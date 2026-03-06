//! Phase 1: Collect — Discovery and system boundary mapping.
//!
//! Algorithm: DEFINE (∂ → π(ς)) + DECOMPOSE (X → P₁ + P₂ + ... + Pₙ)
//! Template: NV-COR-SOP-001 Diagnostic Assessment Tool
//!
//! Collects: organizational context, PV system boundaries, presenting concerns,
//! and the 3-test existence assessment (∃ = ∂(×(ς, ∅))).

use serde::{Deserialize, Serialize};

use nexcore_vigilance::caba::DomainCategory;

/// A bounded subsystem identified during intake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subsystem {
    /// Name of the subsystem (e.g., "Signal Detection", "Case Processing").
    pub name: String,
    /// Which PV domains this subsystem touches.
    pub domains: Vec<DomainCategory>,
    /// Current maturity description (free text from intake).
    pub maturity_description: String,
    /// Whether the subsystem exists (∃), is partially present, or absent (∅).
    pub existence: ExistenceStatus,
}

/// The three-valued existence assessment from the conservation law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExistenceStatus {
    /// ∃ — subsystem exists with boundary, state, and function.
    Exists,
    /// Partial — some components present but conservation law not fully satisfied.
    Partial,
    /// ∅ — subsystem absent.
    Absent,
}

/// Presenting concern from the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concern {
    /// What the client described.
    pub description: String,
    /// Which domains this concern maps to.
    pub domains: Vec<DomainCategory>,
    /// Client-stated priority (1 = highest).
    pub priority: u8,
}

/// Organizational context captured during intake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalContext {
    /// Organization size category.
    pub size: OrgSize,
    /// Regulatory markets (e.g., "US", "EU", "Japan").
    pub markets: Vec<String>,
    /// Product types (e.g., "small molecule", "biologic", "device").
    pub product_types: Vec<String>,
    /// Number of marketed products.
    pub product_count: Option<u32>,
    /// Annual ICSR volume estimate.
    pub annual_icsr_volume: Option<u32>,
}

/// Organization size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrgSize {
    Startup,
    Small,
    Mid,
    Large,
    Enterprise,
}

/// The complete Phase 1 output — a system map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMap {
    /// Organizational context.
    pub context: OrganizationalContext,
    /// Bounded subsystems identified.
    pub subsystems: Vec<Subsystem>,
    /// Presenting concerns.
    pub concerns: Vec<Concern>,
}

impl SystemMap {
    /// All domains touched by at least one subsystem.
    pub fn domains_in_scope(&self) -> Vec<DomainCategory> {
        let mut domains: Vec<DomainCategory> = self
            .subsystems
            .iter()
            .flat_map(|s| s.domains.iter().copied())
            .collect();
        domains.sort_by_key(|d| d.number());
        domains.dedup();
        domains
    }

    /// Subsystems that are absent (∅).
    pub fn gaps(&self) -> Vec<&Subsystem> {
        self.subsystems
            .iter()
            .filter(|s| s.existence == ExistenceStatus::Absent)
            .collect()
    }

    /// Subsystems that are partial.
    pub fn partial(&self) -> Vec<&Subsystem> {
        self.subsystems
            .iter()
            .filter(|s| s.existence == ExistenceStatus::Partial)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_map_gaps() {
        let map = SystemMap {
            context: OrganizationalContext {
                size: OrgSize::Mid,
                markets: vec!["US".into()],
                product_types: vec!["small molecule".into()],
                product_count: Some(3),
                annual_icsr_volume: Some(5000),
            },
            subsystems: vec![
                Subsystem {
                    name: "Signal Detection".into(),
                    domains: vec![DomainCategory::D05SignalDetection],
                    maturity_description: "No formal process".into(),
                    existence: ExistenceStatus::Absent,
                },
                Subsystem {
                    name: "Case Processing".into(),
                    domains: vec![DomainCategory::D04IcsrProcessing],
                    maturity_description: "Manual but functional".into(),
                    existence: ExistenceStatus::Exists,
                },
            ],
            concerns: vec![],
        };

        assert_eq!(map.gaps().len(), 1);
        assert_eq!(map.gaps()[0].name, "Signal Detection");
        assert_eq!(map.domains_in_scope().len(), 2);
    }
}
