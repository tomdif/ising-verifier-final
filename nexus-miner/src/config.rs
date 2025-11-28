//! Configuration for NEXUS miner

use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub chain: ChainConfig,
    pub ipfs: IpfsConfig,
    pub vina: VinaConfig,
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub miner_address: String,
    pub private_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IpfsConfig {
    pub gateway: String,
}

#[derive(Debug, Deserialize)]
pub struct VinaConfig {
    pub path: String,
    pub expected_hash: String,
    pub exhaustiveness: u32,
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    pub size_x: f64,
    pub size_y: f64,
    pub size_z: f64,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
