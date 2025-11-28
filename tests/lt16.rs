use halo2_proofs::dev::MockProver;
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::*,
};
use pasta_curves::pallas::Base as F;

use ising_verifier_final::chips::{Lt16Chip, Lt16Config};

// A tiny test circuit that uses the Lt16Chip on fixed lhs/rhs values.
#[derive(Clone)]
struct Lt16TestConfig {
    lt16: Lt16Config,
}

#[derive(Clone)]
struct Lt16TestCircuit {
    lhs: u16,
    rhs: u16,
}

impl Circuit<F> for Lt16TestCircuit {
    type Config = Lt16TestConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { lhs: 0, rhs: 0 }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let lt16 = Lt16Chip::configure(meta);
        Lt16TestConfig { lt16 }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "lt16 region",
            |mut region| {
                let chip = Lt16Chip::construct(config.lt16.clone());
                chip.assign_and_constrain(&mut region, 0, self.lhs, self.rhs)?;
                Ok(())
            },
        )
    }
}

#[test]
fn test_lt16_passes_when_lhs_less_than_rhs() {
    // lhs = 1234, rhs = 50000 → 1234 < 50000 should satisfy constraints.
    let circuit = Lt16TestCircuit { lhs: 1234, rhs: 50000 };

    let k = 8; // 2^8 = 256 rows, enough for this tiny circuit
    let public_inputs: Vec<Vec<F>> = vec![]; // no instance columns

    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    prover.assert_satisfied();
}

#[test]
fn test_lt16_fails_when_lhs_not_less_than_rhs() {
    // lhs = 50000, rhs = 1234 → 50000 < 1234 is false, constraints should fail.
    let circuit = Lt16TestCircuit { lhs: 50000, rhs: 1234 };

    let k = 8;
    let public_inputs: Vec<Vec<F>> = vec![];

    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    assert!(
        prover.verify().is_err(),
        "proof should not satisfy constraints when lhs >= rhs"
    );
}
