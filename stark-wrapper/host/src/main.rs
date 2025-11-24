//! SP1 Host: Nova STARK Wrapper
//!
//! Orchestrates STARK proof generation from Nova public inputs.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

/// ELF binary of the guest program
const NOVA_VERIFIER_ELF: &[u8] = include_elf!("nova-verifier-program");

/// Public inputs from Nova proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NovaPublicInputs {
    pub problem_commitment: [u8; 32],
    pub spin_commitment: [u8; 32],
    pub energy: u64,
    pub threshold: u64,
    pub verified: bool,
}

/// Verified output from STARK
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedOutput {
    pub problem_commitment: [u8; 32],
    pub spin_commitment: [u8; 32],
    pub energy: i64,
    pub threshold: i64,
    pub valid: bool,
}

/// STARK proof wrapper result
pub struct StarkProofBundle {
    pub proof: Vec<u8>,
    pub public_values: VerifiedOutput,
    pub vkey_hash: [u8; 32],
}

/// Generate STARK proof wrapping Nova public inputs
pub fn generate_stark_proof(inputs: NovaPublicInputs) -> Result<StarkProofBundle> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  STARK Wrapper - Generating Quantum-Resistant Proof");
    println!("═══════════════════════════════════════════════════════════════════");
    
    // Initialize SP1 prover
    let client = ProverClient::from_env();
    
    // Setup stdin with Nova inputs
    let mut stdin = SP1Stdin::new();
    stdin.write(&inputs);
    
    println!("  [1/3] Setting up SP1 prover...");
    let (pk, vk) = client.setup(NOVA_VERIFIER_ELF);
    
    println!("  [2/3] Generating STARK proof...");
    let proof = client.prove(&pk, &stdin).compressed().run()?;
    
    println!("  [3/3] Verifying STARK proof...");
    client.verify(&proof, &vk)?;
    
    // Extract public values
    let public_values: VerifiedOutput = proof.public_values.read();
    
    // Serialize proof
    let proof_bytes = bincode::serialize(&proof)?;
    
    // Get verification key hash
    let vkey_hash = vk.bytes32();
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  STARK Proof Generated Successfully");
    println!("  Proof size: {} KB", proof_bytes.len() / 1024);
    println!("  VKey hash: 0x{}", hex::encode(&vkey_hash[..8]));
    println!("═══════════════════════════════════════════════════════════════════");
    
    Ok(StarkProofBundle {
        proof: proof_bytes,
        public_values,
        vkey_hash: vkey_hash.try_into().unwrap_or([0u8; 32]),
    })
}

fn main() -> Result<()> {
    // Example: Generate STARK proof for sample Nova inputs
    let inputs = NovaPublicInputs {
        problem_commitment: [0xab; 32],
        spin_commitment: [0xcd; 32],
        energy: (1 << 50) - 50000,  // -50000 biased
        threshold: (1 << 50) - 40000, // -40000 biased (energy < threshold)
        verified: true,
    };
    
    println!("Nova Public Inputs:");
    println!("  Problem: 0x{}", hex::encode(&inputs.problem_commitment[..8]));
    println!("  Spins:   0x{}", hex::encode(&inputs.spin_commitment[..8]));
    println!("  Energy:  {} (biased)", inputs.energy);
    println!("  Threshold: {} (biased)", inputs.threshold);
    println!("  Verified: {}", inputs.verified);
    println!();
    
    let bundle = generate_stark_proof(inputs)?;
    
    println!();
    println!("Verified Output:");
    println!("  Energy:    {}", bundle.public_values.energy);
    println!("  Threshold: {}", bundle.public_values.threshold);
    println!("  Valid:     {}", bundle.public_values.valid);
    
    Ok(())
}
