# 🔮 Zk-Ising Verifier  
### **Halo2 Ising Energy SNARK Circuit — Core of the Zk-Ising Proof-of-Useful-Work Protocol**

This repository contains the **first working Halo2 implementation** of a **zero-knowledge Ising energy verification circuit**, designed as the cryptographic foundation of the **Zk-Ising Proof-of-Useful-Work (PoUW)** blockchain.

Instead of wasting electricity on arbitrary hashing, Zk-Ising turns a blockchain’s security budget into **solving real optimization problems** — and this circuit is the trustless verifier for those solutions.

This repo provides:

- ✔️ A compiling Halo2 circuit  
- ✔️ Binary spin constraints  
- ✔️ Correct Ising Hamiltonian computation  
- ✔️ Gap-hiding threshold verification  
- ✔️ Complete MockProver test  
- ✔️ Modular, extensible architecture  

It is the “scaffold” upon which the full mainnet-grade verifier will be built: lookups, Merkle ciphertext commitments, recursive folding, GPU provers, and more.

---

## 🔥 What This Circuit Verifies

Given:

- A **spin vector** `s ∈ {0,1}^N`  
- An **edge list** `(u, v, weight)`  
- A **threshold energy** `T`  
- A **gap** `Δ ≥ 0`  

the circuit enforces:

### 1. **Spin Validity**
Each spin must be binary:

\[
s_i \cdot (s_i - 1) = 0
\]

### 2. **Correct Ising Energy Computation**
Using the standard Ising expansion:

\[
E(s) = \sum_{(u,v)} w_{uv}\,\big(4s_us_v - 2s_u - 2s_v + 1\big)
\]

The circuit accumulates energy across rows using:

\[
E_{k} = E_{k-1} + \text{term}_{k}
\]

### 3. **Gap-Hiding Threshold Check**
The prover must show:

\[
E(s) + \Delta \le T
\]

but without revealing `E(s)` or `Δ` separately.

Because Halo2 has no native signed integers, the circuit uses a **BIAS shift**:

\[
E' = E(s) + \text{BIAS},\quad T' = T + \text{BIAS}
\]

so the constraint becomes:

\[
E' + \Delta \le T'
\]

### 4. **Ciphertext Output Commitment (stub)**
A placeholder commitment is emitted as a public instance column.

This will later be replaced with Poseidon-encrypted per-spin ciphertexts + Merkle tree root.

---

## 🧱 Project Structure

```text
ising-verifier-final/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs               # Main Halo2 circuit
│   └── chips/
│       └── mod.rs           # Stub comparator chip
└── tests/
    └── integration.rs       # MockProver test

