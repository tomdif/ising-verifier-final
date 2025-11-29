package keeper

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"

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
	return Keeper{cdc: cdc, storeKey: storeKey}
}

// CreateCheckpoint initiates a new checkpoint
func (k Keeper) CreateCheckpoint(ctx sdk.Context, height int64, blockHash, valSetHash string) (*types.Checkpoint, error) {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("checkpoint:%d", height))

	if store.Has(key) {
		return nil, errors.New("checkpoint already exists")
	}

	jobs := k.GetPendingDockingJobs(ctx, 10)

	checkpoint := &types.Checkpoint{
		Height:           height,
		BlockHash:        blockHash,
		ValidatorSetHash: valSetHash,
		Status:           types.CheckpointPending,
		CreatedAt:        ctx.BlockTime(),
		DockingJobs:      jobs,
		MinerApprovals:   []types.MinerApproval{},
	}

	bz, _ := json.Marshal(checkpoint)
	store.Set(key, bz)

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		"checkpoint_created",
		sdk.NewAttribute("height", fmt.Sprintf("%d", height)),
		sdk.NewAttribute("jobs", fmt.Sprintf("%d", len(jobs))),
	))

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

// SubmitMinerApproval processes a miner's checkpoint approval
func (k Keeper) SubmitMinerApproval(ctx sdk.Context, approval types.MinerApproval) error {
	miner, err := k.GetMiner(ctx, approval.MinerAddress)
	if err != nil {
		return errors.New("miner not registered")
	}
	if !miner.CanParticipate() {
		return errors.New("miner not eligible")
	}

	checkpoint, err := k.GetPendingCheckpoint(ctx)
	if err != nil {
		return errors.New("no pending checkpoint")
	}

	// Verify hash
	if approval.CheckpointHash != checkpoint.ComputeCheckpointHash() {
		return errors.New("checkpoint hash mismatch")
	}

	// Verify docking result
	if err := k.VerifyDockingResult(&approval.DockingResult, checkpoint); err != nil {
		k.SlashMiner(ctx, approval.MinerAddress, "invalid_result")
		return err
	}

	// Check duplicate
	for _, existing := range checkpoint.MinerApprovals {
		if existing.MinerAddress == approval.MinerAddress {
			return errors.New("already approved")
		}
	}

	// Add approval
	checkpoint.MinerApprovals = append(checkpoint.MinerApprovals, approval)

	// Update miner stats
	miner.JobsCompleted++
	miner.LastActiveAt = ctx.BlockTime()
	miner.Reputation += 10
	if miner.Reputation > 1000 {
		miner.Reputation = 1000
	}
	k.SetMiner(ctx, miner)

	// Check finalization
	totalMiners := k.GetActiveMinerCount(ctx)
	if checkpoint.CanFinalize(totalMiners) {
		k.FinalizeCheckpoint(ctx, checkpoint)
	} else {
		k.SaveCheckpoint(ctx, checkpoint)
	}

	return nil
}

// VerifyDockingResult checks docking result validity
func (k Keeper) VerifyDockingResult(result *types.DockingResult, checkpoint *types.Checkpoint) error {
	var found bool
	for _, job := range checkpoint.DockingJobs {
		if job.JobID == result.JobID {
			found = true
			break
		}
	}
	if !found {
		return errors.New("job not in checkpoint")
	}

	// Verify result hash
	expected := computeResultHash(result)
	if result.ResultHash != expected {
		return errors.New("result hash mismatch")
	}

	return nil
}

func computeResultHash(result *types.DockingResult) string {
	data := fmt.Sprintf("%s|%s|%s|%s",
		result.JobID, result.Affinity, result.PoseHash, result.ADMETHash)
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}

// FinalizeCheckpoint marks checkpoint as finalized
func (k Keeper) FinalizeCheckpoint(ctx sdk.Context, checkpoint *types.Checkpoint) {
	store := ctx.KVStore(k.storeKey)

	now := ctx.BlockTime()
	checkpoint.Status = types.CheckpointFinalized
	checkpoint.FinalizedAt = &now

	k.SaveCheckpoint(ctx, checkpoint)

	// Update latest
	latestKey := []byte("checkpoint:latest")
	bz, _ := json.Marshal(checkpoint.Height)
	store.Set(latestKey, bz)

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		"checkpoint_finalized",
		sdk.NewAttribute("height", fmt.Sprintf("%d", checkpoint.Height)),
		sdk.NewAttribute("approvals", fmt.Sprintf("%d", len(checkpoint.MinerApprovals))),
	))
}

