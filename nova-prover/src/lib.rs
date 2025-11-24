//! Nova Ising Prover - Maximum GPU Acceleration with Pipelining

use ff::{Field, PrimeField};
use nova_snark::{
    provider::{PallasEngine, VestaEngine},
    traits::{circuit::StepCircuit, Engine},
};
use bellpepper_core::{
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use rayon::prelude::*;
use neptune::poseidon::PoseidonConstants;
use neptune::Poseidon;
use neptune::batch_hasher::Batcher;
use neptune::BatchHasher;
use generic_array::typenum::{U2, U4};
use generic_array::GenericArray;
use std::sync::OnceLock;
use std::time::Instant;

pub type E1 = PallasEngine;
pub type E2 = VestaEngine;
pub type F1 = <E1 as Engine>::Scalar;
pub type F2 = <E2 as Engine>::Scalar;

pub const EDGES_PER_STEP: usize = 100_000;
pub const BIAS: u64 = 1 << 50;

// Tuned for RTX 4060 Ti 16GB
const GPU_HASH_BATCH: usize = 8_000_000;   // 8M edges per batch
const GPU_REDUCE_BATCH: usize = 16_000_000; // 16M pairs per reduction batch

static POSEIDON_CONSTANTS_2: OnceLock<PoseidonConstants<F1, U2>> = OnceLock::new();
static POSEIDON_CONSTANTS_4: OnceLock<PoseidonConstants<F1, U4>> = OnceLock::new();

fn get_constants_2() -> &'static PoseidonConstants<F1, U2> {
    POSEIDON_CONSTANTS_2.get_or_init(|| PoseidonConstants::new())
}

fn get_constants_4() -> &'static PoseidonConstants<F1, U4> {
    POSEIDON_CONSTANTS_4.get_or_init(|| PoseidonConstants::new())
}

pub fn i64_to_field(val: i64) -> F1 {
    if val >= 0 { F1::from(val as u64) } else { -F1::from((-val) as u64) }
}

pub fn poseidon_hash_2(a: F1, b: F1) -> F1 {
    Poseidon::new_with_preimage(&[a, b], get_constants_2()).hash()
}

/// GPU-accelerated tree reduction with large batches
fn gpu_tree_reduce(mut elements: Vec<F1>) -> F1 {
    if elements.is_empty() { return F1::ZERO; }
    if elements.len() == 1 { return elements[0]; }
    
    let constants = get_constants_2();
    let mut round = 0;
    
    let gpu_available = elements.len() > 100_000 && 
        Batcher::<F1, U2>::pick_gpu(GPU_REDUCE_BATCH.min(elements.len() / 2)).is_ok();
    
    while elements.len() > 1 {
        round += 1;
        let pairs = elements.len() / 2;
        let has_odd = elements.len() % 2 == 1;
        
        if gpu_available && pairs > 50_000 {
            let mut next = Vec::with_capacity(pairs + if has_odd { 1 } else { 0 });
            
            for chunk_start in (0..pairs).step_by(GPU_REDUCE_BATCH) {
                let chunk_end = (chunk_start + GPU_REDUCE_BATCH).min(pairs);
                let chunk_pairs = chunk_end - chunk_start;
                
                let mut batcher = match Batcher::<F1, U2>::pick_gpu(chunk_pairs) {
                    Ok(b) => b,
                    Err(_) => {
                        let cpu_result: Vec<F1> = (chunk_start..chunk_end)
                            .into_par_iter()
                            .map(|i| Poseidon::new_with_preimage(&[elements[i*2], elements[i*2+1]], constants).hash())
                            .collect();
                        next.extend(cpu_result);
                        continue;
                    }
                };
                
                let preimages: Vec<GenericArray<F1, U2>> = (chunk_start..chunk_end)
                    .into_par_iter()
                    .map(|i| GenericArray::clone_from_slice(&[elements[i*2], elements[i*2+1]]))
                    .collect();
                
                match batcher.hash(&preimages[..]) {
                    Ok(hashes) => next.extend(hashes),
                    Err(_) => {
                        let cpu_result: Vec<F1> = (chunk_start..chunk_end)
                            .into_par_iter()
                            .map(|i| Poseidon::new_with_preimage(&[elements[i*2], elements[i*2+1]], constants).hash())
                            .collect();
                        next.extend(cpu_result);
                    }
                }
            }
            
            if has_odd {
                next.push(elements[elements.len() - 1]);
            }
            elements = next;
        } else {
            let mut next: Vec<F1> = (0..pairs)
                .into_par_iter()
                .map(|i| Poseidon::new_with_preimage(&[elements[i*2], elements[i*2+1]], constants).hash())
                .collect();
            
            if has_odd {
                next.push(elements[elements.len() - 1]);
            }
            elements = next;
        }
    }
    
    println!("    [REDUCE] {} rounds", round);
    elements[0]
}

