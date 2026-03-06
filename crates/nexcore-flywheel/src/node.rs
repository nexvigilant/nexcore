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
    Live,
    Staging,
    Draft,
}

impl FlywheelTier {
    pub const ALL: [FlywheelTier; 3] = [
        FlywheelTier::Live,
        FlywheelTier::Staging,
        FlywheelTier::Draft,
    ];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Active,
    Wiring,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub tier: FlywheelTier,
    pub name: String,
    pub crates: Vec<String>,
    pub status: NodeStatus,
}

impl NodeDescriptor {
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
