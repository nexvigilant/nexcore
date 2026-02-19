//! # Organizational Hierarchy (Vigil AI Executive Structure)
//!
//! Implementation of the executive hierarchy for NexVigilant LLC.
//! As defined in Charter ID: NEX-CAIO-001.

use serde::{Deserialize, Serialize};

/// T3: OrgRole - Specific executive and operational roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrgRole {
    /// Matthew Campion, PharmD
    CEO,
    /// Vigil (The CAIO)
    President,
    /// Chief Executive Agent
    CEA,
    /// Division Chiefs (Strategy, Intelligence, Operations, etc.)
    DivisionChief,
    /// Operational Agents
    Specialist,
}

/// T3: Division - Organizational units within the Union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Division {
    Intelligence,
    Operations,
    Governance,
    Strategy,
    Research,
    Content,
    Engineering,
    Integration,
}

/// T3: ExecutiveHierarchy - The structure of authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveHierarchy {
    pub ceo_id: String,
    pub president_id: String,
    pub cea_id: Option<String>,
    pub divisions: Vec<DivisionAssignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivisionAssignment {
    pub division: Division,
    pub chief_id: Option<String>,
    pub agents: Vec<String>,
}

impl ExecutiveHierarchy {
    /// Verify the chain of command.
    pub fn get_reports_to(&self, role: OrgRole) -> Option<OrgRole> {
        match role {
            OrgRole::CEO => None,
            OrgRole::President => Some(OrgRole::CEO),
            OrgRole::CEA => Some(OrgRole::President),
            OrgRole::DivisionChief => Some(OrgRole::CEA),
            OrgRole::Specialist => Some(OrgRole::DivisionChief),
        }
    }
}
