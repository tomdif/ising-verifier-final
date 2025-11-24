use halo2_proofs::{
    circuit::{AssignedCell, Region},
    plonk::*,
    poly::Rotation,
};
use pasta_curves::pallas::Base as F;

#[derive(Clone, Debug)]
pub struct LessThanConfig {
    pub q_lt: Selector,
    pub a:    Column<Advice>,
    pub b:    Column<Advice>,
    pub diff: Column<Advice>,
}

pub struct LessThanChip {
    config: LessThanConfig,
}

impl LessThanChip {
    pub fn configure(meta: &mut ConstraintSystem<F>) -> LessThanConfig {
        let q_lt = meta.selector();
        let a    = meta.advice_column();
        let b    = meta.advice_column();
        let diff = meta.advice_column();

        // Enable equality on these columns because we use them with "copy_advice"
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(diff);

        // Stub: enforce diff = b - a when q_lt = 1.
        // NOTE: This does NOT yet enforce "a < b". It's a scaffold.
        meta.create_gate("a < b (stub diff constraint)", |vc| {
            let q   = vc.query_selector(q_lt);
            let a_v = vc.query_advice(a,    Rotation::cur());
            let b_v = vc.query_advice(b,    Rotation::cur());
            let d_v = vc.query_advice(diff, Rotation::cur());

            let diff_eq = d_v - (b_v - a_v);

            Constraints::with_selector(q, vec![diff_eq])
        });

        LessThanConfig { q_lt, a, b, diff }
    }

    pub fn construct(config: LessThanConfig) -> Self {
        Self { config }
    }

    pub fn enforce_less_than(
        &self,
        region: &mut Region<'_, F>,
        lhs: &AssignedCell<F, F>,
        rhs: &AssignedCell<F, F>,
    ) -> Result<(), Error> {
        // All values on row 0 of this region
        self.config.q_lt.enable(region, 0)?;

        // Copy lhs and rhs into comparator columns (requires equality enabled)
        lhs.copy_advice(|| "lhs", region, self.config.a, 0)?;
        rhs.copy_advice(|| "rhs", region, self.config.b, 0)?;

        // diff = rhs - lhs
        let diff_val = rhs
            .value()
            .zip(lhs.value())
            .map(|(r, l)| *r - *l);

        region.assign_advice(
            || "diff",
            self.config.diff,
            0,
            || diff_val,
        )?;

        Ok(())
    }
}
