//! Profile witness generation

use ising_nova::collaborative::*;
use ising_nova::{F1};
use ff::Field;
use std::time::Instant;

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
    println!("  WITNESS GENERATION PROFILING");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let n_spins = 64;
    let edges = generate_test_problem(n_spins, 4);
    let total_steps = 100_000u64;
    
    let job_hash = F1::from(12345u64);
    let epoch = 100u64;
    let miner_address = F1::from(0xABCDu64);
    let vrf_randomness = F1::from(0x98765u64);

    // Time seed derivation
    let t = Instant::now();
    let seed = derive_seed(job_hash, epoch, miner_address, vrf_randomness);
    println!("  Seed derivation:       {:>10.3}ms", t.elapsed().as_secs_f64() * 1000.0);

    // Time initial config
    let t = Instant::now();
    let initial_config = derive_initial_config(seed, n_spins);
    println!("  Initial config ({} spins): {:>7.3}ms", n_spins, t.elapsed().as_secs_f64() * 1000.0);

    // Time SA execution without witness storage
    let t = Instant::now();
    let mut spins = initial_config.clone();
    let mut prg = PrgState::new(seed);
    let mut temperature = INITIAL_TEMP_FIXED;
    for _ in 0..total_steps {
        let _ = sa_step(&mut spins, &edges, &mut prg, &mut temperature);
    }
    let sa_only = t.elapsed();
    println!("  SA only ({}K steps):  {:>10.3}ms", total_steps/1000, sa_only.as_secs_f64() * 1000.0);

    // Time SA with witness storage (like in circuit generation)
    let t = Instant::now();
    let mut spins = initial_config.clone();
    let mut prg = PrgState::new(seed);
    let mut temperature = INITIAL_TEMP_FIXED;
    let mut witnesses: Vec<SAStepWitnessField<F1>> = Vec::with_capacity(total_steps as usize);
    for _ in 0..total_steps {
        let (position, delta_e, accepted) = sa_step(&mut spins, &edges, &mut prg, &mut temperature);
        witnesses.push(SAStepWitnessField {
            position: F1::from(position as u64),
            delta_e: ising_nova::i64_to_field(delta_e),
            accepted: if accepted { F1::ONE } else { F1::ZERO },
            prg_output: F1::ZERO,
        });
    }
    let sa_with_witness = t.elapsed();
    println!("  SA + witness storage: {:>10.3}ms", sa_with_witness.as_secs_f64() * 1000.0);

    // Time PRG alone
    let t = Instant::now();
    let mut prg = PrgState::new(seed);
    for _ in 0..total_steps {
        let _ = prg.advance(n_spins);
    }
    let prg_only = t.elapsed();
    println!("  PRG only ({}K calls): {:>10.3}ms", total_steps/1000, prg_only.as_secs_f64() * 1000.0);

    // Time energy delta computation
    let t = Instant::now();
    for i in 0..total_steps {
        let pos = (i as usize) % n_spins;
        let _ = compute_flip_delta(pos, &edges, &spins);
    }
    let delta_only = t.elapsed();
    println!("  Energy delta only:    {:>10.3}ms", delta_only.as_secs_f64() * 1000.0);

    // Breakdown
    println!("\n  ─────────────────────────────────────────────────────────────────");
    println!("  BREAKDOWN:");
    println!("    PRG (Poseidon hashes):  {:>6.1}%", prg_only.as_secs_f64() / sa_only.as_secs_f64() * 100.0);
    println!("    Energy delta:           {:>6.1}%", delta_only.as_secs_f64() / sa_only.as_secs_f64() * 100.0);
    println!("    Witness overhead:       {:>6.1}%", (sa_with_witness.as_secs_f64() - sa_only.as_secs_f64()) / sa_only.as_secs_f64() * 100.0);
    println!("  ─────────────────────────────────────────────────────────────────\n");
}
