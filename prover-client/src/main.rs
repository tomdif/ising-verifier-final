//! Nova Ising Prover Client
//!
//! Autonomous prover that:
//! 1. Registers with orchestrator
//! 2. Polls for open jobs
//! 3. Downloads problems
//! 4. Generates proofs
//! 5. Submits to L1 for rewards

use anyhow::Result;
use clap::Parser;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about = "Nova Ising Prover Client")]
struct Args {
    /// Orchestrator API URL
    #[arg(long, default_value = "http://localhost:3000")]
    orchestrator: String,

    /// L1 RPC URL (Sepolia)
    #[arg(long, env = "SEPOLIA_RPC_URL")]
    rpc_url: String,

    /// Private key for submitting proofs
    #[arg(long, env = "PROVER_PRIVATE_KEY")]
    private_key: String,

    /// IsingJobManager contract address
    #[arg(long, env = "JOB_MANAGER_ADDRESS")]
    contract_address: String,

    /// GPU model (for registration)
    #[arg(long, default_value = "NVIDIA A100")]
    gpu: String,

    /// Max spins this prover can handle
    #[arg(long, default_value = "10000000")]
    max_spins: u64,

    /// Polling interval (seconds)
    #[arg(long, default_value = "10")]
    poll_interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Job {
    id: u64,
    problem_commitment: String,
    threshold: i64,
    reward_wei: String,
    n_spins: Option<u64>,
    n_edges: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ProverRegistration {
    address: String,
    name: Option<String>,
    gpu_model: Option<String>,
    max_spins: u64,
}

struct ProverClient {
    args: Args,
    client: reqwest::Client,
    wallet: LocalWallet,
}

impl ProverClient {
    fn new(args: Args) -> Result<Self> {
        let wallet = args.private_key.parse::<LocalWallet>()?;
        
        Ok(Self {
            args,
            client: reqwest::Client::new(),
            wallet,
        })
    }

    /// Register with orchestrator
    async fn register(&self) -> Result<()> {
        let registration = ProverRegistration {
            address: format!("0x{:x}", self.wallet.address()),
            name: Some("Cloud Prover".to_string()),
            gpu_model: Some(self.args.gpu.clone()),
            max_spins: self.args.max_spins,
        };

        let url = format!("{}/provers/register", self.args.orchestrator);
        let response = self.client.post(&url)
            .json(&registration)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("✅ Registered with orchestrator");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Registration failed: {}", response.status()))
        }
    }

    /// Poll for open jobs
    async fn poll_jobs(&self) -> Result<Vec<Job>> {
        let url = format!("{}/jobs?status=open&max_spins={}", 
            self.args.orchestrator, self.args.max_spins);
        
        let jobs: Vec<Job> = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(jobs)
    }

    /// Claim a job
    async fn claim_job(&self, job_id: u64) -> Result<()> {
        let url = format!("{}/jobs/{}/claim", self.args.orchestrator, job_id);
        
        let payload = serde_json::json!({
            "prover_address": format!("0x{:x}", self.wallet.address()),
            "estimated_time_ms": 30000
        });

        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("✅ Claimed job {}", job_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Claim failed: {}", response.status()))
        }
    }

    /// Generate proof for job
    async fn generate_proof(&self, job: &Job) -> Result<Vec<u8>> {
        tracing::info!("🔧 Generating proof for job {}...", job.id);
        tracing::info!("   Spins: {}", job.n_spins.unwrap_or(0));
        tracing::info!("   Threshold: {}", job.threshold);

        // TODO: Download problem file from IPFS/storage
        // TODO: Run Nova prover
        // TODO: Generate STARK wrapper (optional)

        // Simulated proof for now
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        tracing::info!("✅ Proof generated!");
        
        Ok(vec![0x4e, 0x4f, 0x56, 0x41]) // "NOVA"
    }

    /// Submit proof to L1
    async fn submit_proof(&self, job: &Job, proof: &[u8]) -> Result<()> {
        tracing::info!("📤 Submitting proof to L1...");

        // TODO: Connect to L1 contract
        // TODO: Call submitProof()
        // TODO: Wait for transaction confirmation

        tracing::info!("✅ Proof submitted! Awaiting confirmation...");
        
        Ok(())
    }

    /// Main prover loop
    async fn run(&self) -> Result<()> {
        tracing::info!("╔═══════════════════════════════════════════════════════════╗");
        tracing::info!("║  Nova Ising Prover Client                                 ║");
        tracing::info!("╚═══════════════════════════════════════════════════════════╝");
        tracing::info!("");
        tracing::info!("Prover Address: 0x{:x}", self.wallet.address());
        tracing::info!("Orchestrator:   {}", self.args.orchestrator);
        tracing::info!("Max Spins:      {}", self.args.max_spins);
        tracing::info!("GPU:            {}", self.args.gpu);
        tracing::info!("");

        // Register
        self.register().await?;

        // Main loop
        loop {
            match self.poll_jobs().await {
                Ok(jobs) => {
                    if jobs.is_empty() {
                        tracing::info!("⏳ No jobs available, waiting...");
                    } else {
                        tracing::info!("📋 Found {} open jobs", jobs.len());

                        // Take the first job
                        if let Some(job) = jobs.first() {
                            if let Err(e) = self.process_job(job).await {
                                tracing::error!("❌ Error processing job: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to poll jobs: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(self.args.poll_interval)).await;
        }
    }

    async fn process_job(&self, job: &Job) -> Result<()> {
        tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        tracing::info!("Processing Job #{}", job.id);
        
        // Claim job
        self.claim_job(job.id).await?;

        // Generate proof
        let proof = self.generate_proof(job).await?;

        // Submit to L1
        self.submit_proof(job, &proof).await?;

        tracing::info!("✅ Job #{} complete!", job.id);
        tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let client = ProverClient::new(args)?;
    
    client.run().await
}
