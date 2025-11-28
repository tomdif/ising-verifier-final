//! Integration test using real HIV protease + MK1 inhibitor

use std::process::Command;
use std::fs;

#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_real_docking_determinism() {
    let nexus_miner = env!("CARGO_MANIFEST_DIR");
    
    // Check test files exist
    let receptor = format!("{}/receptor.pdbqt", nexus_miner);
    let ligand = format!("{}/ligand.pdbqt", nexus_miner);
    let vina = format!("{}/vina_1.2.5_linux_x86_64", nexus_miner);
    
    if !std::path::Path::new(&receptor).exists() {
        eprintln!("Skipping: receptor.pdbqt not found");
        return;
    }
    
    // Run docking twice with same seed
    let mut hashes = Vec::new();
    
    for run in 1..=3 {
        let output_path = format!("/tmp/integration_test_{}.pdbqt", run);
        
        let output = Command::new(&vina)
            .args(&[
                "--receptor", &receptor,
                "--ligand", &ligand,
                "--center_x", "13.1",
                "--center_y", "22.5", 
                "--center_z", "5.6",
                "--size_x", "20",
                "--size_y", "20",
                "--size_z", "20",
                "--seed", "42",
                "--exhaustiveness", "8",
                "--cpu", "1",
                "--num_modes", "1",
                "--out", &output_path,
            ])
            .output()
            .expect("Failed to run Vina");
        
        assert!(output.status.success(), "Vina failed");
        
        let content = fs::read_to_string(&output_path).unwrap();
        let hash = format!("{:x}", md5::compute(&content));
        hashes.push(hash);
        
        let _ = fs::remove_file(&output_path);
    }
    
    // All hashes must be identical
    assert_eq!(hashes[0], hashes[1], "Run 1 vs 2 differ!");
    assert_eq!(hashes[1], hashes[2], "Run 2 vs 3 differ!");
    
    println!("✅ Determinism verified: {}", hashes[0]);
}
