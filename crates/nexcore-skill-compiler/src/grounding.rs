//! # GroundsTo implementations for nexcore-skill-compiler types
//!
//! Skill compiler types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `CompilationResult` -- Mapping (mu) dominant as it maps spec to compiled output.
//! - `CompoundSpec`, `CompoundMeta`, `SkillEntry` -- State (varsigma) dominant as config.
//! - `CompositionStrategy`, `ThreadingMode`, `MergeStrategy` -- Comparison (kappa) dominant.
//! - `CompilerError` -- Boundary (partial) dominant as error boundary.
//! - `AnalysisReport`, `SkillAnalysis` -- Mapping (mu) dominant as analysis transformation.
//! - `BuildResult` -- Mapping (mu) dominant as source -> binary transformation.
//! - `GeneratedCrate` -- Mapping (mu) dominant as spec -> code transformation.
//! - `ThreadingConfig`, `FeedbackConfig` -- State (varsigma) dominant as configuration.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::CompilationResult;
use crate::analyzer::{AnalysisReport, SkillAnalysis};
use crate::builder::BuildResult;
use crate::codegen::GeneratedCrate;
use crate::error::CompilerError;
use crate::spec::{
    CompositionStrategy, CompoundMeta, CompoundSpec, FeedbackConfig, MergeStrategy, SkillEntry,
    ThreadingConfig, ThreadingMode,
};

// ---------------------------------------------------------------------------
// Top-level compilation -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// CompilationResult: T3 (mu + varsigma + sigma + lambda + kappa + exists), dominant mu
///
/// Full pipeline result: spec -> analyzed -> generated -> built.
/// Mapping-dominant as the type represents the output of a multi-stage transformation.
impl GroundsTo for CompilationResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- spec -> compiled output
            LexPrimitiva::State,      // varsigma -- analysis report state
            LexPrimitiva::Sequence,   // sigma -- pipeline stages
            LexPrimitiva::Location,   // lambda -- crate_dir, binary_path, skill_md
            LexPrimitiva::Comparison, // kappa -- diamond_compliant check
            LexPrimitiva::Existence,  // exists -- optional binary_path
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Spec types -- varsigma (State) dominant
// ---------------------------------------------------------------------------

