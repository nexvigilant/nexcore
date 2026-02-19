//! Trust Parameters (Ethical and Safety Engine)
//!
//! Scoring, recording evidence, snapshots, decision-making, and network chains.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for trust_score.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustScoreParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Use patient-safety-optimized config.
    pub safety_mode: Option<bool>,
}

/// Parameters for trust_record.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustRecordParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Evidence type: "positive", "negative", or "neutral".
    pub evidence_type: String,
    /// Evidence weight.
    pub weight: Option<f64>,
    /// Advance time by this many units.
    pub time_delta: Option<f64>,
    /// Use patient-safety-optimized config.
    pub safety_mode: Option<bool>,
}

/// Parameters for trust_snapshot.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustSnapshotParams {
    /// Unique entity identifier.
    pub entity_id: String,
}

/// Parameters for trust_decide.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustDecideParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Policy preset: "default", "strict", or "permissive".
    pub policy: Option<String>,
    /// Use patient-safety-optimized config.
    pub safety_mode: Option<bool>,
}

/// Parameters for trust_harm_weight.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustHarmWeightParams {
    /// ICH E2A severity.
    pub severity: String,
    /// WHO-UMC causality term or Naranjo score.
    pub causality: String,
    /// Base evidence weight.
    pub base_weight: Option<f64>,
    /// If provided, record the harm evidence.
    pub entity_id: Option<String>,
}

/// Parameters for trust_velocity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustVelocityParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Threshold for anomaly detection.
    pub anomaly_threshold: Option<f64>,
}

/// Parameters for trust_multi_score.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustMultiScoreParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// If provided, record evidence.
    pub evidence_type: Option<String>,
    /// Dimension to record to: "ability", "benevolence", "integrity", or "all".
    pub dimension: Option<String>,
    /// Evidence weight.
    pub weight: Option<f64>,
}

/// Parameters for trust_network_chain.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustNetworkChainParams {
    /// Array of pairwise trust scores.
    pub scores: Vec<f64>,
    /// Per-hop damping factor.
    pub damping: Option<f64>,
}
