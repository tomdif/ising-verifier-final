use ff::PrimeField;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::*,
    poly::Rotation,
};
use pasta_curves::pallas::Base as F;

mod chips;
use chips::{LessThanChip, LessThanConfig};

const BIAS: u64 = 1 << 50; // shift energies into positive range for unsigned comparison

#[derive(Clone, Debug)]
pub struct IsingConfig {
    // Advice columns
    spin_idx: Column<Advice>,
    spin_val: Column<Advice>,
    edge_u:   Column<Advice>,
    edge_v:   Column<Advice>,
    edge_w:   Column<Advice>,
    su:       Column<Advice>,
    sv:       Column<Advice>,
    term:     Column<Advice>,
    energy:   Column<Advice>,

    // Selectors
    q_spin: Selector,
    q_edge: Selector,

    // Instance columns (public inputs)
    threshold:   Column<Instance>, // T + BIAS
    cipher_root: Column<Instance>,

    // Lookup table for spins: (index, spin)
    table_idx:  TableColumn,
    table_spin: TableColumn,

    // Gadgets
    lt_config: LessThanConfig,
}

#[derive(Clone)]
pub struct IsingCircuit {
    pub edges:     Vec<(u32, u32, i64)>, // (u, v, w)
    pub spins:     Vec<u8>,              // spins in {0,1}
    pub delta:     u64,                  // gap ≥ 0
    pub threshold: i64,                  // off-chain metadata (not enforced yet)
    pub pubkey:    F,
    pub nonce:     u64,
}

impl Circuit<F> for IsingCircuit {
    type Config = IsingConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            edges: self.edges.clone(),
            spins: vec![0; self.spins.len()],
            delta: 0,
            threshold: self.threshold,
            pubkey: F::zero(),
            nonce: 0,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // Advice columns
        let spin_idx = meta.advice_column();
        let spin_val = meta.advice_column();
        let edge_u   = meta.advice_column();
        let edge_v   = meta.advice_column();
        let edge_w   = meta.advice_column();
        let su       = meta.advice_column();
        let sv       = meta.advice_column();
        let term     = meta.advice_column();
        let energy   = meta.advice_column();

        // Selectors
        let q_spin = meta.selector();
        let q_edge = meta.selector();

        // Instance columns
        let threshold   = meta.instance_column();
        let cipher_root = meta.instance_column();
        meta.enable_equality(threshold);
        meta.enable_equality(cipher_root);
        meta.enable_equality(energy);

        // Lookup table columns for spin (idx, val)
        let table_idx  = meta.lookup_table_column();
        let table_spin = meta.lookup_table_column();

        // 1. Enforce spins are binary: s ∈ {0,1}
        meta.create_gate("spin ∈ {0,1}", |vc| {
            let q = vc.query_selector(q_spin);
            let s = vc.query_advice(spin_val, Rotation::cur());
            Constraints::with_selector(q, vec![s.clone() * (s - F::one())])
        });

        // 2. Ising interaction + energy accumulator
        meta.create_gate("ising edge", |vc| {
            let q      = vc.query_selector(q_edge);
            let su_v   = vc.query_advice(su,    Rotation::cur());
            let sv_v   = vc.query_advice(sv,    Rotation::cur());
            let w_v    = vc.query_advice(edge_w,Rotation::cur());
            let term_v = vc.query_advice(term,  Rotation::cur());
            let e_prev = vc.query_advice(energy,Rotation::prev());
            let e_cur  = vc.query_advice(energy,Rotation::cur());

            let four = F::from(4);
            let two  = F::from(2);
            let expr = four * su_v.clone() * sv_v.clone()
                     - two * su_v.clone()
                     - two * sv_v.clone()
                     + F::one();

            let term_calc   = term_v.clone() - (w_v * expr);
            let energy_calc = e_cur - (e_prev + term_v);

            Constraints::with_selector(q, vec![term_calc, energy_calc])
        });

        // 3. Lookups: (edge_u, su) and (edge_v, sv) must be in spin table
        meta.lookup("spin lookup u", |meta| {
            let q   = meta.query_selector(q_edge);
            let idx = meta.query_advice(edge_u, Rotation::cur());
            let sp  = meta.query_advice(su,     Rotation::cur());
            vec![(q.clone() * idx, table_idx), (q * sp, table_spin)]
        });

        meta.lookup("spin lookup v", |meta| {
            let q   = meta.query_selector(q_edge);
            let idx = meta.query_advice(edge_v, Rotation::cur());
            let sp  = meta.query_advice(sv,     Rotation::cur());
            vec![(q.clone() * idx, table_idx), (q * sp, table_spin)]
        });

        let lt_config = LessThanChip::configure(meta);

        IsingConfig {
            spin_idx,
            spin_val,
            edge_u,
            edge_v,
            edge_w,
            su,
            sv,
            term,
            energy,
            q_spin,
            q_edge,
            threshold,
            cipher_root,
            table_idx,
            table_spin,
            lt_config,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // 0. Load spin table (with dummy row)
        self.load_spin_table(&config, &mut layouter)?;

        // 1. Layout spins + edges + energy
        let (spin_cells, final_energy) =
            self.layout_spins_and_edges(&config, layouter.namespace(|| "ising"))?;

        // 2. Enforce gap-hiding: E + Δ ≤ T (via BIAS-shifted comparator)
        self.check_gap_hiding(&config, layouter.namespace(|| "gap-hiding"), final_energy)?;

        // 3. Dummy cipher root: just return pubkey as “root” for now
        let cipher_root =
            self.encrypt_spins(&config, layouter.namespace(|| "encryption"), &spin_cells)?;

        layouter.constrain_instance(cipher_root.cell(), config.cipher_root, 0)?;
        Ok(())
    }
}

