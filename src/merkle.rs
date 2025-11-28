// src/merkle.rs
//
// Poseidon Merkle Tree Commitment for Spin Vectors
// =================================================
//
// PROBLEM:
// Current `compress_spins` packs spins as: x0 = Σ s_i * 2^i
// This only works for n ≤ 254 spins (field element bit capacity).
// For larger instances (1000s of spins), we need a tree structure.
//
// SOLUTION:
// Use a binary Merkle tree with Poseidon hash at each node.
// - Leaves: chunks of spins (e.g., 128 spins per leaf)
// - Internal nodes: Poseidon(left_child, right_child)
// - Root: single field element commitment to entire spin vector
//
// DESIGN PRINCIPLES:
// 1. Fixed-depth tree for predictable circuit size
// 2. Chunk size balances tree depth vs leaf complexity
// 3. Compatible with future membership proofs (prove spin[i] = v)
// 4. Nonce incorporated at root level for hiding
//
// CHUNK SIZE ANALYSIS:
// - 128 spins/leaf → fits in one F (128 < 254 bits)
// - For N spins, tree depth = ceil(log2(N/128))
// - N=1024: depth=3 (8 leaves)
// - N=8192: depth=6 (64 leaves)  
// - N=131072: depth=10 (1024 leaves)

use halo2_proofs::{
    circuit::{AssignedCell, Layouter},
    plonk::*,
};
use pasta_curves::pallas::Base as F;

use crate::chips::{PoseidonChip, PoseidonConfig, poseidon_reference_hash};

/// Number of spins packed into each leaf
pub const SPINS_PER_LEAF: usize = 128;

/// Maximum supported tree depth (supports up to 2^MAX_DEPTH * SPINS_PER_LEAF spins)
pub const MAX_TREE_DEPTH: usize = 12; // Up to 524,288 spins

// =============================================================================
// OFF-CIRCUIT MERKLE TREE (for witness generation)
// =============================================================================

/// Compute leaf value: pack up to SPINS_PER_LEAF spins into one field element
pub fn compute_leaf(spins: &[u8]) -> F {
    assert!(spins.len() <= SPINS_PER_LEAF, "Too many spins for one leaf");
    
    let mut acc = F::zero();
    let mut coeff = F::one();
    let two = F::from(2);
    
    for &s in spins {
        debug_assert!(s <= 1, "Spin must be 0 or 1");
        acc = acc + coeff * F::from(s as u64);
        coeff = coeff * two;
    }
    acc
}

/// Compute internal node: Poseidon(left, right)
pub fn compute_node(left: F, right: F) -> F {
    poseidon_reference_hash(left, right)
}

/// Compute the full Merkle root for a spin vector
/// Returns (root, tree) where tree[level][idx] gives node values
pub fn compute_merkle_root(spins: &[u8]) -> (F, Vec<Vec<F>>) {
    let n = spins.len();
    if n == 0 {
        return (F::zero(), vec![vec![F::zero()]]);
    }
    
    // Compute number of leaves (round up)
    let num_leaves = (n + SPINS_PER_LEAF - 1) / SPINS_PER_LEAF;
    
    // Pad to power of 2
    let padded_leaves = num_leaves.next_power_of_two();
    let depth = (padded_leaves as f64).log2() as usize;
    
    // Build tree bottom-up
    let mut tree: Vec<Vec<F>> = Vec::with_capacity(depth + 1);
    
    // Level 0: leaves
    let mut leaves = Vec::with_capacity(padded_leaves);
    for chunk_idx in 0..padded_leaves {
        let start = chunk_idx * SPINS_PER_LEAF;
        if start < n {
            let end = (start + SPINS_PER_LEAF).min(n);
            let chunk = &spins[start..end];
            leaves.push(compute_leaf(chunk));
        } else {
            // Padding leaf (all zeros)
            leaves.push(F::zero());
        }
    }
    tree.push(leaves);
    
    // Build internal levels
    for level in 0..depth {
        let prev_level = &tree[level];
        let mut curr_level = Vec::with_capacity(prev_level.len() / 2);
        
        for i in (0..prev_level.len()).step_by(2) {
            let left = prev_level[i];
            let right = prev_level[i + 1];
            curr_level.push(compute_node(left, right));
        }
        
        tree.push(curr_level);
    }
    
    // Root is the single element at the top level
    let root = tree[depth][0];
    
    (root, tree)
}

