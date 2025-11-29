package keeper

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"sync"
	"time"

	"nexus-chain/x/dualapproval/types"
)

// Simple in-memory store for standalone mode
type Store struct {
	data map[string][]byte
	mu   sync.RWMutex
}

func NewStore() *Store {
	return &Store{data: make(map[string][]byte)}
}

func (s *Store) Get(key []byte) []byte {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.data[string(key)]
}

func (s *Store) Set(key, value []byte) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.data[string(key)] = value
}

func (s *Store) Has(key []byte) bool {
	s.mu.RLock()
	defer s.mu.RUnlock()
	_, ok := s.data[string(key)]
	return ok
}

type Keeper struct {
	store *Store
}

func NewKeeper() *Keeper {
	return &Keeper{store: NewStore()}
}

// CreateCheckpoint initiates a new checkpoint
func (k *Keeper) CreateCheckpoint(height int64, blockHash, valSetHash string) (*types.Checkpoint, error) {
	key := []byte(fmt.Sprintf("checkpoint:%d", height))

	if k.store.Has(key) {
		return nil, errors.New("checkpoint already exists")
	}

	jobs := k.GetPendingDockingJobs(10)

	checkpoint := &types.Checkpoint{
		Height:           height,
		BlockHash:        blockHash,
		ValidatorSetHash: valSetHash,
		Status:           types.CheckpointPending,
		CreatedAt:        time.Now(),
		DockingJobs:      jobs,
		MinerApprovals:   []types.MinerApproval{},
	}

	bz, _ := json.Marshal(checkpoint)
	k.store.Set(key, bz)

	return checkpoint, nil
}

// GetCheckpoint retrieves a checkpoint by height
func (k *Keeper) GetCheckpoint(height int64) (*types.Checkpoint, error) {
	key := []byte(fmt.Sprintf("checkpoint:%d", height))

	bz := k.store.Get(key)
	if bz == nil {
		return nil, errors.New("checkpoint not found")
	}

	var checkpoint types.Checkpoint
	json.Unmarshal(bz, &checkpoint)
	return &checkpoint, nil
}

// SubmitMinerApproval processes a miner's checkpoint approval
func (k *Keeper) SubmitMinerApproval(approval types.MinerApproval) error {
	miner, err := k.GetMiner(approval.MinerAddress)
	if err != nil {
		return errors.New("miner not registered")
	}
	if !miner.CanParticipate() {
		return errors.New("miner not eligible")
	}

	checkpoint, err := k.GetPendingCheckpoint()
	if err != nil {
		return errors.New("no pending checkpoint")
	}

	// Verify hash
	if approval.CheckpointHash != checkpoint.ComputeCheckpointHash() {
		return errors.New("checkpoint hash mismatch")
	}

	// Verify docking result
	if err := k.VerifyDockingResult(&approval.DockingResult, checkpoint); err != nil {
		k.SlashMiner(approval.MinerAddress, "invalid_result")
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
	miner.LastActiveAt = time.Now()
	miner.Reputation += 10
	if miner.Reputation > 1000 {
		miner.Reputation = 1000
	}
	k.SetMiner(miner)

	// Check finalization
	totalMiners := k.GetActiveMinerCount()
	if checkpoint.CanFinalize(totalMiners) {
		k.FinalizeCheckpoint(checkpoint)
	} else {
		k.SaveCheckpoint(checkpoint)
	}

	return nil
}

// VerifyDockingResult checks docking result validity
func (k *Keeper) VerifyDockingResult(result *types.DockingResult, checkpoint *types.Checkpoint) error {
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
func (k *Keeper) FinalizeCheckpoint(checkpoint *types.Checkpoint) {
	now := time.Now()
	checkpoint.Status = types.CheckpointFinalized
	checkpoint.FinalizedAt = &now

	k.SaveCheckpoint(checkpoint)

	// Update latest
	latestKey := []byte("checkpoint:latest")
	bz, _ := json.Marshal(checkpoint.Height)
	k.store.Set(latestKey, bz)
}

func (k *Keeper) SaveCheckpoint(checkpoint *types.Checkpoint) {
	key := []byte(fmt.Sprintf("checkpoint:%d", checkpoint.Height))
	bz, _ := json.Marshal(checkpoint)
	k.store.Set(key, bz)
}

func (k *Keeper) GetPendingCheckpoint() (*types.Checkpoint, error) {
	bz := k.store.Get([]byte("checkpoint:pending"))
	if bz == nil {
		return nil, errors.New("no pending checkpoint")
	}
	var height int64
	json.Unmarshal(bz, &height)
	return k.GetCheckpoint(height)
}

// RegisterMiner adds a new miner
func (k *Keeper) RegisterMiner(address, pubKey string) error {
	key := []byte(fmt.Sprintf("miner:%s", address))

	if k.store.Has(key) {
		return errors.New("already registered")
	}

	miner := &types.Miner{
		Address:      address,
		PublicKey:    pubKey,
		Reputation:   500,
		RegisteredAt: time.Now(),
		LastActiveAt: time.Now(),
	}

	bz, _ := json.Marshal(miner)
	k.store.Set(key, bz)
	k.incrementMinerCount()

	return nil
}

func (k *Keeper) GetMiner(address string) (*types.Miner, error) {
	bz := k.store.Get([]byte(fmt.Sprintf("miner:%s", address)))
	if bz == nil {
		return nil, errors.New("not found")
	}
	var miner types.Miner
	json.Unmarshal(bz, &miner)
	return &miner, nil
}

func (k *Keeper) SetMiner(miner *types.Miner) {
	bz, _ := json.Marshal(miner)
	k.store.Set([]byte(fmt.Sprintf("miner:%s", miner.Address)), bz)
}

func (k *Keeper) SlashMiner(address, reason string) {
	miner, err := k.GetMiner(address)
	if err != nil {
		return
	}
	if miner.Reputation >= 200 {
		miner.Reputation -= 200
	} else {
		miner.Reputation = 0
		miner.Slashed = true
	}
	k.SetMiner(miner)
}

func (k *Keeper) GetActiveMinerCount() int {
	bz := k.store.Get([]byte("miners:count"))
	if bz == nil {
		return 0
	}
	var count int
	json.Unmarshal(bz, &count)
	return count
}

func (k *Keeper) incrementMinerCount() {
	count := k.GetActiveMinerCount() + 1
	bz, _ := json.Marshal(count)
	k.store.Set([]byte("miners:count"), bz)
}

func (k *Keeper) GetPendingDockingJobs(limit int) []types.DockingJob {
	// In production, would iterate over job:pending: prefix
	return []types.DockingJob{}
}

// EndBlocker called at end of every block
func (k *Keeper) EndBlocker(height int64, blockHash, valSetHash string) {
	if height%types.CheckpointInterval == 0 && height > 0 {
		checkpoint, err := k.CreateCheckpoint(height, blockHash, valSetHash)
		if err == nil {
			bz, _ := json.Marshal(checkpoint.Height)
			k.store.Set([]byte("checkpoint:pending"), bz)
		}
	}
}
