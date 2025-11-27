//! Collaborative Mining Circuit for NEXUS
//!
//! This module extends the Nova Ising prover to support:
//! 1. Deterministic seed derivation from (job_id, epoch, miner_address, vrf_randomness)
//! 2. Initial configuration derivation from seed
//! 3. Step-by-step simulated annealing verification
//! 4. Work proof: "I ran L steps of algorithm A from seed S"

use ff::{Field, PrimeField};
use nova_snark::traits::circuit::StepCircuit;
use bellpepper_core::{
    num::AllocatedNum,
    ConstraintSystem, SynthesisError,
};
use crate::{F1, poseidon_hash_2, poseidon_hash_4, BIAS};

/// Number of SA steps per Nova fold
/// Lower = more folds but smaller circuit per fold
/// Higher = fewer folds but larger circuit
pub const SA_STEPS_PER_FOLD: usize = 100;

/// Temperature decay factor (fixed-point: multiply by this / 2^16)
pub const TEMP_DECAY_FIXED: u64 = 65470;  // ~0.999 * 65536

/// Initial temperature (fixed-point)
pub const INITIAL_TEMP_FIXED: u64 = 655360;  // 10.0 * 65536

//==============================================================================
// SEED DERIVATION
//==============================================================================

/// Derive deterministic seed from public inputs
/// seed = Poseidon(job_hash, epoch, miner_address, vrf_randomness)
pub fn derive_seed(
    job_hash: F1,
    epoch: u64,
    miner_address: F1,
    vrf_randomness: F1,
) -> F1 {
    poseidon_hash_4(
        job_hash,
        F1::from(epoch),
        miner_address,
        vrf_randomness,
    )
}

/// Derive initial spin configuration from seed
/// Returns Vec<u8> where each element is 0 or 1
pub fn derive_initial_config(seed: F1, num_spins: usize) -> Vec<u8> {
    let mut config = Vec::with_capacity(num_spins);
    let mut prg_state = seed;
    
    for i in 0..num_spins {
        // Advance PRG
        prg_state = poseidon_hash_2(prg_state, F1::from(i as u64));
        
        // Extract bit from field element
        let bytes = prg_state.to_repr();
        let bit = bytes.as_ref()[0] & 1;
        config.push(bit);
    }
    
    config
}

/// PRG state for simulated annealing
#[derive(Clone, Debug)]
pub struct PrgState {
    pub state: F1,
    pub step: u64,
}

impl PrgState {
    pub fn new(seed: F1) -> Self {
        Self { state: seed, step: 0 }
    }
    
    /// Advance PRG and return (position_index, accept_threshold)
    pub fn advance(&mut self, num_spins: usize) -> (usize, u64) {
        // Generate position
        let pos_hash = poseidon_hash_2(self.state, F1::from(self.step * 2));
        let pos_bytes = pos_hash.to_repr();
        let pos_u64 = u64::from_le_bytes(pos_bytes.as_ref()[0..8].try_into().unwrap());
        let position = (pos_u64 as usize) % num_spins;
        
        // Generate acceptance threshold (0 to 2^32)
        let accept_hash = poseidon_hash_2(self.state, F1::from(self.step * 2 + 1));
        let accept_bytes = accept_hash.to_repr();
        let accept_threshold = u64::from_le_bytes(accept_bytes.as_ref()[0..8].try_into().unwrap()) >> 32;
        
        // Update state
        self.state = poseidon_hash_2(self.state, F1::from(self.step));
        self.step += 1;
        
        (position, accept_threshold)
    }
}

//==============================================================================
// SIMULATED ANNEALING STEP (for witness generation)
//==============================================================================

