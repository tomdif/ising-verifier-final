//! SP1 Guest Program: Nova Proof Verifier
//!
//! This program runs inside SP1's RISC-V zkVM and verifies
//! the public inputs of a Nova Ising proof.

#![no_main]
sp1_zkvm::entrypoint!(main);

use serde::{Deserialize, Serialize};

/// Public inputs from the Nova proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NovaPublicInputs {
    /// Poseidon commitment to the problem
    pub problem_commitment: [u8; 32],
    /// Poseidon commitment to the spin configuration
    pub spin_commitment: [u8; 32],
    /// Claimed energy value (biased)
    pub energy: u64,
    /// Threshold value (biased)
    pub threshold: u64,
    /// Verified flag from Nova circuit
    pub verified: bool,
}

/// Output committed to the STARK proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedOutput {
    /// Problem commitment (passed through)
    pub problem_commitment: [u8; 32],
    /// Spin commitment (passed through)
    pub spin_commitment: [u8; 32],
    /// Unbiased energy value
    pub energy: i64,
    /// Unbiased threshold value
    pub threshold: i64,
    /// Final verification result
    pub valid: bool,
}

const BIAS: u64 = 1 << 50;

pub fn main() {
    // Read Nova public inputs from host
    let inputs: NovaPublicInputs = sp1_zkvm::io::read();
    
    // Verify the Nova circuit's verification flag
    assert!(inputs.verified, "Nova circuit verification failed");
    
    // Unbias the energy and threshold
    let energy = (inputs.energy as i64) - (BIAS as i64);
    let threshold = (inputs.threshold as i64) - (BIAS as i64);
    
    // Verify energy bound
    assert!(energy <= threshold, "Energy exceeds threshold");
    
    // Verify commitments are non-zero
    assert!(inputs.problem_commitment != [0u8; 32], "Invalid problem commitment");
    assert!(inputs.spin_commitment != [0u8; 32], "Invalid spin commitment");
    
    // Construct verified output
    let output = VerifiedOutput {
        problem_commitment: inputs.problem_commitment,
        spin_commitment: inputs.spin_commitment,
        energy,
        threshold,
        valid: true,
    };
    
    // Commit the output to the STARK proof
    sp1_zkvm::io::commit(&output);
}
