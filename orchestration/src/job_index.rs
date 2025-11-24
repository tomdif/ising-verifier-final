//! Job Index - indexes and caches jobs from L1

use crate::types::*;
use std::collections::HashMap;
use chrono::Utc;

pub struct JobIndex {
    jobs: HashMap<u64, Job>,
    next_id: u64,
}

impl JobIndex {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            next_id: 0,
        }
    }
    
    /// Add a new job (from L1 event or direct submission)
    pub fn add_job(&mut self, mut job: Job) -> u64 {
        let id = self.next_id;
        job.id = id;
        job.status = JobStatus::Open;
        job.created_at = Utc::now();
        self.jobs.insert(id, job);
        self.next_id += 1;
        id
    }
    
    /// Get job by ID
    pub fn get_job(&self, id: u64) -> Option<&Job> {
        self.jobs.get(&id)
    }
    
    /// Get mutable job
    pub fn get_job_mut(&mut self, id: u64) -> Option<&mut Job> {
        self.jobs.get_mut(&id)
    }
    
    /// List open jobs
    pub fn list_open_jobs(&self) -> Vec<&Job> {
        let now = Utc::now();
        self.jobs.values()
            .filter(|j| j.status == JobStatus::Open && j.deadline > now)
            .collect()
    }
    
    /// List jobs with filters
    pub fn list_jobs(&self, 
        status: Option<JobStatus>,
        min_reward: Option<u128>,
        max_spins: Option<u64>,
        limit: usize,
    ) -> Vec<&Job> {
        let now = Utc::now();
        self.jobs.values()
            .filter(|j| {
                // Status filter
                if let Some(ref s) = status {
                    if &j.status != s { return false; }
                }
                // Deadline filter (exclude expired for open jobs)
                if j.status == JobStatus::Open && j.deadline <= now {
                    return false;
                }
                // Reward filter
                if let Some(min) = min_reward {
                    let reward: u128 = j.reward_wei.parse().unwrap_or(0);
                    if reward < min { return false; }
                }
                // Size filter
                if let Some(max) = max_spins {
                    if let Some(n) = j.n_spins {
                        if n > max { return false; }
                    }
                }
                true
            })
            .take(limit)
            .collect()
    }
    
    /// Update job status
    pub fn update_status(&mut self, id: u64, status: JobStatus) -> bool {
        if let Some(job) = self.jobs.get_mut(&id) {
            job.status = status;
            true
        } else {
            false
        }
    }
    
    /// Mark job as claimed
    pub fn claim_job(&mut self, id: u64, prover: &str) -> bool {
        if let Some(job) = self.jobs.get_mut(&id) {
            if job.status == JobStatus::Open {
                job.status = JobStatus::Claimed;
                job.claimed_by = Some(prover.to_string());
                return true;
            }
        }
        false
    }
    
    /// Get statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let total = self.jobs.len();
        let open = self.jobs.values().filter(|j| j.status == JobStatus::Open).count();
        let solved = self.jobs.values().filter(|j| j.status == JobStatus::Solved).count();
        (total, open, solved)
    }
}
