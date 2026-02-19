//! Signal classification and triage methods.
//!
//! This module contains methods for classifying adverse event severity
//! and triaging signals using standardized pharmacovigilance scales.
//!
//! ## Methods
//!
//! - **Hartwig-Siegel** - ADR severity scale (levels 1-7)
//! - **Multi-Method Triage** - Weighted multi-signal consensus scoring (GVP Module IX)
//!
//! ## Use Cases
//!
//! - Adverse event severity grading
//! - Clinical relevance assessment
//! - Signal prioritization
//! - Multi-method signal triage and ranking
//! - Regulatory seriousness determination (ICH E2A)
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::pv::signals::classification::hartwig_siegel::{
//!     assess_severity, SeverityCriteria, SeverityLevel
//! };
//!
//! let criteria = SeverityCriteria::new()
//!     .with_hospitalization();
//!
//! let severity = assess_severity(&criteria);
//! assert!(severity.is_serious()); // Hospitalization makes it "serious"
//! ```

pub mod hartwig_siegel;
pub mod triage;

// Re-export main types and functions
pub use hartwig_siegel::{
    SeverityAssessment, SeverityCategory, SeverityCriteria, SeverityLevel, assess_severity,
    batch_severity_score, full_assessment,
};

// Re-export triage types
pub use triage::{
    DetectionMethod, MethodWeight, NormalizedSignal, SignalInput, SignalInputBuilder,
    ThresholdProfile, TriageCategory, TriageEngine, TriageResult, TriageScore, batch_triage,
};
