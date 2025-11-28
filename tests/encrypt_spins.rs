use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    dev::MockProver,
    plonk::*,
};
use pasta_curves::pallas::Base as F;

use ising_verifier_final::chips::{PoseidonChip, PoseidonConfig};

#[derive(Clone)]
struct TestConfig {
    poseidon: PoseidonConfig,
}

#[derive(Clone)]
struct PoseidonTestCircuit {
    pub x0: F,
    pub x1: F,
}

impl Circuit<F> for PoseidonTestCircuit {
    type Config = TestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            x0: F::zero(),
            x1: F::zero(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // We only use the Poseidon chip; its config already includes an instance column.
        let poseidon = PoseidonChip::configure(meta);
        TestConfig { poseidon }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // Use the Poseidon chip in-circuit
        let chip = PoseidonChip::construct(config.poseidon.clone());

        let out = chip.hash_2(
            &mut layouter.namespace(|| "poseidon_hash_2"),
            self.x0,
            self.x1,
        )?;

        // Expose the result as a public instance. We use the instance column
        // built into PoseidonConfig.
        layouter.constrain_instance(out.cell(), config.poseidon.instance, 0)?;

        Ok(())
    }
}

// Reference Poseidon hash that must match the chip
fn poseidon_hash_2_ref(x0: F, x1: F) -> F {
    // This helper must be defined as `pub fn poseidon_reference_hash` in src/chips/mod.rs,
    // and should use the same POSEIDON_RC / POSEIDON_MDS / RF / RP / N_ROUNDS.
    ising_verifier_final::chips::poseidon_reference_hash(x0, x1)
}

#[test]
fn test_encrypt_spins_matches_poseidon_reference() {
    // Arbitrary test inputs
    let x0 = F::from(123456u64);
    let x1 = F::from(999999u64);

    // Expected result from the native reference implementation
    let expected = poseidon_hash_2_ref(x0, x1);

    // Build the circuit
    let circuit = PoseidonTestCircuit { x0, x1 };

    // One instance column, one row: expected hash
    let public_inputs = vec![vec![expected]];

    let k = 10;
    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();

    prover.assert_satisfied();
}
