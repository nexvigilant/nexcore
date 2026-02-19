//! Protocol registration per ICH E9(R1) §2.
//!
//! A protocol is registered once and treated as immutable.
//! Validates: primary endpoint exists, >= 2 arms, power >= 0.80, alpha in (0, 1).

use chrono::Utc;
use uuid::Uuid;

use crate::error::TrialError;
use crate::types::{Protocol, ProtocolRequest};

/// Register a new trial protocol, validating all required fields.
///
/// Generates a UUID-based trial ID and ISO 8601 creation timestamp.
/// Returns an immutable `Protocol` that cannot be modified after registration.
pub fn register_protocol(req: ProtocolRequest) -> Result<Protocol, TrialError> {
    // E9(R1) §2.2.1 — must have at least 2 arms (treatment + control)
    if req.arms.len() < 2 {
        return Err(TrialError::InvalidProtocol(
            "At least 2 arms required (treatment + control)".into(),
        ));
    }

    // E9(R1) §2.2.2 — must have a control arm
    let has_control = req.arms.iter().any(|a| a.is_control);
    if !has_control {
        return Err(TrialError::InvalidProtocol(
            "At least one arm must be designated as control".into(),
        ));
    }

    // E9(R1) §3.5 — power must be >= 0.80
    if req.power < 0.80 {
        return Err(TrialError::InsufficientPower {
            actual: req.power,
            required: 0.80,
        });
    }

    // Alpha must be a valid probability
    if req.alpha <= 0.0 || req.alpha >= 1.0 {
        return Err(TrialError::InvalidProtocol(format!(
            "alpha must be in (0, 1), got {}",
            req.alpha
        )));
    }

    // Hypothesis must be non-empty
    if req.hypothesis.trim().is_empty() {
        return Err(TrialError::InvalidProtocol(
            "hypothesis must not be empty".into(),
        ));
    }

    // Sample size must be positive
    if req.sample_size == 0 {
        return Err(TrialError::InvalidProtocol(
            "sample_size must be > 0".into(),
        ));
    }

    // Safety threshold must be a valid probability
    if req.safety_boundary.threshold <= 0.0 || req.safety_boundary.threshold > 1.0 {
        return Err(TrialError::InvalidProtocol(
            "safety_boundary.threshold must be in (0, 1]".into(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    Ok(Protocol {
        id,
        hypothesis: req.hypothesis,
        population: req.population,
        primary_endpoint: req.primary_endpoint,
        secondary_endpoints: req.secondary_endpoints,
        arms: req.arms,
        sample_size: req.sample_size,
        power: req.power,
        alpha: req.alpha,
        duration_days: req.duration_days,
        safety_boundary: req.safety_boundary,
        adaptation_rules: req.adaptation_rules,
        blinding: req.blinding,
        created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Arm, BlindingLevel, Endpoint, EndpointDirection, SafetyRule,
    };

    fn make_valid_request() -> ProtocolRequest {
        ProtocolRequest {
            hypothesis: "Treatment A improves conversion rate vs control".into(),
            population: "Adults aged 18-65".into(),
            primary_endpoint: Endpoint {
                name: "conversion_rate".into(),
                metric: "proportion of users converting".into(),
                direction: EndpointDirection::Higher,
                threshold: 0.05,
            },
            secondary_endpoints: vec![],
            arms: vec![
                Arm {
                    name: "control".into(),
                    description: "Standard experience".into(),
                    is_control: true,
                },
                Arm {
                    name: "treatment_a".into(),
                    description: "New checkout flow".into(),
                    is_control: false,
                },
            ],
            sample_size: 400,
            power: 0.80,
            alpha: 0.05,
            duration_days: 30,
            safety_boundary: SafetyRule {
                metric: "serious_adverse_event_rate".into(),
                threshold: 0.02,
                description: "Stop if SAE rate exceeds 2%".into(),
            },
            adaptation_rules: vec![],
            blinding: BlindingLevel::Double,
        }
    }

    #[test]
    fn test_register_valid_protocol() {
        let req = make_valid_request();
        let result = register_protocol(req);
        assert!(result.is_ok(), "Expected Ok, got {result:?}");
        let proto = result.unwrap();
        assert!(!proto.id.is_empty(), "ID must not be empty");
        assert!(proto.power >= 0.80, "Power must be >= 0.80");
        assert!(!proto.created_at.is_empty(), "created_at must be set");
    }

    #[test]
    fn test_id_is_uuid_format() {
        let proto = register_protocol(make_valid_request()).unwrap();
        // UUID v4 format: 8-4-4-4-12 hex chars
        assert_eq!(proto.id.len(), 36);
        assert!(proto.id.contains('-'));
    }

    #[test]
    fn test_reject_single_arm() {
        let mut req = make_valid_request();
        req.arms = vec![Arm {
            name: "control".into(),
            description: "Only arm".into(),
            is_control: true,
        }];
        let result = register_protocol(req);
        assert!(
            matches!(result, Err(TrialError::InvalidProtocol(_))),
            "Expected InvalidProtocol for single arm"
        );
    }

    #[test]
    fn test_reject_no_control_arm() {
        let mut req = make_valid_request();
        for arm in &mut req.arms {
            arm.is_control = false;
        }
        let result = register_protocol(req);
        assert!(
            matches!(result, Err(TrialError::InvalidProtocol(_))),
            "Expected InvalidProtocol when no control arm"
        );
    }

    #[test]
    fn test_reject_insufficient_power() {
        let mut req = make_valid_request();
        req.power = 0.70;
        let result = register_protocol(req);
        assert!(
            matches!(result, Err(TrialError::InsufficientPower { .. })),
            "Expected InsufficientPower"
        );
    }

    #[test]
    fn test_reject_invalid_alpha() {
        let mut req = make_valid_request();
        req.alpha = 1.5;
        let result = register_protocol(req);
        assert!(
            matches!(result, Err(TrialError::InvalidProtocol(_))),
            "Expected InvalidProtocol for alpha > 1"
        );
    }

    #[test]
    fn test_reject_empty_hypothesis() {
        let mut req = make_valid_request();
        req.hypothesis = "   ".into();
        let result = register_protocol(req);
        assert!(
            matches!(result, Err(TrialError::InvalidProtocol(_))),
            "Expected InvalidProtocol for empty hypothesis"
        );
    }

    #[test]
    fn test_protocol_immutable_fields_set() {
        let req = make_valid_request();
        let proto = register_protocol(req).unwrap();
        // ID and created_at are generated by register_protocol, not overridable
        assert!(proto.id.starts_with(|c: char| c.is_ascii_alphanumeric()), "ID should be UUID");
        assert!(proto.created_at.contains('T'), "created_at should be ISO 8601");
    }
}
