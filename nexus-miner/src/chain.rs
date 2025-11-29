//! Chain client for communicating with nexusd

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ChainClient {
    pub node_url: String,
    pub home_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub height: i64,
    pub block_hash: String,
    pub validator_set_hash: String,
    pub status: String,
    pub docking_jobs: Vec<DockingJob>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockingJob {
    pub job_id: String,
    pub target_id: String,
    pub ligand_id: String,
    pub ligand_hash: String,
    pub seed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinerApproval {
    pub miner_address: String,
    pub checkpoint_hash: String,
    pub job_id: String,
    pub affinity: String,
    pub pose_hash: String,
    pub admet_hash: String,
    pub result_hash: String,
    pub signature: String,
}

impl ChainClient {
    pub fn new(node_url: &str, home_dir: &str) -> Self {
        Self {
            node_url: node_url.to_string(),
            home_dir: home_dir.to_string(),
        }
    }

    /// Register as a miner on the chain
    pub fn register_miner(&self, address: &str) -> Result<String, String> {
        let output = Command::new("nexusd")
            .args(["miner", "register", address])
            .args(["--home", &self.home_dir])
            .output()
            .map_err(|e| format!("Failed to execute nexusd: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Get the current pending checkpoint
    pub fn get_pending_checkpoint(&self) -> Result<Option<Checkpoint>, String> {
        // In production, this would query the chain via RPC
        // For now, return a mock checkpoint for testing
        Ok(Some(Checkpoint {
            height: 200,
            block_hash: "abc123".to_string(),
            validator_set_hash: "def456".to_string(),
            status: "pending".to_string(),
            docking_jobs: vec![
                DockingJob {
                    job_id: "job_001".to_string(),
                    target_id: "6LU7".to_string(),
                    ligand_id: "test_ligand".to_string(),
                    ligand_hash: "hash123".to_string(),
                    seed: 12345,
                },
            ],
        }))
    }

    /// Submit a miner approval for a checkpoint
    pub fn submit_approval(&self, approval: &MinerApproval) -> Result<String, String> {
        // In production, this would submit a transaction
        println!("Submitting approval to chain:");
        println!("  Checkpoint: {}", approval.checkpoint_hash);
        println!("  Job: {}", approval.job_id);
        println!("  Affinity: {}", approval.affinity);
        println!("  Result Hash: {}", approval.result_hash);
        
        Ok("tx_hash_placeholder".to_string())
    }

    /// Get miner status
    pub fn get_miner_status(&self, address: &str) -> Result<String, String> {
        let output = Command::new("nexusd")
            .args(["miner", "status", address])
            .args(["--home", &self.home_dir])
            .output()
            .map_err(|e| format!("Failed to execute nexusd: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get latest checkpoint
    pub fn get_latest_checkpoint(&self) -> Result<String, String> {
        let output = Command::new("nexusd")
            .args(["checkpoint", "latest"])
            .args(["--home", &self.home_dir])
            .output()
            .map_err(|e| format!("Failed to execute nexusd: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_client_new() {
        let client = ChainClient::new("http://localhost:26657", "~/.nexusd");
        assert_eq!(client.node_url, "http://localhost:26657");
    }

    #[test]
    fn test_get_pending_checkpoint() {
        let client = ChainClient::new("http://localhost:26657", "~/.nexusd");
        let checkpoint = client.get_pending_checkpoint().unwrap();
        assert!(checkpoint.is_some());
    }
}
