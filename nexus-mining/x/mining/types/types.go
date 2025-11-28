package types

const (
	ModuleName = "mining"
	StoreKey   = ModuleName
)

const (
	LigandStateFresh    = 0
	LigandStatePending  = 1
	LigandStateVerified = 2
	LigandStateDisputed = 3
)

var (
	ParamsKey             = []byte{0x00}
	JobKeyPrefix          = []byte{0x01}
	LigandWorkKeyPrefix   = []byte{0x02}
	AssignmentKeyPrefix   = []byte{0x03}
	RewardPeriodKeyPrefix = []byte{0x05}
	CurrentPeriodKey      = []byte{0x06}
	FreshQueueKeyPrefix   = []byte{0x10}
	PendingQueueKeyPrefix = []byte{0x11}
	VerifiedResultPrefix  = []byte{0x20}
	HitKeyPrefix          = []byte{0x21}
	MinerSharesKeyPrefix  = []byte{0x30}
)

const (
	EventTypeJobCreated     = "job_created"
	EventTypeLigandVerified = "ligand_verified"
	EventTypeLigandDisputed = "ligand_disputed"
	EventTypePeriodEnded    = "period_ended"
	EventTypeMinerRewarded  = "miner_rewarded"
)

type Params struct {
	BlocksPerPeriod       int64  `json:"blocks_per_period"`
	BaseEmission          string `json:"base_emission"`
	HalvingInterval       uint64 `json:"halving_interval"`
	MaxSubmissions        int32  `json:"max_submissions"`
	HitThreshold          string `json:"hit_threshold"`
	HeartbeatGraceSeconds int64  `json:"heartbeat_grace_seconds"`
}

func DefaultParams() Params {
	return Params{
		BlocksPerPeriod:       100,
		BaseEmission:          "359500000000",
		HalvingInterval:       525600,
		MaxSubmissions:        10,
		HitThreshold:          "-7.0",
		HeartbeatGraceSeconds: 45,
	}
}
