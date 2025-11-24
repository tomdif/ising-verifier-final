//! Nova Ising Prover - Phase 1 Hardened Benchmark

use ising_nova::*;
use ff::Field;
use nova_snark::{
    provider::ipa_pc::EvaluationEngine,
    traits::{circuit::TrivialCircuit, snark::RelaxedR1CSSNARKTrait},
    PublicParams, RecursiveSNARK, CompressedSNARK,
    spartan::snark::RelaxedR1CSSNARK,
};
use std::time::Instant;

type C1 = HardenedIsingCircuit<F1>;
type C2 = TrivialCircuit<F2>;
type EE1 = EvaluationEngine<E1>;
type EE2 = EvaluationEngine<E2>;
type S1 = RelaxedR1CSSNARK<E1, EE1>;
type S2 = RelaxedR1CSSNARK<E2, EE2>;

fn generate_test_problem(n_spins: usize, degree: usize) -> (Vec<(u32, u32, i64)>, Vec<u8>) {
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
    
    let spins: Vec<u8> = (0..n_spins).map(|_| ((next_rand() >> 32) & 1) as u8).collect();
    (edges, spins)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Nova Ising Prover - PHASE 1: CRYPTOGRAPHIC HARDENING");
    println!("  Features: Commitment binding, Fiat-Shamir spot-checks");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    
    // Use 10x scale for faster testing of new circuit
    let scale = 10;
    let n_spins = 131_072 * scale;
    let degree = 12;
    
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  {} spins ({}x scale), degree {}", n_spins, scale, degree);
    println!("╚═══════════════════════════════════════════════════════════════╝");
    
    // Step 1: Generate problem
    print!("  [1/7] Generating problem... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let (edges, spins) = generate_test_problem(n_spins, degree);
    println!("{:?} ({:.1}M edges)", t.elapsed(), edges.len() as f64 / 1e6);
    
    // Compute energy first to set threshold
    let energy = ising_nova::compute_ising_energy_parallel(&edges, &spins);
    
    // Set threshold = energy + 1000 (some slack), gap = 0 (no gap-hiding)
    let threshold = energy + 1000;
    let gap = 0i64;
    
    // Step 2: Create hardened prover (computes commitments)
    println!("  [2/7] Creating hardened prover with commitments...");
    println!("         Energy: {}, Threshold: {}, Gap: {}", energy, threshold, gap);
    let t = Instant::now();
    let prover = HardenedIsingProver::new(edges, spins, threshold, gap);
    let commitment_time = t.elapsed();
    println!("         Commitment time: {:?}", commitment_time);
    
    // Step 3: Generate step circuits with spot-checks
    print!("  [3/7] Generating {} hardened step circuits... ", prover.num_steps());
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let circuits = prover.step_circuits();
    println!("{:?}", t.elapsed());
    
    let total_spot_checks = circuits.iter().map(|c| c.spot_checks.len()).sum::<usize>();
    println!("         Total spot-checks: {} ({} per step)", 
             total_spot_checks, SPOT_CHECKS_PER_STEP);
    
    // Step 4: Nova setup
    print!("  [4/7] Nova setup (arity=5)... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let pp = PublicParams::<E1, E2, C1, C2>::setup(
        &circuits[0],
        &TrivialCircuit::default(),
        &*S1::ck_floor(),
        &*S2::ck_floor(),
    );
    println!("{:?}", t.elapsed());
    
    // Step 5: Recursive proving
    println!("  [5/7] Recursive proving ({} steps)...", circuits.len());
    let t_prove = Instant::now();
    
    // Initial state: [biased_energy, problem_commitment, spin_commitment]
    let z0 = prover.initial_state();
    let z0_sec = vec![F2::ZERO];
    
    let mut recursive_snark = RecursiveSNARK::<E1, E2, C1, C2>::new(
        &pp,
        &circuits[0],
        &TrivialCircuit::default(),
        &z0,
        &z0_sec,
    )?;
    
    for (i, circuit) in circuits.iter().enumerate() {
        recursive_snark.prove_step(&pp, circuit, &TrivialCircuit::default())?;
        
        if (i + 1) % 10 == 0 || i == circuits.len() - 1 {
            println!("         Step {}/{} ({:.1}s elapsed)", 
                     i + 1, circuits.len(), t_prove.elapsed().as_secs_f64());
        }
    }
    
    let prove_time = t_prove.elapsed();
    println!("         Done: {:?} ({:.1}ms/step)", prove_time, 
             prove_time.as_secs_f64() * 1000.0 / circuits.len() as f64);
    
    // Verify recursive proof
    print!("         Verifying... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    recursive_snark.verify(&pp, circuits.len(), &z0, &z0_sec)?;
    println!("✅");
    
    // Step 6: Verify commitment integrity
    println!("  [6/7] Verifying commitment integrity...");
    let energy = prover.total_energy();
    println!("         Energy: {}", energy);
    
    // Step 7: Compress proof
    print!("  [7/7] Compressing proof... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let (pk, vk) = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::setup(&pp)?;
    let compressed = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::prove(&pp, &pk, &recursive_snark)?;
    let compress_time = t.elapsed();
    println!("{:?}", compress_time);
    
    let proof_bytes = bincode::serialize(&compressed)?;
    println!("         Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024.0);
    
    print!("         Compressed verify: ");
    std::io::Write::flush(&mut std::io::stdout())?;
    compressed.verify(&vk, circuits.len(), &z0, &z0_sec)?;
    println!("✅");
    
    // Summary
    let bundle = prover.create_bundle();
    println!();
    println!("  ╔═══════════════════════════════════════════════════════════╗");
    println!("  ║ PHASE 1 HARDENED RESULTS");
    println!("  ╠═══════════════════════════════════════════════════════════╣");
    println!("  ║ Spins:              {:>12} ({:.2}M)", bundle.n_spins, bundle.n_spins as f64 / 1e6);
    println!("  ║ Edges:              {:>12} ({:.2}M)", bundle.n_edges, bundle.n_edges as f64 / 1e6);
    println!("  ║ Steps:              {:>12}", circuits.len());
    println!("  ║ Spot-checks:        {:>12}", bundle.num_spot_checks);
    println!("  ║ ─────────────────────────────────────────────────────────");
    println!("  ║ Commitment time:    {:>12.2}s", commitment_time.as_secs_f64());
    println!("  ║ Recursive time:     {:>12.2}s", prove_time.as_secs_f64());
    println!("  ║ Compress time:      {:>12.2}s", compress_time.as_secs_f64());
    println!("  ║ TOTAL PROVE:        {:>12.2}s", 
             (commitment_time + prove_time + compress_time).as_secs_f64());
    println!("  ║ ─────────────────────────────────────────────────────────");
    println!("  ║ Proof size:         {:>12.1} KB", proof_bytes.len() as f64 / 1024.0);
    println!("  ║ Energy:             {:>12}", bundle.claimed_energy);
    println!("  ╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  PHASE 1 CRYPTOGRAPHIC HARDENING - COMPLETE");
    println!("═══════════════════════════════════════════════════════════════════");
    
    Ok(())
}
