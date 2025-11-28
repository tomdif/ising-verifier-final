// benches/real_proof.rs
//
// Generate REAL Halo2 proofs (not MockProver) to measure actual performance.
// This uses the full proving pipeline: setup, prove, verify.
//
// Run with: cargo bench --bench real_proof
// Or for a single run: cargo run --release --bin real_proof

use std::time::Instant;

use halo2_proofs::{
    plonk::{
        create_proof, keygen_pk, keygen_vk, verify_proof, Circuit,
        SingleVerifier,
    },
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use pasta_curves::{pallas, vesta};
use rand::rngs::OsRng;

use ising_verifier_final::{
    compute_ising_energy, compute_public_inputs, IsingCircuit,
};

type F = pallas::Base;

fn main() {
    println!("===========================================");
    println!("  Halo2 Real Proof Benchmark");
    println!("===========================================\n");

    // Test different sizes
    for n_spins in [10, 50, 100, 200, 500] {
        run_benchmark(n_spins);
        println!();
    }
}

fn run_benchmark(n_spins: usize) {
    println!("--- N = {} spins ---", n_spins);

    // Build a simple chain graph: 0-1-2-3-...-N
    let edges: Vec<(u32, u32, i64)> = (0..n_spins - 1)
        .map(|i| (i as u32, (i + 1) as u32, 1i64))
        .collect();

    // Alternating spins: 0,1,0,1,...
    let spins: Vec<u8> = (0..n_spins).map(|i| (i % 2) as u8).collect();

    let delta = 100u64;
    let nonce = 12345u64;

    // Compute energy
    let energy = compute_ising_energy(&edges, &spins);
    let threshold = energy + (delta as i64) + 1000; // Ensure E + delta < T

    println!("  Edges: {}", edges.len());
    println!("  Energy: {}", energy);
    println!("  Threshold: {}", threshold);

    // Create circuit
    let circuit = IsingCircuit {
        edges: edges.clone(),
        spins: spins.clone(),
        delta,
        threshold,
        pubkey: F::one(),
        nonce,
    };

    // Compute public inputs
    let (threshold_plus_bias, cipher_root) = compute_public_inputs(threshold, &spins, nonce);
    let public_inputs = vec![
        vec![threshold_plus_bias],
        vec![cipher_root],
        vec![F::zero()],
    ];

    // Choose k based on circuit size
    // k=12 gives 2^12 = 4096 rows
    // k=14 gives 2^14 = 16384 rows
    // k=16 gives 2^16 = 65536 rows
    let k = if n_spins <= 100 { 12 } else if n_spins <= 500 { 14 } else { 16 };
    println!("  k = {} (2^{} = {} rows)", k, k, 1 << k);

    // =========================================
    // SETUP PHASE
    // =========================================
    println!("\n  [1/4] Setup (generating params)...");
    let setup_start = Instant::now();
    
    let params: Params<vesta::Affine> = Params::new(k);
    
    let setup_time = setup_start.elapsed();
    println!("        Setup time: {:?}", setup_time);

    // =========================================
    // KEY GENERATION
    // =========================================
    println!("  [2/4] Key generation...");
    let keygen_start = Instant::now();

    let vk = keygen_vk(&params, &circuit).expect("keygen_vk failed");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("keygen_pk failed");

    let keygen_time = keygen_start.elapsed();
    println!("        Keygen time: {:?}", keygen_time);

    // =========================================
    // PROOF GENERATION
    // =========================================
    println!("  [3/4] Proof generation...");
    let prove_start = Instant::now();

    let mut transcript = Blake2bWrite::<_, vesta::Affine, Challenge255<_>>::init(vec![]);

    create_proof(
        &params,
        &pk,
        &[circuit.clone()],
        &[&[
            &public_inputs[0][..],
            &public_inputs[1][..],
            &public_inputs[2][..],
        ]],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation failed");

    let proof = transcript.finalize();
    let prove_time = prove_start.elapsed();

    println!("        Prove time: {:?}", prove_time);
    println!("        Proof size: {} bytes", proof.len());

    // =========================================
    // VERIFICATION
    // =========================================
    println!("  [4/4] Verification...");
    let verify_start = Instant::now();

    let mut transcript = Blake2bRead::<_, vesta::Affine, Challenge255<_>>::init(&proof[..]);

    let result = verify_proof(
        &params,
        &vk,
        SingleVerifier::new(&params),
        &[&[
            &public_inputs[0][..],
            &public_inputs[1][..],
            &public_inputs[2][..],
        ]],
        &mut transcript,
    );

    let verify_time = verify_start.elapsed();

    match result {
        Ok(_) => println!("        ✅ Proof verified!"),
        Err(e) => println!("        ❌ Verification failed: {:?}", e),
    }
    println!("        Verify time: {:?}", verify_time);

    // =========================================
    // SUMMARY
    // =========================================
    println!("\n  SUMMARY for N={}:", n_spins);
    println!("    Total time: {:?}", setup_time + keygen_time + prove_time + verify_time);
    println!("    Prove time: {:?}", prove_time);
    println!("    Proof size: {} bytes ({:.1} KB)", proof.len(), proof.len() as f64 / 1024.0);
    println!("    Verify time: {:?}", verify_time);
}