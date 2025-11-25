# Nova Ising Verifier - Complete System

**GPU-accelerated, quantum-resistant proving system for Ising optimization problems**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

A complete end-to-end system for decentralized Ising optimization:
- **Nova IVC** proofs (fast, ~10s for 1.3M spins)
- **STARK wrapper** for quantum-resistant on-chain verification
- **L1 contracts** for job posting and reward distribution
- **Orchestration API** for network coordination
- **Web dashboard** for job management

## Quick Start

### 1. Run the Prover
```bash
cd nova-prover
cargo run --release
```

**Output:** Proves 1.3M spins in ~10s, generates 9.9KB proof

### 2. Start the Orchestrator
```bash
cd orchestration
cargo run --release
```

**API:** http://localhost:3000

### 3. Launch the Dashboard
```bash
cd dashboard
npm install
npm run dev
```

**UI:** http://localhost:3000

## Architecture
```
┌──────────────────────────────────────────────────────────────────┐
│                     Nova Ising System                             │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌──────────────┐         │
│  │   Prover    │───▶│ Orchestrator│───▶│  L1 Contract │         │
│  │   (Rust)    │    │   (Rust)    │    │  (Solidity)  │         │
│  └─────────────┘    └─────────────┘    └──────────────┘         │
│         │                   │                   │                │
│    ~10s prove          Job queue          On-chain verify        │
│    9.9KB proof         PUUB scoring       Reward payout          │
│    GPU accel           REST API           Quantum-safe           │
│                                                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌──────────────┐         │
│  │ STARK Wrap  │    │  Dashboard  │    │  Integration │         │
│  │   (SP1)     │    │  (Next.js)  │    │    Tests     │         │
│  └─────────────┘    └─────────────┘    └──────────────┘         │
│         │                   │                   │                │
│  Quantum-safe          Web UI             E2E validation         │
│  ~30s total            Job browser        All passing            │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

## Components

### Nova Prover (`nova-prover/`)

**Capabilities:**
- GPU-accelerated Poseidon commitments (9.5x speedup)
- Threshold verification (prove E ≤ T in-circuit)
- Gap-hiding support (prove E + Δ ≤ T without revealing E)
- Lt64 comparison gadget (68 constraints)

**Benchmark (10x scale):**
- Spins: 1.31M
- Edges: 15.7M
- Prove time: ~10s
- Proof size: 9.9 KB

**Run:**
```bash
cd nova-prover
cargo run --release
```

### L1 Contracts (`l1-contracts/`)

**Contracts:**
- `IsingJobManager.sol` - Job posting, proof submission, rewards
- `NovaVerifier.sol` - Stub/Optimistic/Full verification modes

**Tests:**
- 33/33 passing with Foundry
- Coverage: Job lifecycle, rewards, admin functions

**Deploy to Sepolia:**
```bash
cd l1-contracts
forge install foundry-rs/forge-std
source .env  # Set PRIVATE_KEY, SEPOLIA_RPC_URL, ETHERSCAN_API_KEY
forge script script/Deploy.s.sol:DeployIsing --rpc-url $SEPOLIA_RPC_URL --broadcast
```

### Orchestration API (`orchestration/`)

**Features:**
- Job indexing and discovery
- Prover registration and matching
- PUUB (Proof-of-Useful-Useful-Work) scoring
- Real-time leaderboard

**Endpoints:**
- `GET /jobs` - List open jobs
- `POST /jobs/:id/claim` - Claim job for proving
- `POST /provers/register` - Register as prover
- `GET /provers/leaderboard` - Top provers by PUUB score

**Run:**
```bash
cd orchestration
cargo run --release
# Listens on http://localhost:3000
```

### STARK Wrapper (`stark-wrapper/`)

**Purpose:** Quantum-resistant on-chain verification

**Flow:**
1. Nova proof (fast, ~10KB, NOT quantum-safe)
2. SP1 STARK wrapper (~30s total, quantum-safe)
3. On-chain verification via SP1NovaVerifier.sol

**Components:**
- `program/` - SP1 guest (RISC-V zkVM)
- `host/` - SP1 orchestrator
- `contracts/SP1NovaVerifier.sol` - On-chain STARK verifier

### Web Dashboard (`dashboard/`)

**Features:**
- Browse open jobs
- Post new jobs (upload problem, set threshold/reward)
- View leaderboard (top provers)
- Responsive Tailwind UI

**Run:**
```bash
cd dashboard
npm install
npm run dev
# Open http://localhost:3000
```

### Integration Test (`integration-test/`)

**E2E validation:**
1. Generate Ising problem
2. Create prover with GPU commitments
3. Export for L1 submission
4. Export for STARK wrapper
5. Simulate on-chain verification

**Run:**
```bash
cd integration-test
cargo run --release
```

## Development Phases

| Phase | Status | Key Features |
|-------|--------|--------------|
| **Phase 0** | ✅ | Prover foundation |
| **Phase 1** | ✅ | Crypto hardening + Lt64 threshold |
| **Phase 2** | ✅ | GPU acceleration (9.5x) |
| **Phase 3** | ✅ | L1 contracts + tests |
| **Phase 4** | ✅ | Orchestration API |
| **Phase 5** | ✅ | STARK wrapper |
| **E2E** | ✅ | Integration tests |
| **UI** | ✅ | Web dashboard |

## Performance

| Metric | Value |
|--------|-------|
| **Spins** | 1.31M (10x scale) |
| **Edges** | 15.7M |
| **Nova Prove Time** | ~10s |
| **STARK Prove Time** | ~30s total |
| **Proof Size (Nova)** | 9.9 KB |
| **Proof Size (STARK)** | ~50 KB |
| **GPU Speedup** | 9.5x |
| **Tests Passing** | 100% (33/33 Foundry + E2E) |

## Security

### Cryptographic Hardening
- ✅ Poseidon commitment binding
- ✅ Fiat-Shamir spot-checks (4 per step)
- ✅ Binary spin constraints
- ✅ Threshold verification (E ≤ T)
- ✅ Gap-hiding ready

### Quantum Resistance
- ✅ STARK wrapper (hash-based, quantum-safe)
- ✅ Poseidon hashes (already quantum-resistant)
- ⚠️ Nova (Pallas/Vesta curves - NOT quantum-safe)

**Solution:** Use STARK wrapper for on-chain verification

## Repository Structure
```
ising-verifier-final/
├── nova-prover/          # Core proving system (Rust)
│   ├── src/
│   │   ├── lib.rs           # HardenedIsingProver
│   │   ├── comparators.rs   # Lt64Chip (68 constraints)
│   │   ├── l1_export.rs     # L1-compatible exports
│   │   └── stark_export.rs  # STARK wrapper exports
│   └── Cargo.toml
│
├── l1-contracts/         # Solidity contracts
│   ├── src/
│   │   ├── IsingJobManager.sol
│   │   └── NovaVerifier.sol
│   ├── test/
│   │   ├── IsingJobManager.t.sol
│   │   └── NovaVerifier.t.sol
│   └── script/Deploy.s.sol
│
├── orchestration/        # Job coordination (Rust)
│   └── src/
│       ├── job_index.rs     # Job discovery
│       ├── matcher.rs       # Prover assignment
│       ├── puub.rs          # PUUB scoring
│       └── api.rs           # REST endpoints
│
├── stark-wrapper/        # Quantum resistance (SP1)
│   ├── program/             # SP1 guest
│   ├── host/                # SP1 orchestrator
│   └── contracts/           # On-chain verifier
│
├── dashboard/            # Web UI (Next.js)
│   ├── app/
│   ├── components/
│   └── pages/api/
│
└── integration-test/     # E2E validation
    └── src/main.rs
```

## License

MIT

## Contact

- GitHub: https://github.com/tomdif/ising-verifier-final
- Email: tomdif@gmail.com

## Acknowledgments

- Nova proving system
- Succinct's SP1 for STARK wrapper
- Neptune for GPU-accelerated Poseidon hashing
