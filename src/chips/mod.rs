use halo2_proofs::{
    circuit::{AssignedCell, Region, Value},
    plonk::*,
    poly::Rotation,
};
use pasta_curves::pallas::Base as F;

// ===================================
// Stub LessThanChip (used by IsingCircuit)
// ===================================

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
        // NOTE: This does NOT enforce "a < b". It's just wiring.
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

// ===================================
// Real 8-bit Less-Than Comparator (Lt8Chip)
// ===================================

#[derive(Clone, Debug)]
pub struct Lt8Config {
    pub q:      Selector,
    pub lhs:    Column<Advice>,
    pub rhs:    Column<Advice>,
    pub d:      Column<Advice>,            // d = rhs - lhs - 1
    pub lhs_b:  [Column<Advice>; 8],
    pub rhs_b:  [Column<Advice>; 8],
    pub d_b:    [Column<Advice>; 8],
}

pub struct Lt8Chip {
    config: Lt8Config,
}

impl Lt8Chip {
    pub fn configure(meta: &mut ConstraintSystem<F>) -> Lt8Config {
        let q      = meta.selector();
        let lhs    = meta.advice_column();
        let rhs    = meta.advice_column();
        let d      = meta.advice_column();

        // 8 bit columns for lhs, rhs, and d each
        let lhs_b = [
            meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(),
            meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(),
        ];
        let rhs_b = [
            meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(),
            meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(),
        ];
        let d_b = [
            meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(),
            meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(),
        ];

        // Gate: for a single row, enforce:
        // - bits are 0 or 1
        // - lhs  = sum(lhs_b[i] * 2^i)
        // - rhs  = sum(rhs_b[i] * 2^i)
        // - d    = sum(d_b[i]   * 2^i)
        // - rhs = lhs + 1 + d
        meta.create_gate("lt8", |vc| {
            let q_expr = vc.query_selector(q);

            let lhs_expr = vc.query_advice(lhs, Rotation::cur());
            let rhs_expr = vc.query_advice(rhs, Rotation::cur());
            let d_expr   = vc.query_advice(d,   Rotation::cur());

            let zero = Expression::Constant(F::zero());
            let one  = Expression::Constant(F::one());

            // sum of bits * 2^i
            let mut lhs_sum = zero.clone();
            let mut rhs_sum = zero.clone();
            let mut d_sum   = zero.clone();

            let mut bit_constraints: Vec<Expression<F>> = Vec::new();

            for i in 0..8 {
                let coeff = Expression::Constant(F::from((1u64) << i));

                // LHS bits
                let lb = vc.query_advice(lhs_b[i], Rotation::cur());
                bit_constraints.push(lb.clone() * (lb.clone() - one.clone()));
                lhs_sum = lhs_sum + coeff.clone() * lb;

                // RHS bits
                let rb = vc.query_advice(rhs_b[i], Rotation::cur());
                bit_constraints.push(rb.clone() * (rb.clone() - one.clone()));
                rhs_sum = rhs_sum + coeff.clone() * rb;

                // D bits
                let db = vc.query_advice(d_b[i], Rotation::cur());
                bit_constraints.push(db.clone() * (db.clone() - one.clone()));
                d_sum = d_sum + coeff * db;
            }

            // Recomposition constraints
            let lhs_recomp = lhs_expr.clone() - lhs_sum;
            let rhs_recomp = rhs_expr.clone() - rhs_sum;
            let d_recomp   = d_expr.clone()   - d_sum;

            // Arithmetic relationship: rhs = lhs + 1 + d
            let one_expr = Expression::Constant(F::one());
            let rhs_minus_lhs_minus_one_minus_d =
                rhs_expr - lhs_expr - one_expr - d_expr;

            let mut constraints = vec![
                lhs_recomp,
                rhs_recomp,
                d_recomp,
                rhs_minus_lhs_minus_one_minus_d,
            ];
            constraints.extend(bit_constraints);

            Constraints::with_selector(q_expr, constraints)
        });

        Lt8Config {
            q,
            lhs,
            rhs,
            d,
            lhs_b,
            rhs_b,
            d_b,
        }
    }

    pub fn construct(config: Lt8Config) -> Self {
        Self { config }
    }

    /// Assign lhs, rhs as u8, compute d = rhs - lhs - 1, and enforce lhs < rhs.
    /// This only works correctly if 0 <= lhs, rhs <= 255.
    pub fn assign_and_constrain(
        &self,
        region: &mut Region<'_, F>,
        row: usize,
        lhs_val: u8,
        rhs_val: u8,
    ) -> Result<(), Error> {
        // Enable the comparator gate at this row
        self.config.q.enable(region, row)?;

        let lhs_f = F::from(lhs_val as u64);
        let rhs_f = F::from(rhs_val as u64);

        // Compute d = rhs - lhs - 1 as integer
        let d_int = (rhs_val as i32) - (lhs_val as i32) - 1;
        let d_u8 = ((d_int % 256) + 256) as u8; // wrap into 0..255
        let d_f = F::from(d_u8 as u64);

        // Assign main values
        region.assign_advice(
            || "lhs",
            self.config.lhs,
            row,
            || Value::known(lhs_f),
        )?;
        region.assign_advice(
            || "rhs",
            self.config.rhs,
            row,
            || Value::known(rhs_f),
        )?;
        region.assign_advice(
            || "d",
            self.config.d,
            row,
            || Value::known(d_f),
        )?;

        // Assign bits for lhs, rhs, d
        let mut lhs_bits = [0u8; 8];
        let mut rhs_bits = [0u8; 8];
        let mut d_bits   = [0u8; 8];

        for i in 0..8 {
            lhs_bits[i] = (lhs_val >> i) & 1;
            rhs_bits[i] = (rhs_val >> i) & 1;
            d_bits[i]   = (d_u8 >> i) & 1;
        }

        for i in 0..8 {
            region.assign_advice(
                || format!("lhs_bit_{}", i),
                self.config.lhs_b[i],
                row,
                || Value::known(F::from(lhs_bits[i] as u64)),
            )?;
            region.assign_advice(
                || format!("rhs_bit_{}", i),
                self.config.rhs_b[i],
                row,
                || Value::known(F::from(rhs_bits[i] as u64)),
            )?;
            region.assign_advice(
                || format!("d_bit_{}", i),
                self.config.d_b[i],
                row,
                || Value::known(F::from(d_bits[i] as u64)),
            )?;
        }

        Ok(())
    }
}
