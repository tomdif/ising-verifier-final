package keeper

import (
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"fmt"

	"github.com/nexus-chain/nexus/x/mining/types"
)

type Keeper struct {
	authority string
}

func NewKeeper(authority string) Keeper {
	return Keeper{authority: authority}
}

func (k Keeper) GetAuthority() string {
	return k.authority
}

func (k Keeper) ComputeSeed(jobId string, ligandId int64) uint64 {
	h := sha256.New()
	h.Write([]byte(jobId))
	h.Write([]byte(fmt.Sprintf("%d", ligandId)))
	hashBytes := h.Sum(nil)
	return binary.BigEndian.Uint64(hashBytes[:8])
}

func ComputeResultHash(jobId string, ligandId int64, seed uint64, score string, poseData string) string {
	h := sha256.New()
	h.Write([]byte(jobId))
	h.Write([]byte(fmt.Sprintf("%d", ligandId)))
	h.Write([]byte(fmt.Sprintf("%d", seed)))
	h.Write([]byte(score))
	h.Write([]byte(poseData))
	return hex.EncodeToString(h.Sum(nil))
}

func GetBondMultiplier(bonds int32) float64 {
	if bonds < 0 {
		bonds = 0
	}
	if bonds <= 8 {
		multipliers := []float64{0.1, 0.447, 0.9, 1.442, 2.074, 2.796, 3.61, 4.518, 5.52}
		return multipliers[bonds]
	}
	result := 5.52
	for i := int32(9); i <= bonds; i++ {
		result *= 1.5
	}
	return result
}

func (k Keeper) GetEmissionForPeriod(periodId uint64, halvingInterval uint64, baseEmission uint64) uint64 {
	halvings := periodId / halvingInterval
	if halvings >= 64 {
		return 0
	}
	return baseEmission >> halvings
}
