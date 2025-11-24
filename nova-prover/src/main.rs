//! Nova Ising Prover - 100X Scale Benchmark (13M spins)

use std::time::Instant;
use ff::Field;
use ising_nova::{
    IsingStepCircuit, IsingNovaProver,
    E1, E2, F1, F2,
};
use nova_snark::{
    traits::{circuit::TrivialCircuit, snark::RelaxedR1CSSNARKTrait},
    PublicParams, RecursiveSNARK, CompressedSNARK,
    provider::ipa_pc::EvaluationEngine,
    spartan::snark::RelaxedR1CSSNARK,
};

type EE1 = EvaluationEngine<E1>;
type EE2 = EvaluationEngine<E2>;
type S1 = RelaxedR1CSSNARK<E1, EE1>;
type S2 = RelaxedR1CSSNARK<E2, EE2>;
type C1 = IsingStepCircuit<F1>;
type C2 = TrivialCircuit<F2>;

fn run_benchmark(n: usize, deg: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  {} spins ({:.1}M), degree {}  ", n, n as f64 / 1_000_000.0, deg);
    println!("╚═══════════════════════════════════════════════════════════════╝");
    
    // Generate problem
    print!("  [1/6] Generating problem... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let edges: Vec<(u32, u32, i64)> = (0..n).flat_map(|i| {
        (1..=deg/2).map(move |d| {
            let j = (i + d) % n;
            let w = ((i + j) % 19) as i64 - 9;
            (i as u32, j as u32, if w == 0 { 1 } else { w })
        })
    }).collect();
    let spins: Vec<u8> = (0..n).map(|i| (i % 2) as u8).collect();
    println!("{:?} ({:.1}M edges)", t.elapsed(), edges.len() as f64 / 1_000_000.0);
    
    // Commitment (skip for very large - takes too long)
    print!("  [2/6] Computing Poseidon commitment... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let prover = IsingNovaProver::new(edges.clone(), spins.clone());
    let bundle = prover.create_bundle();
    println!("{:?}", t.elapsed());
    
    // Circuits
    print!("  [3/6] Generating {} step circuits... ", prover.num_steps());
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let circuits = prover.step_circuits();
    println!("{:?}", t.elapsed());
    
    // Setup
    print!("  [4/6] Nova setup... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let pp = PublicParams::<E1, E2, C1, C2>::setup(
        &circuits[0], &TrivialCircuit::default(),
        &*S1::ck_floor(), &*S2::ck_floor(),
    );
    println!("{:?}", t.elapsed());
    
    // Prove
    println!("  [5/6] Recursive proving ({} steps)...", circuits.len());
    let t = Instant::now();
    let z0 = IsingNovaProver::initial_state();
    let z0_sec = vec![F2::ZERO];
    let mut rs = RecursiveSNARK::<E1, E2, C1, C2>::new(
        &pp, &circuits[0], &TrivialCircuit::default(), &z0, &z0_sec,
    )?;
    for (i, c) in circuits.iter().enumerate() {
        rs.prove_step(&pp, c, &TrivialCircuit::default())?;
        if (i + 1) % 100 == 0 { 
            println!("         Step {}/{} ({:.1}s elapsed)", i + 1, circuits.len(), t.elapsed().as_secs_f64());
        }
    }
    let rec_time = t.elapsed();
    println!("         Done: {:?} ({:.1}ms/step)", rec_time, rec_time.as_millis() as f64 / circuits.len() as f64);
    
    // Verify recursive
    print!("         Verifying... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    rs.verify(&pp, circuits.len(), &z0, &z0_sec)?;
    println!("✅");
    
    // Compress
    print!("  [6/6] Compressing proof... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let t = Instant::now();
    let (pk, vk) = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::setup(&pp)?;
    let compressed = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::prove(&pp, &pk, &rs)?;
    let comp_time = t.elapsed();
    println!("{:?}", comp_time);
    
    let proof_bytes = bincode::serialize(&compressed)?;
    compressed.verify(&vk, circuits.len(), &z0, &z0_sec)?;
    println!("         Compressed verify: ✅");
    
    // Results
    let total = rec_time.as_secs_f64() + comp_time.as_secs_f64();
    println!("\n  ╔═══════════════════════════════════════════════════════════╗");
    println!("  ║ RESULTS: {} spins ({:.2}M)", n, n as f64 / 1_000_000.0);
    println!("  ╠═══════════════════════════════════════════════════════════╣");
    println!("  ║ Edges:              {:>12} ({:.1}M)", bundle.n_edges, bundle.n_edges as f64 / 1_000_000.0);
    println!("  ║ Steps:              {:>12}", circuits.len());
    println!("  ║ Recursive time:     {:>12.2}s", rec_time.as_secs_f64());
    println!("  ║ Compress time:      {:>12.2}s", comp_time.as_secs_f64());
    println!("  ║ ─────────────────────────────────────────────────────────");
    println!("  ║ TOTAL PROVE:        {:>12.2}s", total);
    println!("  ║ Proof size:         {:>12.1} KB", proof_bytes.len() as f64 / 1024.0);
    println!("  ║ Energy:             {:>12}", bundle.claimed_energy);
    println!("  ╚═══════════════════════════════════════════════════════════╝\n");
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Nova Ising Prover - 100X SCALE BENCHMARK");
    println!("  Target: 13,107,200 spins (100x whitepaper)");
    println!("  Expected: ~45 seconds prove time");
    println!("═══════════════════════════════════════════════════════════════════\n");
    
    // Run 100x benchmark
    run_benchmark(131_072 * 100, 12)?;
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  100X BENCHMARK COMPLETE");
    println!("═══════════════════════════════════════════════════════════════════");
    
    Ok(())
}
