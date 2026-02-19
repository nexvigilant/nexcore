//! Randomization schemes per ICH E9(R1) §2.3
//!
//! Tier: T2-P (σ+∂+μ — Randomization)
//!
//! Three schemes: simple (coin flip), block (permuted blocks), stratified (block within strata).
//! Optional deterministic seed for reproducibility and audit trail.

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::error::TrialError;
use crate::types::{ArmAssignment, Stratum};

// ── Simple Randomization ─────────────────────────────────────────────────────

/// Assign subjects to arms using simple (coin-flip) randomization.
///
/// Not guaranteed to be balanced, but unbiased. Suitable for large n.
///
/// # Arguments
/// - `n`: Total number of subjects to assign
/// - `n_arms`: Number of trial arms (must be >= 2)
/// - `seed`: Optional random seed for reproducibility
///
/// # Returns
/// Vector of `ArmAssignment` — one per subject, in enrollment order.
pub fn simple_randomize(
    n: u32,
    n_arms: usize,
    seed: Option<u64>,
) -> Result<Vec<ArmAssignment>, TrialError> {
    if n_arms < 2 {
        return Err(TrialError::InvalidProtocol(
            "n_arms must be >= 2 for randomization".into(),
        ));
    }
    if n == 0 {
        return Err(TrialError::InvalidParameter(
            "n must be > 0".into(),
        ));
    }

    let mut rng = make_rng(seed);
    let mut assignments = Vec::with_capacity(n as usize);

    for subject_id in 0..n {
        let arm_index: usize = rng.random_range(0..n_arms);
        assignments.push(ArmAssignment {
            subject_id,
            arm_index,
            stratum: None,
            block_id: None,
        });
    }

    Ok(assignments)
}

// ── Block Randomization ──────────────────────────────────────────────────────

/// Assign subjects using permuted-block randomization.
///
/// Guarantees balance within every block. Each block contains exactly
/// `block_size / n_arms` subjects per arm.
///
/// # Arguments
/// - `n`: Total subjects to assign
/// - `n_arms`: Number of arms (must divide `block_size` evenly)
/// - `block_size`: Size of each permuted block (must be multiple of `n_arms`)
/// - `seed`: Optional seed for reproducibility
pub fn block_randomize(
    n: u32,
    n_arms: usize,
    block_size: usize,
    seed: Option<u64>,
) -> Result<Vec<ArmAssignment>, TrialError> {
    if n_arms < 2 {
        return Err(TrialError::InvalidProtocol(
            "n_arms must be >= 2".into(),
        ));
    }
    if block_size == 0 || block_size % n_arms != 0 {
        return Err(TrialError::InvalidParameter(format!(
            "block_size ({block_size}) must be a positive multiple of n_arms ({n_arms})"
        )));
    }
    if n == 0 {
        return Err(TrialError::InvalidParameter("n must be > 0".into()));
    }

    let per_arm = block_size / n_arms;
    let mut rng = make_rng(seed);
    let mut assignments = Vec::with_capacity(n as usize);
    let mut subject_id: u32 = 0;
    let mut block_id: u32 = 0;

    while subject_id < n {
        // Build one block: per_arm copies of each arm index
        let mut block: Vec<usize> = (0..n_arms)
            .flat_map(|arm| std::iter::repeat(arm).take(per_arm))
            .collect();
        block.shuffle(&mut rng);

        for arm_index in block {
            if subject_id >= n {
                break;
            }
            assignments.push(ArmAssignment {
                subject_id,
                arm_index,
                stratum: None,
                block_id: Some(block_id),
            });
            subject_id += 1;
        }
        block_id += 1;
    }

    Ok(assignments)
}

// ── Stratified Randomization ─────────────────────────────────────────────────

/// Assign subjects using stratified block randomization.
///
/// Performs permuted-block randomization independently within each stratum,
/// ensuring balance across stratification factors (e.g., age, site).
///
/// # Arguments
/// - `strata`: List of strata — each with an ID and subject count
/// - `n_arms`: Number of arms
/// - `block_size`: Block size (must divide `n_arms` evenly)
/// - `seed`: Optional seed
pub fn stratified_randomize(
    strata: &[Stratum],
    n_arms: usize,
    block_size: usize,
    seed: Option<u64>,
) -> Result<Vec<ArmAssignment>, TrialError> {
    if strata.is_empty() {
        return Err(TrialError::InvalidParameter("strata must not be empty".into()));
    }

    let base_seed = seed.unwrap_or(12_345);
    let mut all_assignments: Vec<ArmAssignment> = Vec::new();
    let mut global_id: u32 = 0;

    for (i, stratum) in strata.iter().enumerate() {
        // Each stratum gets a deterministic but distinct seed derived from base
        let stratum_seed = base_seed.wrapping_add(i as u64 * 97_331);
        let raw = block_randomize(stratum.n, n_arms, block_size, Some(stratum_seed))?;

        for mut a in raw {
            a.subject_id = global_id;
            a.stratum = Some(stratum.id.clone());
            all_assignments.push(a);
            global_id += 1;
        }
    }

    Ok(all_assignments)
}

