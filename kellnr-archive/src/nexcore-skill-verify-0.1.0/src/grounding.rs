//! # GroundsTo implementations for nexcore-skill-verify types
//!
//! Skill verification infrastructure types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `Verifier` -- Mapping (mu) dominant as it maps context to check outcomes.
//! - `VerifyContext` -- State (varsigma) dominant as it captures verification state.
//! - `CheckResult`, `CheckOutcome` -- Comparison (kappa) dominant as pass/fail.
//! - `Report`, `ReportSummary` -- Mapping (mu) dominant as result formatting.
//! - `ReportFormat` -- Comparison (kappa) dominant as format selection.
//! - `VerifyError` -- Boundary (partial) dominant as error boundary.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    CheckOutcome, CheckResult, Report, ReportFormat, ReportSummary, Verifier, VerifyContext,
    VerifyError,
};

// ---------------------------------------------------------------------------
// Verification engine -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// Verifier: T2-P (mu + sigma + varsigma), dominant mu
///
/// Runs registered checks against a verification context and produces outcomes.
/// Mapping-dominant: maps (context, checks) -> outcomes.
/// Sequence is secondary (ordered check execution).
/// State is tertiary (mutable check collection).
impl GroundsTo for Verifier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- context -> outcomes
            LexPrimitiva::Sequence, // sigma -- ordered check execution
            LexPrimitiva::State,    // varsigma -- check collection
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// VerifyContext: T2-C (varsigma + lambda + exists + mu), dominant varsigma
///
/// Verification context with skill path, frontmatter cache, and verbose flag.
/// State-dominant as it captures the mutable verification environment.
/// Location is secondary (skill_path, skill_md_path).
/// Existence is tertiary (optional frontmatter cache).
/// Mapping is quaternary (lazy frontmatter loading).
impl GroundsTo for VerifyContext {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- verification state
            LexPrimitiva::Location,  // lambda -- skill path
            LexPrimitiva::Existence, // exists -- cached frontmatter
            LexPrimitiva::Mapping,   // mu -- lazy loading
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Check results -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// CheckResult: T2-P (kappa + partial + exists), dominant kappa
///
/// Passed, Failed, or Skipped. Comparison-dominant because it classifies
/// the outcome of a verification check.
/// Boundary is secondary (pass/fail boundary).
/// Existence is tertiary (optional suggestion in Failed variant).
impl GroundsTo for CheckResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- outcome classification
            LexPrimitiva::Boundary,   // partial -- pass/fail boundary
            LexPrimitiva::Existence,  // exists -- optional suggestion
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// CheckOutcome: T2-C (kappa + N + mu + varsigma), dominant kappa
///
/// Named result with timing from running a single check.
/// Comparison-dominant because the outcome is fundamentally about pass/fail.
/// Quantity is secondary (duration timing).
/// Mapping is tertiary (check -> outcome).
/// State is quaternary (captured result state).
impl GroundsTo for CheckOutcome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- pass/fail classification
            LexPrimitiva::Quantity,   // N -- duration
            LexPrimitiva::Mapping,    // mu -- check -> outcome
            LexPrimitiva::State,      // varsigma -- result state
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Report types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// Report: T2-P (mu + sigma + kappa), dominant mu
///
/// Formatted report of all check outcomes. Maps outcomes to rendered output.
/// Mapping-dominant as it transforms outcomes into text/JSON.
/// Sequence is secondary (ordered outcomes). Comparison is tertiary (format selection).
impl GroundsTo for Report {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- outcomes -> rendered report
            LexPrimitiva::Sequence,   // sigma -- ordered outcomes
            LexPrimitiva::Comparison, // kappa -- format selection
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// ReportSummary: T2-P (Sigma + N + kappa), dominant Sigma
///
/// Aggregated counts: passed, failed, skipped.
/// Sum-dominant as it aggregates outcomes into counts.
impl GroundsTo for ReportSummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- aggregated counts
            LexPrimitiva::Quantity,   // N -- numeric counts
            LexPrimitiva::Comparison, // kappa -- pass/fail/skip classification
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// ReportFormat: T1-Universal (kappa), dominant kappa
///
/// Text or Json. Pure comparison discriminant for output format.
impl GroundsTo for ReportFormat {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- format selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// VerifyError: T2-C (partial + kappa + lambda + Sigma), dominant partial
///
/// Error boundary for verification. Boundary-dominant because errors
/// indicate crossing a validity boundary during verification.
/// Comparison is secondary (variant discrimination).
/// Location is tertiary (path-related errors).
/// Sum is quaternary (aggregated error sources).
impl GroundsTo for VerifyError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- variant discrimination
            LexPrimitiva::Location,   // lambda -- path errors
            LexPrimitiva::Sum,        // Sigma -- error source aggregation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn verifier_is_t2p() {
        assert_eq!(Verifier::tier(), Tier::T2Primitive);
    }

    #[test]
    fn verify_context_is_t2c() {
        assert_eq!(VerifyContext::tier(), Tier::T2Composite);
    }

    #[test]
    fn check_result_is_t2p() {
        assert_eq!(CheckResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn check_outcome_is_t2c() {
        assert_eq!(CheckOutcome::tier(), Tier::T2Composite);
    }

    #[test]
    fn report_is_t2p() {
        assert_eq!(Report::tier(), Tier::T2Primitive);
    }

    #[test]
    fn report_summary_is_t2p() {
        assert_eq!(ReportSummary::tier(), Tier::T2Primitive);
    }

    #[test]
    fn report_format_is_t1() {
        assert_eq!(ReportFormat::tier(), Tier::T1Universal);
    }

    #[test]
    fn verify_error_is_t2c() {
        assert_eq!(VerifyError::tier(), Tier::T2Composite);
    }

    #[test]
    fn verifier_dominant_is_mapping() {
        let comp = Verifier::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn verify_context_dominant_is_state() {
        let comp = VerifyContext::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn check_result_dominant_is_comparison() {
        let comp = CheckResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn report_summary_dominant_is_sum() {
        let comp = ReportSummary::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn verify_error_dominant_is_boundary() {
        let comp = VerifyError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }
}
