package keeper

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/cosmos/cosmos-sdk/codec"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"nexus-chain/x/dualapproval/types"
)

type Keeper struct {
	cdc      codec.BinaryCodec
	storeKey storetypes.StoreKey
}

func NewKeeper(cdc codec.BinaryCodec, storeKey storetypes.StoreKey) Keeper {
	return Keeper{
		cdc:      cdc,
		storeKey: storeKey,
	}
}

// ============================================================
// CHECKPOINT MANAGEMENT
// ============================================================

// CreateCheckpoint initiates a new checkpoint at the given height
func (k Keeper) CreateCheckpoint(ctx sdk.Context, height int64, blockHash string, valSetHash string) (*types.Checkpoint, error) {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("checkpoint:%d", height))

	// Check if checkpoint already exists
	if store.Has(key) {
		return nil, errors.New("checkpoint already exists for this height")
	}

	// Get pending docking jobs from queue
	jobs := k.GetPendingDockingJobs(ctx, 10) // 10 jobs per checkpoint

	checkpoint := &types.Checkpoint{
		Height:           height,
		BlockHash:        blockHash,
		ValidatorSetHash: valSetHash,
		Status:           types.CheckpointPending,
		CreatedAt:        ctx.BlockTime(),
		DockingJobs:      jobs,
		MinerApprovals:   []types.MinerApproval{},
	}

	// Save checkpoint
	bz, _ := json.Marshal(checkpoint)
	store.Set(key, bz)

	// Emit event
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"checkpoint_created",
			sdk.NewAttribute("height", fmt.Sprintf("%d", height)),
			sdk.NewAttribute("block_hash", blockHash),
			sdk.NewAttribute("jobs_count", fmt.Sprintf("%d", len(jobs))),
		),
	)

	return checkpoint, nil
}

// GetCheckpoint retrieves a checkpoint by height
func (k Keeper) GetCheckpoint(ctx sdk.Context, height int64) (*types.Checkpoint, error) {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("checkpoint:%d", height))

	bz := store.Get(key)
	if bz == nil {
		return nil, errors.New("checkpoint not found")
	}

	var checkpoint types.Checkpoint
	json.Unmarshal(bz, &checkpoint)
	return &checkpoint, nil
}

// GetLatestCheckpoint returns the most recent finalized checkpoint
func (k Keeper) GetLatestCheckpoint(ctx sdk.Context) (*types.Checkpoint, error) {
	store := ctx.KVStore(k.storeKey)
	
	// Get stored latest height
	latestKey := []byte("checkpoint:latest")
	bz := store.Get(latestKey)
	if bz == nil {
		return nil, errors.New("no finalized checkpoints")
	}

	var height int64
	json.Unmarshal(bz, &height)
	return k.GetCheckpoint(ctx, height)
}

// ============================================================
// MINER APPROVAL SUBMISSION
// ============================================================

// SubmitMinerApproval processes a miner's checkpoint approval
func (k Keeper) SubmitMinerApproval(ctx sdk.Context, approval types.MinerApproval) error {
	// Verify miner is registered and eligible
	miner, err := k.GetMiner(ctx, approval.MinerAddress)
	if err != nil {
		return errors.New("miner not registered")
	}
	if !miner.CanParticipate() {
		return errors.New("miner not eligible to participate")
	}

	// Get the checkpoint
	// Parse height from checkpoint hash (in real impl, would lookup by hash)
	checkpoint, err := k.GetPendingCheckpoint(ctx)
	if err != nil {
		return errors.New("no pending checkpoint")
	}

	// Verify checkpoint hash matches
	expectedHash := checkpoint.ComputeCheckpointHash()
	if approval.CheckpointHash != expectedHash {
		return errors.New("checkpoint hash mismatch")
	}

	// Verify docking result
	if err := k.VerifyDockingResult(ctx, &approval.DockingResult, checkpoint); err != nil {
		// Slash miner for invalid result
		k.SlashMiner(ctx, approval.MinerAddress, "invalid_docking_result")
		return fmt.Errorf("invalid docking result: %w", err)
	}

	// Check for duplicate approval
	for _, existing := range checkpoint.MinerApprovals {
		if existing.MinerAddress == approval.MinerAddress {
			return errors.New("miner already approved this checkpoint")
		}
	}

	// Add approval
	checkpoint.MinerApprovals = append(checkpoint.MinerApprovals, approval)

	// Update miner stats
	miner.JobsCompleted++
	miner.LastActiveAt = ctx.BlockTime()
	miner.Reputation += 10 // Reward for participation
	if miner.Reputation > 1000 {
		miner.Reputation = 1000
	}
	k.SetMiner(ctx, miner)

	// Check if checkpoint can be finalized
	totalMiners := k.GetActiveMinerCount(ctx)
	if checkpoint.CanFinalize(totalMiners) {
		k.FinalizeCheckpoint(ctx, checkpoint)
	} else {
		// Save updated checkpoint
		k.SaveCheckpoint(ctx, checkpoint)
	}

	// Emit event
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"miner_approval",
			sdk.NewAttribute("miner", approval.MinerAddress),
			sdk.NewAttribute("checkpoint_height", fmt.Sprintf("%d", checkpoint.Height)),
			sdk.NewAttribute("total_approvals", fmt.Sprintf("%d", len(checkpoint.MinerApprovals))),
		),
	)

	return nil
}

