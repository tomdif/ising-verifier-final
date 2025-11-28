use std::time::Instant;

use halo2_proofs::{
    plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier},
    poly::commitment::Params,
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use pasta_curves::{pallas, vesta};
use rand::rngs::OsRng;

use ising_verifier_final::{compute_ising_energy, compute_public_inputs, IsingCircuit};

type F = pallas::Base;

fn main() {
    println!("===========================================");
    println!("  Halo2 Real Proof Benchmark");
    println!("===========================================\n");

    for n_spins in [10, 50, 100, 200] {
        run_benchmark(n_spins);
        println!();
    }
}

fn run_benchmark(n_spins: usize) {
    println!("--- N = {} spins ---", n_spins);

    let edges: Vec<(u32, u32, i64)> = (0..n_spins - 1)
        .map(|i| (i as u32, (i + 1) as u32, 1i64))
        .collect();

    let spins: Vec<u8> = (0..n_spins).map(|i| (i % 2) as u8).collect();
    let delta = 100u64;
    let nonce = 12345u64;

    let energy = compute_ising_energy(&edges, &spins);
    let threshold = energy + (delta as i64) + 1000;

    println!("  Edges: {}", edges.len());
    println!("  Energy: {}", energy);

    let circuit = IsingCircuit {
        edges: edges.clone(),
        spins: spins.clone(),
        delta,
        threshold,
        pubkey: F::one(),
        nonce,
    };

    let (threshold_plus_bias, cipher_root) = compute_public_inputs(threshold, &spins, nonce);
    let public_inputs = vec![
        vec![threshold_plus_bias],
        vec![cipher_root],
        vec![F::zero()],
    ];

    let k = if n_spins <= 100 { 12 } else if n_spins <= 500 { 14 } else { 16 };
    println!("  k = {} ({} rows)", k, 1 << k);

    println!("\n  [1/4] Setup...");
    let t = Instant::now();
    let params: Params<vesta::Affine> = Params::new(k);
    println!("        {:?}", t.elapsed());

    println!("  [2/4] Keygen...");
    let t = Instant::now();
    let vk = keygen_vk(&params, &circuit).expect("keygen_vk failed");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("keygen_pk failed");
    println!("        {:?}", t.elapsed());

    println!("  [3/4] Prove...");
    let t = Instant::now();
    let mut transcript = Blake2bWrite::<_, vesta::Affine, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        &pk,
        &[circuit.clone()],
        &[&[&public_inputs[0][..], &public_inputs[1][..], &public_inputs[2][..]]],
        OsRng,
        &mut transcript,
    )
    .expect("proof generation failed");
    let proof = transcript.finalize();
    let prove_time = t.elapsed();
    println!("        {:?}", prove_time);
    println!("        Proof size: {} bytes", proof.len());

    println!("  [4/4] Verify...");
    let t = Instant::now();
    let mut transcript = Blake2bRead::<_, vesta::Affine, Challenge255<_>>::init(&proof[..]);
    let result = verify_proof(
        &params,
        &vk,
        SingleVerifier::new(&params),
        &[&[&public_inputs[0][..], &public_inputs[1][..], &public_inputs[2][..]]],
        &mut transcript,
    );
    println!("        {:?}", t.elapsed());
    match result {
        Ok(_) => println!("        ✅ Verified!"),
        Err(e) => println!("        ❌ Failed: {:?}", e),
    }

    println!("\n  SUMMARY: Prove={:?}, Size={} bytes", prove_time, proof.len());
}