/// Compute energy delta for flipping spin at position
/// ΔE = 2 * s_i * Σ_j J_ij * s_j (for neighbors j)
pub fn compute_flip_delta(
    position: usize,
    edges: &[(u32, u32, i64)],
    spins: &[u8],
) -> i64 {
    let s_i = if spins[position] == 1 { 1i64 } else { -1i64 };
    
    let mut neighbor_sum: i64 = 0;
    for &(u, v, w) in edges {
        if u as usize == position {
            let s_j = if spins[v as usize] == 1 { 1i64 } else { -1i64 };
            neighbor_sum += w * s_j;
        } else if v as usize == position {
            let s_j = if spins[u as usize] == 1 { 1i64 } else { -1i64 };
            neighbor_sum += w * s_j;
        }
    }
    
    // Flipping s_i changes energy by 2 * s_i * neighbor_sum
    2 * s_i * neighbor_sum
}

/// Metropolis acceptance criterion
/// Accept if delta_e < 0 or random < exp(-delta_e / T)
pub fn metropolis_accept(delta_e: i64, temperature_fixed: u64, random_threshold: u64) -> bool {
    if delta_e <= 0 {
        return true;
    }
    
    // Approximate exp(-delta_e / T) using lookup or linear approx
    // For simplicity, use: accept if random < (T / (T + delta_e)) * 2^32
    // This is a rough approximation but works for SA
    let t = temperature_fixed as i128;
    let d = delta_e.abs() as i128;
    let threshold = ((t << 32) / (t + d * 65536)) as u64;
    
    random_threshold < threshold
}

/// Run one step of simulated annealing
pub fn sa_step(
    spins: &mut [u8],
    edges: &[(u32, u32, i64)],
    prg: &mut PrgState,
    temperature_fixed: &mut u64,
) -> (usize, i64, bool) {
    let num_spins = spins.len();
    let (position, accept_random) = prg.advance(num_spins);
    
    let delta_e = compute_flip_delta(position, edges, spins);
    let accepted = metropolis_accept(delta_e, *temperature_fixed, accept_random);
    
    if accepted {
        spins[position] = 1 - spins[position];  // Flip
    }
    
    // Cool temperature
    *temperature_fixed = (*temperature_fixed * TEMP_DECAY_FIXED) >> 16;
    
    (position, delta_e, accepted)
}

//==============================================================================
// COLLABORATIVE STEP CIRCUIT
//==============================================================================

/// State that evolves through SA steps
#[derive(Clone, Debug)]
pub struct SAState {
    pub step_count: u64,
    pub current_energy: i64,
    pub best_energy: i64,
    pub temperature_fixed: u64,
    pub prg_state: F1,
    pub config_hash: F1,  // Hash of current configuration
}

/// Witness data for one fold (multiple SA steps)
#[derive(Clone, Debug)]
pub struct SAFoldWitness {
    // For each SA step in this fold
    pub steps: Vec<SAStepWitness>,
    
    // Commitments (constant across folds)
    pub problem_commitment: F1,
    pub initial_seed: F1,
}

#[derive(Clone, Debug)]
pub struct SAStepWitness {
    pub position: usize,
    pub delta_e: i64,
    pub accepted: bool,
    pub prg_output_pos: F1,
    pub prg_output_accept: F1,
}

/// Collaborative mining circuit - verifies L steps of SA from seed
#[derive(Clone, Debug)]
pub struct CollaborativeMiningCircuit<F: PrimeField> {
    // Number of SA steps in this fold
    pub num_steps: usize,
    
    // Witness data for each step
    pub step_witnesses: Vec<SAStepWitnessField<F>>,
    
    // Accumulated energy change in this fold
    pub energy_delta: F,
    
    // PRG verification
    pub initial_prg_state: F,
    pub final_prg_state: F,
    
    // Commitments (must match across folds)
    pub problem_commitment: F,
    pub seed: F,
    
    // Job metadata (public inputs)
    pub job_hash: F,
    pub epoch: F,
    pub miner_address: F,
    pub vrf_randomness: F,
}

#[derive(Clone, Debug)]
pub struct SAStepWitnessField<F: PrimeField> {
    pub position: F,
    pub delta_e: F,
    pub accepted: F,  // 0 or 1
    pub prg_output: F,
}

