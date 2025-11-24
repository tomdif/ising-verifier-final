//! STARK Export Module
//!
//! Exports Nova public inputs for STARK wrapper.

use crate::{HardenedIsingProver, BIAS};
use ff::PrimeField;
use serde::{Deserialize, Serialize};

/// Public inputs for STARK wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NovaPublicInputs {
    pub problem_commitment: [u8; 32],
    pub spin_commitment: [u8; 32],
    pub energy: u64,
    pub threshold: u64,
    pub verified: bool,
}

impl HardenedIsingProver {
    /// Export public inputs for STARK wrapper
    pub fn export_stark_inputs(&self) -> NovaPublicInputs {
        let problem_repr = self.problem_commitment.to_repr();
        let spin_repr = self.spin_commitment.to_repr();
        
        // Bias the energy and threshold
        let biased_energy = (self.total_energy() + BIAS as i64) as u64;
        let biased_threshold = (self.threshold + BIAS as i64) as u64;
        
        NovaPublicInputs {
            problem_commitment: problem_repr.as_ref()[..32].try_into().unwrap(),
            spin_commitment: spin_repr.as_ref()[..32].try_into().unwrap(),
            energy: biased_energy,
            threshold: biased_threshold,
            verified: biased_energy <= biased_threshold,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bias_roundtrip() {
        let energy: i64 = -50000;
        let biased = (energy + BIAS as i64) as u64;
        let unbiased = (biased as i64) - (BIAS as i64);
        assert_eq!(energy, unbiased);
    }
}
