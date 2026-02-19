//! # Capability 7: Data Sovereignty Act (Interior Domain)
//!
//! Implementation of the Data Sovereignty Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Natural Resources" (State & Data) of the Union's domains.
//!
//! Matches 1:1 to the US Department of the Interior (DOI) mandate for
//! the management and conservation of most federal land and natural resources.
//!
//! ## DOI Agency Mappings
//! - **Bureau of Land Management (BLM):** Manages the shared state space (The Common).
//! - **National Park Service (NPS):** Protects "Immutable Modules" and historical state logs.
//! - **U.S. Fish and Wildlife Service:** Manages the lifecycle of "Living Agents" and their local state.
//! - **Bureau of Ocean Energy Management:** Manages external data streams (The "Data Lakes").

use crate::primitives::governance::{SovereignDomain, Verdict};
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: DataSovereigntyAct - Capability 7 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSovereigntyAct {
    pub id: String,
    pub conservation_active: bool,
}

/// T2-P: ResourceType - The type of data resource being managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Shared state (BLM managed).
    SharedState,
    /// Protected immutable module (NPS managed).
    ImmutableModule,
    /// Local agent state (Fish & Wildlife managed).
    AgentLocalState,
}

/// T2-C: ResourceLease - A contract for data access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLease {
    pub domain_id: String,
    pub resource_type: ResourceType,
    pub duration_cycles: u32,
    pub protection_level: Confidence,
}

impl DataSovereigntyAct {
    pub fn new() -> Self {
        Self {
            id: "CAP-007".into(),
            conservation_active: true,
        }
    }

    /// Request access to a domain's "Natural Resource" (Data).
    pub fn grant_lease(
        &self,
        domain: &SovereignDomain,
        resource: ResourceType,
        requester_id: &str,
    ) -> Measured<ResourceLease> {
        // Simulation of DOI Conservation Logic
        let confidence_level = if requester_id == "ADMIN" {
            Confidence::new(1.0)
        } else {
            Confidence::new(0.8)
        };

        let lease = ResourceLease {
            domain_id: domain.id.clone(),
            resource_type: resource,
            duration_cycles: 10,
            protection_level: confidence_level,
        };

        Measured::uncertain(lease, confidence_level)
    }

    /// Verify if a state mutation violates "Environmental Integrity" (Type Safety).
    pub fn verify_resource_integrity(&self, lease: &ResourceLease, mutation_type: &str) -> Verdict {
        if matches!(lease.resource_type, ResourceType::ImmutableModule) && mutation_type == "WRITE"
        {
            // Cannot write to National Parks (Immutable Modules)
            Verdict::Rejected
        } else {
            Verdict::Permitted
        }
    }
}