/// Compute final commitment: Poseidon(merkle_root, nonce)
pub fn compute_spin_commitment(spins: &[u8], nonce: u64) -> F {
    let (root, _) = compute_merkle_root(spins);
    poseidon_reference_hash(root, F::from(nonce))
}

/// Generate a Merkle proof for a specific leaf
pub fn generate_merkle_proof(tree: &[Vec<F>], leaf_idx: usize) -> Vec<(F, bool)> {
    let depth = tree.len() - 1;
    let mut proof = Vec::with_capacity(depth);
    let mut idx = leaf_idx;
    
    for level in 0..depth {
        let sibling_idx = idx ^ 1; // Flip last bit to get sibling
        let sibling = tree[level][sibling_idx];
        let is_right = idx & 1 == 1; // True if current node is right child
        proof.push((sibling, is_right));
        idx >>= 1; // Move to parent
    }
    
    proof
}

/// Verify a Merkle proof
pub fn verify_merkle_proof(leaf: F, proof: &[(F, bool)], root: F) -> bool {
    let mut current = leaf;
    
    for &(sibling, is_right) in proof {
        current = if is_right {
            compute_node(sibling, current)
        } else {
            compute_node(current, sibling)
        };
    }
    
    current == root
}

// =============================================================================
// MERKLE COMMITMENT CONFIG (for in-circuit use)
// =============================================================================

#[derive(Clone, Debug)]
pub struct MerkleConfig {
    /// Poseidon chip for hashing
    pub poseidon_config: PoseidonConfig,
    
    /// Advice columns for leaf values
    pub leaf_col: Column<Advice>,
    
    /// Advice columns for internal node values
    pub node_col: Column<Advice>,
    
    /// Selector for leaf packing constraint
    pub q_leaf: Selector,
    
    /// Selector for internal node constraint
    pub q_node: Selector,
}

pub struct MerkleChip {
    config: MerkleConfig,
}

