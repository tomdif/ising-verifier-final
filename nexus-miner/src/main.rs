mod chain;
mod config;
mod worker;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nexus-miner")]
#[command(about = "NEXUS Network Miner - Proof of Useful Work")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Start mining
    Start {
        /// Miner address
        #[arg(long)]
        address: String,
    },

    /// Register as a miner
    Register {
        /// Miner address
        #[arg(long)]
        address: String,
    },

    /// Check miner status
    Status {
        /// Miner address
        #[arg(long)]
        address: String,
    },

    /// Run a test docking job
    TestDock {
        /// Target protein PDB ID
        #[arg(long, default_value = "6LU7")]
        target: String,

        /// Ligand SMILES
        #[arg(long, default_value = "CCO")]
        ligand: String,
    },

    /// Verify Vina installation
    VerifyVina,
}

fn main() {
    let cli = Cli::parse();

    // Load config
    let config = config::load_config(&cli.config).unwrap_or_else(|e| {
        eprintln!("Warning: Could not load config: {}", e);
        config::Config::default()
    });

    let chain_client = chain::ChainClient::new(
        &config.chain.node_url,
        &config.chain.home_dir,
    );

    match cli.command {
        Commands::Start { address } => {
            println!("Starting NEXUS miner...");
            println!("  Address: {}", address);
            println!("  Node: {}", config.chain.node_url);
            println!("");
            
            // Check miner is registered
            match chain_client.get_miner_status(&address) {
                Ok(status) => println!("{}", status),
                Err(e) => {
                    eprintln!("Error checking miner status: {}", e);
                    eprintln!("Register first with: nexus-miner register --address {}", address);
                    return;
                }
            }

            println!("\nWaiting for checkpoints...");
            println!("[Press Ctrl+C to stop]");
            
            // Main mining loop
            loop {
                match chain_client.get_pending_checkpoint() {
                    Ok(Some(checkpoint)) => {
                        println!("\nCheckpoint {} received!", checkpoint.height);
                        println!("  Jobs: {}", checkpoint.docking_jobs.len());
                        
                        for job in &checkpoint.docking_jobs {
                            println!("\nProcessing job: {}", job.job_id);
                            
                            // Run docking
                            match worker::run_docking(&config, &job.target_id, &job.ligand_id, job.seed) {
                                Ok(result) => {
                                    println!("  Affinity: {} kcal/mol", result.affinity);
                                    println!("  Result hash: {}", result.result_hash);
                                    
                                    // Submit approval
                                    let approval = chain::MinerApproval {
                                        miner_address: address.clone(),
                                        checkpoint_hash: checkpoint.block_hash.clone(),
                                        job_id: job.job_id.clone(),
                                        affinity: result.affinity.clone(),
                                        pose_hash: result.pose_hash.clone(),
                                        admet_hash: result.admet_hash.clone(),
                                        result_hash: result.result_hash.clone(),
                                        signature: "sig_placeholder".to_string(),
                                    };
                                    
                                    match chain_client.submit_approval(&approval) {
                                        Ok(tx) => println!("  Submitted: {}", tx),
                                        Err(e) => eprintln!("  Error submitting: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("  Docking error: {}", e),
                            }
                        }
                    }
                    Ok(None) => {
                        // No pending checkpoint, wait
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    }
                    Err(e) => {
                        eprintln!("Error fetching checkpoint: {}", e);
                        std::thread::sleep(std::time::Duration::from_secs(5));
                    }
                }
            }
        }

        Commands::Register { address } => {
            println!("Registering miner: {}", address);
            match chain_client.register_miner(&address) {
                Ok(result) => println!("{}", result),
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::Status { address } => {
            match chain_client.get_miner_status(&address) {
                Ok(status) => println!("{}", status),
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::TestDock { target, ligand } => {
            println!("Running test docking...");
            println!("  Target: {}", target);
            println!("  Ligand: {}", ligand);
            
            match worker::run_docking(&config, &target, &ligand, 12345) {
                Ok(result) => {
                    println!("\nResult:");
                    println!("  Affinity: {} kcal/mol", result.affinity);
                    println!("  Pose hash: {}", result.pose_hash);
                    println!("  ADMET hash: {}", result.admet_hash);
                    println!("  Result hash: {}", result.result_hash);
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Commands::VerifyVina => {
            println!("Verifying Vina installation...");
            match worker::verify_vina(&config) {
                Ok(version) => {
                    println!("✅ Vina found: {}", version);
                    println!("✅ Ready to mine!");
                }
                Err(e) => {
                    eprintln!("❌ Vina not found: {}", e);
                    eprintln!("\nDownload from: https://github.com/ccsb-scripps/AutoDock-Vina/releases");
                }
            }
        }
    }
}
