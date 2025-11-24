//! Common types for the orchestration system

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Job status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Open,
    Claimed,
    Proving,
    Solved,
    Expired,
}

/// Ising job from L1
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub problem_commitment: String,  // hex
    pub threshold: i64,
    pub reward_wei: String,
    pub deadline: DateTime<Utc>,
    pub poster: String,              // address
    pub status: JobStatus,
    pub n_spins: Option<u64>,        // metadata
    pub n_edges: Option<u64>,        // metadata
    pub claimed_by: Option<String>,  // prover address
    pub created_at: DateTime<Utc>,
}

/// Registered prover
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prover {
    pub address: String,
    pub name: Option<String>,
    pub gpu_model: Option<String>,
    pub max_spins: u64,
    pub registered_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub active: bool,
}

/// PUUB (Proof of Useful Useful Work) record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PuubRecord {
    pub prover: String,
    pub job_id: u64,
    pub energy_achieved: i64,
    pub threshold: i64,
    pub proof_time_ms: u64,
    pub n_spins: u64,
    pub n_edges: u64,
    pub timestamp: DateTime<Utc>,
    pub reward_wei: String,
}

/// Prover statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProverStats {
    pub address: String,
    pub total_proofs: u64,
    pub total_spins_proved: u64,
    pub total_edges_proved: u64,
    pub total_rewards_wei: String,
    pub avg_proof_time_ms: u64,
    pub success_rate: f64,
    pub puub_score: u64,
}

/// Job claim request
#[derive(Clone, Debug, Deserialize)]
pub struct ClaimRequest {
    pub prover_address: String,
    pub estimated_time_ms: Option<u64>,
}

/// Prover registration request
#[derive(Clone, Debug, Deserialize)]
pub struct RegisterProverRequest {
    pub address: String,
    pub name: Option<String>,
    pub gpu_model: Option<String>,
    pub max_spins: u64,
}
