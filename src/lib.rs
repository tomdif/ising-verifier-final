use ff::PrimeField;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::*,
    poly::Rotation,
};
use pasta_curves::pallas::Base as F;
use poseidon_halo2::{PoseidonChip, PoseidonConfig as PoseidonGadgetConfig};

mod chips;
use chips::{LessThanChip, LessThanConfig};

const BIAS: u64 = 1 << 50; // Makes all energies positive

#[derive(Clone, Debug)]
pub struct IsingConfig {
    spin_idx: Column<Advice>, spin_val: Column<Advice>,
    edge_u: Column<Advice>, edge_v: Column<Advice>, edge_w: Column<Advice>,
    su: Column<Advice>, sv: Column<Advice>,
    term: Column<Advice>, energy: Column<Advice>,
    q_spin: Selector, q_edge: Selector,
    threshold: Column<Instance>, cipher_root: Column<Instance>,
    table_idx: TableColumn, table_spin: TableColumn,
    lt_config: LessThanConfig,
    poseidon: PoseidonGadgetConfig<9, 8, 57>,
}

#[derive(Clone)]
pub struct IsingCircuit {
    pub edges: Vec<(u32, u32, i64)>,
    pub spins: Vec<u8>,
    pub delta: u64,
    pub threshold: i64,
    pub pubkey: F,
    pub nonce: u64,
}

impl Circuit<F> for IsingCircuit {
    type Config = IsingConfig;
    type FloorPlanner = SimpleFloorPlanner;
    // ... (without_witnesses omitted for brevity)

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // ... (full configure from final version — abbreviated here)
        // You can paste the full one from my last message — it works
        unimplemented!("Full configure will be in final repo")
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        unimplemented!("Full synthesize will be in final repo")
    }
}
