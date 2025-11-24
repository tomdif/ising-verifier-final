//! PUUB - Proof of Useful Useful Work tracking

use crate::types::*;
use std::collections::HashMap;
use chrono::Utc;

pub struct PuubTracker {
    records: Vec<PuubRecord>,
    scores: HashMap<String, u64>,  // prover -> score
}

impl PuubTracker {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            scores: HashMap::new(),
        }
    }
    
    /// Record a completed proof
    pub fn record_proof(&mut self, record: PuubRecord) {
        // Calculate PUUB score contribution
        // Formula: (n_spins * n_edges) / proof_time_ms * quality_factor
        let work_units = record.n_spins * record.n_edges;
        let time_factor = if record.proof_time_ms > 0 {
            1000 / record.proof_time_ms.max(1)
        } else {
            1
        };
        
        // Quality factor: how much better than threshold
        let quality = if record.threshold != 0 {
            let improvement = record.threshold - record.energy_achieved;
            (improvement.abs() as u64).min(1000)
        } else {
            100
        };
        
        let score_delta = (work_units / 1_000_000) * time_factor * quality / 100;
        
        // Update cumulative score
        let prover = record.prover.clone();
        *self.scores.entry(prover).or_insert(0) += score_delta.max(1);
        
        self.records.push(record);
    }
    
    /// Get PUUB score for prover
    pub fn get_score(&self, prover: &str) -> u64 {
        *self.scores.get(prover).unwrap_or(&0)
    }
    
    /// Get proof history for prover
    pub fn get_history(&self, prover: &str, limit: usize) -> Vec<&PuubRecord> {
        self.records.iter()
            .filter(|r| r.prover == prover)
            .rev()
            .take(limit)
            .collect()
    }
    
    /// Get prover statistics
    pub fn get_stats(&self, prover: &str) -> ProverStats {
        let records: Vec<_> = self.records.iter()
            .filter(|r| r.prover == prover)
            .collect();
        
        let total_proofs = records.len() as u64;
        let total_spins: u64 = records.iter().map(|r| r.n_spins).sum();
        let total_edges: u64 = records.iter().map(|r| r.n_edges).sum();
        let total_time: u64 = records.iter().map(|r| r.proof_time_ms).sum();
        let total_rewards: u128 = records.iter()
            .map(|r| r.reward_wei.parse::<u128>().unwrap_or(0))
            .sum();
        
        ProverStats {
            address: prover.to_string(),
            total_proofs,
            total_spins_proved: total_spins,
            total_edges_proved: total_edges,
            total_rewards_wei: total_rewards.to_string(),
            avg_proof_time_ms: if total_proofs > 0 { total_time / total_proofs } else { 0 },
            success_rate: 1.0, // All recorded proofs are successful
            puub_score: self.get_score(prover),
        }
    }
    
    /// Get leaderboard
    pub fn leaderboard(&self, limit: usize) -> Vec<(String, u64)> {
        let mut scores: Vec<_> = self.scores.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        scores.sort_by(|a, b| b.1.cmp(&a.1));
        scores.truncate(limit);
        scores
    }
}