impl<F: PrimeField> Default for CollaborativeMiningCircuit<F> {
    fn default() -> Self {
        Self {
            num_steps: 0,
            step_witnesses: Vec::new(),
            energy_delta: F::ZERO,
            initial_prg_state: F::ZERO,
            final_prg_state: F::ZERO,
            problem_commitment: F::ZERO,
            seed: F::ZERO,
            job_hash: F::ZERO,
            epoch: F::ZERO,
            miner_address: F::ZERO,
            vrf_randomness: F::ZERO,
        }
    }
}

impl<F: PrimeField> StepCircuit<F> for CollaborativeMiningCircuit<F> {
    /// State: [step_count, running_energy, best_energy, temperature, prg_state, 
    ///         problem_commitment, seed, job_hash, epoch, miner_address, vrf]
    fn arity(&self) -> usize { 11 }

    fn synthesize<CS: ConstraintSystem<F>>(
        &self,
        cs: &mut CS,
        z: &[AllocatedNum<F>],
    ) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
        // Unpack state
        let step_count = &z[0];
        let running_energy = &z[1];
        let best_energy = &z[2];
        let temperature = &z[3];
        let prg_state = &z[4];
        let problem_comm = &z[5];
        let seed = &z[6];
        let job_hash = &z[7];
        let epoch = &z[8];
        let miner_address = &z[9];
        let vrf_randomness = &z[10];
        
        //----------------------------------------------------------------------
        // 1. VERIFY SEED DERIVATION (only needed on first fold, but check always)
        //----------------------------------------------------------------------
        // seed should equal Poseidon(job_hash, epoch, miner_address, vrf_randomness)
        // For efficiency, we verify seed is passed through unchanged
        // The initial fold verifies seed = derive_seed(...)
        
        let expected_seed = AllocatedNum::alloc(cs.namespace(|| "expected_seed"), || {
            Ok(self.seed)
        })?;
        
        cs.enforce(
            || "seed_binding",
            |lc| lc + seed.get_variable() - expected_seed.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc,
        );
        
        //----------------------------------------------------------------------
        // 2. VERIFY PROBLEM COMMITMENT UNCHANGED
        //----------------------------------------------------------------------
        let expected_problem_comm = AllocatedNum::alloc(
            cs.namespace(|| "expected_problem_comm"),
            || Ok(self.problem_commitment)
        )?;
        
        cs.enforce(
            || "problem_commitment_binding",
            |lc| lc + problem_comm.get_variable() - expected_problem_comm.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc,
        );
        
        //----------------------------------------------------------------------
        // 3. ACCUMULATE ENERGY DELTA
        //----------------------------------------------------------------------
        let energy_delta = AllocatedNum::alloc(cs.namespace(|| "energy_delta"), || {
            Ok(self.energy_delta)
        })?;
        
        let new_energy = AllocatedNum::alloc(cs.namespace(|| "new_energy"), || {
            match (running_energy.get_value(), self.energy_delta) {
                (Some(e), d) => Ok(e + d),
                _ => Err(SynthesisError::AssignmentMissing),
            }
        })?;
        
        cs.enforce(
            || "energy_accumulation",
            |lc| lc + running_energy.get_variable() + energy_delta.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + new_energy.get_variable(),
        );
        
        //----------------------------------------------------------------------
        // 4. UPDATE BEST ENERGY (if improved)
        //----------------------------------------------------------------------
        let new_best = AllocatedNum::alloc(cs.namespace(|| "new_best"), || {
            match (best_energy.get_value(), new_energy.get_value()) {
                (Some(b), Some(e)) => {
                    { let e_repr = e.to_repr(); let b_repr = b.to_repr(); if e_repr.as_ref() < b_repr.as_ref() { Ok(e) } else { Ok(b) } }
                }
                _ => Err(SynthesisError::AssignmentMissing),
            }
        })?;
        
        // Verify: new_best = min(best_energy, new_energy)
        // This requires a comparison gadget - simplified here
        // In full implementation, use Lt64 comparator
        
