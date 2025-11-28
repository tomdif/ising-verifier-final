use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod client;
mod config;
mod hasher;
mod ipfs;
mod worker;

#[derive(Parser)]
#[command(name = "nexus-miner")]
#[command(about = "NEXUS molecular docking miner")]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start,
    Status,
    Earnings,
    VerifyWasm,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("nexus_miner=info")
        .init();

    let cli = Cli::parse();
    let config = config::Config::load(&cli.config)?;

    match cli.command {
        Commands::Start => {
            info!("Starting NEXUS miner...");
            run_miner(config).await?;
        }
        Commands::Status => {
            let chain = client::ChainClient::new(&config.chain)?;
            let stats = chain.get_miner_stats().await?;
            println!("Period: {} | Shares: {}", stats.period_id, stats.shares);
        }
        Commands::Earnings => {
            let chain = client::ChainClient::new(&config.chain)?;
            let stats = chain.get_miner_stats().await?;
            println!("Estimated: {} NEX", stats.estimated_reward);
        }
        Commands::VerifyWasm => {
            let hash = hasher::hash_file(&config.wasm.vina_path)?;
            println!("SHA256: {}", hash);
            if hash == config.wasm.expected_hash {
                println!("VERIFIED");
            } else {
                println!("MISMATCH");
            }
        }
    }
    Ok(())
}

async fn run_miner(config: config::Config) -> Result<()> {
    let wasm_hash = hasher::hash_file(&config.wasm.vina_path)?;
    if wasm_hash != config.wasm.expected_hash {
        anyhow::bail!("WASM hash mismatch");
    }

    let chain = client::ChainClient::new(&config.chain)?;
    let ipfs = ipfs::IpfsClient::new(&config.ipfs.gateway)?;
    let worker = worker::WasmWorker::new(&config.wasm.vina_path)?;

    loop {
        let assignment = chain.request_work().await?;
        info!("Got work: job={} ligand={}", assignment.job_id, assignment.ligand_id);

        let work = chain.get_full_assignment(&assignment.assignment_id).await?;
        let ligand = fetch_ligand(assignment.ligand_id).await?;

        let result = worker.run_docking(
            &work.protein_pdbqt, &ligand,
            work.center_x, work.center_y, work.center_z,
            work.size_x, work.size_y, work.size_z,
            work.exhaustiveness, assignment.seed,
        )?;

        let hash = hasher::compute_result_hash(
            &assignment.job_id, assignment.ligand_id,
            assignment.seed, result.score, &result.pose_pdbqt,
        );

        let cid = ipfs.upload(&result.pose_pdbqt).await?;

        let resp = chain.submit_result(
            &assignment.job_id, assignment.ligand_id,
            &hash, &cid, result.score, result.bonds,
        ).await?;

        info!("Status: {} - {}", resp.status, resp.message);
        chain.heartbeat(&assignment.assignment_id).await?;
    }
}

async fn fetch_ligand(cid: i64) -> Result<String> {
    let url = format!("https://zinc.docking.org/substances/{}/formats/pdbqt", cid);
    Ok(reqwest::get(&url).await?.text().await?)
}
