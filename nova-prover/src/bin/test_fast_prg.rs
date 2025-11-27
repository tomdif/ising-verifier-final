//! Test faster PRG approaches

use ising_nova::{F1, poseidon_hash_2};
use ff::{Field, PrimeField};
use std::time::Instant;

// Current: 2 hashes per step
fn prg_current(seed: F1, steps: u64, n_spins: usize) -> (usize, u64) {
    let mut state = seed;
    let mut last_pos = 0;
    let mut last_accept = 0u64;
    
    for step in 0..steps {
        // Two hashes per step
        let pos_hash = poseidon_hash_2(state, F1::from(step * 2));
        let accept_hash = poseidon_hash_2(state, F1::from(step * 2 + 1));
        
        let pos_bytes = pos_hash.to_repr();
        last_pos = (u64::from_le_bytes(pos_bytes.as_ref()[0..8].try_into().unwrap()) as usize) % n_spins;
        
        let accept_bytes = accept_hash.to_repr();
        last_accept = u64::from_le_bytes(accept_bytes.as_ref()[0..8].try_into().unwrap()) >> 32;
        
        state = poseidon_hash_2(state, F1::from(step));
    }
    (last_pos, last_accept)
}

// Optimized: 1 hash per step, extract both values from 32-byte output
fn prg_optimized(seed: F1, steps: u64, n_spins: usize) -> (usize, u64) {
    let mut state = seed;
    let mut last_pos = 0;
    let mut last_accept = 0u64;
    
    for step in 0..steps {
        // One hash, extract both values
        let hash = poseidon_hash_2(state, F1::from(step));
        let bytes = hash.to_repr();
        
        // First 8 bytes for position
        last_pos = (u64::from_le_bytes(bytes.as_ref()[0..8].try_into().unwrap()) as usize) % n_spins;
        // Next 8 bytes for acceptance
        last_accept = u64::from_le_bytes(bytes.as_ref()[8..16].try_into().unwrap()) >> 32;
        
        state = hash;  // Reuse hash as new state
    }
    (last_pos, last_accept)
}

// Batched: 1 hash per 4 steps
fn prg_batched(seed: F1, steps: u64, n_spins: usize) -> (usize, u64) {
    let mut state = seed;
    let mut last_pos = 0;
    let mut last_accept = 0u64;
    
    let mut step = 0u64;
    while step < steps {
        let hash = poseidon_hash_2(state, F1::from(step / 4));
        let bytes = hash.to_repr();
        
        // Extract 4 pairs of (pos, accept) from 32 bytes
        for i in 0..4 {
            if step + i >= steps { break; }
            let offset = (i * 4) as usize;  // 4 bytes per pair
            let val = u32::from_le_bytes(bytes.as_ref()[offset..offset+4].try_into().unwrap());
            last_pos = (val as usize >> 16) % n_spins;  // Upper 16 bits for position
            last_accept = (val & 0xFFFF) as u64 * 65536;  // Lower 16 bits scaled
        }
        step += 4;
        state = hash;
    }
    (last_pos, last_accept)
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  PRG OPTIMIZATION BENCHMARK");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let seed = F1::from(12345u64);
    let steps = 100_000u64;
    let n_spins = 64;

    // Warmup
    let _ = prg_current(seed, 1000, n_spins);
    let _ = prg_optimized(seed, 1000, n_spins);
    let _ = prg_batched(seed, 1000, n_spins);

    // Benchmark current
    let t = Instant::now();
    let (pos1, acc1) = prg_current(seed, steps, n_spins);
    let current_time = t.elapsed();
    println!("  Current (2 hash/step):    {:>8.2}ms  pos={}, acc={}", 
             current_time.as_secs_f64() * 1000.0, pos1, acc1);

    // Benchmark optimized
    let t = Instant::now();
    let (pos2, acc2) = prg_optimized(seed, steps, n_spins);
    let opt_time = t.elapsed();
    println!("  Optimized (1 hash/step):  {:>8.2}ms  pos={}, acc={}", 
             opt_time.as_secs_f64() * 1000.0, pos2, acc2);

    // Benchmark batched
    let t = Instant::now();
    let (pos3, acc3) = prg_batched(seed, steps, n_spins);
    let batch_time = t.elapsed();
    println!("  Batched (1 hash/4 steps): {:>8.2}ms  pos={}, acc={}", 
             batch_time.as_secs_f64() * 1000.0, pos3, acc3);

    println!("\n  ─────────────────────────────────────────────────────────────────");
    println!("  Speedup (optimized): {:.1}x", current_time.as_secs_f64() / opt_time.as_secs_f64());
    println!("  Speedup (batched):   {:.1}x", current_time.as_secs_f64() / batch_time.as_secs_f64());
    println!("  ─────────────────────────────────────────────────────────────────\n");
}
