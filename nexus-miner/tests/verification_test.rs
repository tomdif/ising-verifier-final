//! Test that deterministic inputs produce matching hashes

use sha2::{Sha256, Digest};

fn compute_seed(job_id: &str, ligand_id: i64) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(job_id.as_bytes());
    hasher.update(ligand_id.to_le_bytes());
    let result = hasher.finalize();
    u64::from_be_bytes(result[0..8].try_into().unwrap())
}

fn compute_result_hash(
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

fn main() {
    println!("=== NEXUS Hash Verification Test ===\n");

    // Simulate a job
    let job_id = "dock-1ABC-100";
    let ligand_id: i64 = 12345;

    // Both miners compute the same seed
    let seed = compute_seed(job_id, ligand_id);
    println!("Job ID:    {}", job_id);
    println!("Ligand ID: {}", ligand_id);
    println!("Seed:      {}\n", seed);

    // Miner A runs WASM Vina and gets result
    let score_a: f64 = -7.523456;
    let pose_a = "MODEL 1\nATOM      1  C   LIG     1       1.234   2.345   3.456\nENDMDL";

    // Miner B runs WASM Vina with same inputs - gets identical result
    let score_b: f64 = -7.523456;
    let pose_b = "MODEL 1\nATOM      1  C   LIG     1       1.234   2.345   3.456\nENDMDL";

    // Both compute hash
    let hash_a = compute_result_hash(job_id, ligand_id, seed, score_a, pose_a);
    let hash_b = compute_result_hash(job_id, ligand_id, seed, score_b, pose_b);

    println!("Miner A hash: {}", hash_a);
    println!("Miner B hash: {}", hash_b);
    println!();

    if hash_a == hash_b {
        println!("✅ MATCH! Both miners verified.");
    } else {
        println!("❌ MISMATCH! Results differ.");
    }

    // Now test what happens with a slight difference
    println!("\n=== Testing Mismatch Detection ===\n");

    let score_c: f64 = -7.523457; // Tiny difference
    let hash_c = compute_result_hash(job_id, ligand_id, seed, score_c, pose_a);

    println!("Miner A hash: {}", hash_a);
    println!("Miner C hash: {} (score off by 0.000001)", hash_c);
    println!();

    if hash_a == hash_c {
        println!("❌ False match - this shouldn't happen!");
    } else {
        println!("✅ Correctly detected mismatch!");
    }
}
