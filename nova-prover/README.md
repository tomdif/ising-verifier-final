# Nova Ising Prover

Zero-knowledge proof for Ising optimization using Nova folding.

## Performance
- 50K spins, degree 12: 13.9s
- Proof size: 9.8 KB (constant)
- Verify: ~30ms

## Build
```bash
cd nova-prover
cargo build --release
cargo run --release
```
