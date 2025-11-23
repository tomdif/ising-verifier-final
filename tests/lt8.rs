use halo2_proofs::dev::MockProver;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::*,
};
use pasta_curves::pallas::Base as F;

use ising_verifier_final::chips::{Lt8Chip, Lt8Config};

// A tiny test circuit that uses the Lt8Chip on fixed lhs/rhs values.
#[derive(Clone)]
struct Lt8TestConfig {
    lt8: Lt8Config,
}

#[derive(Clone)]
struct Lt8TestCircuit {
    lhs: u8,
    rhs: u8,
}

impl Circuit<F> for Lt8TestCircuit {
    type Config = Lt8TestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { lhs: 0, rhs: 0 }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let lt8 = Lt8Chip::configure(meta);
        Lt8TestConfig { lt8 }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "lt8 region",
            |mut region| {
                let chip = Lt8Chip::construct(config.lt8.clone());
                chip.assign_and_constrain(&mut region, 0, self.lhs, self.rhs)?;
                Ok(())
            },
        )
    }
}

#[test]
fn test_lt8_passes_when_lhs_less_than_rhs() {
    // lhs = 5, rhs = 10 → 5 < 10 should satisfy constraints.
    let circuit = Lt8TestCircuit { lhs: 5, rhs: 10 };

    let k = 6; // small circuit: 2^6 = 64 rows
    let public_inputs: Vec<Vec<F>> = vec![]; // no instance columns

    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    prover.assert_satisfied();
}

#[test]
fn test_lt8_fails_when_lhs_not_less_than_rhs() {
    // lhs = 12, rhs = 7 → 12 < 7 is false, constraints should fail.
    let circuit = Lt8TestCircuit { lhs: 12, rhs: 7 };

    let k = 6;
    let public_inputs: Vec<Vec<F>> = vec![];

    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    assert!(
        prover.verify().is_err(),
        "proof should not satisfy constraints when lhs >= rhs"
    );
}