// VerifyDockingResult checks that a docking result is valid
func (k Keeper) VerifyDockingResult(ctx sdk.Context, result *types.DockingResult, checkpoint *types.Checkpoint) error {
	// Find the job in checkpoint
	var job *types.DockingJob
	for _, j := range checkpoint.DockingJobs {
		if j.JobID == result.JobID {
			job = &j
			break
		}
	}
	if job == nil {
		return errors.New("job not found in checkpoint")
	}

	// Verify result hash is correctly computed
	expectedResultHash := computeResultHash(result)
	if result.ResultHash != expectedResultHash {
		return errors.New("result hash mismatch")
	}

	// In production: validators would re-run docking to verify
	// For now, we trust the hash-based verification
	// Full re-execution would be done by a subset of validators

	return nil
}

func computeResultHash(result *types.DockingResult) string {
	data := fmt.Sprintf("%s|%s|%s|%s",
		result.JobID,
		result.Affinity,
		result.PoseHash,
		result.ADMETHash,
	)
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}

// FinalizeCheckpoint marks a checkpoint as finalized
func (k Keeper) FinalizeCheckpoint(ctx sdk.Context, checkpoint *types.Checkpoint) {
	store := ctx.KVStore(k.storeKey)

	now := ctx.BlockTime()
	checkpoint.Status = types.CheckpointFinalized
	checkpoint.FinalizedAt = &now

	// Save checkpoint
	k.SaveCheckpoint(ctx, checkpoint)

	// Update latest finalized
	latestKey := []byte("checkpoint:latest")
	bz, _ := json.Marshal(checkpoint.Height)
	store.Set(latestKey, bz)

	// Emit event
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"checkpoint_finalized",
			sdk.NewAttribute("height", fmt.Sprintf("%d", checkpoint.Height)),
			sdk.NewAttribute("approvals", fmt.Sprintf("%d", len(checkpoint.MinerApprovals))),
			sdk.NewAttribute("block_hash", checkpoint.BlockHash),
		),
	)
}

// SaveCheckpoint persists a checkpoint
func (k Keeper) SaveCheckpoint(ctx sdk.Context, checkpoint *types.Checkpoint) {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("checkpoint:%d", checkpoint.Height))
	bz, _ := json.Marshal(checkpoint)
	store.Set(key, bz)
}

// GetPendingCheckpoint returns the current unfinalized checkpoint
func (k Keeper) GetPendingCheckpoint(ctx sdk.Context) (*types.Checkpoint, error) {
	store := ctx.KVStore(k.storeKey)
	
	pendingKey := []byte("checkpoint:pending")
	bz := store.Get(pendingKey)
	if bz == nil {
		return nil, errors.New("no pending checkpoint")
	}

	var height int64
	json.Unmarshal(bz, &height)
	return k.GetCheckpoint(ctx, height)
}

// ============================================================
// MINER MANAGEMENT
// ============================================================

// RegisterMiner adds a new miner to the network
func (k Keeper) RegisterMiner(ctx sdk.Context, address string, pubKey string) error {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("miner:%s", address))

	if store.Has(key) {
		return errors.New("miner already registered")
	}

	miner := &types.Miner{
		Address:       address,
		PublicKey:     pubKey,
		ComputePower:  0,
		JobsCompleted: 0,
		Reputation:    500, // Start at neutral
		RegisteredAt:  ctx.BlockTime(),
		LastActiveAt:  ctx.BlockTime(),
		Slashed:       false,
	}

	bz, _ := json.Marshal(miner)
	store.Set(key, bz)

	// Update miner count
	k.incrementMinerCount(ctx)

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"miner_registered",
			sdk.NewAttribute("address", address),
		),
	)

	return nil
}

// GetMiner retrieves a miner by address
func (k Keeper) GetMiner(ctx sdk.Context, address string) (*types.Miner, error) {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("miner:%s", address))

	bz := store.Get(key)
	if bz == nil {
		return nil, errors.New("miner not found")
	}

	var miner types.Miner
	json.Unmarshal(bz, &miner)
	return &miner, nil
}

