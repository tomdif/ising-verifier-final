# NEXUS Dual Approval Consensus

## Overview

NEXUS uses a two-layer consensus model:
```
┌─────────────────────────────────────────────────────────────────┐
│                     DUAL APPROVAL MODEL                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Layer 1: VALIDATORS (Tendermint PoS)                         │
│   ├── Fast block production (~3 seconds)                       │
│   ├── 67% stake threshold for consensus                        │
│   └── Handles all transactions                                  │
│                                                                 │
│                         ▼                                       │
│                                                                 │
│   Layer 2: MINERS (Proof of Useful Work)                       │
│   ├── Checkpoint every 200 blocks (~10 minutes)                │
│   ├── Must complete docking job to approve                     │
│   ├── 67% miner approval for finality                          │
│   └── Creates irreversible finality                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Security Properties

### Attack Resistance

To successfully attack NEXUS, an attacker needs BOTH:

1. **67% of validator stake** - Requires massive capital
2. **51% of miner compute** - Requires hardware + expertise

These are different resource types, making combined acquisition extremely difficult.

### Finality Guarantees

| Block State | Time | Reversibility |
|-------------|------|---------------|
| Proposed | 0s | Highly reversible |
| Validator approved | ~3s | Reversible with 67% stake |
| Miner checkpoint | ~10 min | Irreversible |

## Checkpoint Flow
```
Block 200 produced by validators
          │
          ▼
    Checkpoint created
    - Contains 10 docking jobs
    - Broadcast to miners
          │
          ▼
    Miners compete to:
    1. Download job data
    2. Run Vina docking
    3. Compute ADMET
    4. Submit approval + results
          │
          ▼
    Approvals collected
    - Each approval contains:
      - Checkpoint hash
      - Docking result hash
      - Miner signature
          │
          ▼
    When 67% approve:
    - Checkpoint finalized
    - Blocks 1-200 irreversible
    - Miners rewarded
```

## Miner Incentives

### Rewards

- **Block rewards**: NEX tokens for valid approvals
- **Job fees**: Share of docking job submission fees
- **Reputation bonus**: Higher reputation = priority job assignment

### Penalties

- **Invalid results**: -200 reputation, potential slashing
- **Missed checkpoints**: -50 reputation
- **At 0 reputation**: Banned from participation

## Economic Model
```
Job Submitter pays 100 NEX
         │
         ├── 70 NEX → Checkpoint miners (split among approvers)
         ├── 20 NEX → Validators (block rewards)
         └── 10 NEX → Treasury (burned or development)
```

## Implementation Status

- [x] Checkpoint types and structures
- [x] Miner registration and management
- [x] Approval submission and verification
- [x] Docking result hash verification
- [x] EndBlocker hook for checkpoint creation
- [ ] Message handlers (CLI/API)
- [ ] Integration tests
- [ ] Validator re-execution logic
- [ ] Token distribution

## Files
```
nexus-chain/x/dualapproval/
├── types/
│   ├── types.go      # Core data structures
│   └── msgs.go       # Transaction message types
├── keeper/
│   └── keeper.go     # State management and logic
└── module.go         # Module registration (TODO)
```
