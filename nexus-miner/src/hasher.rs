//! File hashing utilities

use anyhow::Result;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::Read;

pub fn hash_file(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}
