package types

import (
	"encoding/json"
	"errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	TypeMsgRegisterMiner    = "register_miner"
	TypeMsgSubmitApproval   = "submit_approval"
	TypeMsgSubmitDockingJob = "submit_docking_job"
)

// MsgRegisterMiner registers a new miner
type MsgRegisterMiner struct {
	Address string `json:"address"`
	PubKey  string `json:"pub_key"`
}

func (msg MsgRegisterMiner) Route() string { return ModuleName }
func (msg MsgRegisterMiner) Type() string  { return TypeMsgRegisterMiner }
func (msg MsgRegisterMiner) ValidateBasic() error {
	if msg.Address == "" {
		return errors.New("address cannot be empty")
	}
	return nil
}
func (msg MsgRegisterMiner) GetSigners() []sdk.AccAddress {
	addr, _ := sdk.AccAddressFromBech32(msg.Address)
	return []sdk.AccAddress{addr}
}
func (msg MsgRegisterMiner) GetSignBytes() []byte {
	bz, _ := json.Marshal(msg)
	return bz
}

// MsgSubmitApproval submits a miner's checkpoint approval
type MsgSubmitApproval struct {
	MinerAddress   string        `json:"miner_address"`
	CheckpointHash string        `json:"checkpoint_hash"`
	DockingResult  DockingResult `json:"docking_result"`
	Signature      string        `json:"signature"`
}

func (msg MsgSubmitApproval) Route() string { return ModuleName }
func (msg MsgSubmitApproval) Type() string  { return TypeMsgSubmitApproval }
func (msg MsgSubmitApproval) ValidateBasic() error {
	if msg.MinerAddress == "" {
		return errors.New("miner address cannot be empty")
	}
	if msg.CheckpointHash == "" {
		return errors.New("checkpoint hash cannot be empty")
	}
	return nil
}
func (msg MsgSubmitApproval) GetSigners() []sdk.AccAddress {
	addr, _ := sdk.AccAddressFromBech32(msg.MinerAddress)
	return []sdk.AccAddress{addr}
}
func (msg MsgSubmitApproval) GetSignBytes() []byte {
	bz, _ := json.Marshal(msg)
	return bz
}

// MsgSubmitDockingJob submits a new docking job
type MsgSubmitDockingJob struct {
	Submitter  string `json:"submitter"`
	TargetID   string `json:"target_id"`
	LigandID   string `json:"ligand_id"`
	LigandHash string `json:"ligand_hash"`
}

func (msg MsgSubmitDockingJob) Route() string { return ModuleName }
func (msg MsgSubmitDockingJob) Type() string  { return TypeMsgSubmitDockingJob }
func (msg MsgSubmitDockingJob) ValidateBasic() error {
	if msg.Submitter == "" {
		return errors.New("submitter cannot be empty")
	}
	if msg.TargetID == "" {
		return errors.New("target ID cannot be empty")
	}
	return nil
}
func (msg MsgSubmitDockingJob) GetSigners() []sdk.AccAddress {
	addr, _ := sdk.AccAddressFromBech32(msg.Submitter)
	return []sdk.AccAddress{addr}
}
func (msg MsgSubmitDockingJob) GetSignBytes() []byte {
	bz, _ := json.Marshal(msg)
	return bz
}
