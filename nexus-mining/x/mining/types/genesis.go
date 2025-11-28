package types

type GenesisState struct {
	Params          Params `json:"params"`
	CurrentPeriodId uint64 `json:"current_period_id"`
}

func DefaultGenesis() *GenesisState {
	return &GenesisState{
		Params:          DefaultParams(),
		CurrentPeriodId: 1,
	}
}

func (gs GenesisState) Validate() error {
	if gs.Params.BlocksPerPeriod <= 0 {
		return ErrInvalidParams
	}
	return nil
}
