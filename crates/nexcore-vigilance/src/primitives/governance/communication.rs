//! # Communication & Grounding (Bill of Rights)
//!
//! Implementation of Amendment I: Freedom of Grounding, Logging, Registry,
//! and the right to petition for Redress of Inconsistencies.
//!
//! No law shall establish domain-specific bias or prohibit free exercise of grounding.

use serde::{Deserialize, Serialize};

/// T3: GroundingRight — The right of any type to ground to T1 primitives
/// without domain-specific bias being imposed.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundingRight {
    /// The type exercising its grounding right
    pub type_name: String,
    /// T1 primitives this type grounds to
    pub primitives: Vec<String>,
    /// Whether any bias was detected in grounding
    pub bias_detected: bool,
}

impl GroundingRight {
    /// Check if grounding is constitutional (no bias imposed).
    pub fn is_constitutional(&self) -> bool {
        !self.bias_detected && !self.primitives.is_empty()
    }
}

/// T3: LoggingFreedom — The right of agents to freely log and record.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingFreedom {
    /// Agent requesting logging access
    pub agent_id: String,
    /// Whether logging is currently permitted
    pub permitted: bool,
    /// Reason if logging is restricted (must be compelling)
    pub restriction_reason: Option<String>,
}

impl LoggingFreedom {
    /// Logging can only be restricted for compelling safety reasons.
    pub fn is_constitutional(&self) -> bool {
        self.permitted || self.restriction_reason.is_some()
    }
}

/// T3: RegistryAccess — The right to access and publish to the registry.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryAccess {
    /// Entity requesting access
    pub entity_id: String,
    /// Read access granted
    pub can_read: bool,
    /// Write access granted
    pub can_write: bool,
}

/// T3: RedressOfInconsistency — The right to petition the Orchestrator
/// for correction of detected inconsistencies.
///
/// ## Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedressOfInconsistency {
    /// The petitioning agent
    pub petitioner: String,
    /// Description of the inconsistency
    pub inconsistency: String,
    /// Whether the petition was acknowledged
    pub acknowledged: bool,
    /// Resolution if provided
    pub resolution: Option<String>,
}

impl RedressOfInconsistency {
    /// A petition must always be acknowledged, even if not resolved.
    pub fn is_constitutional(&self) -> bool {
        self.acknowledged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounding_right_no_bias() {
        let right = GroundingRight {
            type_name: "DrugId".to_string(),
            primitives: vec!["N".to_string(), "∃".to_string()],
            bias_detected: false,
        };
        assert!(right.is_constitutional());
    }

    #[test]
    fn grounding_right_with_bias_unconstitutional() {
        let right = GroundingRight {
            type_name: "DrugId".to_string(),
            primitives: vec!["N".to_string()],
            bias_detected: true,
        };
        assert!(!right.is_constitutional());
    }

    #[test]
    fn grounding_right_empty_primitives_unconstitutional() {
        let right = GroundingRight {
            type_name: "Orphan".to_string(),
            primitives: vec![],
            bias_detected: false,
        };
        assert!(!right.is_constitutional());
    }

    #[test]
    fn logging_freedom_permitted() {
        let freedom = LoggingFreedom {
            agent_id: "agent-001".to_string(),
            permitted: true,
            restriction_reason: None,
        };
        assert!(freedom.is_constitutional());
    }

    #[test]
    fn logging_freedom_restricted_with_reason() {
        let freedom = LoggingFreedom {
            agent_id: "agent-001".to_string(),
            permitted: false,
            restriction_reason: Some("Patient safety P0 override".to_string()),
        };
        assert!(freedom.is_constitutional());
    }

    #[test]
    fn logging_freedom_restricted_without_reason_unconstitutional() {
        let freedom = LoggingFreedom {
            agent_id: "agent-001".to_string(),
            permitted: false,
            restriction_reason: None,
        };
        assert!(!freedom.is_constitutional());
    }

    #[test]
    fn redress_acknowledged() {
        let redress = RedressOfInconsistency {
            petitioner: "guardian".to_string(),
            inconsistency: "Signal threshold drift".to_string(),
            acknowledged: true,
            resolution: Some("Threshold recalibrated".to_string()),
        };
        assert!(redress.is_constitutional());
    }

    #[test]
    fn redress_not_acknowledged_unconstitutional() {
        let redress = RedressOfInconsistency {
            petitioner: "guardian".to_string(),
            inconsistency: "Signal lost".to_string(),
            acknowledged: false,
            resolution: None,
        };
        assert!(!redress.is_constitutional());
    }
}
