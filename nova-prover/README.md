# Nova Ising Prover

Zero-knowledge proof system for Ising model optimization using [Nova folding](https://github.com/microsoft/Nova).

## 🎯 Whitepaper Targets - ACHIEVED

| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| Variables | 131,072 | 131,072 | ✅ |
| Graph degree | ≤12 | 12 | ✅ |
| Prove time | ≤10 seconds | **0.95 seconds** | ✅ 10x better |
| Proof size | ≤280 KB | **9.8 KB** | ✅ 28x better |

## Performance
```
131,072 spins, degree 12, 786,432 edges:
  Steps: 8
  Recursive prove: 0.39s
  Compression: 0.57s
  TOTAL: 0.95s
  Proof size: 9.8 KB
  Verify: 23ms
```

## Architecture

Nova IVC (Incremental Verifiable Computation) with Spartan compression:

1. **Folding**: Each step processes 100K edges, accumulating energy
2. **Compression**: Final IVC proof compressed to ~10KB SNARK
3. **Verification**: Constant-time (~23ms) regardless of problem size

### Why Nova beats Halo2 for this use case

| Approach | 131K vars | Scaling |
|----------|-----------|---------|
| Halo2 (monolithic) | ~1 hour | O(n log n) FFTs |
| **Nova (folding)** | **0.95s** | **O(n) linear** |

## Usage
```rust
use ising_nova::{IsingNovaProver, EDGES_PER_STEP};

let edges = vec![(0, 1, 1), (1, 2, -1), ...];
let spins = vec![0, 1, 1, 0, ...];

let prover = IsingNovaProver::new(edges, spins, delta, threshold);
let circuits = prover.step_circuits();
// ... run Nova IVC
```

## Build & Run
```bash
cargo build --release
cargo run --release
```

## Dependencies

- `nova-snark` - Nova proof system
- `bellpepper` - Circuit building
- `rayon` - Parallel computation

## License

MIT
