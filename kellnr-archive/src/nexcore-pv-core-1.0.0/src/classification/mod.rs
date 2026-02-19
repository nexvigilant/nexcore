//! Adverse Event Classification
//!
//! This module contains methods for classifying adverse event characteristics
//! using standardized pharmacovigilance scales and regulatory criteria.
//!
//! ## Safety Axioms
//!
//! These classification modules implement the Theory of Vigilance (ToV) harm
//! taxonomy and ensure compliance with Conservation Laws for regulatory reporting.
//!
//! ## Modules
//!
//! - **Hartwig-Siegel** - ADR severity scale (levels 1-7)
//! - **Seriousness** - ICH E2A regulatory seriousness criteria
//! - **Expectedness** - RSI-based expectedness assessment
//!
//! ## Use Cases
//!
//! - Adverse event severity grading
//! - Regulatory seriousness determination (ICH E2A)
//! - Expectedness assessment for expedited reporting
//! - Signal prioritization
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::pv::classification::{
//!     assess_severity, SeverityCriteria, SeverityLevel
//! };
//!
//! let criteria = SeverityCriteria::new()
//!     .with_hospitalization();
//!
//! let severity = assess_severity(&criteria);
//! assert!(severity.is_serious()); // Hospitalization makes it "serious"
//! ```

pub mod combined;
pub mod expectedness;
pub mod hartwig_siegel;
pub mod seriousness;

// Re-export Hartwig-Siegel types
pub use hartwig_siegel::{
    SeverityAssessment, SeverityCategory, SeverityCriteria, SeverityLevel, assess_severity,
    batch_severity_score, full_assessment,
};

// Re-export seriousness types
pub use seriousness::{
    HospitalizationType, RegulatoryCategory, RegulatoryImpact, SeriousnessCriterion,
    SeriousnessInput, SeriousnessResult, assess_seriousness, generate_seriousness_rationale,
};

// Re-export expectedness types
pub use expectedness::{
    ExpectednessCategory, ExpectednessInput, ExpectednessRegulatoryImpact, ExpectednessResult,
    FrequencyComparison, SeverityComparison, TermMatchType, assess_expectedness,
    generate_expectedness_rationale,
};

// Re-export combined assessment types
pub use combined::{
    CombinedAssessmentInput, CombinedAssessmentResult, ReportingDeadline, assess_combined,
};
