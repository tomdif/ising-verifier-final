# NEXUS Mining Module

Decentralized molecular docking for drug discovery, powered by verified computation.

## Overview

NEXUS miners perform molecular docking using AutoDock Vina compiled to WebAssembly.
Results are verified through hash matching - when two independent miners produce the
same result hash, the result is confirmed and both miners earn rewards.

## Key Design

1. WASM-Based Deterministic Execution
2. Hash-Matching Verification (no ZK proofs needed)
3. No Slashing - wrong results just get no reward
4. Network-Assigned Work - prevents cherry-picking
5. Shares Credited on Verification

## Structure

nexus-mining/
  go.mod
  proto/nexus/mining/v1/   - Protobuf definitions
  x/mining/keeper/         - State management
  x/mining/types/          - Type definitions
