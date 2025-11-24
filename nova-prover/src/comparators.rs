//! Comparison Chips for Nova Ising Prover
//! 
//! Implements Lt64Chip for 64-bit less-than comparisons using
//! bit decomposition and range checking.

use bellpepper_core::{
    num::AllocatedNum,
    ConstraintSystem, SynthesisError, LinearCombination,
};
use ff::PrimeField;

/// Decompose a field element into `n_bits` bits (little-endian)
/// Returns allocated bits and enforces they reconstruct the original value
pub fn decompose_into_bits<F: PrimeField, CS: ConstraintSystem<F>>(
    cs: &mut CS,
    value: &AllocatedNum<F>,
    n_bits: usize,
) -> Result<Vec<AllocatedNum<F>>, SynthesisError> {
    let val = value.get_value();
    
    // Extract bits from the value
    let bits_values: Option<Vec<bool>> = val.map(|v| {
        let repr = v.to_repr();
        let bytes = repr.as_ref();
        (0..n_bits).map(|i| {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if byte_idx < bytes.len() {
                (bytes[byte_idx] >> bit_idx) & 1 == 1
            } else {
                false
            }
        }).collect()
    });
    
    // Allocate each bit
    let mut bits = Vec::with_capacity(n_bits);
    for i in 0..n_bits {
        let bit_val = bits_values.as_ref().map(|bv| bv[i]);
        let bit = AllocatedNum::alloc(
            cs.namespace(|| format!("bit_{}", i)),
            || Ok(if bit_val.ok_or(SynthesisError::AssignmentMissing)? {
                F::ONE
            } else {
                F::ZERO
            }),
        )?;
        
        // Enforce bit constraint: b * (b - 1) = 0
        cs.enforce(
            || format!("bit_{}_binary", i),
            |lc| lc + bit.get_variable(),
            |lc| lc + bit.get_variable() - CS::one(),
            |lc| lc,
        );
        
        bits.push(bit);
    }
    
    // Enforce reconstruction: sum(bit_i * 2^i) = value
    let mut coeff = F::ONE;
    let mut reconstruction: LinearCombination<F> = LinearCombination::zero();
    for bit in &bits {
        reconstruction = reconstruction + (coeff, bit.get_variable());
        coeff = coeff.double();
    }
    
    cs.enforce(
        || "bit_decomposition",
        |_| reconstruction,
        |lc| lc + CS::one(),
        |lc| lc + value.get_variable(),
    );
    
    Ok(bits)
}

/// Lt64Chip: Compare two 64-bit values
/// 
/// Uses the subtraction method: a < b iff (b - a + 2^64 - 1) has bit 64 set
/// 
/// Returns 1 if a < b, 0 otherwise.
/// Constraints: ~65 (for bit decomposition of diff)
pub fn lt64<F: PrimeField, CS: ConstraintSystem<F>>(
    cs: &mut CS,
    a: &AllocatedNum<F>,
    b: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
    let a_val = a.get_value();
    let b_val = b.get_value();
    
    // Extract 64-bit values for witness computation
    let (a_u64, b_u64) = match (a_val, b_val) {
        (Some(av), Some(bv)) => {
            let a_bytes = av.to_repr();
            let b_bytes = bv.to_repr();
            let au = u64::from_le_bytes(a_bytes.as_ref()[0..8].try_into().unwrap());
            let bu = u64::from_le_bytes(b_bytes.as_ref()[0..8].try_into().unwrap());
            (Some(au), Some(bu))
        }
        _ => (None, None),
    };
    
    // Compute diff = b - a + offset where offset = 2^64 - 1
    // If a < b: diff >= 2^64, so bit 64 = 1
    // If a >= b: diff < 2^64, so bit 64 = 0
    let offset = F::from(u64::MAX);
    
    let diff = AllocatedNum::alloc(cs.namespace(|| "diff"), || {
        match (a_val, b_val) {
            (Some(av), Some(bv)) => Ok(bv - av + offset),
            _ => Err(SynthesisError::AssignmentMissing),
        }
    })?;
    
    // Enforce diff = b - a + offset
    cs.enforce(
        || "diff_computation",
        |lc| lc + diff.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc + b.get_variable() - a.get_variable() + (offset, CS::one()),
    );
    
    // Decompose diff into 65 bits
    let diff_bits = decompose_into_bits(&mut cs.namespace(|| "diff_bits"), &diff, 65)?;
    
    // Result is bit 64 (the MSB indicating overflow)
    let result = &diff_bits[64];
    
    // Allocate result as a separate variable for cleaner API
    let result_out = AllocatedNum::alloc(cs.namespace(|| "lt64_result"), || {
        result.get_value().ok_or(SynthesisError::AssignmentMissing)
    })?;
    
    // Enforce result_out = diff_bits[64]
    cs.enforce(
        || "result_equals_bit64",
        |lc| lc + result_out.get_variable() - result.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc,
    );
    
    Ok(result_out)
}