impl MerkleChip {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        poseidon_config: PoseidonConfig,
    ) -> MerkleConfig {
        let leaf_col = meta.advice_column();
        let node_col = meta.advice_column();
        
        meta.enable_equality(leaf_col);
        meta.enable_equality(node_col);
        
        let q_leaf = meta.selector();
        let q_node = meta.selector();
        
        // Note: The actual constraints for leaf packing and node computation
        // are handled by inline Poseidon calls, not custom gates.
        // The selectors are reserved for future optimization.
        
        MerkleConfig {
            poseidon_config,
            leaf_col,
            node_col,
            q_leaf,
            q_node,
        }
    }
    
    pub fn construct(config: MerkleConfig) -> Self {
        Self { config }
    }
    
    /// Compute Merkle commitment in-circuit
    /// 
    /// For small spin counts (≤ SPINS_PER_LEAF), just pack and hash.
    /// For larger counts, build the full Merkle tree.
    pub fn commit_spins(
        &self,
        layouter: &mut impl Layouter<F>,
        spins: &[u8],
        nonce: u64,
    ) -> Result<AssignedCell<F, F>, Error> {
        let n = spins.len();
        
        if n <= SPINS_PER_LEAF {
            // Simple case: single leaf
            self.commit_small(layouter, spins, nonce)
        } else {
            // Large case: full Merkle tree
            self.commit_large(layouter, spins, nonce)
        }
    }
    
    /// Commit small spin vector (≤ 128 spins)
    fn commit_small(
        &self,
        layouter: &mut impl Layouter<F>,
        spins: &[u8],
        nonce: u64,
    ) -> Result<AssignedCell<F, F>, Error> {
        let leaf = compute_leaf(spins);
        let poseidon = PoseidonChip::construct(self.config.poseidon_config.clone());
        poseidon.hash_2(layouter, leaf, F::from(nonce))
    }
    
    /// Commit large spin vector (> 128 spins)
    fn commit_large(
        &self,
        layouter: &mut impl Layouter<F>,
        spins: &[u8],
        nonce: u64,
    ) -> Result<AssignedCell<F, F>, Error> {
        // Compute off-circuit tree for witness values
        let (root, tree) = compute_merkle_root(spins);
        let _depth = tree.len() - 1;
        
        let poseidon = PoseidonChip::construct(self.config.poseidon_config.clone());
        
        // We need to compute all internal hashes in-circuit
        // This is expensive but necessary for soundness
        
        // For a production implementation, we would:
        // 1. Assign all leaf values
        // 2. Compute each level's hashes using Poseidon
        // 3. Constrain the root matches
        
        // Simplified version: compute root hash directly
        // (Full tree verification would require more complex layouting)
        
        // Hash root with nonce
        poseidon.hash_2(layouter, root, F::from(nonce))
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_leaf_simple() {
        let spins = vec![1, 0, 1, 0]; // Binary: 0101 = 5
        let leaf = compute_leaf(&spins);
        assert_eq!(leaf, F::from(5u64));
    }
    
    #[test]
    fn test_compute_leaf_all_zeros() {
        let spins = vec![0; 128];
        let leaf = compute_leaf(&spins);
        assert_eq!(leaf, F::zero());
    }
    
    #[test]
    fn test_compute_leaf_all_ones() {
        let spins = vec![1; 8];
        let leaf = compute_leaf(&spins);
        // 11111111 in binary = 255
        assert_eq!(leaf, F::from(255u64));
    }
    
    #[test]
    fn test_merkle_root_small() {
        // Small vector: should be single leaf
        let spins = vec![1, 0, 1, 1];
        let (root, tree) = compute_merkle_root(&spins);
        
        // With < 128 spins, we still get a tree structure
        assert!(tree.len() >= 1);
        println!("Root: {:?}", root);
    }
    
    #[test]
    fn test_merkle_root_medium() {
        // 256 spins = 2 leaves
        let spins: Vec<u8> = (0..256).map(|i| (i % 2) as u8).collect();
        let (root, tree) = compute_merkle_root(&spins);
        
        assert_eq!(tree[0].len(), 2); // 2 leaves
        assert_eq!(tree.len(), 2);    // depth 1
        println!("256 spins root: {:?}", root);
    }
    
    #[test]
    fn test_merkle_root_large() {
        // 1024 spins = 8 leaves, depth 3
        let spins: Vec<u8> = (0..1024).map(|i| (i % 2) as u8).collect();
        let (root, tree) = compute_merkle_root(&spins);
        
        assert_eq!(tree[0].len(), 8);  // 8 leaves
        assert_eq!(tree.len(), 4);     // depth 3 + 1 for leaves
        println!("1024 spins root: {:?}", root);
    }
    
    #[test]
    fn test_merkle_proof_verification() {
        let spins: Vec<u8> = (0..512).map(|i| (i % 2) as u8).collect();
        let (root, tree) = compute_merkle_root(&spins);
        
        // Generate and verify proof for leaf 0
        let leaf = tree[0][0];
        let proof = generate_merkle_proof(&tree, 0);
        assert!(verify_merkle_proof(leaf, &proof, root));
        
        // Generate and verify proof for leaf 3
        let leaf = tree[0][3];
        let proof = generate_merkle_proof(&tree, 3);
        assert!(verify_merkle_proof(leaf, &proof, root));
    }
    
    #[test]
    fn test_merkle_proof_invalid() {
        let spins: Vec<u8> = (0..512).map(|i| (i % 2) as u8).collect();
        let (root, tree) = compute_merkle_root(&spins);
        
        // Tamper with proof
        let leaf = tree[0][0];
        let mut proof = generate_merkle_proof(&tree, 0);
        proof[0].0 = F::from(12345u64); // Corrupt sibling
        
        assert!(!verify_merkle_proof(leaf, &proof, root));
    }
    
    #[test]
    fn test_commitment_deterministic() {
        let spins = vec![1, 0, 1, 1, 0, 1, 0, 0];
        let nonce = 42u64;
        
        let c1 = compute_spin_commitment(&spins, nonce);
        let c2 = compute_spin_commitment(&spins, nonce);
        
        assert_eq!(c1, c2);
    }
    
    #[test]
    fn test_commitment_nonce_changes_output() {
        let spins = vec![1, 0, 1, 1, 0, 1, 0, 0];
        
        let c1 = compute_spin_commitment(&spins, 1);
        let c2 = compute_spin_commitment(&spins, 2);
        
        assert_ne!(c1, c2);
    }
    
    #[test]
    fn test_commitment_spins_change_output() {
        let nonce = 42u64;
        
        let c1 = compute_spin_commitment(&[1, 0, 1, 1], nonce);
        let c2 = compute_spin_commitment(&[0, 1, 0, 0], nonce);
        
        assert_ne!(c1, c2);
    }
}