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
    if bonds > 30 {
        bonds = 30
    }
    // Power law: (bonds + 1)^1.2 - precomputed for 0-30
    multipliers := []float64{
        1.00,   // 0
        2.30,   // 1
        3.76,   // 2
        5.34,   // 3
        7.01,   // 4
        8.76,   // 5
        10.56,  // 6
        12.43,  // 7
        14.35,  // 8
        16.31,  // 9
        18.32,  // 10
        20.37,  // 11
        22.45,  // 12
        24.58,  // 13
        26.73,  // 14
        28.92,  // 15
        31.14,  // 16
        33.39,  // 17
        35.67,  // 18
        37.97,  // 19
        40.30,  // 20
        42.66,  // 21
        45.04,  // 22
        47.44,  // 23
        49.87,  // 24
        52.32,  // 25
        54.79,  // 26
        57.28,  // 27
        59.79,  // 28
        62.32,  // 29
        64.88,  // 30
    }
    return multipliers[bonds]
}

func (k Keeper) GetEmissionForPeriod(periodId uint64, halvingInterval uint64, baseEmission uint64) uint64 {
	halvings := periodId / halvingInterval
	if halvings >= 64 {
		return 0
	}
	return baseEmission >> halvings
}
