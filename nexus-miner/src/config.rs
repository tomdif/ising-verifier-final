use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub chain: ChainConfig,
    pub mining: MiningConfig,
    pub vina: VinaConfig,
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub node_url: String,
    pub home_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct MiningConfig {
    pub num_workers: u32,
    pub data_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct VinaConfig {
    pub binary_path: String,
    pub exhaustiveness: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chain: ChainConfig {
                node_url: "http://localhost:26657".to_string(),
                home_dir: "~/.nexusd".to_string(),
            },
            mining: MiningConfig {
                num_workers: 4,
                data_dir: "./data".to_string(),
            },
            vina: VinaConfig {
                binary_path: "./vina_1.2.5_linux_x86_64".to_string(),
                exhaustiveness: 8,
            },
        }
    }
}

pub fn load_config(path: &Path) -> Result<Config, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    toml::from_str(&contents)
        .map_err(|e| format!("Failed to parse config: {}", e))
}
