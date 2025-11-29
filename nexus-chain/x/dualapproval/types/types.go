package types

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"time"
)

const (
	ModuleName = "dualapproval"
	StoreKey   = ModuleName

	CheckpointInterval   = 200 // Every 200 blocks (~10 min at 3s blocks)
	MinCheckpointSigners = 5   // Minimum miners to finalize
	CheckpointThreshold  = 67  // 67% approval required
)

type CheckpointStatus int

const (
	CheckpointPending CheckpointStatus = iota
	CheckpointApproved
	CheckpointFinalized
	CheckpointExpired
)

// Checkpoint is a miner-approved finality point
type Checkpoint struct {
	Height           int64            `json:"height"`
	BlockHash        string           `json:"block_hash"`
	ValidatorSetHash string           `json:"validator_set_hash"`
	Status           CheckpointStatus `json:"status"`
	CreatedAt        time.Time        `json:"created_at"`
	FinalizedAt      *time.Time       `json:"finalized_at,omitempty"`
	DockingJobs      []DockingJob     `json:"docking_jobs"`
	MinerApprovals   []MinerApproval  `json:"miner_approvals"`
}

// DockingJob is work miners must complete to approve checkpoint
type DockingJob struct {
	JobID      string `json:"job_id"`
	TargetID   string `json:"target_id"`
	LigandID   string `json:"ligand_id"`
	LigandHash string `json:"ligand_hash"`
	Seed       uint32 `json:"seed"`
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

// DockingResult is verifiable output from a miner
type DockingResult struct {
	JobID      string `json:"job_id"`
	Affinity   string `json:"affinity"`
	PoseHash   string `json:"pose_hash"`
	ADMETHash  string `json:"admet_hash"`
	ResultHash string `json:"result_hash"`
}

// Miner represents a registered miner
type Miner struct {
	Address       string    `json:"address"`
	PublicKey     string    `json:"public_key"`
	ComputePower  uint64    `json:"compute_power"`
	JobsCompleted uint64    `json:"jobs_completed"`
	Reputation    uint64    `json:"reputation"` // 0-1000
	RegisteredAt  time.Time `json:"registered_at"`
	LastActiveAt  time.Time `json:"last_active_at"`
	Slashed       bool      `json:"slashed"`
}

func (c *Checkpoint) ComputeCheckpointHash() string {
	data := fmt.Sprintf("%d|%s|%s|%d",
		c.Height, c.BlockHash, c.ValidatorSetHash, len(c.DockingJobs))
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}

func (c *Checkpoint) ApprovalPercentage(totalMiners int) int {
	if totalMiners == 0 {
		return 0
	}
	return (len(c.MinerApprovals) * 100) / totalMiners
}

func (c *Checkpoint) CanFinalize(totalMiners int) bool {
	return len(c.MinerApprovals) >= MinCheckpointSigners &&
		c.ApprovalPercentage(totalMiners) >= CheckpointThreshold
}

func (m *Miner) CanParticipate() bool {
	return !m.Slashed && m.Reputation >= 100
}
