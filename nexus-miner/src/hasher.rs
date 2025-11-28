use anyhow::Result;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{BufReader, Read};

pub fn hash_file(path: &str) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

pub fn compute_result_hash(
    job_id: &str,
    ligand_id: i64,
    seed: u64,
    score: f64,
    pose_data: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(job_id.as_bytes());
    hasher.update(ligand_id.to_le_bytes());
    hasher.update(seed.to_le_bytes());
    hasher.update(format!("{:.6}", score).as_bytes());
    hasher.update(pose_data.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn compute_seed(job_id: &str, ligand_id: i64) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(job_id.as_bytes());
    hasher.update(ligand_id.to_le_bytes());
    let result = hasher.finalize();
    u64::from_be_bytes(result[0..8].try_into().unwrap())
}
