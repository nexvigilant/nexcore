//! Drug Tools Parameters
//! Tier: T3 (MCP tool parameters for drug entity queries)
//!
//! Four tools covering the full drug safety query surface:
//! - `drug_profile`:      Complete drug profile JSON
//! - `drug_signals`:      Safety signal portfolio (PRR/ROR/IC)
//! - `drug_compare`:      Cross-drug signal comparison
//! - `drug_class_members`: List all drugs in a class from the catalog

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for `drug_profile` — retrieve the complete drug entity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DrugProfileParams {
    /// Generic (INN) drug name (e.g. "tirzepatide", "semaglutide", "pembrolizumab").
    /// Case-insensitive. All 10 catalog drugs are supported.
    pub drug_name: String,
}

/// Parameters for `drug_signals` — retrieve all PV signals for a drug.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DrugSignalsParams {
    /// Generic (INN) drug name (e.g. "tirzepatide", "semaglutide", "pembrolizumab").
    /// Case-insensitive.
    pub drug_name: String,
}

/// Parameters for `drug_compare` — compare safety profiles of two drugs.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DrugCompareParams {
    /// First drug generic name (e.g. "tirzepatide").
    pub drug_a: String,
    /// Second drug generic name (e.g. "semaglutide").
    pub drug_b: String,
}

/// Parameters for `drug_class_members` — list all catalog drugs in a class.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DrugClassMembersParams {
    /// Drug class name. Supported values (case-insensitive):
    /// "GLP1ReceptorAgonist", "GLP1GIPDualAgonist", "AntiAmyloid",
    /// "CheckpointInhibitor", "JAKInhibitor", "SGLT2Inhibitor",
    /// "EGFRTKInhibitor", "AntiTNF", "AntiIL17", "Anticoagulant".
    /// Pass "all" to list every drug in the catalog.
    pub drug_class: String,
}
