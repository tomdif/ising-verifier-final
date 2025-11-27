//! Benchmark different fold sizes

use ising_nova::collaborative::*;
use ising_nova::{F1, F2, E1, E2};
use ff::Field;
use nova_snark::{
    provider::ipa_pc::EvaluationEngine,
    traits::{circuit::TrivialCircuit, circuit::StepCircuit, snark::RelaxedR1CSSNARKTrait},
    PublicParams, RecursiveSNARK, CompressedSNARK,
    spartan::snark::RelaxedR1CSSNARK,
};
use std::time::Instant;

type C1 = CollaborativeMiningCircuit<F1>;
type C2 = TrivialCircuit<F2>;
type EE1 = EvaluationEngine<E1>;
type EE2 = EvaluationEngine<E2>;
type S1 = RelaxedR1CSSNARK<E1, EE1>;
type S2 = RelaxedR1CSSNARK<E2, EE2>;

fn generate_test_problem(n_spins: usize, degree: usize) -> Vec<(u32, u32, i64)> {
    use std::collections::HashSet;
    let mut edges = Vec::new();
    let mut seen = HashSet::new();
    let mut rng_state = 12345u64;
    let mut next_rand = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        rng_state
    };
    for u in 0..n_spins {
        for _ in 0..degree {
            let v = (next_rand() as usize) % n_spins;
            if u != v {
                let (a, b) = if u < v { (u, v) } else { (v, u) };
                if seen.insert((a, b)) {
                    let w = ((next_rand() % 201) as i64) - 100;
                    edges.push((a as u32, b as u32, w));
                }
            }
        }
    }
    edges
}

fn run_benchmark(edges: &[(u32, u32, i64)], n_spins: usize, total_steps: u64, steps_per_fold: usize) -> Result<(f64, f64, f64), Box<dyn std::error::Error>> {
    let job_hash = F1::from(12345u64);
    let epoch = 100u64;
    let miner_address = F1::from(0xABCDu64);
    let vrf_randomness = F1::from(0x98765u64);

    // Generate circuits with custom steps_per_fold
    let t_gen = Instant::now();
    let prover = CollaborativeProverCustom::new(
        edges.to_vec(),
        n_spins,
        job_hash,
        epoch,
        miner_address,
        vrf_randomness,
        total_steps,
        steps_per_fold,
    );
    let (circuits, _final_state) = prover.generate_circuits();
    let gen_time = t_gen.elapsed().as_secs_f64();

    // Nova setup
    let pp = PublicParams::<E1, E2, C1, C2>::setup(
        &circuits[0],
        &TrivialCircuit::default(),
        &*S1::ck_floor(),
        &*S2::ck_floor(),
    );

    // Recursive proving
    let t_prove = Instant::now();
    let z0 = prover.initial_state();
    let z0_sec = vec![F2::ZERO];

    let mut recursive_snark = RecursiveSNARK::<E1, E2, C1, C2>::new(
        &pp,
        &circuits[0],
        &TrivialCircuit::default(),
        &z0,
        &z0_sec,
    )?;

    for circuit in circuits.iter() {
        recursive_snark.prove_step(&pp, circuit, &TrivialCircuit::default())?;
    }
    let prove_time = t_prove.elapsed().as_secs_f64();

    // Compress
    let t_compress = Instant::now();
    let (pk, _vk) = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::setup(&pp)?;
    let _compressed = CompressedSNARK::<E1, E2, C1, C2, S1, S2>::prove(&pp, &pk, &recursive_snark)?;
    let compress_time = t_compress.elapsed().as_secs_f64();

    Ok((gen_time, prove_time, compress_time))
}

// Custom prover with configurable steps_per_fold
pub struct CollaborativeProverCustom {
    pub edges: Vec<(u32, u32, i64)>,
    pub num_spins: usize,
    pub problem_commitment: F1,
    pub job_hash: F1,
    pub epoch: u64,
    pub miner_address: F1,
    pub vrf_randomness: F1,
    pub seed: F1,
    pub initial_config: Vec<u8>,
    pub initial_energy: i64,
    pub total_steps: u64,
    pub steps_per_fold: usize,
}

impl CollaborativeProverCustom {
    pub fn new(
        edges: Vec<(u32, u32, i64)>,
        num_spins: usize,
        job_hash: F1,
        epoch: u64,
        miner_address: F1,
        vrf_randomness: F1,
        total_steps: u64,
        steps_per_fold: usize,
    ) -> Self {
        let problem_commitment = ising_nova::commit_ising_problem(num_spins, &edges);
        let seed = derive_seed(job_hash, epoch, miner_address, vrf_randomness);
        let initial_config = derive_initial_config(seed, num_spins);
        let initial_energy = ising_nova::compute_ising_energy(&edges, &initial_config);
        
        Self {
            edges, num_spins, problem_commitment, job_hash, epoch,
            miner_address, vrf_randomness, seed, initial_config,
            initial_energy, total_steps, steps_per_fold,
        }
    }
    
