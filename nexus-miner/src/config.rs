use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub chain: ChainConfig,
    pub miner: MinerConfig,
    pub wasm: WasmConfig,
    pub ipfs: IpfsConfig,
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub node: String,
    pub chain_id: String,
}

#[derive(Debug, Deserialize)]
pub struct MinerConfig {
    pub key_file: String,
    pub heartbeat_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct WasmConfig {
    pub vina_path: String,
    pub expected_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct IpfsConfig {
    pub gateway: String,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
