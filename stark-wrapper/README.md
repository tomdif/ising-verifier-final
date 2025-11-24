# Nova STARK Wrapper

Wraps Nova proofs in a STARK for quantum-resistant on-chain verification.

## Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                    Proof Generation Flow                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  Nova Prover │───▶│  SP1 Program │───▶│ STARK Proof  │       │
│  │  (off-chain) │    │  (RISC-V)    │    │ (on-chain)   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                   │                   │                │
│    ~10KB proof         Verifies:           ~50KB proof          │
│    ~10s time       - Commitments          Quantum-safe          │
│                    - Energy bound                               │
│                    - Threshold check                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Why STARK Wrapper?

| Property | Nova | STARK | Nova + STARK |
|----------|------|-------|--------------|
| Proof size | ~10 KB | ~50 KB | ~50 KB |
| Prove time | Fast | Slower | ~30s total |
| Verify cost | Medium | Low | Low |
| Quantum-safe | ❌ | ✅ | ✅ |

## Components

### Guest Program (program/)
RISC-V program that runs inside SP1:
- Reads Nova public inputs
- Verifies commitment structure
- Checks energy ≤ threshold
- Outputs verified public values

### Host (host/)
Orchestrates proof generation:
- Takes Nova proof + public inputs
- Runs SP1 prover
- Generates STARK proof
- Exports for on-chain verification

### Contracts
- `SP1NovaVerifier.sol` - On-chain STARK verification