/// Prep preimages from edge slice
fn prep_preimages(edges: &[(u32, u32, i64)]) -> Vec<GenericArray<F1, U4>> {
    edges.par_iter().map(|&(u, v, w)| {
        GenericArray::clone_from_slice(&[
            F1::from(u as u64), F1::from(v as u64), i64_to_field(w), F1::ZERO
        ])
    }).collect()
}

/// GPU hash with pipelining using rayon::join
fn gpu_hash_edges_pipelined(edges: &[(u32, u32, i64)]) -> Vec<F1> {
    let total = edges.len();
    let num_chunks = (total + GPU_HASH_BATCH - 1) / GPU_HASH_BATCH;
    
    let gpu_available = Batcher::<F1, U4>::pick_gpu(GPU_HASH_BATCH.min(total)).is_ok();
    
    if !gpu_available {
        println!("    [HASH] CPU fallback");
        let constants = get_constants_4();
        return edges.par_iter()
            .map(|&(u, v, w)| {
                Poseidon::new_with_preimage(&[
                    F1::from(u as u64), F1::from(v as u64), i64_to_field(w), F1::ZERO
                ], constants).hash()
            })
            .collect();
    }
    
    println!("    [HASH] GPU pipelined, {} chunks of {}M", num_chunks, GPU_HASH_BATCH / 1_000_000);
    
    let chunks: Vec<_> = edges.chunks(GPU_HASH_BATCH).collect();
    let mut all_hashes = Vec::with_capacity(total);
    
    // Prep first batch
    let mut current_preimages = prep_preimages(chunks[0]);
    
    for i in 0..num_chunks {
        // Pipeline: prep next while GPU hashes current
        let (hashes, next_preimages) = if i + 1 < num_chunks {
            let next_chunk = chunks[i + 1];
            rayon::join(
                || {
                    let mut batcher = Batcher::<F1, U4>::pick_gpu(current_preimages.len()).unwrap();
                    let h = batcher.hash(&current_preimages[..]).unwrap();
                    drop(batcher);
                    h
                },
                || prep_preimages(next_chunk)
            )
        } else {
            let mut batcher = Batcher::<F1, U4>::pick_gpu(current_preimages.len()).unwrap();
            let h = batcher.hash(&current_preimages[..]).unwrap();
            drop(batcher);
            (h, Vec::new())
        };
        
        all_hashes.extend(hashes);
        current_preimages = next_preimages;
        
        print!("\r    [HASH] {}/{}", i + 1, num_chunks);
        use std::io::Write;
        std::io::stdout().flush().ok();
    }
    println!(" done");
    
    all_hashes
}

/// Commit to Ising problem
pub fn commit_ising_problem(n_spins: usize, edges: &[(u32, u32, i64)]) -> F1 {
    let t0 = Instant::now();
    let edge_hashes = gpu_hash_edges_pipelined(edges);
    let hash_time = t0.elapsed();
    
    let t1 = Instant::now();
    let edges_commitment = gpu_tree_reduce(edge_hashes);
    let reduce_time = t1.elapsed();
    
    println!("    [TIME] Hash: {:.2}s, Reduce: {:.2}s, Total: {:.2}s", 
             hash_time.as_secs_f64(), reduce_time.as_secs_f64(),
             hash_time.as_secs_f64() + reduce_time.as_secs_f64());
    
    poseidon_hash_2(F1::from(n_spins as u64), edges_commitment)
}

/// Commit to spin configuration
pub fn commit_spins(spins: &[u8]) -> F1 {
    let packed: Vec<F1> = spins.par_chunks(64).map(|chunk| {
        let mut val: u64 = 0;
        for (i, &s) in chunk.iter().enumerate() { val |= (s as u64) << i; }
        F1::from(val)
    }).collect();
    
    gpu_tree_reduce(packed)
}

