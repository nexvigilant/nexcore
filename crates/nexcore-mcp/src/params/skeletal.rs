//! Params for skeletal system (structural knowledge framework) tools.

use serde::Deserialize;

/// Assess structural health of project knowledge framework.
#[derive(Debug, Deserialize)]
pub struct SkeletalHealthParams {}

/// Evaluate Wolff's Law reinforcement (add knowledge where corrections concentrate).
#[derive(Debug, Deserialize)]
pub struct SkeletalWolffsLawParams {
    /// Domain or area to check for stress concentration
    pub domain: String,
    /// Number of recent corrections in this domain
    #[serde(default)]
    pub correction_count: Option<u32>,
}

/// Get project skeleton structure snapshot.
#[derive(Debug, Deserialize)]
pub struct SkeletalStructureParams {}
