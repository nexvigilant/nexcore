//! PV Intelligence Parameters
//! Tier: T3 (Domain MCP tool parameters — competitive PV analysis)
//!
//! Read-only tools that expose nexcore-pv-intelligence query methods:
//! - `pv_intelligence_head_to_head`: Compare two drugs' safety signal portfolios
//! - `pv_intelligence_class_effects`: Find signals shared across a drug class
//! - `pv_intelligence_safety_gaps`: Find off-label or weak signals for a disease
//! - `pv_intelligence_safest_company`: Rank companies by safety portfolio for a disease
//! - `pv_intelligence_therapeutic_landscape`: Map the competitive landscape for a therapeutic area
//! - `pv_intelligence_pipeline_overlap`: Find companies competing in the same indication

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for `pv_intelligence_head_to_head` — compare two drugs' safety portfolios.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvIntelligenceHeadToHeadParams {
    /// First drug node ID (e.g. "semaglutide", "tirzepatide")
    pub drug_a: String,

    /// Second drug node ID (e.g. "liraglutide", "dulaglutide")
    pub drug_b: String,
}

/// Parameters for `pv_intelligence_class_effects` — find class-wide adverse event signals.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvIntelligenceClassEffectsParams {
    /// Drug class label to analyse (e.g. "GLP-1 Receptor Agonist", "PD-1 Checkpoint Inhibitor")
    pub drug_class: String,
}

/// Parameters for `pv_intelligence_safety_gaps` — find unlabelled or elevated signals for a disease.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvIntelligenceSafetyGapsParams {
    /// Disease node ID to scan for safety gaps (e.g. "t2dm", "nsclc", "ra")
    pub disease_id: String,
}

/// Parameters for `pv_intelligence_safest_company` — rank companies by safety for a disease.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvIntelligenceSafestCompanyParams {
    /// Disease node ID to rank companies against (e.g. "t2dm", "obesity", "nsclc")
    pub disease_id: String,
}

/// Parameters for `pv_intelligence_therapeutic_landscape` — map the competitive landscape.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvIntelligenceTherapeuticLandscapeParams {
    /// Therapeutic area to map (e.g. "Metabolic", "Oncology", "Immunology", "Neuroscience")
    pub therapeutic_area: String,
}

/// Parameters for `pv_intelligence_pipeline_overlap` — find companies competing in the same indication.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvIntelligencePipelineOverlapParams {
    /// Disease node ID to check for pipeline overlap (e.g. "t2dm", "breast-cancer", "psoriasis")
    pub disease_id: String,
}
