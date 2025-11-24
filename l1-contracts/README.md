# Nova Ising L1 Integration

## Overview

On-chain infrastructure for the Nova Ising prover system:
- Job posting with Ising problem commitments
- Proof verification 
- Reward distribution

## Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                         L1 Contract                              │
├─────────────────────────────────────────────────────────────────┤
│  Job Registry           │  Verifier             │  Treasury      │
│  ─────────────          │  ────────             │  ────────      │
│  • Post job             │  • Verify proof       │  • Deposits    │
│  • Problem commitment   │  • Check threshold    │  • Payouts     │
│  • Threshold T          │  • Validate energy    │  • Slashing    │
│  • Reward amount        │                       │                │
│  • Deadline             │                       │                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Nova Proof Bundle                           │
├─────────────────────────────────────────────────────────────────┤
│  • problem_commitment (Poseidon hash)                            │
│  • spin_commitment (Poseidon hash)                               │
│  • claimed_energy                                                │
│  • threshold                                                     │
│  • verified flag (E ≤ T)                                         │
│  • compressed_proof (~10 KB)                                     │
└─────────────────────────────────────────────────────────────────┘
```

## Job Metadata Standard
```solidity
struct IsingJob {
    bytes32 problemCommitment;  // Poseidon hash of (n_spins, edges)
    int64 threshold;            // Energy threshold T
    uint256 reward;             // Reward in wei
    uint256 deadline;           // Block number deadline
    address poster;             // Job poster address
    address solver;             // Solver who claimed (0 if unclaimed)
    JobStatus status;           // OPEN, CLAIMED, VERIFIED, EXPIRED
}
```

## Proof Submission
```solidity
struct ProofSubmission {
    uint256 jobId;
    bytes32 spinCommitment;     // Commitment to solution
    int64 claimedEnergy;        // Claimed energy E
    bytes proof;                // Compressed Nova proof (~10 KB)
}
```