    pub fn num_folds(&self) -> usize {
        ((self.total_steps as usize) + self.steps_per_fold - 1) / self.steps_per_fold
    }
    
    pub fn generate_circuits(&self) -> (Vec<CollaborativeMiningCircuit<F1>>, SAState) {
        let mut circuits = Vec::with_capacity(self.num_folds());
        let mut spins = self.initial_config.clone();
        let mut prg = PrgState::new(self.seed);
        let mut temperature = INITIAL_TEMP_FIXED;
        let mut current_energy = self.initial_energy;
        let mut best_energy = self.initial_energy;
        let mut step_count = 0u64;
        
        for _ in 0..self.num_folds() {
            let steps_this_fold = std::cmp::min(
                self.steps_per_fold,
                (self.total_steps - step_count) as usize
            );
            
            let initial_prg_state = prg.state;
            let mut energy_delta = 0i64;
            let mut step_witnesses = Vec::with_capacity(steps_this_fold);
            
            for _ in 0..steps_this_fold {
                let (position, delta_e, accepted) = sa_step(&mut spins, &self.edges, &mut prg, &mut temperature);
                
                if accepted {
                    energy_delta += delta_e;
                    current_energy += delta_e;
                    if current_energy < best_energy {
                        best_energy = current_energy;
                    }
                }
                
                step_witnesses.push(SAStepWitnessField {
                    position: F1::from(position as u64),
                    delta_e: ising_nova::i64_to_field(delta_e),
                    accepted: if accepted { F1::ONE } else { F1::ZERO },
                    prg_output: F1::ZERO,
                });
                
                step_count += 1;
            }
            
            circuits.push(CollaborativeMiningCircuit {
                num_steps: steps_this_fold,
                step_witnesses,
                energy_delta: ising_nova::i64_to_field(energy_delta),
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
            step_count, current_energy, best_energy,
            temperature_fixed: temperature, prg_state: prg.state,
            config_hash: F1::ZERO,
        };
        
        (circuits, final_state)
    }
    
    pub fn initial_state(&self) -> Vec<F1> {
        vec![
            F1::ZERO,
            F1::from((self.initial_energy + ising_nova::BIAS as i64) as u64),
            F1::from((self.initial_energy + ising_nova::BIAS as i64) as u64),
            F1::from(INITIAL_TEMP_FIXED),
            self.seed,
            self.problem_commitment,
            self.seed,
            self.job_hash,
            F1::from(self.epoch),
            self.miner_address,
            self.vrf_randomness,
        ]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  FOLD SIZE BENCHMARK - Finding optimal steps per fold");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    let n_spins = 64;
    let degree = 4;
    let total_steps = 100_000u64;

    println!("  Generating problem...");
    let edges = generate_test_problem(n_spins, degree);
    println!("  {} spins, {} edges, {} total steps", n_spins, edges.len(), total_steps);
    println!();

    let fold_sizes = vec![10_000, 15_000, 20_000, 25_000, 30_000, 35_000, 40_000, 45_000, 50_000];
    
    println!("  {:>12} {:>6} {:>10} {:>10} {:>10} {:>10}", 
             "Steps/Fold", "Folds", "Gen(s)", "Prove(s)", "Compress", "TOTAL");
    println!("  {}", "-".repeat(64));

    let mut best_total = f64::MAX;
    let mut best_fold_size = 0;

    for fold_size in fold_sizes {
        let folds = (total_steps as usize + fold_size - 1) / fold_size;
        
        match run_benchmark(&edges, n_spins, total_steps, fold_size) {
            Ok((gen, prove, compress)) => {
                let total = gen + prove + compress;
                let marker = if total < best_total { " ★" } else { "" };
                if total < best_total {
                    best_total = total;
                    best_fold_size = fold_size;
                }
                println!("  {:>12} {:>6} {:>10.2} {:>10.2} {:>10.2} {:>10.2}{}", 
                         fold_size, folds, gen, prove, compress, total, marker);
            }
            Err(e) => {
                println!("  {:>12} {:>6} ERROR: {}", fold_size, folds, e);
            }
        }
    }

    println!();
    println!("  ═══════════════════════════════════════════════════════════════");
    println!("  OPTIMAL: {} steps/fold = {:.2}s total", best_fold_size, best_total);
    println!("  ═══════════════════════════════════════════════════════════════");

    Ok(())
}
