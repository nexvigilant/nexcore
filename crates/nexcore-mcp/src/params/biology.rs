//! Biology & Endocrine Subsystem Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Biological metrics applied to agentic systems (Hormones, Cytokines).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for getting a specific hormone level
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HormoneGetParams {
    /// Hormone name: cortisol, dopamine, serotonin, adrenaline, oxytocin, melatonin
    pub hormone: String,
}

/// Parameters for applying a stimulus to the endocrine system
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HormoneStimulusParams {
    /// Stimulus type (e.g., "error", "task_completed")
    pub stimulus_type: String,
    /// Intensity (0.0-1.0)
    pub intensity: Option<f64>,
    /// Count for count-based stimuli
    pub count: Option<u32>,
    /// Recoverable flag for critical errors
    pub recoverable: Option<bool>,
}

// ============================================================================
// Molecular Biology Parameters
// ============================================================================

/// Parameters for translating a single codon
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularTranslateCodonParams {
    /// RNA codon (e.g., "AUG")
    pub codon: String,
}

/// Parameters for translating mRNA to protein
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularTranslateMrnaParams {
    /// mRNA sequence
    pub mrna: String,
    /// Start translation from first AUG codon
    #[serde(default)]
    pub from_start: Option<bool>,
}

/// Parameters for Central Dogma stage mapping
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularCentralDogmaParams {
    /// Stage: transcription, translation, folding, etc.
    pub stage: String,
}

/// Parameters for ADME pharmacokinetic phase mapping
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularAdmePhaseParams {
    /// Phase: absorption, distribution, metabolism, elimination
    pub phase: String,
}

// ============================================================================
// Cytokine Parameters
// ============================================================================

/// Parameters for emitting a cytokine signal.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CytokineEmitParams {
    /// Cytokine family (e.g., "il1", "il6")
    pub family: String,
    /// Signal name
    pub name: String,
    /// Severity level
    #[serde(default)]
    pub severity: Option<String>,
    /// Scope: autocrine, paracrine, endocrine, systemic
    #[serde(default)]
    pub scope: Option<String>,
    /// Optional JSON payload data
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Parameters for listing cytokine families.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CytokineListParams {
    /// Optional filter by family name
    #[serde(default)]
    pub family_filter: Option<String>,
}

/// Parameters for querying recent cytokine signals
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CytokineRecentParams {
    /// Maximum number of recent cytokines to return
    #[serde(default = "default_cytokine_recent_limit")]
    pub limit: u32,
    /// Optional family filter
    #[serde(default)]
    pub family: Option<String>,
}

fn default_cytokine_recent_limit() -> u32 {
    20
}

/// Parameters for computing chemotactic gradient routing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemotaxisGradientParams {
    /// Array of gradient samples
    pub gradients: Vec<GradientSample>,
}

/// A single gradient sample for chemotaxis computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GradientSample {
    /// Source identifier
    pub source: String,
    /// Cytokine family
    pub family: String,
    /// Signal concentration [0.0, 1.0]
    pub concentration: f64,
    /// Distance from agent
    pub distance: f64,
    /// Tropism: "positive" or "negative"
    #[serde(default = "default_tropism")]
    pub tropism: String,
}

fn default_tropism() -> String {
    "positive".to_string()
}

/// Parameters for endocytosis pool operations.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EndocytosisInternalizeParams {
    /// Cytokine family to internalize
    pub family: String,
    /// Signal name
    pub name: String,
    /// Severity
    #[serde(default)]
    pub severity: Option<String>,
    /// Pool capacity
    #[serde(default = "default_pool_capacity")]
    pub pool_capacity: usize,
}

fn default_pool_capacity() -> usize {
    10
}
