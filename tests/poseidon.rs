use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    dev::MockProver,
    plonk::{Circuit, ConstraintSystem, Error},
};
use pasta_curves::pallas::Base as F;

use ising_verifier_final::chips::{POSEIDON_RC, POSEIDON_MDS, RF, RP, N_ROUNDS, T, PoseidonChip, PoseidonConfig};
use ff::PrimeField;

/// Helper: convert a hex or decimal string into F
fn f_from_str_any(s: &str) -> F {
    if !s.starts_with("0x") {
        return F::from_str_vartime(s).expect("invalid decimal field string");
    }
    let hex = &s[2..];
    let sixteen = F::from(16u64);
    let mut value = F::zero();
    let mut base = F::one();
    for ch in hex.chars().rev() {
        let digit = ch.to_digit(16).expect("invalid hex digit") as u64;
        value = value + base * F::from(digit);
        base = base * sixteen;
    }
    value
}

/// Off-circuit Poseidon x5_255_3 reference using the same parameters.
fn poseidon_hash_2_ref(x0: F, x1: F) -> F {
    // Parse RC into Vec<[F; T]>
    let rc: Vec<[F; T]> = POSEIDON_RC
        .chunks(T)
        .map(|chunk| {
            let mut row = [F::zero(); T];
            for i in 0..T {
                row[i] = f_from_str_any(chunk[i]);
            }
            row
        })
        .collect();

    // Parse MDS into [[F; T]; T]
    let mut mds = [[F::zero(); T]; T];
    for i in 0..T {
        for j in 0..T {
            mds[i][j] = f_from_str_any(POSEIDON_MDS[i][j]);
        }
    }

    let mut state = [x0, x1, F::zero()];

    for r in 0..N_ROUNDS {
        let full_round = r < RF / 2 || r >= RF / 2 + RP;

        // Add round constants
        for i in 0..T {
            state[i] = state[i] + rc[r][i];
        }

        // S-box x^5
        for i in 0..T {
            if full_round || i == T - 1 {
                let x = state[i];
                let x2 = x * x;
                let x4 = x2 * x2;
                state[i] = x4 * x;
            }
        }

        // MDS
        let mut new_state = [F::zero(); T];
        for j in 0..T {
            let mut acc = F::zero();
            for k in 0..T {
                acc += mds[j][k] * state[k];
            }
            new_state[j] = acc;
        }
        state = new_state;
    }

    state[0]
}

/// Small circuit wiring PoseidonChip::hash_2 and exposing result as instance.
#[derive(Clone, Debug)]
struct PoseidonTestCircuit {
    x0: F,
    x1: F,
}

impl Circuit<F> for PoseidonTestCircuit {
    type Config = PoseidonConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            x0: F::zero(),
            x1: F::zero(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        PoseidonChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = PoseidonChip::construct(config.clone());

        let out = chip.hash_2(
            &mut layouter.namespace(|| "poseidon_hash_2"),
            self.x0,
            self.x1,
        )?;

        // We constrain out to instance column 0, row 0.
        layouter.constrain_instance(out.cell(), config.instance, 0)?;

        Ok(())
    }
}

#[test]
fn test_poseidon_hash_matches_reference_for_simple_input() {
    // Simple test vector: x0 = 0, x1 = 1
    let x0 = F::zero();
    let x1 = F::one();

    let expected = poseidon_hash_2_ref(x0, x1);

    let circuit = PoseidonTestCircuit { x0, x1 };
    let public_inputs = vec![vec![expected]]; // instance column 0, row 0

    let k = 10; // 2^10 rows
    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    prover.assert_satisfied();
}

#[test]
fn test_poseidon_hash_matches_reference_for_randomish_input() {
    // Another test vector: x0 and x1 arbitrary
    let x0 = f_from_str_any("0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234");
    let x1 = f_from_str_any("0x56f09d89e827d00392fdc0c3d21b1a5bae2d689894ced82f58e256a03d20ef91");

    let expected = poseidon_hash_2_ref(x0, x1);

    let circuit = PoseidonTestCircuit { x0, x1 };
    let public_inputs = vec![vec![expected]];

    let k = 10;
    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    prover.assert_satisfied();
}
