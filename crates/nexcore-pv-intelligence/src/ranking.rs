//! Result types for intelligence graph queries.
//!
//! All result types derive `Serialize`/`Deserialize` for cross-boundary
//! transport and `Clone`/`Debug` for test ergonomics.
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Ranked company lists | Sequence | σ |
//! | Numeric scores / counts | Quantity | N |
//! | Optional safer drug | Void | ∅ |
//! | Advantage label | Sum | Σ |
//! | Serialize / Deserialize | Persistence | π |

use serde::{Deserialize, Serialize};

/// A company ranked by safety portfolio quality for a target disease.
///
/// Lower `avg_signal_strength` is safer (closer to background rate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyRanking {
    /// Company node ID
    pub company: String,
    /// Drug IDs owned by this company that treat the target disease
    pub drugs: Vec<String>,
    /// Average PRR across all signals on those drugs (lower = safer)
    pub avg_signal_strength: f64,
    /// Total boxed warnings across those drugs
    pub boxed_warnings: u32,
    /// 1-based rank (1 = safest)
    pub rank: u32,
}

/// Head-to-head safety comparison between two drugs.
///
/// `safer_drug` is `None` when neither drug has a material advantage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadToHead {
    /// First drug node ID
    pub drug_a: String,
    /// Second drug node ID
    pub drug_b: String,
    /// Adverse events detected on both drugs
    pub shared_signals: Vec<SharedSignal>,
    /// Events unique to drug A (not present on drug B)
    pub unique_to_a: Vec<String>,
    /// Events unique to drug B (not present on drug A)
    pub unique_to_b: Vec<String>,
    /// Which drug has the overall safety advantage, if determinable
    pub safer_drug: Option<String>,
}

/// A single adverse event observed on both drugs in a head-to-head comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedSignal {
    /// MedDRA preferred term or adverse event label
    pub event: String,
    /// PRR for drug A
    pub prr_a: f64,
    /// PRR for drug B
    pub prr_b: f64,
    /// Which drug has lower PRR: `"drug_a"`, `"drug_b"`, or `"comparable"`
    pub advantage: String,
}

/// An off-label or unlabelled signal that represents a competitive gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGap {
    /// Drug node ID
    pub drug: String,
    /// Adverse event term
    pub event: String,
    /// Proportional Reporting Ratio
    pub prr: f64,
    /// Whether the event appears in the current label
    pub on_label: bool,
    /// Plain-English description of the opportunity or risk
    pub opportunity: String,
}

/// Competitive landscape for a therapeutic area.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TherapeuticLandscape {
    /// Therapeutic area label
    pub area: String,
    /// Companies with drugs in this area, ranked by safety
    pub companies: Vec<CompanyRanking>,
    /// Total drugs in this area across all companies
    pub total_drugs: usize,
    /// Total signal count across all drugs in this area
    pub total_signals: usize,
    /// Company with the most drugs in this area, if determinable
    pub dominant_company: Option<String>,
}

/// Two companies competing in the same indication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOverlap {
    /// First company node ID
    pub company_a: String,
    /// Second company node ID
    pub company_b: String,
    /// Disease node ID they both target
    pub shared_indication: String,
    /// Qualitative phase of competition (e.g. `"head-to-head"`, `"emerging"`)
    pub competition_phase: String,
}

/// Result for a class-effect analysis across drugs sharing a mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassEffectResult {
    /// Drug class label
    pub drug_class: String,
    /// Adverse event common to drugs in this class
    pub event: String,
    /// Drug IDs exhibiting the class effect
    pub drugs: Vec<String>,
    /// Number of drugs in the class exhibiting this effect
    pub affected_count: usize,
}