/// Less-than-or-equal for 64-bit values
/// Returns 1 if a <= b, 0 otherwise
pub fn le64<F: PrimeField, CS: ConstraintSystem<F>>(
    cs: &mut CS,
    a: &AllocatedNum<F>,
    b: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
    // a <= b iff NOT (b < a) iff 1 - lt64(b, a)
    let b_lt_a = lt64(&mut cs.namespace(|| "b_lt_a"), b, a)?;
    
    let result = AllocatedNum::alloc(cs.namespace(|| "le64_result"), || {
        b_lt_a.get_value()
            .map(|v| F::ONE - v)
            .ok_or(SynthesisError::AssignmentMissing)
    })?;
    
    // Enforce result = 1 - b_lt_a
    cs.enforce(
        || "le64_from_lt64",
        |lc| lc + result.get_variable() + b_lt_a.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc + CS::one(),
    );
    
    Ok(result)
}

/// Verify that energy is below threshold (gap-hiding)
/// Returns 1 if energy + gap <= threshold, 0 otherwise
/// 
/// This enables gap-hiding: prover proves E + Δ ≤ T without revealing exact E
pub fn verify_threshold<F: PrimeField, CS: ConstraintSystem<F>>(
    cs: &mut CS,
    energy: &AllocatedNum<F>,
    gap: &AllocatedNum<F>,
    threshold: &AllocatedNum<F>,
) -> Result<AllocatedNum<F>, SynthesisError> {
    // Compute energy + gap
    let energy_plus_gap = AllocatedNum::alloc(cs.namespace(|| "energy_plus_gap"), || {
        match (energy.get_value(), gap.get_value()) {
            (Some(e), Some(g)) => Ok(e + g),
            _ => Err(SynthesisError::AssignmentMissing),
        }
    })?;
    
    // Enforce energy_plus_gap = energy + gap
    cs.enforce(
        || "sum_constraint",
        |lc| lc + energy_plus_gap.get_variable(),
        |lc| lc + CS::one(),
        |lc| lc + energy.get_variable() + gap.get_variable(),
    );
    
    // Check energy_plus_gap <= threshold
    le64(&mut cs.namespace(|| "threshold_check"), &energy_plus_gap, threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use crate::F1 as F;
    use ff::Field;
    
    #[test]
    fn test_decompose_bits() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let val = AllocatedNum::alloc(cs.namespace(|| "val"), || Ok(F::from(0b10110101u64))).unwrap();
        let bits = decompose_into_bits(&mut cs.namespace(|| "decompose"), &val, 8).unwrap();
        
        assert!(cs.is_satisfied());
        assert_eq!(bits.len(), 8);
        
        // Check bits (little-endian): 10110101 = 181
        // bit 0 = 1, bit 1 = 0, bit 2 = 1, bit 3 = 0, bit 4 = 1, bit 5 = 1, bit 6 = 0, bit 7 = 1
        let expected = [1, 0, 1, 0, 1, 1, 0, 1];
        for (i, &exp) in expected.iter().enumerate() {
            let bit_val = bits[i].get_value().unwrap();
            assert_eq!(bit_val, F::from(exp as u64), "bit {} mismatch", i);
        }
    }
    
    #[test]
    fn test_lt64_less() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(F::from(100u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(F::from(200u64))).unwrap();
        
        let result = lt64(&mut cs.namespace(|| "lt64"), &a, &b).unwrap();
        
        assert!(cs.is_satisfied(), "constraints not satisfied");
        assert_eq!(result.get_value().unwrap(), F::ONE, "100 < 200 should be true");
    }
    
    #[test]
    fn test_lt64_greater() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(F::from(500u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(F::from(200u64))).unwrap();
        
        let result = lt64(&mut cs.namespace(|| "lt64"), &a, &b).unwrap();
        
        assert!(cs.is_satisfied(), "constraints not satisfied");
        assert_eq!(result.get_value().unwrap(), F::ZERO, "500 < 200 should be false");
    }
    
    #[test]
    fn test_lt64_equal() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(F::from(42u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(F::from(42u64))).unwrap();
        
        let result = lt64(&mut cs.namespace(|| "lt64"), &a, &b).unwrap();
        
        assert!(cs.is_satisfied(), "constraints not satisfied");
        assert_eq!(result.get_value().unwrap(), F::ZERO, "42 < 42 should be false");
    }
    
    #[test]
    fn test_lt64_large_values() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(F::from(u64::MAX - 100))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(F::from(u64::MAX - 50))).unwrap();
        
        let result = lt64(&mut cs.namespace(|| "lt64"), &a, &b).unwrap();
        
        assert!(cs.is_satisfied(), "constraints not satisfied");
        assert_eq!(result.get_value().unwrap(), F::ONE, "MAX-100 < MAX-50 should be true");
    }
    
    #[test]
    fn test_le64_equal() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(F::from(42u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(F::from(42u64))).unwrap();
        
        let result = le64(&mut cs.namespace(|| "le64"), &a, &b).unwrap();
        
        assert!(cs.is_satisfied(), "constraints not satisfied");
        assert_eq!(result.get_value().unwrap(), F::ONE, "42 <= 42 should be true");
    }
    
    #[test]
    fn test_verify_threshold_pass() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let energy = AllocatedNum::alloc(cs.namespace(|| "energy"), || Ok(F::from(100u64))).unwrap();
        let gap = AllocatedNum::alloc(cs.namespace(|| "gap"), || Ok(F::from(50u64))).unwrap();
        let threshold = AllocatedNum::alloc(cs.namespace(|| "threshold"), || Ok(F::from(200u64))).unwrap();
        
        let result = verify_threshold(&mut cs.namespace(|| "verify"), &energy, &gap, &threshold).unwrap();
        
        assert!(cs.is_satisfied(), "constraints not satisfied");
        assert_eq!(result.get_value().unwrap(), F::ONE, "100 + 50 <= 200 should pass");
    }
    
    #[test]
    fn test_verify_threshold_fail() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let energy = AllocatedNum::alloc(cs.namespace(|| "energy"), || Ok(F::from(100u64))).unwrap();
        let gap = AllocatedNum::alloc(cs.namespace(|| "gap"), || Ok(F::from(150u64))).unwrap();
        let threshold = AllocatedNum::alloc(cs.namespace(|| "threshold"), || Ok(F::from(200u64))).unwrap();
        
        let result = verify_threshold(&mut cs.namespace(|| "verify"), &energy, &gap, &threshold).unwrap();
        
        assert!(cs.is_satisfied(), "constraints not satisfied");
        assert_eq!(result.get_value().unwrap(), F::ZERO, "100 + 150 <= 200 should fail");
    }
    
    #[test]
    fn test_constraint_count() {
        let mut cs = TestConstraintSystem::<F>::new();
        
        let a = AllocatedNum::alloc(cs.namespace(|| "a"), || Ok(F::from(100u64))).unwrap();
        let b = AllocatedNum::alloc(cs.namespace(|| "b"), || Ok(F::from(200u64))).unwrap();
        
        let _ = lt64(&mut cs.namespace(|| "lt64"), &a, &b).unwrap();
        
        println!("Lt64 constraint count: {}", cs.num_constraints());
        // Should be around 65-70 constraints (65 bit constraints + decomposition + result)
        assert!(cs.num_constraints() < 100, "too many constraints: {}", cs.num_constraints());
    }
}
