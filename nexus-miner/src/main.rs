//! NEXUS Miner - Native Vina Edition
//! 
//! Mines NEXUS tokens by performing molecular docking computations
//! using native AutoDock Vina 1.2.5 (deterministic).

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};

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
    /// Start mining
    Start,
    /// Show miner status
    Status,
    /// Show earnings
    Earnings,
    /// Verify Vina binary
    VerifyVina,
    /// Test docking locally
    TestDock {
        #[arg(long)]
        receptor: String,
        #[arg(long)]
        ligand: String,
    },
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
        Commands::VerifyVina => {
            verify_vina(&config)?;
        }
        Commands::TestDock { receptor, ligand } => {
            test_dock(&config, &receptor, &ligand)?;
        }
    }
    Ok(())
}

fn verify_vina(config: &config::Config) -> Result<()> {
    let hash = hasher::hash_file(&config.vina.path)?;
    println!("Vina binary: {}", config.vina.path);
    println!("SHA256: {}", hash);
    
    if hash == config.vina.expected_hash {
        println!("✅ VERIFIED - matches expected hash");
        Ok(())
    } else {
        println!("❌ MISMATCH");
        println!("Expected: {}", config.vina.expected_hash);
        anyhow::bail!("Vina binary hash mismatch")
    }
}

fn test_dock(config: &config::Config, receptor: &str, ligand: &str) -> Result<()> {
    println!("Testing local docking...");
    
    let dock_config = worker::DockingConfig {
        vina_path: config.vina.path.clone(),
        center: (config.vina.center_x, config.vina.center_y, config.vina.center_z),
        size: (config.vina.size_x, config.vina.size_y, config.vina.size_z),
        exhaustiveness: config.vina.exhaustiveness,
        cpu: 1,
    };
    
    let result = worker::execute_docking(
        &dock_config,
        receptor,
        ligand,
        "test-job",
        "test-ligand",
    )?;
    
    println!("✅ Docking complete!");
    println!("  Affinity: {:.3} kcal/mol", result.affinity);
    println!("  Seed: {}", result.seed);
    println!("  Bonds: {}", result.num_bonds);
    println!("  Hash: {}", result.result_hash);
    
    Ok(())
}

async fn run_miner(config: config::Config) -> Result<()> {
    // Verify Vina binary
    let vina_hash = hasher::hash_file(&config.vina.path)?;
    if vina_hash != config.vina.expected_hash {
        anyhow::bail!("Vina binary hash mismatch! Expected: {}", config.vina.expected_hash);
    }
    info!("Vina binary verified");

    let chain = client::ChainClient::new(&config.chain)?;
    let ipfs = ipfs::IpfsClient::new(&config.ipfs.gateway)?;
    
    let dock_config = worker::DockingConfig {
        vina_path: config.vina.path.clone(),
        center: (config.vina.center_x, config.vina.center_y, config.vina.center_z),
        size: (config.vina.size_x, config.vina.size_y, config.vina.size_z),
        exhaustiveness: config.vina.exhaustiveness,
        cpu: 1, // Fixed for cross-machine determinism
    };

    info!("Requesting work from chain...");
    
    loop {
        // Request work assignment
        let assignment = match chain.request_work().await {
            Ok(a) => a,
            Err(e) => {
                warn!("No work available: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue;
            }
        };
        
        info!("Got assignment: job={} ligand={}", 
            assignment.job_id, assignment.ligand_id);

        // Fetch full job data
        let job = chain.get_job(&assignment.job_id).await?;
        
        // Download ligand from IPFS
        let ligand_pdbqt = ipfs.download(&assignment.ligand_cid).await?;
        
        // Save to temp files
        let receptor_path = format!("/tmp/receptor_{}.pdbqt", assignment.job_id);
        let ligand_path = format!("/tmp/ligand_{}.pdbqt", assignment.ligand_id);
        std::fs::write(&receptor_path, &job.receptor_pdbqt)?;
        std::fs::write(&ligand_path, &ligand_pdbqt)?;
        
        // Run docking
        let result = worker::execute_docking(
            &dock_config,
            &receptor_path,
            &ligand_path,
            &assignment.job_id,
            &assignment.ligand_id,
        )?;
        
        info!("Docking complete: affinity={:.3} hash={}", 
            result.affinity, &result.result_hash[..16]);
        
        // Upload pose to IPFS
        let pose_cid = ipfs.upload(&result.pose_pdbqt).await?;
        
        // Submit result to chain
        chain.submit_result(
            &assignment.job_id,
            &assignment.ligand_id,
            &result.result_hash,
            &pose_cid,
            result.affinity,
            result.num_bonds,
        ).await?;
        
        info!("Result submitted successfully");
        
        // Cleanup
        let _ = std::fs::remove_file(&receptor_path);
        let _ = std::fs::remove_file(&ligand_path);
    }
}
