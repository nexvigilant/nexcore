//! Flywheel node descriptors and tier classification.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Tier classification | Sum | Σ |
//! | Node identity | Existence | ∃ |
//! | Status tracking | State | ς |
//! | Crate membership | Mapping | μ |

use serde::{Deserialize, Serialize};
use std::fmt;

/// The three flywheel tiers, ordered by readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlywheelTier {
    /// Production-ready, actively processing events.
    Live,
    /// Wired and tested, awaiting promotion.
    Staging,
    /// Planned but not yet wired.
    Draft,
}

impl FlywheelTier {
    /// All tiers in readiness order (Live, Staging, Draft).
    pub const ALL: [FlywheelTier; 3] = [
        FlywheelTier::Live,
        FlywheelTier::Staging,
        FlywheelTier::Draft,
    ];

    /// Advance to the next tier (Draft→Staging→Live). Live stays Live.
    #[must_use]
    pub fn promote(self) -> Self {
        match self {
            Self::Draft => Self::Staging,
            Self::Staging => Self::Live,
            Self::Live => Self::Live,
        }
    }
}

impl fmt::Display for FlywheelTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Live => write!(f, "live"),
            Self::Staging => write!(f, "staging"),
            Self::Draft => write!(f, "draft"),
        }
    }
}

/// Operational status of a flywheel node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    /// Processing events in production.
    Active,
    /// Emitters connected, under integration testing.
    Wiring,
    /// Defined but not yet wired.
    Dormant,
}

impl fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Wiring => write!(f, "wiring"),
            Self::Dormant => write!(f, "dormant"),
        }
    }
}

/// Descriptor for a single flywheel node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDescriptor {
    /// Current tier classification.
    pub tier: FlywheelTier,
    /// Node identifier (e.g. "homeostasis").
    pub name: String,
    /// Workspace crates that compose this node.
    pub crates: Vec<String>,
    /// Operational status.
    pub status: NodeStatus,
}

impl NodeDescriptor {
    /// Create a new node descriptor.
    pub fn new(
        tier: FlywheelTier,
        name: impl Into<String>,
        crates: Vec<String>,
        status: NodeStatus,
    ) -> Self {
        Self {
            tier,
            name: name.into(),
            crates,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_promote_draft_to_staging() {
        assert_eq!(FlywheelTier::Draft.promote(), FlywheelTier::Staging);
    }
    #[test]
    fn tier_promote_staging_to_live() {
        assert_eq!(FlywheelTier::Staging.promote(), FlywheelTier::Live);
    }
    #[test]
    fn tier_promote_live_stays_live() {
        assert_eq!(FlywheelTier::Live.promote(), FlywheelTier::Live);
    }
    #[test]
    fn tier_display() {
        assert_eq!(format!("{}", FlywheelTier::Live), "live");
        assert_eq!(format!("{}", FlywheelTier::Staging), "staging");
        assert_eq!(format!("{}", FlywheelTier::Draft), "draft");
    }
    #[test]
    fn tier_all_has_three() {
        assert_eq!(FlywheelTier::ALL.len(), 3);
    }
    #[test]
    fn status_display() {
        assert_eq!(format!("{}", NodeStatus::Active), "active");
        assert_eq!(format!("{}", NodeStatus::Wiring), "wiring");
        assert_eq!(format!("{}", NodeStatus::Dormant), "dormant");
    }
    #[test]
    fn node_descriptor_new() {
        let n = NodeDescriptor::new(
            FlywheelTier::Live,
            "test",
            vec!["crate-a".into()],
            NodeStatus::Active,
        );
        assert_eq!(n.name, "test");
        assert_eq!(n.tier, FlywheelTier::Live);
        assert_eq!(n.crates.len(), 1);
    }
    #[test]
    fn tier_serialization_roundtrip() {
        let json = serde_json::to_string(&FlywheelTier::Staging).expect("ser");
        let back: FlywheelTier = serde_json::from_str(&json).expect("de");
        assert_eq!(back, FlywheelTier::Staging);
    }
    #[test]
    fn status_serialization_roundtrip() {
        let json = serde_json::to_string(&NodeStatus::Wiring).expect("ser");
        let back: NodeStatus = serde_json::from_str(&json).expect("de");
        assert_eq!(back, NodeStatus::Wiring);
    }
    #[test]
    fn descriptor_serialization_roundtrip() {
        let n = NodeDescriptor::new(FlywheelTier::Draft, "x", vec![], NodeStatus::Dormant);
        let json = serde_json::to_string(&n).expect("ser");
        let back: NodeDescriptor = serde_json::from_str(&json).expect("de");
        assert_eq!(back.name, "x");
        assert_eq!(back.tier, FlywheelTier::Draft);
    }
}
