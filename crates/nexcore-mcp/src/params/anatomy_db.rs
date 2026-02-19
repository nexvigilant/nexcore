//! Parameters for anatomy DB tools.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyQueryParams {
    /// SQL query (SELECT/WITH/EXPLAIN/PRAGMA only)
    pub sql: String,
    /// Max rows to return (default 500, max 1000)
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyStatusParams {}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordCytokineParams {
    /// Signal ID from cytokine_emit
    pub signal_id: String,
    /// Cytokine family (IL-1, IL-2, etc.)
    pub family: String,
    /// Signal name
    pub name: String,
    /// Severity level
    #[serde(default)]
    pub severity: Option<String>,
    /// Scope (autocrine/paracrine/endocrine/systemic)
    #[serde(default)]
    pub scope: Option<String>,
    /// JSON payload
    #[serde(default)]
    pub payload: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordHormonesParams {
    pub cortisol: f64,
    pub dopamine: f64,
    pub serotonin: f64,
    pub adrenaline: f64,
    pub oxytocin: f64,
    pub melatonin: f64,
    #[serde(default)]
    pub mood_score: Option<f64>,
    #[serde(default)]
    pub risk_tolerance: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordGuardianTickParams {
    /// Iteration ID (e.g., "iter-3")
    pub iteration_id: String,
    /// Number of signals detected
    pub signals_detected: i64,
    /// Number of actions taken
    pub actions_taken: i64,
    /// Maximum threat level
    pub max_threat_level: String,
    /// Duration in milliseconds
    pub duration_ms: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordImmunityEventParams {
    /// Antibody ID that matched
    #[serde(default)]
    pub antibody_id: Option<String>,
    /// PAMP or DAMP
    #[serde(default)]
    pub threat_type: Option<String>,
    /// Detection confidence
    #[serde(default)]
    pub confidence: Option<f64>,
    /// Action taken (block/warn/log)
    #[serde(default)]
    pub action_taken: Option<String>,
    /// Snippet of content that triggered
    #[serde(default)]
    pub content_snippet: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordSynapseParams {
    /// Synapse identifier
    pub synapse_id: String,
    /// Current amplitude
    pub amplitude: f64,
    /// Number of observations
    #[serde(default)]
    pub observation_count: Option<i64>,
    /// Peak amplitude reached
    #[serde(default)]
    pub peak_amplitude: Option<f64>,
    /// Status: accumulating/consolidated/decayed
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordEnergyParams {
    pub atp: f64,
    pub adp: f64,
    pub amp: f64,
    /// Energy charge EC = (ATP + 0.5*ADP) / total
    pub charge: f64,
    /// Metabolic regime: anabolic/catabolic/balanced/crisis
    #[serde(default)]
    pub regime: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordTranscriptaseParams {
    /// Data source name
    #[serde(default)]
    pub source: Option<String>,
    /// Number of fields inferred
    #[serde(default)]
    pub field_count: Option<i64>,
    /// Number of observations in sample
    #[serde(default)]
    pub observation_count: Option<i64>,
    /// Inferred schema as JSON string
    pub schema_json: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordRibosomeParams {
    /// Unique contract identifier
    pub contract_id: String,
    /// Schema JSON
    pub schema_json: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordPhenotypeParams {
    /// Primitives as JSON array string
    pub primitives: String,
    /// Whether mutation uses unsafe
    #[serde(default)]
    pub uses_unsafe: bool,
    /// Whether mutation is lethal
    #[serde(default)]
    pub is_lethal: bool,
    /// Verdict text
    #[serde(default)]
    pub verdict: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyRecordOrganSignalParams {
    /// Source organ ID
    pub source_organ: String,
    /// Target organ ID
    pub target_organ: String,
    /// Signal type: data/trigger/feedback
    #[serde(default)]
    pub signal_type: Option<String>,
    /// Payload data
    #[serde(default)]
    pub payload: Option<String>,
    /// Signal latency in ms
    #[serde(default)]
    pub latency_ms: Option<i64>,
}
