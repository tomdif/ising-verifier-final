# Cloud Mining Guide

## Run a Prover on Any Cloud

### Prerequisites
1. Cloud GPU instance (AWS, GCP, Azure, or quantum cloud)
2. Rust installed
3. Sepolia testnet ETH (~0.01 for gas)

### Supported Clouds

| Provider | GPU | Cost/hour | Setup Time |
|----------|-----|-----------|------------|
| AWS EC2 | g5.xlarge (A10G) | $1.006 | 5 min |
| GCP | a2-highgpu-1g (A100) | $3.67 | 5 min |
| Azure | NC6s v3 (V100) | $3.06 | 5 min |
| Lambda Labs | 1x A100 | $1.10 | 2 min |

### Quick Start
```bash
# 1. SSH into your cloud instance
ssh user@your-cloud-instance

# 2. Clone repo
git clone https://github.com/tomdif/ising-verifier-final.git
cd ising-verifier-final/prover-client

# 3. Configure
export PROVER_PRIVATE_KEY="0xYOUR_PRIVATE_KEY"
export SEPOLIA_RPC_URL="https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY"
export JOB_MANAGER_ADDRESS="0xYOUR_DEPLOYED_CONTRACT"

# 4. Run
cargo run --release -- \
  --orchestrator https://orchestrator.example.com \
  --gpu "NVIDIA A100" \
  --max-spins 10000000
```

### Docker Deployment
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin ising-prover-client

FROM nvidia/cuda:12.0-runtime
COPY --from=builder /app/target/release/ising-prover-client /usr/local/bin/
CMD ["ising-prover-client"]
```

### Economics

**Example Job:**
- Spins: 1M
- Reward: 0.01 ETH ($20)
- Prove time: 10s
- Gas cost: ~0.001 ETH ($2)
- **Net profit: $18**

**Hourly potential:** 360 jobs × $18 = **$6,480/hour**
(Assuming continuous job availability)

### Auto-scaling

The prover client automatically:
- ✅ Registers with orchestrator
- ✅ Polls for matching jobs
- ✅ Claims jobs competitively
- ✅ Generates proofs
- ✅ Submits to L1
- ✅ Tracks earnings

