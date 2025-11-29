use crate::config::Config;
use sha2::{Sha256, Digest};
use std::process::Command;

#[derive(Debug)]
pub struct DockingResult {
    pub affinity: String,
    pub pose_hash: String,
    pub admet_hash: String,
    pub result_hash: String,
}

pub fn verify_vina(config: &Config) -> Result<String, String> {
    let output = Command::new(&config.vina.binary_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to run Vina: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn run_docking(config: &Config, target: &str, ligand: &str, seed: u32) -> Result<DockingResult, String> {
    // This is a simplified version - full implementation would:
    // 1. Fetch receptor PDBQT from data dir or IPFS
    // 2. Convert ligand SMILES to PDBQT
    // 3. Run Vina with deterministic seed
    // 4. Parse output and compute hashes

    let receptor_path = format!("{}/receptors/{}.pdbqt", config.mining.data_dir, target);
    
    // Check if receptor exists
    if !std::path::Path::new(&receptor_path).exists() {
        return Err(format!("Receptor not found: {}", receptor_path));
    }

    // For demo, return mock result with deterministic hashes
    let affinity = "-5.23".to_string();
    
    // Compute deterministic hashes
    let pose_data = format!("{}|{}|{}|pose", target, ligand, seed);
    let pose_hash = hex::encode(Sha256::digest(pose_data.as_bytes()));
    
    let admet_data = format!("{}|admet", ligand);
    let admet_hash = hex::encode(Sha256::digest(admet_data.as_bytes()));
    
    let result_data = format!("{}|{}|{}|{}", affinity, pose_hash, admet_hash, seed);
    let result_hash = hex::encode(Sha256::digest(result_data.as_bytes()));

    Ok(DockingResult {
        affinity,
        pose_hash,
        admet_hash,
        result_hash,
    })
}
