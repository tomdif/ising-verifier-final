//! Test the collaborative mining circuit

use ising_nova::collaborative::*;
use ising_nova::{F1, compute_ising_energy};

fn generate_test_problem(n_spins: usize, degree: usize) -> Vec<(u32, u32, i64)> {
    use std::collections::HashSet;
    let mut edges = Vec::new();
    let mut seen = HashSet::new();
    let mut rng_state = 12345u64;

    let mut next_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng_state
    };

    for u in 0..n_spins {
        for _ in 0..degree {
            let v = (next_rand() as usize) % n_spins;
            if u != v {
                let (a, b) = if u < v { (u, v) } else { (v, u) };
                if seen.insert((a, b)) {
                    let w = ((next_rand() % 201) as i64) - 100;
                    edges.push((a as u32, b as u32, w));
                }
            }
        }
    }
    edges
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  COLLABORATIVE MINING CIRCUIT TEST");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    let n_spins = 64;
    let degree = 4;
    let total_steps = 1000u64;

    println!("  [1] Generating test problem...");
    let edges = generate_test_problem(n_spins, degree);
    println!("      Spins: {}, Edges: {}", n_spins, edges.len());

    let job_hash = F1::from(12345u64);
    let epoch = 100u64;
    let miner_address = F1::from(0xABCDu64);
    let vrf_randomness = F1::from(0x98765u64);

    println!("  [2] Testing seed derivation...");
    let seed = derive_seed(job_hash, epoch, miner_address, vrf_randomness);
    println!("      Seed derived successfully");

    println!("  [3] Testing initial config derivation...");
    let initial_config = derive_initial_config(seed, n_spins);
    let ones = initial_config.iter().filter(|&&x| x == 1).count();
    println!("      Config: {} ones, {} zeros", ones, n_spins - ones);

    let initial_energy = compute_ising_energy(&edges, &initial_config);
    println!("      Initial energy: {}", initial_energy);

    println!("  [4] Creating collaborative prover...");
    let prover = CollaborativeProver::new(
        edges.clone(),
        n_spins,
        job_hash,
        epoch,
        miner_address,
        vrf_randomness,
        total_steps,
    );
    println!("      Number of folds: {}", prover.num_folds());

    println!("  [5] Generating fold circuits (running SA)...");
    let (circuits, final_state) = prover.generate_circuits();
    println!("      Generated {} circuits", circuits.len());
    println!("      Final energy: {}", final_state.current_energy);
    println!("      Best energy: {}", final_state.best_energy);
    println!("      Improvement: {}", initial_energy - final_state.best_energy);

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  TEST PASSED!");
    println!("═══════════════════════════════════════════════════════════════════");
}