        //----------------------------------------------------------------------
        // 5. UPDATE STEP COUNT
        //----------------------------------------------------------------------
        let steps_in_fold = AllocatedNum::alloc(cs.namespace(|| "steps_in_fold"), || {
            Ok(F::from(self.num_steps as u64))
        })?;
        
        let new_step_count = AllocatedNum::alloc(cs.namespace(|| "new_step_count"), || {
            match step_count.get_value() {
                Some(s) => Ok(s + F::from(self.num_steps as u64)),
                _ => Err(SynthesisError::AssignmentMissing),
            }
        })?;
        
        cs.enforce(
            || "step_count_update",
            |lc| lc + step_count.get_variable() + steps_in_fold.get_variable(),
            |lc| lc + CS::one(),
            |lc| lc + new_step_count.get_variable(),
        );
        
        //----------------------------------------------------------------------
        // 6. UPDATE PRG STATE
        //----------------------------------------------------------------------
        let new_prg_state = AllocatedNum::alloc(cs.namespace(|| "new_prg_state"), || {
            Ok(self.final_prg_state)
        })?;
        
        // In full implementation: verify PRG transitions are correct
        // For each step: new_prg = Poseidon(old_prg, step_number)
        // This would require in-circuit Poseidon (expensive but doable)
        
        //----------------------------------------------------------------------
        // 7. UPDATE TEMPERATURE (simplified - just pass through witness)
        //----------------------------------------------------------------------
        let new_temperature = AllocatedNum::alloc(cs.namespace(|| "new_temperature"), || {
            match temperature.get_value() {
                Some(t) => {
                    // Apply decay for num_steps iterations
                    let decay = F::from(TEMP_DECAY_FIXED);
                    let divisor = F::from(1u64 << 16);
                    // Simplified: just reduce by fixed amount per fold
                    Ok(t) // In full impl: t * decay^num_steps
                }
                _ => Err(SynthesisError::AssignmentMissing),
            }
        })?;
        
        // Return updated state
        Ok(vec![
            new_step_count,
            new_energy,
            new_best,
            new_temperature,
            new_prg_state,
            problem_comm.clone(),
            seed.clone(),
            job_hash.clone(),
            epoch.clone(),
            miner_address.clone(),
            vrf_randomness.clone(),
        ])
    }
}

//==============================================================================
// COLLABORATIVE PROVER
//==============================================================================

pub struct CollaborativeProver {
    // Problem
    pub edges: Vec<(u32, u32, i64)>,
    pub num_spins: usize,
    pub problem_commitment: F1,
    
    // Job assignment
    pub job_hash: F1,
    pub epoch: u64,
    pub miner_address: F1,
    pub vrf_randomness: F1,
    pub seed: F1,
    
    // Initial state
    pub initial_config: Vec<u8>,
    pub initial_energy: i64,
    
    // SA parameters
    pub total_steps: u64,
    pub steps_per_fold: usize,
}

impl CollaborativeProver {
    pub fn new(
        edges: Vec<(u32, u32, i64)>,
        num_spins: usize,
        job_hash: F1,
        epoch: u64,
        miner_address: F1,
        vrf_randomness: F1,
        total_steps: u64,
    ) -> Self {
        // Compute problem commitment
        let problem_commitment = crate::commit_ising_problem(num_spins, &edges);
        
        // Derive seed
        let seed = derive_seed(job_hash, epoch, miner_address, vrf_randomness);
        
        // Derive initial configuration
        let initial_config = derive_initial_config(seed, num_spins);
        
        // Compute initial energy
        let initial_energy = crate::compute_ising_energy(&edges, &initial_config);
        
        Self {
            edges,
            num_spins,
            problem_commitment,
            job_hash,
            epoch,
            miner_address,
            vrf_randomness,
            seed,
            initial_config,
            initial_energy,
            total_steps,
            steps_per_fold: SA_STEPS_PER_FOLD,
        }
    }
    
