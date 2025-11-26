//! Proof verification endpoint for NEXUS chain integration

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub job_id: String,
    pub problem_commitment: String,
    pub spin_commitment: String,
    pub claimed_energy: i64,
    pub threshold: i64,
    pub proof: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub energy: i64,
    pub meets_threshold: bool,
    pub error: Option<String>,
}

pub async fn verify_proof(
    State(_state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<VerifyRequest>,
) -> Json<VerifyResponse> {
    let problem_bytes = match hex::decode(&req.problem_commitment) {
        Ok(b) if b.len() == 32 => b,
        _ => return Json(VerifyResponse {
            valid: false, energy: 0, meets_threshold: false,
            error: Some("Invalid problem_commitment hex".into()),
        }),
    };

    let spin_bytes = match hex::decode(&req.spin_commitment) {
        Ok(b) if b.len() == 32 => b,
        _ => return Json(VerifyResponse {
            valid: false, energy: 0, meets_threshold: false,
            error: Some("Invalid spin_commitment hex".into()),
        }),
    };

    let proof_bytes = match hex::decode(&req.proof) {
        Ok(b) => b,
        Err(_) => return Json(VerifyResponse {
            valid: false, energy: 0, meets_threshold: false,
            error: Some("Invalid proof hex".into()),
        }),
    };

    // Structural validation (real verification would deserialize Nova proof)
    let valid = proof_bytes.len() >= 4
        && problem_bytes.len() == 32
        && spin_bytes.len() == 32;

    let meets_threshold = req.claimed_energy <= req.threshold;

    Json(VerifyResponse {
        valid,
        energy: req.claimed_energy,
        meets_threshold,
        error: if valid { None } else { Some("Proof too short".into()) },
    })
}
