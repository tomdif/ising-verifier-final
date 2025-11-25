# Ising Prover Client

**Autonomous prover for the Nova Ising marketplace**

Connects to orchestrator, claims jobs, generates proofs, and submits to L1 for rewards.

## Quick Start
```bash
# Configure
export PROVER_PRIVATE_KEY="0xYOUR_KEY"
export SEPOLIA_RPC_URL="https://rpc.sepolia.org"
export JOB_MANAGER_ADDRESS="0xCONTRACT_ADDRESS"

# Run
cargo run --release -- \
  --orchestrator http://localhost:3000 \
  --gpu "NVIDIA A100" \
  --max-spins 10000000
```

## Options

| Flag | Description | Default |
|------|-------------|---------|
| `--orchestrator` | Orchestrator API URL | http://localhost:3000 |
| `--rpc-url` | Sepolia RPC URL | From env |
| `--private-key` | Prover private key | From env |
| `--contract-address` | JobManager address | From env |
| `--gpu` | GPU model name | NVIDIA A100 |
| `--max-spins` | Max problem size | 10000000 |
| `--poll-interval` | Poll interval (sec) | 10 |

## Features

- ✅ Auto-registration with orchestrator
- ✅ Continuous job polling
- ✅ Competitive job claiming
- ✅ GPU-accelerated proving
- ✅ Automatic L1 submission
- ✅ Retry logic and error handling

## Cloud Deployment

See [CLOUD_MINING.md](CLOUD_MINING.md) for detailed cloud setup guides.

