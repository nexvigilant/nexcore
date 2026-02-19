//! CEP (Cognitive Evolution Pipeline) Parameters
//! Patent: NV-2026-001
//!
//! SEE, SPEAK, DECOMPOSE, COMPOSE stages for domain cognitive evolution.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for executing a CEP stage
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CepExecuteStageParams {
    /// Stage to execute: SEE, SPEAK, DECOMPOSE, COMPOSE, TRANSLATE, VALIDATE, DEPLOY, IMPROVE
    pub stage: String,
    /// Domain being processed
    pub domain: String,
    /// Input data for the stage (JSON)
    #[serde(default)]
    pub input: Option<serde_json::Value>,
}

/// Parameters for CEP pipeline status
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CepPipelineStatusParams {
    /// Pipeline execution ID (optional)
    #[serde(default)]
    pub execution_id: Option<String>,
}

/// Parameters for validating primitive extraction
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CepValidateExtractionParams {
    /// Coverage score (0.0-1.0)
    pub coverage: f64,
    /// Minimality score (0.0-1.0)
    pub minimality: f64,
    /// Independence score (0.0-1.0)
    pub independence: f64,
    /// Custom coverage threshold
    #[serde(default)]
    pub coverage_threshold: Option<f64>,
    /// Custom minimality threshold
    #[serde(default)]
    pub minimality_threshold: Option<f64>,
    /// Custom independence threshold
    #[serde(default)]
    pub independence_threshold: Option<f64>,
}

/// Parameters for extracting primitives from a domain
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveExtractParams {
    /// Domain to extract primitives from
    pub domain: String,
    /// Extraction mode: "standard" or "deep"
    #[serde(default)]
    pub mode: Option<String>,
    /// Source documents or corpus (optional)
    #[serde(default)]
    pub sources: Option<Vec<String>>,
}

/// Parameters for domain translation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainTranslateParams {
    /// Source domain
    pub source_domain: String,
    /// Target domain
    pub target_domain: String,
    /// Concept to translate
    pub concept: String,
    /// Concept tier (T1, T2, T3) if known
    #[serde(default)]
    pub tier: Option<String>,
}

/// Parameters for classifying a primitive's tier
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveTierClassifyParams {
    /// Number of domains the concept appears in
    pub domain_count: usize,
}

/// Parameters for getting tier summary
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TierSummaryParams {
    /// Count of T1 Universal primitives
    pub t1_count: usize,
    /// Count of T2 Cross-Domain primitives
    pub t2_count: usize,
    /// Count of T3 Domain-Specific primitives
    pub t3_count: usize,
}
