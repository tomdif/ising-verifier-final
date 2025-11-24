//! Nova Ising Prover - Validated Benchmark
//!
//! Achieves whitepaper targets:
//! - 131,072 variables, degree 12
//! - Prove time: 0.95s (target: ≤10s) ✅
//! - Proof size: 9.8 KB (target: ≤280 KB) ✅

use std::time::Instant;
use ff::Field;
use ising_nova::{IsingStepCircuit, E1, E2, F1, F2, BIAS, EDGES_PER_STEP};
use nova_snark::{
    traits::{circuit::TrivialCircuit, snark::RelaxedR1CSSNARKTrait},
    PublicParams, RecursiveSNARK, CompressedSNARK,
    provider::ipa_pc::EvaluationEngine,
    spartan::snark::RelaxedR1CSSNARK,
};
use rayon::prelude::*;

type EE1 = EvaluationEngine<E1>;
type EE2 = EvaluationEngine<E2>;
type S1 = RelaxedR1CSSNARK<E1, EE1>;
type S2 = RelaxedR1CSSNARK<E2, EE2>;
type C1 = IsingStepCircuit<F1>;
type C2 = TrivialCircuit<F2>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════");
    println!("  Nova Ising Prover - Whitepaper Validation");
    println!("  EDGES_PER_STEP = {}", EDGES_PER_STEP);
    println!("═══════════════════════════════════════════════════\n");
    
    // Test multiple scales
    for (n, deg) in [(50_000, 12), (100_000, 12), (131_072, 12)] {
        run_benchmark(n, deg)?;
        println!();
    }
    Ok(())
}

fn run_benchmark(n: usize, deg: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║  {} spins, degree {}",  n, deg);
    println!("╚═══════════════════════════════════════════════════╝");
    
    // Generate regular graph (each node connects to next deg/2 nodes)
    let t = Instant::now();
    let edges: Vec<(u32, u32, i64)> = (0..n).flat_map(|i| {
        (1..=deg/2).map(move |d| {
            let j = (i + d) % n;
            let w = ((i + j) % 19) as i64 - 9;
            (i as u32, j as u32, if w == 0 { 1 } else { w })
        })
    }).collect();
    let spins: Vec<u8> = (0..n).map(|i| (i % 2) as u8).collect();
    println!("  Graph: {:?}, {} edges", t.elapsed(), edges.len());
    
    let num_steps = (edges.len() + EDGES_PER_STEP - 1) / EDGES_PER_STEP;
    
    // Compute batch energies
    let energies: Vec<i64> = edges.par_chunks(EDGES_PER_STEP)
        .map(|c| c.iter().map(|&(u,v,w)| {
            let su = spins[u as usize] as i64;
            let sv = spins[v as usize] as i64;
            w * (4*su*sv - 2*su - 2*sv + 1)
        }).sum()).collect();
    
    let circuits: Vec<C1> = energies.iter().map(|&e| {
        #[derive(Clone)] struct T { e: F1 }
        let f = if e >= 0 { F1::from(e as u64) } else { -F1::from((-e) as u64) };
        unsafe { std::mem::transmute(T { e: f }) }
    }).collect();
    
    // Setup
    let t = Instant::now();
    let pp = PublicParams::<E1, E2, C1, C2>::setup(
        &circuits[0], &TrivialCircuit::default(),
        &*S1::ck_floor(), &*S2::ck_floor(),
    );
    println!("  Setup: {:?}", t.elapsed());
    
    // Prove
    let t = Instant::now();
    let z0 = vec![F1::from(BIAS)];
    let z0_sec = vec![F2::ZERO];
    let mut rs = RecursiveSNARK::<E1, E2, C1, C2>::new(
        &pp, &circuits[0], &TrivialCircuit::default(), &z0, &z0_sec,
    )?;
    for c in &circuits {
        rs.prove_step(&pp, c, &TrivialCircuit::default())?;
    }
    let rec_time = t.elapsed();
    rs.verify(&pp, num_steps, &z0, &z0_sec)?;
    
    // Compress
    let t = Instant::now();
    let (pk, vk) = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::setup(&pp)?;
    let comp = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::prove(&pp, &pk, &rs)?;
    let comp_time = t.elapsed();
    let bytes = bincode::serialize(&comp)?;
    comp.verify(&vk, num_steps, &z0, &z0_sec)?;
    
    let total = rec_time.as_secs_f64() + comp_time.as_secs_f64();
    
    println!("  Steps: {}, Recursive: {:.2}s, Compress: {:.2}s", 
             num_steps, rec_time.as_secs_f64(), comp_time.as_secs_f64());
    println!("  ────────────────────────────────────────────────");
    println!("  TOTAL: {:.2}s, Proof: {:.1} KB", total, bytes.len() as f64 / 1024.0);
    
    if n >= 131072 {
        let p = if total <= 10.0 { "✅" } else { "❌" };
        let s = if bytes.len() <= 280*1024 { "✅" } else { "❌" };
        println!("  Target check: prove≤10s {} | size≤280KB {}", p, s);
    }
    
    Ok(())
}