impl IsingCircuit {
    fn load_spin_table(
        &self,
        config: &IsingConfig,
        layouter: &mut impl Layouter<F>,
    ) -> Result<(), Error> {
        layouter.assign_table(
            || "spin table",
            |mut table| {
                // Dummy (0,0) row for q=0 lookups
                table.assign_cell(
                    || "idx_dummy",
                    config.table_idx,
                    0,
                    || Value::known(F::zero()),
                )?;
                table.assign_cell(
                    || "spin_dummy",
                    config.table_spin,
                    0,
                    || Value::known(F::zero()),
                )?;

                for (i, &s) in self.spins.iter().enumerate() {
                    let row = i + 1;
                    table.assign_cell(
                        || "idx",
                        config.table_idx,
                        row,
                        || Value::known(F::from(i as u64)),
                    )?;
                    table.assign_cell(
                        || "spin",
                        config.table_spin,
                        row,
                        || Value::known(F::from(s as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }

    fn layout_spins_and_edges(
        &self,
        config: &IsingConfig,
        mut layouter: impl Layouter<F>,
    ) -> Result<(Vec<AssignedCell<F, F>>, AssignedCell<F, F>), Error> {
        layouter.assign_region(
            || "ising",
            |mut region| {
                let mut spin_cells = Vec::with_capacity(self.spins.len());

                // Phase 1: spins at rows 0..N-1
                for (i, &s) in self.spins.iter().enumerate() {
                    config.q_spin.enable(&mut region, i)?;
                    region.assign_advice(
                        || "spin_idx",
                        config.spin_idx,
                        i,
                        || Value::known(F::from(i as u64)),
                    )?;
                    let cell = region.assign_advice(
                        || "spin_val",
                        config.spin_val,
                        i,
                        || Value::known(F::from(s as u64)),
                    )?;
                    spin_cells.push(cell);
                }

                let start = self.spins.len();

                // E0 + BIAS at row = start
                let mut energy = region.assign_advice(
                    || "E0+BIAS",
                    config.energy,
                    start,
                    || Value::known(F::from(BIAS)),
                )?;

                let mut row = start;

                // Phase 2: edges at rows start+1..start+M
                for &(u, v, w_raw) in &self.edges {
                    row += 1;
                    config.q_edge.enable(&mut region, row)?;

                    let w = if w_raw < 0 {
                        -F::from(w_raw.unsigned_abs())
                    } else {
                        F::from(w_raw as u64)
                    };

                    region.assign_advice(
                        || "u",
                        config.edge_u,
                        row,
                        || Value::known(F::from(u as u64)),
                    )?;
                    region.assign_advice(
                        || "v",
                        config.edge_v,
                        row,
                        || Value::known(F::from(v as u64)),
                    )?;
                    region.assign_advice(
                        || "w",
                        config.edge_w,
                        row,
                        || Value::known(w),
                    )?;

                    // Copy spins into su/sv
                    let _su_cell = spin_cells[u as usize].copy_advice(
                        || "su",
                        &mut region,
                        config.su,
                        row,
                    )?;
                    let _sv_cell = spin_cells[v as usize].copy_advice(
                        || "sv",
                        &mut region,
                        config.sv,
                        row,
                    )?;

                    // Compute term and new energy as witness
                    let su_val = F::from(self.spins[u as usize] as u64);
                    let sv_val = F::from(self.spins[v as usize] as u64);
                    let expr = F::from(4) * su_val * sv_val
                             - F::from(2) * su_val
                             - F::from(2) * sv_val
                             + F::one();
                    let term_val = w * expr;

                    region.assign_advice(
                        || "term",
                        config.term,
                        row,
                        || Value::known(term_val),
                    )?;
                    energy = region.assign_advice(
                        || "energy",
                        config.energy,
                        row,
                        || energy.value().map(|e| *e + term_val),
                    )?;
                }

                Ok((spin_cells, energy))
            },
        )
    }

    fn check_gap_hiding(
        &self,
        config: &IsingConfig,
        mut layouter: impl Layouter<F>,
        final_energy: AssignedCell<F, F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "gap-hiding",
            |mut region| {
                // Public instance: T+BIAS
                let threshold = region.assign_advice_from_instance(
                    || "T+BIAS",
                    config.threshold,
                    0,
                    config.energy,
                    0,
                )?;

                // final_energy is E+BIAS, so lhs = (E+BIAS) + delta
                let delta_f = F::from(self.delta);
                let lhs_val = final_energy.value().map(|e| *e + delta_f);

                let lhs_cell = region.assign_advice(
                    || "E+Δ+BIAS",
                    config.energy,
                    1,
                    || lhs_val,
                )?;

                let lt_chip = LessThanChip::construct(config.lt_config.clone());
                lt_chip.enforce_less_than(&mut region, &lhs_cell, &threshold)
            },
        )
    }

    fn encrypt_spins(
        &self,
        config: &IsingConfig,
        mut layouter: impl Layouter<F>,
        _spin_cells: &[AssignedCell<F, F>],
    ) -> Result<AssignedCell<F, F>, Error> {
        // For now, just return pubkey as “cipher root”
        layouter.assign_region(
            || "cipher root",
            |mut region| {
                let cell = region.assign_advice(
                    || "cipher_root",
                    config.energy, // any advice column is fine
                    0,
                    || Value::known(self.pubkey),
                )?;
                Ok(cell)
            },
        )
    }
}
