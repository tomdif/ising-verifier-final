//! L1 Export Module - exports proofs for IsingJobManager contract

use crate::HardenedIsingProver;
use ff::PrimeField;

/// L1-compatible proof submission
pub struct L1ProofSubmission {
    pub job_id: u64,
    pub spin_commitment: [u8; 32],
    pub claimed_energy: i64,
    pub proof: Vec<u8>,
}

/// L1-compatible job posting
pub struct L1JobPosting {
    pub problem_commitment: [u8; 32],
    pub threshold: i64,
    pub n_spins: usize,
    pub n_edges: usize,
}

impl HardenedIsingProver {
    pub fn export_job_posting(&self) -> L1JobPosting {
        let repr = self.problem_commitment.to_repr();
        let bytes: [u8; 32] = repr.as_ref()[..32].try_into().unwrap();
        L1JobPosting {
            problem_commitment: bytes,
            threshold: self.threshold,
            n_spins: self.n_spins,
            n_edges: self.edges.len(),
        }
    }
    
    pub fn export_proof_submission(&self, job_id: u64, proof: &[u8]) -> L1ProofSubmission {
        let repr = self.spin_commitment.to_repr();
        let bytes: [u8; 32] = repr.as_ref()[..32].try_into().unwrap();
        L1ProofSubmission {
            job_id,
            spin_commitment: bytes,
            claimed_energy: self.total_energy(),
            proof: proof.to_vec(),
        }
    }
}

impl L1JobPosting {
    pub fn to_hex(&self) -> String {
        format!("problem: 0x{}, threshold: {}", hex::encode(&self.problem_commitment), self.threshold)
    }
}

impl L1ProofSubmission {
    pub fn to_hex(&self) -> String {
        format!("job: {}, spin: 0x{}, energy: {}, proof: {} bytes",
            self.job_id, hex::encode(&self.spin_commitment), 
            self.claimed_energy, self.proof.len())
    }
}
