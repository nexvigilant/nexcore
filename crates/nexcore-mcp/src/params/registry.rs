//! Registry Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Skill ecosystem compliance assessment, promotion, and ToV monitoring.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for assessing a single skill's compliance
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryAssessSkillParams {
    /// Skill name
    pub name: String,
}

/// Parameters for assessing all skills (no required params)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryAssessAllParams {
    /// If true, write assessments back to DB (default: false)
    #[serde(default)]
    pub apply: bool,
}

/// Parameters for generating a gap report (no required params)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryGapReportParams {
    /// Placeholder — no params needed, but MCP requires a struct
    #[serde(default)]
    pub _unused: Option<String>,
}

/// Parameters for querying promotable skills
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryPromotableParams {
    /// Target compliance tier: Bronze, Silver, Gold, Platinum, Diamond
    pub target_tier: String,
}

/// Parameters for generating a promotion plan
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryPromotionPlanParams {
    /// Skill name
    pub name: String,
    /// Target compliance tier: Bronze, Silver, Gold, Platinum, Diamond
    pub target_tier: String,
}

/// Parameters for ToV safety distance (no required params)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryTovSafetyParams {
    /// Placeholder — no params needed
    #[serde(default)]
    pub _unused: Option<String>,
}

/// Parameters for ToV harm indicators (no required params)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryTovHarmParams {
    /// Placeholder — no params needed
    #[serde(default)]
    pub _unused: Option<String>,
}

/// Parameters for ToV safety check
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegistryTovIsSafeParams {
    /// Safety threshold (default: 0.3)
    #[serde(default = "default_safety_threshold")]
    pub threshold: f64,
}

fn default_safety_threshold() -> f64 {
    0.3
}