fn field_to_bytes(f: F1) -> [u8; 32] {
    let repr = f.to_repr();
    let mut out = [0u8; 32];
    out.copy_from_slice(repr.as_ref());
    out
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct IsingProofBundle {
    pub problem_commitment: [u8; 32],
    pub spin_commitment: [u8; 32],
    pub claimed_energy: i64,
    pub n_spins: usize,
    pub n_edges: usize,
}

impl IsingProofBundle {
    pub fn new(n_spins: usize, edges: &[(u32, u32, i64)], spins: &[u8], energy: i64) -> Self {
        Self {
            problem_commitment: field_to_bytes(commit_ising_problem(n_spins, edges)),
            spin_commitment: field_to_bytes(commit_spins(spins)),
            claimed_energy: energy,
            n_spins,
            n_edges: edges.len(),
        }
    }
    
    pub fn verify_problem(&self, n_spins: usize, edges: &[(u32, u32, i64)]) -> bool {
        field_to_bytes(commit_ising_problem(n_spins, edges)) == self.problem_commitment
    }
    
    pub fn verify_spins(&self, spins: &[u8]) -> bool {
        field_to_bytes(commit_spins(spins)) == self.spin_commitment
    }
}

pub fn compute_ising_energy(edges: &[(u32, u32, i64)], spins: &[u8]) -> i64 {
    edges.iter().map(|&(u, v, w)| {
        let su = spins[u as usize] as i64;
        let sv = spins[v as usize] as i64;
        w * (4 * su * sv - 2 * su - 2 * sv + 1)
    }).sum()
}

pub fn compute_ising_energy_parallel(edges: &[(u32, u32, i64)], spins: &[u8]) -> i64 {
    edges.par_iter().map(|&(u, v, w)| {
        let su = spins[u as usize] as i64;
        let sv = spins[v as usize] as i64;
        w * (4 * su * sv - 2 * su - 2 * sv + 1)
    }).sum()
}

#[derive(Clone, Debug)]
pub struct IsingStepCircuit<F: PrimeField> {
    pub batch_energy: F,
}

impl<F: PrimeField> Default for IsingStepCircuit<F> {
    fn default() -> Self { Self { batch_energy: F::ZERO } }
}

impl<F: PrimeField> IsingStepCircuit<F> {
    pub fn new(energy: i64) -> Self {
        Self { batch_energy: Self::i64_to_f(energy) }
    }
    fn i64_to_f(val: i64) -> F {
        if val >= 0 { F::from(val as u64) } else { -F::from((-val) as u64) }
    }
}

impl<F: PrimeField> StepCircuit<F> for IsingStepCircuit<F> {
    fn arity(&self) -> usize { 1 }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self, cs: &mut CS, z: &[AllocatedNum<F>],
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

pub struct IsingNovaProver {
    pub edges: Vec<(u32, u32, i64)>,
    pub spins: Vec<u8>,
    pub n_spins: usize,
}

impl IsingNovaProver {
    pub fn new(edges: Vec<(u32, u32, i64)>, spins: Vec<u8>) -> Self {
        let n_spins = spins.len();
        Self { edges, spins, n_spins }
    }
    
    pub fn num_steps(&self) -> usize {
        ((self.edges.len() + EDGES_PER_STEP - 1) / EDGES_PER_STEP).max(1)
    }
    
    pub fn step_circuits(&self) -> Vec<IsingStepCircuit<F1>> {
        if self.edges.is_empty() { return vec![IsingStepCircuit::default()]; }
        let energies: Vec<i64> = self.edges.par_chunks(EDGES_PER_STEP)
            .map(|chunk| compute_ising_energy(chunk, &self.spins))
            .collect();
        energies.into_iter().map(IsingStepCircuit::new).collect()
    }
    
    pub fn total_energy(&self) -> i64 {
        compute_ising_energy_parallel(&self.edges, &self.spins)
    }
    
    pub fn create_bundle(&self) -> IsingProofBundle {
        IsingProofBundle::new(self.n_spins, &self.edges, &self.spins, self.total_energy())
    }
    
    pub fn initial_state() -> Vec<F1> { vec![F1::from(BIAS)] }
}