/// Compute a simple audit hash over the assignment list.
///
/// Uses a deterministic XOR-rotate accumulator over (subject_id, arm_index) pairs.
/// Sufficient for integrity checking (not cryptographic).
pub fn randomization_hash(assignments: &[ArmAssignment]) -> u64 {
    let mut acc: u64 = 0xcafe_babe_dead_beef;
    for a in assignments {
        let pair = (u64::from(a.subject_id) << 32) | (a.arm_index as u64);
        acc = acc.rotate_left(13).wrapping_add(pair).wrapping_mul(6_364_136_223_846_793_005);
    }
    acc
}

// ── Internal ─────────────────────────────────────────────────────────────────

fn make_rng(seed: Option<u64>) -> StdRng {
    let s = seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xcafe_babe_dead_beef)
    });
    StdRng::seed_from_u64(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_randomization_all_assigned() {
        let assignments = simple_randomize(100, 2, Some(42));
        assert!(assignments.is_ok());
        let a = assignments.unwrap();
        assert_eq!(a.len(), 100);
        // All arm indices valid
        assert!(a.iter().all(|x| x.arm_index < 2));
    }

    #[test]
    fn test_block_randomization_balanced_within_blocks() {
        let assignments = block_randomize(100, 2, 4, Some(42));
        assert!(assignments.is_ok());
        let a = assignments.unwrap();
        assert_eq!(a.len(), 100);

        // Each block of 4 should have exactly 2 of each arm
        let blocks: std::collections::BTreeMap<u32, Vec<usize>> =
            a.iter().fold(std::collections::BTreeMap::new(), |mut acc, x| {
                acc.entry(x.block_id.unwrap_or(0)).or_default().push(x.arm_index);
                acc
            });

        // All complete blocks (4 subjects) must have exactly 2 per arm
        for (_bid, arms) in blocks.iter().filter(|(_, v)| v.len() == 4) {
            let arm0 = arms.iter().filter(|&&x| x == 0).count();
            let arm1 = arms.iter().filter(|&&x| x == 1).count();
            assert_eq!(arm0, 2, "Block must have 2 subjects in arm 0");
            assert_eq!(arm1, 2, "Block must have 2 subjects in arm 1");
        }
    }

    #[test]
    fn test_block_randomization_exact_count() {
        // When n is exact multiple of block_size, all assignments accounted for
        let assignments = block_randomize(40, 2, 4, Some(7)).unwrap();
        assert_eq!(assignments.len(), 40);
        let arm0 = assignments.iter().filter(|a| a.arm_index == 0).count();
        let arm1 = assignments.iter().filter(|a| a.arm_index == 1).count();
        assert_eq!(arm0, 20);
        assert_eq!(arm1, 20);
    }

    #[test]
    fn test_stratified_randomization_balanced_per_stratum() {
        let strata = vec![
            Stratum { id: "age_under_50".into(), n: 60 },
            Stratum { id: "age_over_50".into(), n: 40 },
        ];
        let assignments = stratified_randomize(&strata, 2, 4, Some(99));
        assert!(assignments.is_ok());
        let a = assignments.unwrap();
        assert_eq!(a.len(), 100);

        // Each stratum should appear the right number of times
        let s1 = a.iter().filter(|x| x.stratum.as_deref() == Some("age_under_50")).count();
        let s2 = a.iter().filter(|x| x.stratum.as_deref() == Some("age_over_50")).count();
        assert_eq!(s1, 60);
        assert_eq!(s2, 40);
    }

    #[test]
    fn test_reproducible_with_same_seed() {
        let a1 = block_randomize(50, 2, 4, Some(1234)).unwrap();
        let a2 = block_randomize(50, 2, 4, Some(1234)).unwrap();
        let indices1: Vec<usize> = a1.iter().map(|x| x.arm_index).collect();
        let indices2: Vec<usize> = a2.iter().map(|x| x.arm_index).collect();
        assert_eq!(indices1, indices2, "Same seed must produce same assignments");
    }

    #[test]
    fn test_different_seeds_differ() {
        let a1 = block_randomize(100, 2, 4, Some(1)).unwrap();
        let a2 = block_randomize(100, 2, 4, Some(2)).unwrap();
        let indices1: Vec<usize> = a1.iter().map(|x| x.arm_index).collect();
        let indices2: Vec<usize> = a2.iter().map(|x| x.arm_index).collect();
        assert_ne!(indices1, indices2, "Different seeds should produce different sequences");
    }

    #[test]
    fn test_invalid_block_size() {
        let result = block_randomize(10, 3, 4, Some(1)); // 4 % 3 != 0
        assert!(matches!(result, Err(TrialError::InvalidParameter(_))));
    }

    #[test]
    fn test_randomization_hash_deterministic() {
        let a = block_randomize(20, 2, 4, Some(42)).unwrap();
        let h1 = randomization_hash(&a);
        let h2 = randomization_hash(&a);
        assert_eq!(h1, h2);
    }
}
