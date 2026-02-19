//! Observatory Personalization Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for `observatory_personalize_get`
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PersonalizeGetParams {
    /// User profile type: "default", "power-user", "accessibility", "mobile"
    #[serde(default)]
    pub profile: Option<String>,
}

/// Parameters for `observatory_personalize_set`
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PersonalizeSetParams {
    /// Quality preset: "low", "medium", "high", "cinematic"
    #[serde(default)]
    pub quality: Option<String>,
    /// Theme: "default", "warm", "clinical", "high-contrast"
    #[serde(default)]
    pub theme: Option<String>,
    /// CVD mode: "normal", "deuteranopia", "protanopia", "tritanopia"
    #[serde(default)]
    pub cvd_mode: Option<String>,
    /// Default explorer: "graph", "career", "learning", "state", "math"
    #[serde(default)]
    pub default_explorer: Option<String>,
    /// Default layout: "force", "hierarchy", "radial", "grid"
    #[serde(default)]
    pub default_layout: Option<String>,
    /// Enable Web Worker layout computation
    #[serde(default)]
    pub enable_worker_layout: Option<bool>,
    /// Enabled post-processing effects (bloom, ssao, vignette)
    #[serde(default)]
    pub post_processing: Option<Vec<String>>,
}

/// Parameters for `observatory_personalize_detect`
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PersonalizeDetectParams {
    /// GPU renderer string from WebGL debug info
    #[serde(default)]
    pub gpu_renderer: Option<String>,
    /// Device pixel ratio (window.devicePixelRatio)
    #[serde(default)]
    pub device_pixel_ratio: Option<f64>,
    /// Logical CPU cores (navigator.hardwareConcurrency)
    #[serde(default)]
    pub hardware_concurrency: Option<u32>,
    /// Device memory in GB (navigator.deviceMemory)
    #[serde(default)]
    pub device_memory: Option<f64>,
    /// User prefers reduced motion (prefers-reduced-motion: reduce)
    #[serde(default)]
    pub prefers_reduced_motion: Option<bool>,
    /// User contrast preference: "no-preference", "more", "less"
    #[serde(default)]
    pub prefers_contrast: Option<String>,
}

/// Parameters for `observatory_personalize_validate`
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PersonalizeValidateParams {
    /// Explorer type to validate against: "graph", "career", "learning", "state", "math"
    #[serde(default)]
    pub explorer: Option<String>,
    /// Quality preset
    #[serde(default)]
    pub quality: Option<String>,
    /// Theme
    #[serde(default)]
    pub theme: Option<String>,
    /// CVD mode
    #[serde(default)]
    pub cvd_mode: Option<String>,
    /// Layout
    #[serde(default)]
    pub layout: Option<String>,
    /// Post-processing effects
    #[serde(default)]
    pub post_processing: Option<Vec<String>>,
}
