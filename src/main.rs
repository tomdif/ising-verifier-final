use std::time::Instant;
use ff::Field;
use ising_nova::{
    compute_ising_energy, IsingNovaProver, IsingStepCircuit, 
    E1, E2, F1, F2, EDGES_PER_STEP,
};
use nova_snark::{
    traits::{circuit::TrivialCircuit, snark::RelaxedR1CSSNARKTrait},
    PublicParams, RecursiveSNARK, CompressedSNARK,
    provider::ipa_pc::EvaluationEngine,
    spartan::snark::RelaxedR1CSSNARK,
};
use std::collections::HashSet;

type EE1 = EvaluationEngine<E1>;
type EE2 = EvaluationEngine<E2>;
type S1 = RelaxedR1CSSNARK<E1, EE1>;
type S2 = RelaxedR1CSSNARK<E2, EE2>;
type C1 = IsingStepCircuit<F1>;
type C2 = TrivialCircuit<F2>;

/// Generate a random degree-d graph using simple PRNG
fn generate_degree_d_graph(n_spins: usize, degree: usize, seed: u64) -> Vec<(u32, u32, i64)> {
    let mut edges = HashSet::new();
    let mut rng_state = seed;
    
    // Simple LCG PRNG
    let mut next_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng_state
    };
    
    // Target: each node has approximately `degree` edges
    // Total edges ≈ n * degree / 2
    let target_edges = n_spins * degree / 2;
    
    while edges.len() < target_edges {
        let u = (next_rand() as usize) % n_spins;
        let v = (next_rand() as usize) % n_spins;
        if u != v {
            let (a, b) = if u < v { (u, v) } else { (v, u) };
            edges.insert((a as u32, b as u32));
        }
    }
    
    // Convert to vec with random weights in [-10, 10]
    edges.into_iter()
        .map(|(u, v)| {
            let w = ((next_rand() % 21) as i64) - 10;  // -10 to +10
            (u, v, if w == 0 { 1 } else { w })
        })
        .collect()
}

/// Generate random spins
fn generate_random_spins(n_spins: usize, seed: u64) -> Vec<u8> {
    let mut rng_state = seed;
    (0..n_spins).map(|_| {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 63) as u8
    }).collect()
}

fn main() {
    println!("=============================================");
    println!("  Nova Ising Prover - Degree-12 Graphs");
    println!("  EDGES_PER_STEP = {}", EDGES_PER_STEP);
    println!("=============================================\n");
    
    let degree = 12;
    
    for n_spins in [1_000, 5_000, 10_000, 20_000, 50_000] {
        if let Err(e) = run_benchmark(n_spins, degree) {
            println!("  ❌ Error: {:?}\n", e);
        }
        println!();
    }
}

fn run_benchmark(n_spins: usize, degree: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("========== N = {} spins, degree = {} ==========", n_spins, degree);
    
    // Generate degree-12 graph
    let edges = generate_degree_d_graph(n_spins, degree, 12345);
    let spins = generate_random_spins(n_spins, 67890);
    
    let actual_edges = edges.len();
    let avg_degree = (2 * actual_edges) as f64 / n_spins as f64;
    
    let energy = compute_ising_energy(&edges, &spins);
    let prover = IsingNovaProver::new(edges, spins, 100, energy + 1000);
    let num_steps = prover.num_steps();
    let circuits = prover.step_circuits();
    
    println!("  Edges: {} (avg degree: {:.1}), Steps: {}", actual_edges, avg_degree, num_steps);
    
    // Setup
    println!("\n  [1/5] Setup...");
    let t = Instant::now();
    let pp = PublicParams::<E1, E2, C1, C2>::setup(
        &circuits[0],
        &TrivialCircuit::default(),
        &*S1::ck_floor(),
        &*S2::ck_floor(),
    );
    println!("        {:?}", t.elapsed());
    
    // Recursive proving
    println!("  [2/5] Recursive proving ({} steps)...", num_steps);
    let t = Instant::now();
    let z0 = IsingNovaProver::initial_state();
    let z0_secondary = vec![F2::ZERO];
    
    let mut rs = RecursiveSNARK::<E1, E2, C1, C2>::new(
        &pp,
        &circuits[0],
        &TrivialCircuit::default(),
        &z0,
        &z0_secondary,
    )?;
    
    for (i, circuit) in circuits.iter().enumerate() {
        rs.prove_step(&pp, circuit, &TrivialCircuit::default())?;
        if num_steps >= 50 && (i + 1) % 50 == 0 {
            println!("        Step {}/{} ({:.1}s)", i + 1, num_steps, t.elapsed().as_secs_f64());
        }
    }
    let recursive_time = t.elapsed();
    let total_steps = circuits.len();
    println!("        {:?} ({:.1}ms/step)", recursive_time, 
             recursive_time.as_millis() as f64 / total_steps as f64);
    
    // Verify recursive proof
    println!("  [3/5] Verifying recursive proof...");
    let t = Instant::now();
    rs.verify(&pp, total_steps, &z0, &z0_secondary)?;
    println!("        {:?} ✅", t.elapsed());
    
    // Compress
    println!("  [4/5] Compressing proof...");
    let t = Instant::now();
    let (pk, vk) = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::setup(&pp)?;
    let compressed = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::prove(&pp, &pk, &rs)?;
    let compress_time = t.elapsed();
    
    let proof_bytes = bincode::serialize(&compressed)?;
    let proof_size_kb = proof_bytes.len() as f64 / 1024.0;
    println!("        {:?}, size: {:.1} KB", compress_time, proof_size_kb);
    
    // Verify compressed
    println!("  [5/5] Verifying compressed proof...");
    let t = Instant::now();
    compressed.verify(&vk, total_steps, &z0, &z0_secondary)?;
    let verify_time = t.elapsed();
    println!("        {:?} ✅", verify_time);
    
    let total_prove = recursive_time.as_secs_f64() + compress_time.as_secs_f64();
    
    println!("\n  ╔══════════════════════════════════════════════════╗");
    println!("  ║ RESULTS: {} spins, degree {}", n_spins, degree);
    println!("  ╠══════════════════════════════════════════════════╣");
    println!("  ║ Edges:            {:>12}", actual_edges);
    println!("  ║ Steps:            {:>12}", num_steps);
    println!("  ║ Recursive prove:  {:>12.2}s", recursive_time.as_secs_f64());
    println!("  ║ Compress:         {:>12.2}s", compress_time.as_secs_f64());
    println!("  ║ TOTAL PROVE:      {:>12.2}s", total_prove);
    println!("  ║ Proof size:       {:>12.1} KB", proof_size_kb);
    println!("  ║ Verify time:      {:>12.1}ms", verify_time.as_millis() as f64);
    println!("  ╚══════════════════════════════════════════════════╝");
    
    // Extrapolation
    if n_spins >= 10000 {
        let ms_per_step = recursive_time.as_millis() as f64 / total_steps as f64;
        let target_edges = 131072 * 12 / 2;  // 786K edges
        let target_steps = (target_edges + EDGES_PER_STEP - 1) / EDGES_PER_STEP;
        let projected_recursive = target_steps as f64 * ms_per_step / 1000.0;
        let projected_total = projected_recursive + 0.5;  // + compression
        println!("\n  → Projected for 131K spins, degree 12:");
        println!("    {} edges, {} steps", target_edges, target_steps);
        println!("    Recursive: {:.1}s, Total: {:.1}s", projected_recursive, projected_total);
        println!("    With GPU (4x): {:.1}s", projected_total / 4.0);
    }
    
    Ok(())
}
