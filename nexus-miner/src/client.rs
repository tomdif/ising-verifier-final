use anyhow::Result;
use serde::Deserialize;
use crate::config::ChainConfig;

pub struct ChainClient {
    node_url: String,
    http: reqwest::Client,
    miner_address: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkAssignment {
    pub assignment_id: String,
    pub job_id: String,
    pub ligand_id: i64,
    pub seed: u64,
}

#[derive(Debug, Deserialize)]
pub struct FullWorkData {
    pub protein_pdbqt: String,
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub size_x: f64,
    pub size_y: f64,
    pub size_z: f64,
    pub exhaustiveness: i32,
}

#[derive(Debug, Deserialize)]
pub struct SubmitResponse {
    pub status: String,
    pub message: String,
    pub shares: String,
}

#[derive(Debug, Deserialize)]
pub struct MinerStats {
    pub period_id: u64,
    pub shares: String,
    pub estimated_reward: String,
}

impl ChainClient {
    pub fn new(config: &ChainConfig) -> Result<Self> {
        Ok(ChainClient {
            node_url: config.node.clone(),
            http: reqwest::Client::new(),
            miner_address: "nexus1...".to_string(),
        })
    }

    pub async fn request_work(&self) -> Result<WorkAssignment> {
        let url = format!("{}/nexus/mining/v1/request_work", self.node_url);
        let resp = self.http.post(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn get_full_assignment(&self, id: &str) -> Result<FullWorkData> {
        let url = format!("{}/nexus/mining/v1/assignment/{}", self.node_url, id);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn submit_result(
        &self, job_id: &str, ligand_id: i64,
        hash: &str, ipfs: &str, score: f64, bonds: i32,
    ) -> Result<SubmitResponse> {
        let url = format!("{}/nexus/mining/v1/submit", self.node_url);
        let resp = self.http.post(&url)
            .json(&serde_json::json!({
                "miner": self.miner_address,
                "job_id": job_id,
                "ligand_id": ligand_id,
                "result_hash": hash,
                "pose_ipfs": ipfs,
                "score": format!("{:.6}", score),
                "bonds": bonds,
            }))
            .send().await?;
        Ok(resp.json().await?)
    }

    pub async fn heartbeat(&self, id: &str) -> Result<()> {
        let url = format!("{}/nexus/mining/v1/heartbeat", self.node_url);
        self.http.post(&url)
            .json(&serde_json::json!({"assignment_id": id}))
            .send().await?;
        Ok(())
    }

    pub async fn get_miner_stats(&self) -> Result<MinerStats> {
        let url = format!("{}/nexus/mining/v1/miner/{}/stats", self.node_url, self.miner_address);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }
}

