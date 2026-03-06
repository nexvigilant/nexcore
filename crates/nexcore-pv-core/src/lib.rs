#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

//! # NexVigilant Core — PV Core
//!
//! Pharmacovigilance signal detection, medical coding, causality assessment,
//! and pharmacokinetic calculations.
//!
//! ## Modules
//!
//! - **signals** - Signal detection algorithms (PRR, ROR, IC, EBGM, MaxSPRT, CuSum)
//! - **coding** - Medical coding (MedDRA, fuzzy search, glossary)
//! - **causality** - Causality assessment (Naranjo, WHO-UMC, RUCAM)
//! - **classification** - Severity classification (Hartwig-Siegel ADR scale)
//! - **faers** - FAERS data parsing and processing
//! - **thresholds** - Signal detection threshold criteria
//! - **pk** - Pharmacokinetics (AUC, steady state, mass balance, ionization, saturation)
//! - **thermodynamic** - Binding thermodynamics (Gibbs energy, Kd, kinetics, Arrhenius)
//! - **comppv** - Comprehensive PV: 11 Conservation Laws validation framework
//! - **hierarchy** - Safety level hierarchy and Tree of Vigilance (ToV) mapping
//! - **risk** - Risk analytics (Safety-at-Risk, Expected Shortfall, Monte Carlo)
//! - **regulatory** - Regulatory compliance bridge (EMA, FDA, WHO, ICH) + ICH Glossary (894+ terms, O(1) lookup)
//! - **minesweeper** - CSP framework for signal investigation prioritization (Safety Axioms)
//! - **pharmakon** - The Pharmakon Principle (ToV §34): benefit/harm duality, therapeutic windows
//! - **ivf** - Intervention Vigilance Framework (ToV §35): five IVF axioms
//! - **clinical_trial** - FDA effectiveness endpoints, substantial evidence, safety reporting

// Re-export nexcore-tov grounded types at crate root for internal use
#![warn(missing_docs)]
pub use nexcore_tov::grounded;

/// Safety margin d(s) calculation result (inlined from vigilance::tov)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SafetyMargin {
    /// Signed distance to harm boundary
    pub distance: f64,
    /// Interpretation of the distance
    pub interpretation: String,
    /// Recommended action
    pub action: String,
}

impl SafetyMargin {
    /// Calculate safety margin d(s) based on formal ToV axioms.
    ///
    /// d(s) = min(metrics - thresholds)
    #[must_use]
    pub fn calculate(prr: f64, ror_lower: f64, ic025: f64, eb05: f64, n: u32) -> Self {
        let prr_t = 2.0;
        let ror_t = 1.0;
        let ic_t = 0.0;
        let eb_t = 2.0;

        let d_prr = (prr_t - prr) / prr_t;
        let d_ror = (ror_t - ror_lower) / ror_t;
        let d_ic = ic_t - ic025;
        let d_eb = (eb_t - eb05) / eb_t;

        let distance = d_prr.min(d_ror).min(d_ic).min(d_eb);

        let epistemic_factor = if n < 3 {
            0.1
        } else if n < 5 {
            0.5
        } else {
            1.0
        };
        let weighted_distance = distance * epistemic_factor;

        let (interpretation, action) = if weighted_distance > 0.5 {
            ("Robustly Safe", "Routine surveillance")
        } else if weighted_distance > 0.0 {
            ("Safe (Low Margin)", "Enhanced monitoring")
        } else if weighted_distance > -0.5 {
            ("Potential Signal", "Signal validation required")
        } else {
            (
                "Confirmed Axiomatic Violation",
                "Immediate regulatory action",
            )
        };

        Self {
            distance: (weighted_distance * 100.0).round() / 100.0,
            interpretation: interpretation.to_string(),
            action: action.to_string(),
        }
    }

    /// Scores the epistemic trust of a result based on ToV hierarchy completeness.
    #[must_use]
    pub fn score_epistemic_trust(levels_covered: &[u8], sources: usize) -> f64 {
        let coverage = levels_covered.len() as f64 / 8.0;
        let source_factor = (sources as f64).ln_1p() / 5.0f64.ln_1p();
        (coverage * 0.7 + source_factor * 0.3).clamp(0.0, 1.0)
    }
}

// Foundation-equivalent utilities (inlined from vigilance::foundation)
pub mod foundation_compat;

pub mod benefit_risk;
pub mod causality;
pub mod classification;
pub mod clinical_trial;
pub mod coding;
pub mod compat;
pub mod comppv;
pub mod definition;
pub mod expectedness;
pub mod faers;
pub mod grounding;
pub mod hierarchy;
pub mod icsr;
pub mod ivf;
pub mod landscape;
pub mod minesweeper;
pub mod periodic_reporting;
pub mod pharmakon;
pub mod pk;
pub mod regulatory;
pub mod risk;
pub mod risk_management;
pub mod signals;
pub mod temporal;
pub mod thermodynamic;
pub mod thresholds;
pub mod types;

// Structural modules: prelude, cross-domain transfer, T1 primitive inventory, T2/T3 composites
pub mod composites;
pub mod prelude;
pub mod primitives;
pub mod transfer;

pub use classification::{
    SeverityAssessment, SeverityCategory, SeverityCriteria, SeverityLevel, assess_severity,
    batch_severity_score, full_assessment,
};
pub use comppv::{ConservationLaw, ConservationValidationReport, LawValidationResult};
pub use hierarchy::{LEVEL_METADATA, SafetyLevel, ToVLevel, map_to_tov_level};
pub use landscape::{LandscapeAnalysis, LandscapeEntry};
pub use minesweeper::{
    AdjacencyType, BeliefState, CSPGrid, CampionSignalResult, Cell, CellStatus, Evidence,
    TemporalWindow, cell_to_signal_score,
};
/// Backward-compatible re-export.
#[deprecated(note = "use RegulatoryViolation — F2 equivocation fix")]
#[allow(deprecated)]
pub use regulatory::Violation;
pub use regulatory::{
    ComplianceBridge, ComplianceReport, RegulatoryAuthority, RegulatoryViolation,
};
pub use signals::safe_detector::SafeSignalDetector;
// ICH Glossary - high-performance O(1) regulatory term lookup
pub use regulatory::ich_glossary::{
    Guideline, IchCategory, TOTAL_TERM_COUNT, Term, glossary_metadata, lookup_term, search_terms,
};
// Compatibility wrappers that use pv::types
pub use compat::{
    FisherResult, calculate_chi_square, calculate_ebgm, calculate_ic, calculate_prr, calculate_ror,
    fisher_exact_test,
};
pub use definition::{
    AssessmentResult, CausalityLevel, CollectionResult, DetectionResult, Pharmacovigilance,
    PreventionResult, PvCycleResult, RegulatoryAction, UnderstandingResult,
};
pub use icsr::{
    CaseId, Drug, DrugRole, Icsr, IcsrBuilder, Patient, Reaction, ReactionOutcome, Seriousness,
};
pub use thresholds::{ContextualThreshold, SignalCriteria, ThresholdRegistry};
pub use types::{CompleteSignalResult, ContingencyTable, SignalResult};

// Preemptive PV — three-tier signal detection (Reactive → Predictive → Preemptive).
// Re-exported from nexcore-preemptive-pv for unified access through pv-core.
pub use nexcore_preemptive_pv as preemptive_pv;
