//! NEXUS Mining Worker - Native Vina Execution

use anyhow::{Result, anyhow};
use sha2::{Sha256, Digest};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::process::Command;
use std::fs;

#[derive(Debug, Clone)]
pub struct DockingResult {
    pub job_id: String,
    pub ligand_id: String,
    pub seed: u32,  // Vina uses 32-bit seed
    pub affinity: f64,
    pub pose_pdbqt: String,
    pub result_hash: String,
    pub num_bonds: u32,
}

/// Generate deterministic 32-bit seed from job_id and ligand_id
/// Vina's --seed argument accepts values 0 to 2^31-1
pub fn generate_seed(job_id: &str, ligand_id: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    job_id.hash(&mut hasher);
    ligand_id.hash(&mut hasher);
    // Truncate to 31 bits to ensure positive i32 compatible value
    (hasher.finish() & 0x7FFFFFFF) as u32
}

#[derive(Debug, Clone)]
pub struct AtomPosition {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn parse_pdbqt_atoms(pdbqt: &str) -> Vec<AtomPosition> {
    pdbqt.lines()
        .filter(|line| line.starts_with("ATOM") || line.starts_with("HETATM"))
        .filter(|line| line.len() >= 54)
        .map(|line| AtomPosition {
            name: line[12..16].trim().to_string(),
            x: line[30..38].trim().parse().unwrap_or(0.0),
            y: line[38..46].trim().parse().unwrap_or(0.0),
            z: line[46..54].trim().parse().unwrap_or(0.0),
        })
        .collect()
}

pub fn compute_result_hash(
    job_id: &str,
    ligand_id: &str,
    seed: u32,
    affinity: f64,
    pose_atoms: &[AtomPosition],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(job_id.as_bytes());
    hasher.update(ligand_id.as_bytes());
    hasher.update(seed.to_le_bytes());
    hasher.update(format!("{:.6}", affinity).as_bytes());
    for atom in pose_atoms {
        hasher.update(atom.name.as_bytes());
        hasher.update(format!("{:.3}", atom.x).as_bytes());
        hasher.update(format!("{:.3}", atom.y).as_bytes());
        hasher.update(format!("{:.3}", atom.z).as_bytes());
    }
    hex::encode(hasher.finalize())
}

pub fn parse_affinity(vina_output: &str) -> Option<f64> {
    for line in vina_output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == "1" {
            return parts[1].parse().ok();
        }
    }
    None
}

pub fn count_bonds(pdbqt: &str) -> u32 {
    pdbqt.lines()
        .find(|line| line.starts_with("TORSDOF"))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct DockingConfig {
    pub vina_path: String,
    pub center: (f64, f64, f64),
    pub size: (f64, f64, f64),
    pub exhaustiveness: u32,
    pub cpu: u32,
}

impl Default for DockingConfig {
    fn default() -> Self {
        Self {
            vina_path: "./vina_1.2.5_linux_x86_64".to_string(),
            center: (0.0, 0.0, 0.0),
            size: (20.0, 20.0, 20.0),
            exhaustiveness: 8,
            cpu: 1,
        }
    }
}

pub fn execute_docking(
    config: &DockingConfig,
    receptor_path: &str,
    ligand_path: &str,
    job_id: &str,
    ligand_id: &str,
) -> Result<DockingResult> {
    let seed = generate_seed(job_id, ligand_id);
    let output_path = format!("/tmp/nexus_dock_{}_{}.pdbqt", 
        job_id.replace("/", "_"), ligand_id);
    
    let output = Command::new(&config.vina_path)
        .args(&[
            "--receptor", receptor_path,
            "--ligand", ligand_path,
            "--center_x", &config.center.0.to_string(),
            "--center_y", &config.center.1.to_string(),
            "--center_z", &config.center.2.to_string(),
            "--size_x", &config.size.0.to_string(),
            "--size_y", &config.size.1.to_string(),
            "--size_z", &config.size.2.to_string(),
            "--seed", &seed.to_string(),
            "--exhaustiveness", &config.exhaustiveness.to_string(),
            "--cpu", &config.cpu.to_string(),
            "--num_modes", "1",
            "--out", &output_path,
        ])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Vina failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let affinity = parse_affinity(&stdout)
        .ok_or_else(|| anyhow!("Failed to parse affinity from Vina output"))?;
    
    let pose_pdbqt = fs::read_to_string(&output_path)?;
    let atoms = parse_pdbqt_atoms(&pose_pdbqt);
    let result_hash = compute_result_hash(job_id, ligand_id, seed, affinity, &atoms);
    
    let ligand_content = fs::read_to_string(ligand_path).unwrap_or_default();
    let num_bonds = count_bonds(&ligand_content);
    
    let _ = fs::remove_file(&output_path);
    
    Ok(DockingResult {
        job_id: job_id.to_string(),
        ligand_id: ligand_id.to_string(),
        seed,
        affinity,
        pose_pdbqt,
        result_hash,
        num_bonds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_seed_determinism() {
        let s1 = generate_seed("job-123", "lig-456");
        let s2 = generate_seed("job-123", "lig-456");
        assert_eq!(s1, s2);
        // Verify it's in valid range for Vina
        assert!(s1 <= 0x7FFFFFFF);
    }
    
    #[test]
    fn test_seed_range() {
        // Test various inputs produce valid seeds
        for i in 0..100 {
            let seed = generate_seed(&format!("job-{}", i), "ligand");
            assert!(seed <= 0x7FFFFFFF, "Seed {} out of range", seed);
        }
    }
    
    #[test]
    fn test_parse_affinity() {
        let out = "   1       -7.144          0          0";
        assert_eq!(parse_affinity(out), Some(-7.144));
    }
}