func (k Keeper) SaveCheckpoint(ctx sdk.Context, checkpoint *types.Checkpoint) {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("checkpoint:%d", checkpoint.Height))
	bz, _ := json.Marshal(checkpoint)
	store.Set(key, bz)
}

func (k Keeper) GetPendingCheckpoint(ctx sdk.Context) (*types.Checkpoint, error) {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get([]byte("checkpoint:pending"))
	if bz == nil {
		return nil, errors.New("no pending checkpoint")
	}
	var height int64
	json.Unmarshal(bz, &height)
	return k.GetCheckpoint(ctx, height)
}

// RegisterMiner adds a new miner
func (k Keeper) RegisterMiner(ctx sdk.Context, address, pubKey string) error {
	store := ctx.KVStore(k.storeKey)
	key := []byte(fmt.Sprintf("miner:%s", address))

	if store.Has(key) {
		return errors.New("already registered")
	}

	miner := &types.Miner{
		Address:      address,
		PublicKey:    pubKey,
		Reputation:   500,
		RegisteredAt: ctx.BlockTime(),
		LastActiveAt: ctx.BlockTime(),
	}

	bz, _ := json.Marshal(miner)
	store.Set(key, bz)
	k.incrementMinerCount(ctx)

	return nil
}

func (k Keeper) GetMiner(ctx sdk.Context, address string) (*types.Miner, error) {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get([]byte(fmt.Sprintf("miner:%s", address)))
	if bz == nil {
		return nil, errors.New("not found")
	}
	var miner types.Miner
	json.Unmarshal(bz, &miner)
	return &miner, nil
}

func (k Keeper) SetMiner(ctx sdk.Context, miner *types.Miner) {
	store := ctx.KVStore(k.storeKey)
	bz, _ := json.Marshal(miner)
	store.Set([]byte(fmt.Sprintf("miner:%s", miner.Address)), bz)
}

func (k Keeper) SlashMiner(ctx sdk.Context, address, reason string) {
	miner, err := k.GetMiner(ctx, address)
	if err != nil {
		return
	}
	if miner.Reputation >= 200 {
		miner.Reputation -= 200
	} else {
		miner.Reputation = 0
		miner.Slashed = true
	}
	k.SetMiner(ctx, miner)

	ctx.EventManager().EmitEvent(sdk.NewEvent(
		"miner_slashed",
		sdk.NewAttribute("address", address),
		sdk.NewAttribute("reason", reason),
	))
}

func (k Keeper) GetActiveMinerCount(ctx sdk.Context) int {
	store := ctx.KVStore(k.storeKey)
	bz := store.Get([]byte("miners:count"))
	if bz == nil {
		return 0
	}
	var count int
	json.Unmarshal(bz, &count)
	return count
}

func (k Keeper) incrementMinerCount(ctx sdk.Context) {
	store := ctx.KVStore(k.storeKey)
	count := k.GetActiveMinerCount(ctx) + 1
	bz, _ := json.Marshal(count)
	store.Set([]byte("miners:count"), bz)
}

func (k Keeper) GetPendingDockingJobs(ctx sdk.Context, limit int) []types.DockingJob {
	store := ctx.KVStore(k.storeKey)
	iterator := sdk.KVStorePrefixIterator(store, []byte("job:pending:"))
	defer iterator.Close()

	jobs := []types.DockingJob{}
	for count := 0; iterator.Valid() && count < limit; iterator.Next() {
		var job types.DockingJob
		json.Unmarshal(iterator.Value(), &job)
		jobs = append(jobs, job)
		count++
	}
	return jobs
}

// EndBlocker called at end of every block
func (k Keeper) EndBlocker(ctx sdk.Context) {
	height := ctx.BlockHeight()

	if height%types.CheckpointInterval == 0 && height > 0 {
		blockHash := fmt.Sprintf("%X", ctx.BlockHeader().LastBlockId.Hash)
		valSetHash := fmt.Sprintf("%X", ctx.BlockHeader().ValidatorsHash)

		checkpoint, err := k.CreateCheckpoint(ctx, height, blockHash, valSetHash)
		if err == nil {
			store := ctx.KVStore(k.storeKey)
			bz, _ := json.Marshal(checkpoint.Height)
			store.Set([]byte("checkpoint:pending"), bz)
		}
	}
}
