//! # Capability 32: International Development Act (Cross-Union Sharing)
//!
//! Implementation of the International Development Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Inter-System Resource Sharing" and "External Union Development".
//!
//! Matches 1:1 to the US Agency for International Development (USAID) mandate
//! to lead the U.S. Government's international development and disaster
//! assistance efforts.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: InternationalDevelopmentAct - Capability 32 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternationalDevelopmentAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether cross-union assistance is active.
    pub sharing_active: bool,
}

/// T2-P: StabilityIndex - The quantified health of an external system.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct StabilityIndex(pub f64);

/// T2-C: AssistancePackage - A transfer of resources to another system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistancePackage {
    /// The identifier of the target system.
    pub target_system: String,
    /// The type of resource being shared.
    pub resource_type: String,
    /// The quantity being transferred.
    pub quantity: u64,
}

impl InternationalDevelopmentAct {
    /// Creates a new instance of the InternationalDevelopmentAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-032".into(),
            sharing_active: true,
        }
    }

    /// Dispatch an assistance package to an external system.
    pub fn dispatch_assistance(&self, target: &str, resource: &str) -> Measured<AssistancePackage> {
        let package = AssistancePackage {
            target_system: target.to_string(),
            resource_type: resource.to_string(),
            quantity: 1000,
        };

        Measured::uncertain(package, Confidence::new(0.95))
    }
}
