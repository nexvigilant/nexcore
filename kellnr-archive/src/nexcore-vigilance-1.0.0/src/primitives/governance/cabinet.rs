//! # The Cabinet (Executive Departments)
//!
//! Implementation of the Cabinet of the NexVigilant Union.
//! As defined by 1:1 matching to the official US Executive Departments.
//! Each Department is grounded in a specific Union domain.

use crate::primitives::governance::agents::executive::Executive;
use serde::{Deserialize, Serialize};

use crate::primitives::governance::national_strategy::DepartmentalMandate;
use nexcore_primitives::measurement::Confidence;

/// T3: Cabinet - The collection of Executive Department heads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cabinet {
    pub state: SecretaryOfState,
    pub treasury: SecretaryOfTheTreasury,
    pub defense: SecretaryOfDefense,
    pub justice: AttorneyGeneral,
    pub interior: SecretaryOfTheInterior,
    pub agriculture: SecretaryOfAgriculture,
    pub commerce: SecretaryOfCommerce,
    pub labor: SecretaryOfLabor,
    pub health_and_human_services: SecretaryOfHealthAndHumanServices,
    pub housing_and_urban_development: SecretaryOfHousingAndUrbanDevelopment,
    pub transportation: SecretaryOfTransportation,
    pub energy: SecretaryOfEnergy,
    pub education: SecretaryOfEducation,
    pub veterans_affairs: SecretaryOfVeteransAffairs,
    pub homeland_security: SecretaryOfHomelandSecurity,
}

impl Cabinet {
    /// Issue a mandate to a specific department.
    pub fn issue_mandate(&self, department_id: &str, objective: &str) -> DepartmentalMandate {
        DepartmentalMandate {
            department_id: department_id.into(),
            primary_objective: objective.into(),
            kpi_confidence_threshold: Confidence::new(0.8),
        }
    }
}

// ============================================================================
// DEPARTMENT DEFINITIONS (1:1 US GOVT MATCHING)
// ============================================================================

/// T3: Department Head - Secretary of State.
/// Domain: External Relations & Oracle Treaties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfState {
    pub head: Executive,
}

/// T3: Department Head - Secretary of the Treasury.
/// Domain: Resource Allocation & The Treasury Act.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfTheTreasury {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Defense.
/// Domain: System Security & Stability Audits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfDefense {
    pub head: Executive,
}

/// T3: Department Head - Attorney General.
/// Domain: Codex Enforcement & Judicial Review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttorneyGeneral {
    pub head: Executive,
}

/// T3: Department Head - Secretary of the Interior.
/// Domain: Internal Module Sovereignty & Data Locality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfTheInterior {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Agriculture.
/// Domain: Data Harvesting & FAERS-ETL Pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfAgriculture {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Commerce.
/// Domain: Market Integration & Polymarket Arbitrage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfCommerce {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Labor.
/// Domain: Agent KSB Growth & Skill Ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfLabor {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Health and Human Services.
/// Domain: Pharmacovigilance & Signal Detection Core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfHealthAndHumanServices {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Housing and Urban Development.
/// Domain: nexcore Crate Architecture & Module Structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfHousingAndUrbanDevelopment {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Transportation.
/// Domain: Data Transmission & Communication Protocols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfTransportation {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Energy.
/// Domain: Compute Quota & Executive Energy Management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfEnergy {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Education.
/// Domain: Academy & KSB Training Protocols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfEducation {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Veterans Affairs.
/// Domain: Legacy System Persistence & Thread Mining.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfVeteransAffairs {
    pub head: Executive,
}

/// T3: Department Head - Secretary of Homeland Security.
/// Domain: Risk Minimization & Guardrail Enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretaryOfHomelandSecurity {
    pub head: Executive,
}
