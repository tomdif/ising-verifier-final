//! Matcher - assigns jobs to provers

use crate::types::*;
use std::collections::HashMap;
use chrono::Utc;

pub struct Matcher {
    provers: HashMap<String, Prover>,
    assignments: HashMap<u64, String>, // job_id -> prover_address
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            provers: HashMap::new(),
            assignments: HashMap::new(),
        }
    }
    
    /// Register a new prover
    pub fn register_prover(&mut self, req: RegisterProverRequest) -> Prover {
        let now = Utc::now();
        let prover = Prover {
            address: req.address.clone(),
            name: req.name,
            gpu_model: req.gpu_model,
            max_spins: req.max_spins,
            registered_at: now,
            last_seen: now,
            active: true,
        };
        self.provers.insert(req.address.clone(), prover.clone());
        prover
    }
    
    /// Get prover by address
    pub fn get_prover(&self, address: &str) -> Option<&Prover> {
        self.provers.get(address)
    }
    
    /// Update prover last seen
    pub fn heartbeat(&mut self, address: &str) {
        if let Some(prover) = self.provers.get_mut(address) {
            prover.last_seen = Utc::now();
            prover.active = true;
        }
    }
    
    /// Check if prover can handle job
    pub fn can_handle(&self, prover_addr: &str, job: &Job) -> bool {
        if let Some(prover) = self.provers.get(prover_addr) {
            if !prover.active { return false; }
            if let Some(n_spins) = job.n_spins {
                if n_spins > prover.max_spins { return false; }
            }
            true
        } else {
            false
        }
    }
    
    /// Assign job to prover (first-come-first-served)
    pub fn assign_job(&mut self, job_id: u64, prover_addr: &str) -> bool {
        if self.assignments.contains_key(&job_id) {
            return false;
        }
        self.assignments.insert(job_id, prover_addr.to_string());
        true
    }
    
    /// Get job assignment
    pub fn get_assignment(&self, job_id: u64) -> Option<&String> {
        self.assignments.get(&job_id)
    }
    
    /// Complete job assignment
    pub fn complete_job(&mut self, job_id: u64) {
        self.assignments.remove(&job_id);
    }
    
    /// List active provers
    pub fn list_active_provers(&self) -> Vec<&Prover> {
        self.provers.values().filter(|p| p.active).collect()
    }
    
    /// Get prover count
    pub fn prover_count(&self) -> usize {
        self.provers.len()
    }
}