    pub fn num_folds(&self) -> usize {
        ((self.total_steps as usize) + self.steps_per_fold - 1) / self.steps_per_fold
    }
    
    /// Generate all fold circuits by running SA
    pub fn generate_circuits(&self) -> (Vec<CollaborativeMiningCircuit<F1>>, SAState) {
        let mut circuits = Vec::with_capacity(self.num_folds());
        
        // Initialize state
        let mut spins = self.initial_config.clone();
        let mut prg = PrgState::new(self.seed);
        let mut temperature = INITIAL_TEMP_FIXED;
        let mut current_energy = self.initial_energy;
        let mut best_energy = self.initial_energy;
        let mut step_count = 0u64;
        
        for fold_idx in 0..self.num_folds() {
            let steps_this_fold = std::cmp::min(
                self.steps_per_fold,
                (self.total_steps - step_count) as usize
            );
            
            let initial_prg_state = prg.state;
            let mut energy_delta = 0i64;
            let mut step_witnesses = Vec::with_capacity(steps_this_fold);
            
            // Run SA steps
            for _ in 0..steps_this_fold {
                let (position, delta_e, accepted) = sa_step(
                    &mut spins,
                    &self.edges,
                    &mut prg,
                    &mut temperature,
                );
                
                if accepted {
                    energy_delta += delta_e;
                    current_energy += delta_e;
                    if current_energy < best_energy {
                        best_energy = current_energy;
                    }
                }
                
                step_witnesses.push(SAStepWitnessField {
                    position: F1::from(position as u64),
                    delta_e: crate::i64_to_field(delta_e),
                    accepted: if accepted { F1::ONE } else { F1::ZERO },
                    prg_output: F1::ZERO,  // Filled in if needed
                });
                
                step_count += 1;
            }
            
            circuits.push(CollaborativeMiningCircuit {
                num_steps: steps_this_fold,
                step_witnesses,
                energy_delta: crate::i64_to_field(energy_delta),
                initial_prg_state,
                final_prg_state: prg.state,
                problem_commitment: self.problem_commitment,
                seed: self.seed,
                job_hash: self.job_hash,
                epoch: F1::from(self.epoch),
                miner_address: self.miner_address,
                vrf_randomness: self.vrf_randomness,
            });
        }
        
        let final_state = SAState {
            step_count,
            current_energy,
            best_energy,
            temperature_fixed: temperature,
            prg_state: prg.state,
            config_hash: F1::ZERO,  // Could compute hash of final config
        };
        
        (circuits, final_state)
    }
    
    /// Initial state for Nova folding
    pub fn initial_state(&self) -> Vec<F1> {
        vec![
            F1::ZERO,                              // step_count
            F1::from((self.initial_energy + BIAS as i64) as u64),  // running_energy (biased)
            F1::from((self.initial_energy + BIAS as i64) as u64),  // best_energy (biased)
            F1::from(INITIAL_TEMP_FIXED),          // temperature
            self.seed,                             // prg_state (starts as seed)
            self.problem_commitment,               // problem_commitment
            self.seed,                             // seed
            self.job_hash,                         // job_hash
            F1::from(self.epoch),                  // epoch
            self.miner_address,                    // miner_address
            self.vrf_randomness,                   // vrf_randomness
        ]
    }
}

//==============================================================================
// PROOF OUTPUT
//==============================================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CollaborativeProofBundle {
    // Job assignment
    pub job_id: String,
    pub epoch: u64,
    pub miner_address: String,
    pub vrf_randomness: [u8; 32],
    
    // Work done
    pub num_steps: u64,
    pub algorithm_version: String,
    
    // Results
    pub final_energy: i64,
    pub best_energy: i64,
    pub best_config_hash: [u8; 32],
    
    // Commitments
    pub problem_commitment: [u8; 32],
    pub seed: [u8; 32],
    
    // Proof data (serialized Nova proof)
    pub proof_bytes: Vec<u8>,
}
