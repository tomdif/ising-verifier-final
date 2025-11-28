# WASM Vina Determinism for NEXUS

## Overview

NEXUS uses AutoDock Vina compiled to WebAssembly (via Webina) to achieve
deterministic molecular docking.

## The Determinism Problem

Traditional molecular docking has sources of non-determinism:
- Random number generation
- Floating-point variations across CPUs
- Threading race conditions
- Memory allocation order

## NEXUS Solution

### 1. WebAssembly Guarantees

WASM provides hardware-independent execution:
- IEEE 754 floats: Strictly defined behavior
- Deterministic semantics: Same bytecode = same execution
- No undefined behavior

### 2. Fixed WASM Binary

All miners run the exact same vina.wasm file:
- SHA256 hash verified before execution
- Network consensus on official binary

### 3. Deterministic Seeding

Random seed derived from job parameters:
    seed = SHA256(job_id + ligand_id)[0:8]

All miners get the same seed for the same work.

### 4. Single-Threaded Mode

Threading disabled in wasmtime:
    config.wasm_threads(false)

Eliminates race conditions.

### 5. NaN Canonicalization

Floating-point edge cases handled consistently:
    config.cranelift_nan_canonicalization(true)

## Verification Flow

    Miner A: inputs -> WASM Vina -> hash_A
    Miner B: inputs -> WASM Vina -> hash_B

    IF hash_A == hash_B: VERIFIED (both earn rewards)

## Result Hash

    hash = SHA256(job_id + ligand_id + seed + score + pose_pdbqt)

Any difference in execution produces different hash.

## Webina Source

From Durrant Lab: github.com/durrantlab/webina
- Vina 1.2.3 compiled to WASM via Emscripten
- Open source (Apache 2.0)
- Proven identical results with same seed

## Performance

WASM is slower than native:
- Native Vina: 10-30 seconds per ligand
- WASM Vina: 30-90 seconds per ligand

Acceptable because correctness > speed for verification.
