//! Nova Ising Prover - Full Demo with Commitment Verification

use std::time::Instant;
use ff::Field;
use ising_nova::{
    IsingStepCircuit, IsingProofBundle, IsingNovaProver,
    E1, E2, F1, F2, BIAS, EDGES_PER_STEP,
    commit_ising_problem, commit_spins,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Nova Ising Prover - FULL DEMO WITH COMMITMENT");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    // ========================================================================
    // STEP 1: Define Problem
    // ========================================================================
    let n = 50_000;
    let deg = 12;
    
    print!("[1/6] Generating problem ({} spins, degree {})... ", n, deg);
    let t = Instant::now();
    let edges: Vec<(u32, u32, i64)> = (0..n).flat_map(|i| {
        (1..=deg/2).map(move |d| {
            let j = (i + d) % n;
            let w = ((i + j) % 19) as i64 - 9;
            (i as u32, j as u32, if w == 0 { 1 } else { w })
        })
    }).collect();
    let spins: Vec<u8> = (0..n).map(|i| (i % 2) as u8).collect();
    println!("{:?}", t.elapsed());
    println!("       {} edges, {} spins", edges.len(), spins.len());
    
    // ========================================================================
    // STEP 2: Create Prover & Compute Commitments
    // ========================================================================
    print!("\n[2/6] Computing commitments... ");
    let t = Instant::now();
    let prover = IsingNovaProver::new(edges.clone(), spins.clone());
    let bundle = prover.create_bundle();
    println!("{:?}", t.elapsed());
    
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│                    PROOF BUNDLE                             │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│  Problem commitment: 0x{}...  │", hex::encode(&bundle.problem_commitment[..16]));
    println!("│  Spin commitment:    0x{}...  │", hex::encode(&bundle.spin_commitment[..16]));
    println!("│  Claimed energy:     {:>10}                            │", bundle.claimed_energy);
    println!("│  N spins:            {:>10}                            │", bundle.n_spins);
    println!("│  N edges:            {:>10}                            │", bundle.n_edges);
    println!("└─────────────────────────────────────────────────────────────┘");
    
    // ========================================================================
    // STEP 3: Generate Step Circuits
    // ========================================================================
    print!("\n[3/6] Generating {} step circuits... ", prover.num_steps());
    let t = Instant::now();
    let circuits = prover.step_circuits();
    println!("{:?}", t.elapsed());
    
    // ========================================================================
    // STEP 4: Nova Setup & Proving
    // ========================================================================
    print!("[4/6] Nova setup... ");
    let t = Instant::now();
    let pp = PublicParams::<E1, E2, C1, C2>::setup(
        &circuits[0], &TrivialCircuit::default(),
        &*S1::ck_floor(), &*S2::ck_floor(),
    );
    println!("{:?}", t.elapsed());
    
    print!("[5/6] Recursive proving ({} steps)... ", circuits.len());
    let t = Instant::now();
    let z0 = IsingNovaProver::initial_state();
    let z0_sec = vec![F2::ZERO];
    let mut rs = RecursiveSNARK::<E1, E2, C1, C2>::new(
        &pp, &circuits[0], &TrivialCircuit::default(), &z0, &z0_sec,
    )?;
    for c in &circuits {
        rs.prove_step(&pp, c, &TrivialCircuit::default())?;
    }
    let rec_time = t.elapsed();
    rs.verify(&pp, circuits.len(), &z0, &z0_sec)?;
    println!("{:?} ✅", rec_time);
    
    // ========================================================================
    // STEP 5: Compress Proof
    // ========================================================================
    print!("[6/6] Compressing proof... ");
    let t = Instant::now();
    let (pk, vk) = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::setup(&pp)?;
    let compressed = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::prove(&pp, &pk, &rs)?;
    let comp_time = t.elapsed();
    println!("{:?}", comp_time);
    
    let proof_bytes = bincode::serialize(&compressed)?;
    let bundle_bytes = bincode::serialize(&bundle)?;
    
    print!("       Verifying... ");
    let t = Instant::now();
    compressed.verify(&vk, circuits.len(), &z0, &z0_sec)?;
    println!("{:?} ✅", t.elapsed());
    
    // ========================================================================
    // RESULTS
    // ========================================================================
    let total = rec_time.as_secs_f64() + comp_time.as_secs_f64();
    
    println!("\n╔═════════════════════════════════════════════════════════════╗");
    println!("║                      FINAL RESULTS                          ║");
    println!("╠═════════════════════════════════════════════════════════════╣");
    println!("║  Total prove time:     {:>8.2}s                            ║", total);
    println!("║  Proof size:           {:>8.1} KB                          ║", proof_bytes.len() as f64 / 1024.0);
    println!("║  Bundle size:          {:>8} bytes                        ║", bundle_bytes.len());
    println!("╚═════════════════════════════════════════════════════════════╝");
    
    // ========================================================================
    // COMMITMENT VERIFICATION DEMO
    // ========================================================================
    println!("\n╔═════════════════════════════════════════════════════════════╗");
    println!("║              COMMITMENT VERIFICATION DEMO                   ║");
    println!("╠═════════════════════════════════════════════════════════════╣");
    
    // Test 1: Correct problem
    let ok1 = bundle.verify_problem(n, &edges);
    println!("║  Same problem:         {}                              ║", 
             if ok1 { "✅ VALID" } else { "❌ INVALID" });
    
    // Test 2: Correct spins
    let ok2 = bundle.verify_spins(&spins);
    println!("║  Same spins:           {}                              ║",
             if ok2 { "✅ VALID" } else { "❌ INVALID" });
    
    // Test 3: Wrong problem (different edge)
    let mut wrong_edges = edges.clone();
    wrong_edges[0] = (0, 1, 999);  // Tampered weight
    let ok3 = bundle.verify_problem(n, &wrong_edges);
    println!("║  Tampered edge:        {}                            ║",
             if ok3 { "❌ FALSE POSITIVE!" } else { "✅ REJECTED" });
    
    // Test 4: Wrong spins
    let mut wrong_spins = spins.clone();
    wrong_spins[0] = 1 - wrong_spins[0];  // Flip one spin
    let ok4 = bundle.verify_spins(&wrong_spins);
    println!("║  Tampered spin:        {}                            ║",
             if ok4 { "❌ FALSE POSITIVE!" } else { "✅ REJECTED" });
    
    // Test 5: Completely different problem
    let tiny_edges = vec![(0u32, 1u32, 1i64)];
    let ok5 = bundle.verify_problem(2, &tiny_edges);
    println!("║  Different problem:    {}                            ║",
             if ok5 { "❌ FALSE POSITIVE!" } else { "✅ REJECTED" });
    
    println!("╚═════════════════════════════════════════════════════════════╝");
    
    if ok1 && ok2 && !ok3 && !ok4 && !ok5 {
        println!("\n🎉 SUCCESS: Proof is cryptographically bound to this problem!");
        println!("   A malicious prover cannot substitute a different problem.");
    } else {
        println!("\n❌ FAILURE: Commitment verification has issues!");
    }
    
    Ok(())
}
