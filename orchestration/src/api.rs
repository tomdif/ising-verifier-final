//! API handlers for the orchestrator

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{AppState, Job, JobStatus, ClaimRequest, RegisterProverRequest, ProverStats};

/// Query parameters for listing jobs
#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    status: Option<String>,
    min_reward: Option<String>,
    max_spins: Option<u64>,
    limit: Option<usize>,
}

/// List jobs
pub async fn list_jobs(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(query): Query<ListJobsQuery>,
) -> Json<Vec<Job>> {
    let state = state.read().await;
    
    let status = query.status.and_then(|s| match s.as_str() {
        "open" => Some(JobStatus::Open),
        "claimed" => Some(JobStatus::Claimed),
        "solved" => Some(JobStatus::Solved),
        _ => None,
    });
    
    let min_reward = query.min_reward.and_then(|r| r.parse().ok());
    let limit = query.limit.unwrap_or(100);
    
    let jobs = state.job_index.list_jobs(status, min_reward, query.max_spins, limit);
    Json(jobs.into_iter().cloned().collect())
}

/// Get single job
pub async fn get_job(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<u64>,
) -> Result<Json<Job>, StatusCode> {
    let state = state.read().await;
    state.job_index.get_job(id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Claim job for proving
pub async fn claim_job(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<u64>,
    Json(req): Json<ClaimRequest>,
) -> Result<Json<Job>, StatusCode> {
    let mut state = state.write().await;
    
    // Check if prover can handle job
    let job = state.job_index.get_job(id).ok_or(StatusCode::NOT_FOUND)?;
    if !state.matcher.can_handle(&req.prover_address, job) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Try to claim
    if !state.job_index.claim_job(id, &req.prover_address) {
        return Err(StatusCode::CONFLICT);
    }
    
    state.matcher.assign_job(id, &req.prover_address);
    state.matcher.heartbeat(&req.prover_address);
    
    state.job_index.get_job(id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Register new prover
pub async fn register_prover(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(req): Json<RegisterProverRequest>,
) -> Json<crate::Prover> {
    let mut state = state.write().await;
    let prover = state.matcher.register_prover(req);
    Json(prover)
}

/// Get prover statistics
pub async fn get_prover_stats(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(addr): Path<String>,
) -> Json<ProverStats> {
    let state = state.read().await;
    let stats = state.puub.get_stats(&addr);
    Json(stats)
}

/// Get leaderboard
#[derive(Serialize)]
pub struct LeaderboardEntry {
    address: String,
    puub_score: u64,
    rank: usize,
}

pub async fn get_leaderboard(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<Vec<LeaderboardEntry>> {
    let state = state.read().await;
    let leaders = state.puub.leaderboard(100);
    let entries: Vec<_> = leaders.into_iter()
        .enumerate()
        .map(|(i, (addr, score))| LeaderboardEntry {
            address: addr,
            puub_score: score,
            rank: i + 1,
        })
        .collect();
    Json(entries)
}

/// Get PUUB score
#[derive(Serialize)]
pub struct PuubScoreResponse {
    address: String,
    score: u64,
}

pub async fn get_puub_score(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(addr): Path<String>,
) -> Json<PuubScoreResponse> {
    let state = state.read().await;
    let score = state.puub.get_score(&addr);
    Json(PuubScoreResponse { address: addr, score })
}

/// Get PUUB history
pub async fn get_puub_history(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(addr): Path<String>,
) -> Json<Vec<crate::PuubRecord>> {
    let state = state.read().await;
    let history = state.puub.get_history(&addr, 100);
    Json(history.into_iter().cloned().collect())
}
