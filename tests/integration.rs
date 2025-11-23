use halo2_proofs::dev::MockProver;
use pasta_curves::pallas::Base as F;

use ising_verifier_final::IsingCircuit;

// Must match BIAS in src/lib.rs
const BIAS: u64 = 1 << 50;

/// Simple sanity test:
/// - Build a tiny 3-spin Ising instance
/// - Run MockProver
/// - Ensure the circuit verifies structurally
#[test]
fn test_small_ising_mockprover_runs() {
    // Graph:
    //   0 --(+1)-- 1
    //   1 --(-1)-- 2
    //   0 --(+1)-- 2
    let edges = vec![
        (0u32, 1u32,  1i64),
        (1u32, 2u32, -1i64),
        (0u32, 2u32,  1i64),
    ];

    // Spins: [1, 0, 1]
    let spins = vec![1u8, 0u8, 1u8];

    let circuit = IsingCircuit {
        edges,
        spins,
        delta: 10,         // arbitrary gap
        threshold: -5,     // not enforced directly; public instance carries T+BIAS
        pubkey: F::one(),  // dummy pubkey
        nonce:  42,
    };

    // Instance column 0: threshold + BIAS
    let threshold_plus_bias = F::from(BIAS);

    // Instance column 1: cipher_root
    // encrypt_spins currently returns pubkey as "cipher root",
    // so we set it to F::one() to match the circuit.
    let cipher_root = F::one();

    let public_inputs = vec![
        vec![threshold_plus_bias], // for config.threshold
        vec![cipher_root],         // for config.cipher_root
    ];

    let k = 10; // 2^10 rows = 1024
    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();

    prover.assert_satisfied();
}
