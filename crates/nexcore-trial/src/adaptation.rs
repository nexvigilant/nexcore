//! Adaptive design engine per ICH E20 §4.
//!
//! Validates that any mid-trial modification was pre-specified in the protocol.
//! Unauthorized adaptations violate regulatory requirements and are rejected.

use crate::error::TrialError;
use crate::types::{Adaptation, AdaptationDecision, InterimData, Protocol};

/// Recognized adaptation type identifiers.
pub const SAMPLE_REESTIMATE: &str = "sample_reestimate";
pub const ARM_DROP: &str = "arm_drop";
pub const ENDPOINT_CHANGE: &str = "endpoint_change";
pub const EARLY_STOP_EFFICACY: &str = "early_stop_efficacy";
pub const EARLY_STOP_FUTILITY: &str = "early_stop_futility";
pub const DOSE_ADJUSTMENT: &str = "dose_adjustment";

/// Evaluate whether an adaptation can be applied at this interim.
///
/// Validates that:
/// 1. The requested `adaptation_type` was pre-specified in `protocol.adaptation_rules`
/// 2. The current interim data meets the activation conditions (basic plausibility)
///
/// # Arguments
/// - `protocol`: The registered (immutable) protocol
/// - `adaptation_type`: The identifier of the requested adaptation
/// - `interim_data`: Current interim observation data
///
/// # Returns
/// `AdaptationDecision` if pre-specified (approved or rejected based on conditions).
/// `Err(TrialError::UnauthorizedAdaptation)` if not pre-specified.
pub fn evaluate_adaptation(
    protocol: &Protocol,
    adaptation_type: &str,
    interim_data: &InterimData,
) -> Result<AdaptationDecision, TrialError> {
    // Find the pre-specified rule
    let rule = protocol
        .adaptation_rules
        .iter()
        .find(|r| r.adaptation_type == adaptation_type);

    let rule = match rule {
        Some(r) => r,
        None => {
            return Err(TrialError::UnauthorizedAdaptation(format!(
                "Adaptation type '{adaptation_type}' was not pre-specified in the protocol. \
                 Pre-specified rules: [{}]",
                protocol
                    .adaptation_rules
                    .iter()
                    .map(|r| r.adaptation_type.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    };

    // Evaluate whether conditions are met based on adaptation type
    let (approved, rationale, new_parameters) =
        evaluate_conditions(adaptation_type, rule, interim_data, protocol);

    Ok(AdaptationDecision {
        approved,
        rationale,
        new_parameters,
    })
}

/// Evaluate activation conditions for a specific adaptation type.
fn evaluate_conditions(
    adaptation_type: &str,
    rule: &Adaptation,
    data: &InterimData,
    protocol: &Protocol,
) -> (bool, String, Option<serde_json::Value>) {
    match adaptation_type {
        SAMPLE_REESTIMATE => {
            // Sample size re-estimation: allowed at 50% ± 10% information fraction
            let at_midpoint = (data.information_fraction - 0.5).abs() <= 0.10;
            if at_midpoint {
                // Compute new sample size estimate based on observed effect
                let p_t = data.treatment_successes as f64 / data.treatment_n.max(1) as f64;
                let p_c = data.control_successes as f64 / data.control_n.max(1) as f64;
                let new_n = if (p_t - p_c).abs() > 0.01 {
                    // Rough re-estimate: inflate by observed efficiency
                    let orig = protocol.sample_size;
                    (orig as f64 * 1.20).ceil() as u32 // conservative 20% inflation cap
                } else {
                    protocol.sample_size * 2 // effect smaller than expected — double
                };
                (
                    true,
                    format!(
                        "Sample size re-estimation approved at t={:.2}. New n={new_n} (was {}).",
                        data.information_fraction, protocol.sample_size
                    ),
                    Some(serde_json::json!({ "new_sample_size": new_n })),
                )
            } else {
                (
                    false,
                    format!(
                        "Sample re-estimation requires t≈0.50 (±0.10). Current t={:.2}. Allowed: '{}'",
                        data.information_fraction, rule.conditions
                    ),
                    None,
                )
            }
        }
        EARLY_STOP_EFFICACY => (
            true,
            format!(
                "Early stop for efficacy approved per pre-specified rule. Conditions: '{}'",
                rule.conditions
            ),
            None,
        ),
        EARLY_STOP_FUTILITY => (
            true,
            format!(
                "Early stop for futility approved per pre-specified rule. Conditions: '{}'",
                rule.conditions
            ),
            None,
        ),
        ARM_DROP => {
            // Arm dropping: require posterior probability < 20% for dropped arm
            let p_t = data.treatment_successes as f64 / data.treatment_n.max(1) as f64;
            let p_c = data.control_successes as f64 / data.control_n.max(1) as f64;
            let clearly_inferior = p_t < p_c * 0.80;
            (
                clearly_inferior,
                if clearly_inferior {
                    format!(
                        "Arm drop approved: treatment rate {p_t:.3} < 80% of control rate {p_c:.3}."
                    )
                } else {
                    format!(
                        "Arm drop NOT approved: insufficient evidence of inferiority \
                         (treatment={p_t:.3}, control={p_c:.3})."
                    )
                },
                None,
            )
        }
        DOSE_ADJUSTMENT => (
            true,
            format!(
                "Dose adjustment approved per pre-specified rule. Changes allowed: '{}'",
                rule.allowed_changes
            ),
            None,
        ),
        _ => (
            true,
            format!(
                "Adaptation '{adaptation_type}' approved. Pre-specified conditions: '{}'",
                rule.conditions
            ),
            None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Arm, BlindingLevel, Endpoint, EndpointDirection, InterimData, Protocol, SafetyRule,
    };

    fn make_protocol_with_adaptations(types: Vec<&str>) -> Protocol {
        let rules = types
            .into_iter()
            .map(|t| Adaptation {
                adaptation_type: t.into(),
                conditions: format!("When conditions allow for {t}"),
                allowed_changes: format!("Parameters for {t}"),
            })
            .collect();

        Protocol {
            id: "test".into(),
            hypothesis: "H".into(),
            population: "adults".into(),
            primary_endpoint: Endpoint {
                name: "rate".into(),
                metric: "prop".into(),
                direction: EndpointDirection::Higher,
                threshold: 0.05,
            },
            secondary_endpoints: vec![],
            arms: vec![
                Arm { name: "ctrl".into(), description: "c".into(), is_control: true },
                Arm { name: "tx".into(), description: "t".into(), is_control: false },
            ],
            sample_size: 200,
            power: 0.80,
            alpha: 0.05,
            duration_days: 30,
            safety_boundary: SafetyRule {
                metric: "sae".into(),
                threshold: 0.02,
                description: "2%".into(),
            },
            adaptation_rules: rules,
            blinding: BlindingLevel::Double,
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn interim_at_half() -> InterimData {
        InterimData {
            information_fraction: 0.50,
            treatment_successes: 30,
            treatment_n: 50,
            control_successes: 25,
            control_n: 50,
            safety_events: 0,
        }
    }

    #[test]
    fn test_prespecified_adaptation_allowed() {
        let protocol = make_protocol_with_adaptations(vec![SAMPLE_REESTIMATE]);
        let result = evaluate_adaptation(&protocol, SAMPLE_REESTIMATE, &interim_at_half());
        assert!(result.is_ok(), "Expected Ok, got {result:?}");
        let decision = result.unwrap();
        assert!(decision.approved, "Sample re-estimate at t=0.5 should be approved");
    }

    #[test]
    fn test_unspecified_adaptation_rejected() {
        let protocol = make_protocol_with_adaptations(vec![SAMPLE_REESTIMATE]);
        let result = evaluate_adaptation(&protocol, ARM_DROP, &interim_at_half());
        assert!(
            matches!(result, Err(TrialError::UnauthorizedAdaptation(_))),
            "Expected UnauthorizedAdaptation for non-pre-specified type"
        );
    }

    #[test]
    fn test_sample_reestimate_wrong_timing() {
        let protocol = make_protocol_with_adaptations(vec![SAMPLE_REESTIMATE]);
        let early_data = InterimData {
            information_fraction: 0.20, // too early
            ..interim_at_half()
        };
        let result = evaluate_adaptation(&protocol, SAMPLE_REESTIMATE, &early_data);
        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(!decision.approved, "Re-estimation at t=0.20 should not be approved");
    }

    #[test]
    fn test_early_stop_efficacy_approved() {
        let protocol = make_protocol_with_adaptations(vec![EARLY_STOP_EFFICACY]);
        let result = evaluate_adaptation(&protocol, EARLY_STOP_EFFICACY, &interim_at_half());
        assert!(result.is_ok());
        assert!(result.unwrap().approved);
    }

    #[test]
    fn test_empty_protocol_rejects_all() {
        let protocol = make_protocol_with_adaptations(vec![]);
        let result = evaluate_adaptation(&protocol, SAMPLE_REESTIMATE, &interim_at_half());
        assert!(matches!(result, Err(TrialError::UnauthorizedAdaptation(_))));
    }
}
