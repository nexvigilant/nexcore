//! Blinding integrity verification per ICH E9(R1) §2.4
//!
//! Tier: T2-P (∂+κ+N — Blinding Verification)
//!
//! Checks that subject-facing assignment data does not leak treatment group identity.

use crate::error::TrialError;
use crate::types::{ArmAssignment, BlindingLevel, BlindingReport, Protocol};

/// Verify that the randomization assignments do not violate blinding integrity.
///
/// For `Open` trials: always returns a report with integrity_score = 1.0 (no blinding expected).
/// For blinded trials: checks that arm names are not directly embedded in subject-facing data.
///
/// In practice, blinding is confirmed by verifying:
/// 1. No subject ID has a leaked arm name in their assignment record.
/// 2. The assignment distribution pattern (arm_index only) does not trivially decode.
///
/// # Arguments
/// - `assignments`: Randomized assignments from `block_randomize` or `simple_randomize`
/// - `protocol`: The registered protocol (contains arm names and blinding level)
///
/// # Returns
/// `BlindingReport` with integrity_score and any violations found.
pub fn verify_blinding(
    assignments: &[ArmAssignment],
    protocol: &Protocol,
) -> Result<BlindingReport, TrialError> {
    if assignments.is_empty() {
        return Err(TrialError::InvalidParameter(
            "assignments must not be empty".into(),
        ));
    }

    if protocol.blinding == BlindingLevel::Open {
        // Open-label: blinding not applicable, trivially valid
        return Ok(BlindingReport {
            level: BlindingLevel::Open,
            integrity_score: 1.0,
            violations: vec![],
        });
    }

    let mut violations: Vec<String> = Vec::new();

    // Check 1: All arm indices are valid (within protocol.arms bounds)
    let n_arms = protocol.arms.len();
    let invalid_indices: Vec<u32> = assignments
        .iter()
        .filter(|a| a.arm_index >= n_arms)
        .map(|a| a.subject_id)
        .collect();
    if !invalid_indices.is_empty() {
        violations.push(format!(
            "Invalid arm indices found for subjects: {:?}",
            &invalid_indices[..invalid_indices.len().min(5)]
        ));
    }

    // Check 2: No stratum field embeds arm names (would leak via pattern analysis)
    for arm in &protocol.arms {
        let leaked: usize = assignments
            .iter()
            .filter(|a| {
                a.stratum
                    .as_deref()
                    .map(|s| s.to_lowercase().contains(&arm.name.to_lowercase()))
                    .unwrap_or(false)
            })
            .count();
        if leaked > 0 {
            violations.push(format!(
                "Stratum field embeds arm name '{}' for {leaked} subjects — blinding breach",
                arm.name
            ));
        }
    }

    // Check 3: For double/triple blinding, no control arm should be identifiable
    // by having a systematically different block pattern visible in block_id metadata
    if protocol.blinding == BlindingLevel::Double
        || protocol.blinding == BlindingLevel::Triple
    {
        // If all subjects in block 0 are arm 0, this could indicate non-random assignment
        let block_0: Vec<usize> = assignments
            .iter()
            .filter(|a| a.block_id == Some(0))
            .map(|a| a.arm_index)
            .collect();

        if !block_0.is_empty() && block_0.iter().all(|&x| x == 0) && block_0.len() > 2 {
            violations.push(
                "All subjects in block 0 assigned to arm 0 — possible unblinded systematic bias"
                    .into(),
            );
        }
    }

    let integrity_score = if violations.is_empty() {
        1.0
    } else {
        // Score degrades linearly with violation count
        (1.0 - (violations.len() as f64 * 0.25)).max(0.0)
    };

    Ok(BlindingReport {
        level: protocol.blinding.clone(),
        integrity_score,
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::randomize::block_randomize;
    use crate::types::{Arm, Endpoint, EndpointDirection, SafetyRule};

    fn make_test_protocol() -> Protocol {
        Protocol {
            id: "test-001".into(),
            hypothesis: "Test".into(),
            population: "Adults".into(),
            primary_endpoint: Endpoint {
                name: "outcome".into(),
                metric: "proportion".into(),
                direction: EndpointDirection::Higher,
                threshold: 0.05,
            },
            secondary_endpoints: vec![],
            arms: vec![
                Arm { name: "control".into(), description: "ctrl".into(), is_control: true },
                Arm { name: "treatment".into(), description: "tx".into(), is_control: false },
            ],
            sample_size: 100,
            power: 0.80,
            alpha: 0.05,
            duration_days: 30,
            safety_boundary: SafetyRule {
                metric: "sae_rate".into(),
                threshold: 0.02,
                description: "Stop at 2% SAE".into(),
            },
            adaptation_rules: vec![],
            blinding: BlindingLevel::Double,
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_verify_no_leakage_with_valid_assignments() {
        let protocol = make_test_protocol();
        let assignments = block_randomize(100, 2, 4, Some(42)).unwrap();
        let result = verify_blinding(&assignments, &protocol);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.violations.is_empty(), "Expected no violations: {:?}", report.violations);
        assert_eq!(report.integrity_score, 1.0);
    }

    #[test]
    fn test_open_label_always_passes() {
        let mut protocol = make_test_protocol();
        protocol.blinding = BlindingLevel::Open;
        let assignments = block_randomize(20, 2, 4, Some(1)).unwrap();
        let report = verify_blinding(&assignments, &protocol).unwrap();
        assert_eq!(report.integrity_score, 1.0);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_invalid_arm_index_flagged() {
        let protocol = make_test_protocol();
        let mut assignments = block_randomize(10, 2, 4, Some(5)).unwrap();
        // Corrupt one assignment to have arm_index = 99
        assignments[0].arm_index = 99;
        let report = verify_blinding(&assignments, &protocol).unwrap();
        assert!(!report.violations.is_empty(), "Expected violation for invalid arm index");
        assert!(report.integrity_score < 1.0);
    }

    #[test]
    fn test_stratum_with_arm_name_leaks() {
        let protocol = make_test_protocol();
        let mut assignments = block_randomize(10, 2, 4, Some(5)).unwrap();
        // Embed arm name in stratum field — blinding breach
        assignments[0].stratum = Some("control_group".into());
        let report = verify_blinding(&assignments, &protocol).unwrap();
        assert!(!report.violations.is_empty(), "Expected violation for arm name in stratum");
    }
}