// SetMiner updates a miner
func (k Keeper) SetMiner(ctx sdk.Context, miner *types.Miner) {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("miner:%s", miner.Address))
	bz, _ := json.Marshal(miner)
	store.Set(key, bz)
}

// SlashMiner penalizes a miner for misbehavior
func (k Keeper) SlashMiner(ctx sdk.Context, address string, reason string) {
	miner, err := k.GetMiner(ctx, address)
	if err != nil {
		return
	}

	// Reduce reputation
	if miner.Reputation >= 200 {
		miner.Reputation -= 200
	} else {
		miner.Reputation = 0
		miner.Slashed = true
	}

	k.SetMiner(ctx, miner)

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"miner_slashed",
			sdk.NewAttribute("address", address),
			sdk.NewAttribute("reason", reason),
			sdk.NewAttribute("new_reputation", fmt.Sprintf("%d", miner.Reputation)),
		),
	)
}

// GetActiveMinerCount returns number of active miners
func (k Keeper) GetActiveMinerCount(ctx sdk.Context) int {
	store := ctx.KVStore(k.storeKey)
	key := []byte("miners:count")
	bz := store.Get(key)
	if bz == nil {
		return 0
	}
	var count int
	json.Unmarshal(bz, &count)
	return count
}

func (k Keeper) incrementMinerCount(ctx sdk.Context) {
	store := ctx.KVStore(k.storeKey)
	key := []byte("miners:count")
	count := k.GetActiveMinerCount(ctx) + 1
	bz, _ := json.Marshal(count)
	store.Set(key, bz)
}

// ============================================================
// DOCKING JOB QUEUE
// ============================================================

// SubmitDockingJob adds a job to the queue
func (k Keeper) SubmitDockingJob(ctx sdk.Context, job types.DockingJob) error {
	store := ctx.KVStore(k.storeKey)
	
	// Add to pending queue
	key := []byte(fmt.Sprintf("job:pending:%s", job.JobID))
	bz, _ := json.Marshal(job)
	store.Set(key, bz)

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			"docking_job_submitted",
			sdk.NewAttribute("job_id", job.JobID),
			sdk.NewAttribute("target", job.TargetID),
		),
	)

	return nil
}

// GetPendingDockingJobs retrieves jobs for a checkpoint
func (k Keeper) GetPendingDockingJobs(ctx sdk.Context, limit int) []types.DockingJob {
	store := ctx.KVStore(k.storeKey)
	iterator := sdk.KVStorePrefixIterator(store, []byte("job:pending:"))
	defer iterator.Close()

	jobs := []types.DockingJob{}
	count := 0

	for ; iterator.Valid() && count < limit; iterator.Next() {
		var job types.DockingJob
		json.Unmarshal(iterator.Value(), &job)
		jobs = append(jobs, job)
		count++
	}

	return jobs
}

// ============================================================
// END BLOCK HOOK
// ============================================================

// EndBlocker is called at the end of every block
func (k Keeper) EndBlocker(ctx sdk.Context) {
	height := ctx.BlockHeight()

	// Check if this is a checkpoint block
	if height % types.CheckpointInterval == 0 && height > 0 {
		// Get block hash from context
		blockHash := fmt.Sprintf("%X", ctx.BlockHeader().LastBlockId.Hash)
		valSetHash := fmt.Sprintf("%X", ctx.BlockHeader().ValidatorsHash)

		// Create new checkpoint
		checkpoint, err := k.CreateCheckpoint(ctx, height, blockHash, valSetHash)
		if err == nil {
			// Mark as pending
			store := ctx.KVStore(k.storeKey)
			pendingKey := []byte("checkpoint:pending")
			bz, _ := json.Marshal(checkpoint.Height)
			store.Set(pendingKey, bz)
		}
	}

	// Expire old pending checkpoints (after 100 blocks)
	k.expireOldCheckpoints(ctx, height)
}

func (k Keeper) expireOldCheckpoints(ctx sdk.Context, currentHeight int64) {
	checkpoint, err := k.GetPendingCheckpoint(ctx)
	if err != nil {
		return
	}

	// If checkpoint is more than 100 blocks old and not finalized, expire it
	if currentHeight - checkpoint.Height > 100 && checkpoint.Status == types.CheckpointPending {
		checkpoint.Status = types.CheckpointExpired
		k.SaveCheckpoint(ctx, checkpoint)

		// Penalize miners who didn't participate
		// (In production, would track who was assigned)

		ctx.EventManager().EmitEvent(
			sdk.NewEvent(
				"checkpoint_expired",
				sdk.NewAttribute("height", fmt.Sprintf("%d", checkpoint.Height)),
			),
		)
	}
}
