//! Full Nova proof test for collaborative mining

use ising_nova::collaborative::*;
use ising_nova::{F1, F2, E1, E2};
use ff::Field;
use nova_snark::{
    provider::ipa_pc::EvaluationEngine,
    traits::{circuit::TrivialCircuit, circuit::StepCircuit, snark::RelaxedR1CSSNARKTrait},
    PublicParams, RecursiveSNARK, CompressedSNARK,
    spartan::snark::RelaxedR1CSSNARK,
};
use std::time::Instant;

type C1 = CollaborativeMiningCircuit<F1>;
type C2 = TrivialCircuit<F2>;
type EE1 = EvaluationEngine<E1>;
type EE2 = EvaluationEngine<E2>;
type S1 = RelaxedR1CSSNARK<E1, EE1>;
type S2 = RelaxedR1CSSNARK<E2, EE2>;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  COLLABORATIVE MINING - FULL NOVA PROOF TEST");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    // Small problem for testing
    let n_spins = 64;
    let degree = 4;
    let total_steps = 100_000u64;  // 5 folds of 100 steps each

    println!("  Configuration:");
    println!("    Spins: {}", n_spins);
    println!("    SA Steps: {}", total_steps);
    println!("    Steps per fold: {}", SA_STEPS_PER_FOLD);
    println!();

    // Step 1: Generate problem
    print!("  [1/7] Generating problem... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let edges = generate_test_problem(n_spins, degree);
    println!("{:?} ({} edges)", t.elapsed(), edges.len());

    // Step 2: Create job assignment
    let job_hash = F1::from(12345u64);
    let epoch = 100u64;
    let miner_address = F1::from(0xABCDu64);
    let vrf_randomness = F1::from(0x98765u64);

    // Step 3: Create prover and generate circuits
    print!("  [2/7] Creating prover and generating circuits... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let prover = CollaborativeProver::new(
        edges.clone(),
        n_spins,
        job_hash,
        epoch,
        miner_address,
        vrf_randomness,
        total_steps,
    );
    let (circuits, final_state) = prover.generate_circuits();
    println!("{:?}", t.elapsed());
    println!("         {} folds, final energy: {}, best: {}", 
             circuits.len(), final_state.current_energy, final_state.best_energy);

    // Step 4: Nova setup
    print!("  [3/7] Nova setup (arity={})... ", circuits[0].arity());
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
    println!("  [4/7] Recursive proving ({} folds)...", circuits.len());
    let t_prove = Instant::now();

    let z0 = prover.initial_state();
    let z0_sec = vec![F2::ZERO];

    println!("         Initial state: {} elements", z0.len());

    let mut recursive_snark = RecursiveSNARK::<E1, E2, C1, C2>::new(
        &pp,
        &circuits[0],
        &TrivialCircuit::default(),
        &z0,
        &z0_sec,
    )?;

    for (i, circuit) in circuits.iter().enumerate() {
        let step_start = Instant::now();
        recursive_snark.prove_step(&pp, circuit, &TrivialCircuit::default())?;
        println!("         Fold {}/{}: {:?}", i + 1, circuits.len(), step_start.elapsed());
    }

    let prove_time = t_prove.elapsed();
    println!("         Total recursive time: {:?}", prove_time);

    // Step 6: Verify recursive proof
    print!("  [5/7] Verifying recursive proof... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    recursive_snark.verify(&pp, circuits.len(), &z0, &z0_sec)?;
    println!("{:?} ✅", t.elapsed());

    // Step 7: Compress proof
    print!("  [6/7] Compressing proof... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let (pk, vk) = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::setup(&pp)?;
    let compressed = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::prove(&pp, &pk, &recursive_snark)?;
    let compress_time = t.elapsed();
    println!("{:?}", compress_time);

    let proof_bytes = bincode::serialize(&compressed)?;
    println!("         Proof size: {:.2} KB", proof_bytes.len() as f64 / 1024.0);

    // Step 8: Verify compressed proof
    print!("  [7/7] Verifying compressed proof... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    compressed.verify(&vk, circuits.len(), &z0, &z0_sec)?;
    println!("{:?} ✅", t.elapsed());

    // Summary
    println!();
    println!("  ╔═══════════════════════════════════════════════════════════════╗");
    println!("  ║ COLLABORATIVE MINING PROOF - RESULTS                          ║");
    println!("  ╠═══════════════════════════════════════════════════════════════╣");
    println!("  ║ Problem:                                                      ║");
    println!("  ║   Spins:              {:>10}                              ║", n_spins);
    println!("  ║   Edges:              {:>10}                              ║", edges.len());
    println!("  ║   SA Steps:           {:>10}                              ║", total_steps);
    println!("  ║   Folds:              {:>10}                              ║", circuits.len());
    println!("  ╠═══════════════════════════════════════════════════════════════╣");
    println!("  ║ Results:                                                      ║");
    println!("  ║   Initial energy:     {:>10}                              ║", prover.initial_energy);
    println!("  ║   Final energy:       {:>10}                              ║", final_state.current_energy);
    println!("  ║   Best energy:        {:>10}                              ║", final_state.best_energy);
    println!("  ╠═══════════════════════════════════════════════════════════════╣");
    println!("  ║ Timing:                                                       ║");
    println!("  ║   Recursive prove:    {:>10.2}s                             ║", prove_time.as_secs_f64());
    println!("  ║   Compress:           {:>10.2}s                             ║", compress_time.as_secs_f64());
    println!("  ║   TOTAL:              {:>10.2}s                             ║", (prove_time + compress_time).as_secs_f64());
    println!("  ╠═══════════════════════════════════════════════════════════════╣");
    println!("  ║ Proof size:           {:>10.2} KB                           ║", proof_bytes.len() as f64 / 1024.0);
    println!("  ╚═══════════════════════════════════════════════════════════════╝");
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  COLLABORATIVE MINING PROOF - SUCCESS!");
    println!("═══════════════════════════════════════════════════════════════════");

    Ok(())
}
