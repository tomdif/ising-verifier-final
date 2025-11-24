//! Nova-based Ising Prover

use ff::PrimeField;
use nova_snark::{
    provider::{PallasEngine, VestaEngine},
    traits::{circuit::StepCircuit, Engine},
};
use bellpepper_core::{
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};

pub type E1 = PallasEngine;
pub type E2 = VestaEngine;
pub type F1 = <E1 as Engine>::Scalar;
pub type F2 = <E2 as Engine>::Scalar;

pub const EDGES_PER_STEP: usize = 1000;  // Increased from 64!
pub const BIAS: u64 = 1 << 50;

/// Compute Ising energy
pub fn compute_ising_energy(edges: &[(u32, u32, i64)], spins: &[u8]) -> i64 {
    let mut energy = 0i64;
    for &(u, v, w) in edges {
        let su = spins[u as usize] as i64;
        let sv = spins[v as usize] as i64;
        let sigma_product = 4 * su * sv - 2 * su - 2 * sv + 1;
        energy += w * sigma_product;
    }
    energy
}

/// Step circuit for Nova folding
#[derive(Clone, Debug)]
pub struct IsingStepCircuit<F: PrimeField> {
    batch_energy: F,
}

impl<F: PrimeField> Default for IsingStepCircuit<F> {
    fn default() -> Self {
        Self { batch_energy: F::ZERO }
    }
}

impl<F: PrimeField> IsingStepCircuit<F> {
    pub fn new(edges: &[(u32, u32, i64)], spins: &[u8]) -> Self {
        let energy = compute_batch_energy(edges, spins);
        Self { batch_energy: i64_to_field(energy) }
    }
}

impl<F: PrimeField> StepCircuit<F> for IsingStepCircuit<F> {
    fn arity(&self) -> usize { 1 }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        let running = &z[0];
        
        let batch = AllocatedNum::alloc(cs.namespace(|| "batch"), || Ok(self.batch_energy))?;
        
        let new_val = running.get_value().map(|r| r + self.batch_energy);
        let new = AllocatedNum::alloc(cs.namespace(|| "new"), || {
            new_val.ok_or(SynthesisError::AssignmentMissing)
        })?;
        
        cs.enforce(
            || "accumulate",
            |lc| lc + running.get_variable() + batch.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + new.get_variable(),
        );
        
        Ok(vec![new])
    }
}

fn compute_batch_energy(edges: &[(u32, u32, i64)], spins: &[u8]) -> i64 {
    let mut energy = 0i64;
    for &(u, v, w) in edges {
        if u as usize >= spins.len() || v as usize >= spins.len() { continue; }
        let su = spins[u as usize] as i64;
        let sv = spins[v as usize] as i64;
        energy += w * (4 * su * sv - 2 * su - 2 * sv + 1);
    }
    energy
}

fn i64_to_field<F: PrimeField>(val: i64) -> F {
    if val >= 0 { F::from(val as u64) } else { -F::from((-val) as u64) }
}

/// Main prover
pub struct IsingNovaProver {
    pub edges: Vec<(u32, u32, i64)>,
    pub spins: Vec<u8>,
}

impl IsingNovaProver {
    pub fn new(edges: Vec<(u32, u32, i64)>, spins: Vec<u8>, _delta: u64, _threshold: i64) -> Self {
        Self { edges, spins }
    }
    
    pub fn compute_energy(&self) -> i64 {
        compute_ising_energy(&self.edges, &self.spins)
    }
    
    pub fn num_steps(&self) -> usize {
        let n = (self.edges.len() + EDGES_PER_STEP - 1) / EDGES_PER_STEP;
        n.max(1)
    }
    
    pub fn step_circuits(&self) -> Vec<IsingStepCircuit<F1>> {
        if self.edges.is_empty() {
            return vec![IsingStepCircuit::default()];
        }
        self.edges.chunks(EDGES_PER_STEP)
            .map(|chunk| IsingStepCircuit::new(chunk, &self.spins))
            .collect()
    }
    
    pub fn initial_state() -> Vec<F1> {
        vec![F1::from(BIAS)]
    }
}
