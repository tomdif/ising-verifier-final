# NEXUS Miner

Miner client for NEXUS molecular docking network.

## Overview

NEXUS miners perform molecular docking using AutoDock Vina compiled to
WebAssembly. The WASM binary guarantees deterministic execution - given
identical inputs, every miner produces identical outputs.

## How It Works

1. Request Work - Miner asks the network for a work assignment
2. Download Inputs - Fetch protein PDBQT, ligand, docking box
3. Run Docking - Execute WASM Vina with deterministic seed
4. Compute Hash - SHA256(jobId + ligandId + seed + score + poseData)
5. Submit Result - Send hash, score, bonds, and IPFS CID of pose
6. Verification - When another miner submits matching hash, both earn rewards

## WASM Determinism

We use Webina (github.com/durrantlab/webina), which compiles AutoDock Vina
to WebAssembly. Key determinism factors:

- Fixed Binary: All miners use exact same .wasm file (SHA256 verified)
- Fixed Seed: Network provides seed = SHA256(jobId + ligandId)
- No Threading: Single-threaded mode eliminates race conditions
- IEEE 754 Floats: WASM guarantees consistent floating-point behavior

## Installation

    git clone https://github.com/nexus-chain/nexus-miner
    cd nexus-miner
    ./scripts/download-wasm.sh
    cargo build --release
    ./target/release/nexus-miner --config config.toml start

## Bond Multipliers

Results with more hydrogen bonds earn more shares:

    Bonds 0: 0.10x    Bonds 5: 2.80x
    Bonds 1: 0.45x    Bonds 6: 3.61x
    Bonds 2: 0.90x    Bonds 7: 4.52x
    Bonds 3: 1.44x    Bonds 8: 5.52x
    Bonds 4: 2.07x    Bonds 9+: exponential

## License

Apache-2.0
