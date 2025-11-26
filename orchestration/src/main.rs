//! Nova Ising Orchestrator

mod job_index;
mod matcher;
mod puub;
mod api;
mod types;
mod verify;

use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

pub use types::*;
pub use job_index::JobIndex;
pub use matcher::Matcher;
pub use puub::PuubTracker;

pub struct AppState {
    pub job_index: JobIndex,
    pub matcher: Matcher,
    pub puub: PuubTracker,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            job_index: JobIndex::new(),
            matcher: Matcher::new(),
            puub: PuubTracker::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(RwLock::new(AppState::new()));

    let app = Router::new()
        .route("/jobs", get(api::list_jobs))
        .route("/jobs/:id", get(api::get_job))
        .route("/jobs/:id/claim", axum::routing::post(api::claim_job))
        .route("/provers/register", axum::routing::post(api::register_prover))
        .route("/provers/:addr/stats", get(api::get_prover_stats))
        .route("/provers/leaderboard", get(api::get_leaderboard))
        .route("/puub/score/:addr", get(api::get_puub_score))
        .route("/puub/history/:addr", get(api::get_puub_history))
        .route("/verify", axum::routing::post(verify::verify_proof))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("Orchestrator listening on {}", addr);
    tracing::info!("NEXUS verify endpoint: POST /verify");
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
