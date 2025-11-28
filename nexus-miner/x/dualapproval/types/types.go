package types

import "fmt"

import (
	"crypto/sha256"
	"encoding/hex"
	"time"
)

const (
	ModuleName = "dualapproval"
	StoreKey   = ModuleName

	// Checkpoint every 200 blocks (~10 min at 3s blocks)
	CheckpointInterval = 200

	// Minimum miners needed to finalize checkpoint
	MinCheckpointSigners = 5

	// Threshold for checkpoint approval (67%)
	CheckpointThreshold = 67
)

// CheckpointStatus represents the state of a checkpoint
type CheckpointStatus int

const (
	CheckpointPending CheckpointStatus = iota
	CheckpointApproved
	CheckpointFinalized
	CheckpointExpired
)

// Checkpoint represents a miner-approved finality point
type Checkpoint struct {
	Height           int64              `json:"height"`
	BlockHash        string             `json:"block_hash"`
	ValidatorSetHash string             `json:"validator_set_hash"`
	Status           CheckpointStatus   `json:"status"`
	CreatedAt        time.Time          `json:"created_at"`
	FinalizedAt      *time.Time         `json:"finalized_at,omitempty"`
	DockingJobs      []DockingJob       `json:"docking_jobs"`
	MinerApprovals   []MinerApproval    `json:"miner_approvals"`
}

// DockingJob is the work miners must complete
type DockingJob struct {
	JobID      string `json:"job_id"`
	TargetID   string `json:"target_id"`   // e.g., "6LU7" (COVID Mpro)
	LigandID   string `json:"ligand_id"`   // e.g., compound identifier
	LigandHash string `json:"ligand_hash"` // SHA256 of ligand PDBQT
	Seed       uint32 `json:"seed"`        // Deterministic seed
	Assigned   bool   `json:"assigned"`
}

// MinerApproval is a miner's signature on a checkpoint
type MinerApproval struct {
	MinerAddress   string        `json:"miner_address"`
	CheckpointHash string        `json:"checkpoint_hash"`
	DockingResult  DockingResult `json:"docking_result"`
	Signature      string        `json:"signature"`
	Timestamp      time.Time     `json:"timestamp"`
}

// DockingResult is the verifiable output from a miner
type DockingResult struct {
	JobID       string `json:"job_id"`
	Affinity    string `json:"affinity"`    // Fixed precision string
	PoseHash    string `json:"pose_hash"`   // SHA256 of output PDBQT
	ADMETHash   string `json:"admet_hash"`  // SHA256 of ADMET properties
	ResultHash  string `json:"result_hash"` // Combined hash
}

// ComputeCheckpointHash creates deterministic hash of checkpoint data
func (c *Checkpoint) ComputeCheckpointHash() string {
	data := fmt.Sprintf("%d|%s|%s|%d",
		c.Height,
		c.BlockHash,
		c.ValidatorSetHash,
		len(c.DockingJobs),
	)
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}

// ApprovalPercentage returns current approval level
func (c *Checkpoint) ApprovalPercentage(totalMiners int) int {
	if totalMiners == 0 {
		return 0
	}
	return (len(c.MinerApprovals) * 100) / totalMiners
}

// CanFinalize checks if checkpoint has enough approvals
func (c *Checkpoint) CanFinalize(totalMiners int) bool {
	return len(c.MinerApprovals) >= MinCheckpointSigners &&
		c.ApprovalPercentage(totalMiners) >= CheckpointThreshold
}

// Miner represents a registered miner
type Miner struct {
	Address       string    `json:"address"`
	PublicKey     string    `json:"public_key"`
	ComputePower  uint64    `json:"compute_power"`  // Benchmarked capacity
	JobsCompleted uint64    `json:"jobs_completed"`
	Reputation    uint64    `json:"reputation"`     // 0-1000
	RegisteredAt  time.Time `json:"registered_at"`
	LastActiveAt  time.Time `json:"last_active_at"`
	Slashed       bool      `json:"slashed"`
}

// CanParticipate checks if miner is eligible for checkpoints
func (m *Miner) CanParticipate() bool {
	return !m.Slashed && m.Reputation >= 100
}