/// CompoundSpec: T2-C (varsigma + sigma + kappa + exists), dominant varsigma
///
/// Top-level compound skill specification parsed from TOML.
/// State-dominant as it captures the full configuration state.
impl GroundsTo for CompoundSpec {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- configuration state
            LexPrimitiva::Sequence,   // sigma -- ordered skills list
            LexPrimitiva::Comparison, // kappa -- strategy selection
            LexPrimitiva::Existence,  // exists -- optional threading/feedback
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// CompoundMeta: T2-P (varsigma + mu + sigma), dominant varsigma
///
/// Compound-level metadata: name, description, strategy, tags.
impl GroundsTo for CompoundMeta {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- identity state
            LexPrimitiva::Mapping,  // mu -- TOML -> metadata
            LexPrimitiva::Sequence, // sigma -- ordered tags
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// SkillEntry: T2-P (varsigma + partial + N), dominant varsigma
///
/// A sub-skill entry with name, required flag, and timeout.
impl GroundsTo for SkillEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- entry configuration
            LexPrimitiva::Boundary, // partial -- required/optional boundary
            LexPrimitiva::Quantity, // N -- timeout_seconds
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// ThreadingConfig: T2-P (varsigma + kappa), dominant varsigma
///
/// Result-passing configuration between skills.
impl GroundsTo for ThreadingConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- config state
            LexPrimitiva::Comparison, // kappa -- mode/strategy selection
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// FeedbackConfig: T2-P (varsigma + N + partial), dominant varsigma
///
/// Feedback loop parameters: max_iterations, convergence_field, threshold.
impl GroundsTo for FeedbackConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- loop configuration
            LexPrimitiva::Quantity, // N -- max_iterations, threshold
            LexPrimitiva::Boundary, // partial -- convergence boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Strategy enums -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// CompositionStrategy: T1-Universal (kappa), dominant kappa
///
/// Sequential, Parallel, FeedbackLoop. Pure comparison discriminant.
impl GroundsTo for CompositionStrategy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- strategy selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// ThreadingMode: T1-Universal (kappa), dominant kappa
///
/// Result-passing mode selection (currently only JsonAccumulator).
impl GroundsTo for ThreadingMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- mode selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// MergeStrategy: T1-Universal (kappa), dominant kappa
///
/// DeepMerge vs Overwrite. Pure comparison discriminant.
impl GroundsTo for MergeStrategy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- merge strategy selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ---------------------------------------------------------------------------
// Analysis types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// AnalysisReport: T2-C (mu + kappa + sigma + partial), dominant mu
///
/// Result of analyzing a compound spec. Maps spec -> compatibility report.
impl GroundsTo for AnalysisReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- spec -> report
            LexPrimitiva::Comparison, // kappa -- can_compile decision
            LexPrimitiva::Sequence,   // sigma -- ordered warnings/blockers
            LexPrimitiva::Boundary,   // partial -- blocker boundary
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// SkillAnalysis: T2-C (mu + exists + kappa + sigma), dominant mu
///
/// Per-skill analysis result. Maps skill name -> discovery info.
impl GroundsTo for SkillAnalysis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- name -> analysis
            LexPrimitiva::Existence,  // exists -- found/executable booleans
            LexPrimitiva::Comparison, // kappa -- found/not-found discrimination
            LexPrimitiva::Sequence,   // sigma -- methods list
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Build/Codegen types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// BuildResult: T2-P (mu + lambda + varsigma), dominant mu
///
/// Result of cargo build. Maps source -> compiled binary.
impl GroundsTo for BuildResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- source -> binary
            LexPrimitiva::Location, // lambda -- binary_path
            LexPrimitiva::State,    // varsigma -- stdout/stderr capture
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// GeneratedCrate: T2-C (mu + lambda + sigma + varsigma), dominant mu
///
/// Output of the codegen stage. Maps spec -> generated crate filesystem.
impl GroundsTo for GeneratedCrate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- spec -> generated code
            LexPrimitiva::Location, // lambda -- root, main_rs, cargo_toml, skill_md paths
            LexPrimitiva::Sequence, // sigma -- generation order
            LexPrimitiva::State,    // varsigma -- file system state
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// CompilerError: T2-C (partial + kappa + Sigma + lambda), dominant partial
///
/// Error boundary for the compilation pipeline. Boundary-dominant.
impl GroundsTo for CompilerError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- error variant discrimination
            LexPrimitiva::Sum,        // Sigma -- aggregated error sources
            LexPrimitiva::Location,   // lambda -- path-related errors
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
    fn compilation_result_is_t3() {
        assert_eq!(CompilationResult::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn compound_spec_is_t2c() {
        assert_eq!(CompoundSpec::tier(), Tier::T2Composite);
    }

    #[test]
    fn composition_strategy_is_t1() {
        assert_eq!(CompositionStrategy::tier(), Tier::T1Universal);
    }

    #[test]
    fn threading_mode_is_t1() {
        assert_eq!(ThreadingMode::tier(), Tier::T1Universal);
    }

    #[test]
    fn merge_strategy_is_t1() {
        assert_eq!(MergeStrategy::tier(), Tier::T1Universal);
    }

    #[test]
    fn analysis_report_is_t2c() {
        assert_eq!(AnalysisReport::tier(), Tier::T2Composite);
    }

    #[test]
    fn build_result_is_t2p() {
        assert_eq!(BuildResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn generated_crate_is_t2c() {
        assert_eq!(GeneratedCrate::tier(), Tier::T2Composite);
    }

    #[test]
    fn compiler_error_is_t2c() {
        assert_eq!(CompilerError::tier(), Tier::T2Composite);
    }

    #[test]
    fn compilation_result_dominant_is_mapping() {
        let comp = CompilationResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn compound_spec_dominant_is_state() {
        let comp = CompoundSpec::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn compiler_error_dominant_is_boundary() {
        let comp = CompilerError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }
}
