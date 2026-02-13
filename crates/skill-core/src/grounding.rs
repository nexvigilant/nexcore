//! # GroundsTo implementations for skill-core types
//!
//! Core skill infrastructure types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `SkillMetadata`, `SkillContext` -- State (varsigma) dominant as they encapsulate
//!   skill configuration and execution context.
//! - `ComplianceLevel`, `Trigger` -- Comparison (kappa) dominant as they classify
//!   and discriminate between categories.
//! - `SkillOutput`, `OutputContent` -- Mapping (mu) dominant as they transform
//!   execution results into presentable output.
//! - `SkillRegistry` -- Mapping (mu) dominant as it maps names to skill instances.
//! - `SkillChain` -- Sequence (sigma) dominant as it sequences skill execution.
//! - `SkillParallel` -- Sum (Sigma) dominant as it aggregates parallel results.
//! - `SkillError` -- Boundary (partial) dominant as it represents error boundaries.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    ComplianceLevel, OutputContent, SkillChain, SkillContext, SkillError, SkillMetadata,
    SkillOutput, SkillParallel, SkillRegistry, Trigger,
};

// ---------------------------------------------------------------------------
// State-dominant types -- varsigma (State) dominant
// ---------------------------------------------------------------------------

/// SkillMetadata: T2-C (varsigma + mu + sigma + exists), dominant varsigma
///
/// Metadata captures the static configuration state of a skill: name, version,
/// compliance level, dependencies. State-dominant because it encapsulates
/// identity and configuration. Mapping is secondary (maps frontmatter to structured data).
/// Sequence is tertiary (ordered dependency list). Existence is quaternary (optional paired_agent).
impl GroundsTo for SkillMetadata {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- encapsulated configuration state
            LexPrimitiva::Mapping,   // mu -- frontmatter -> structured metadata
            LexPrimitiva::Sequence,  // sigma -- ordered dependency list
            LexPrimitiva::Existence, // exists -- optional paired_agent
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// SkillContext: T2-C (varsigma + mu + sigma + exists), dominant varsigma
///
/// Execution context provided to a skill: input, parsed args, params, cwd, session.
/// State-dominant as it captures the complete execution environment.
/// Mapping is secondary (raw input mapped to parsed args/params).
/// Sequence is tertiary (ordered args list). Existence is quaternary (optional session_id).
impl GroundsTo for SkillContext {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- execution context snapshot
            LexPrimitiva::Mapping,   // mu -- input -> parsed args
            LexPrimitiva::Sequence,  // sigma -- ordered argument list
            LexPrimitiva::Existence, // exists -- optional session_id
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Classification enums -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// ComplianceLevel: T1-Universal (kappa), dominant kappa
///
/// Four-level quality classification: Bronze < Silver < Gold < Platinum.
/// Pure comparison -- the entire type exists to compare skill quality against
/// discrete thresholds via its Ord implementation.
impl GroundsTo for ComplianceLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- ordered quality classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// Trigger: T2-P (kappa + mu + partial), dominant kappa
///
/// Pattern matching discriminant: Command, Pattern, Keyword, Always.
/// Comparison-dominant because the matches() method compares input against
/// patterns to determine activation. Mapping is secondary (input -> bool match).
/// Boundary is tertiary (threshold between match and non-match).
impl GroundsTo for Trigger {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- input pattern comparison
            LexPrimitiva::Mapping,    // mu -- input -> match result
            LexPrimitiva::Boundary,   // partial -- match/no-match boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Mapping-dominant types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// SkillOutput: T2-P (mu + varsigma + sigma), dominant mu
///
/// Output produced by skill execution. Maps execution results into structured
/// content with metadata and suggestions. Mapping-dominant because the type
/// transforms raw results into presentable output.
/// State is secondary (metadata key-value context).
/// Sequence is tertiary (ordered suggestions list).
impl GroundsTo for SkillOutput {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- execution result -> presentable output
            LexPrimitiva::State,    // varsigma -- metadata context
            LexPrimitiva::Sequence, // sigma -- ordered suggestions
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// OutputContent: T2-P (mu + rho + Sigma), dominant mu
///
/// Content types that skills produce: Text, Markdown, Json, Table, Multi.
/// Mapping-dominant as each variant transforms data into a representation.
/// Recursion is secondary (Multi variant is self-referential).
/// Sum is tertiary (enum aggregates multiple output forms).
impl GroundsTo for OutputContent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- data -> representation
            LexPrimitiva::Recursion, // rho -- Multi variant is recursive
            LexPrimitiva::Sum,       // Sigma -- enum sum type of output forms
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SkillRegistry: T2-P (mu + varsigma + exists), dominant mu
///
/// Runtime skill lookup. Maps skill names to skill instances via HashMap.
/// Mapping-dominant: the core operation is name -> Skill lookup.
/// State is secondary (mutable skill collection).
/// Existence is tertiary (get() returns Option, testing existence).
impl GroundsTo for SkillRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- name -> skill instance
            LexPrimitiva::State,     // varsigma -- mutable registry state
            LexPrimitiva::Existence, // exists -- optional skill lookup
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Composition types
// ---------------------------------------------------------------------------

/// SkillChain: T2-C (sigma + mu + varsigma + rho), dominant sigma
///
/// Sequential skill execution: A -> B. Chains two skills where the output
/// of the first feeds into the second. Sequence-dominant because ordering
/// is the fundamental concern. Mapping is secondary (output threading).
/// State is tertiary (context transformation between steps).
/// Recursion is quaternary (chain can contain chains).
impl GroundsTo for SkillChain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // sigma -- ordered A->B execution
            LexPrimitiva::Mapping,   // mu -- output threading between steps
            LexPrimitiva::State,     // varsigma -- context transformation
            LexPrimitiva::Recursion, // rho -- chains can contain chains
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// SkillParallel: T2-C (Sigma + mu + varsigma + sigma), dominant Sigma
///
/// Parallel skill execution: A | B. Runs two skills concurrently and
/// aggregates results via Multi output. Sum-dominant because the core
/// operation is aggregation of parallel results.
/// Mapping is secondary (result combination). State is tertiary (context duplication).
/// Sequence is quaternary (result collection ordering).
impl GroundsTo for SkillParallel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- parallel result aggregation
            LexPrimitiva::Mapping,  // mu -- result combination
            LexPrimitiva::State,    // varsigma -- context duplication
            LexPrimitiva::Sequence, // sigma -- result collection order
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error type -- partial (Boundary) dominant
// ---------------------------------------------------------------------------

/// SkillError: T2-P (partial + kappa + Sigma), dominant partial
///
/// Error boundary for skill execution. Boundary-dominant because errors
/// define the edge between valid and invalid execution.
/// Comparison is secondary (error variant discrimination).
/// Sum is tertiary (enum sum of error cases).
impl GroundsTo for SkillError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- error variant discrimination
            LexPrimitiva::Sum,        // Sigma -- sum of error cases
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // Tier classification tests

    #[test]
    fn compliance_level_is_t1() {
        assert_eq!(ComplianceLevel::tier(), Tier::T1Universal);
    }

    #[test]
    fn trigger_is_t2p() {
        assert_eq!(Trigger::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_metadata_is_t2c() {
        assert_eq!(SkillMetadata::tier(), Tier::T2Composite);
    }

    #[test]
    fn skill_context_is_t2c() {
        assert_eq!(SkillContext::tier(), Tier::T2Composite);
    }

    #[test]
    fn skill_output_is_t2p() {
        assert_eq!(SkillOutput::tier(), Tier::T2Primitive);
    }

    #[test]
    fn output_content_is_t2p() {
        assert_eq!(OutputContent::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_registry_is_t2p() {
        assert_eq!(SkillRegistry::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_chain_is_t2c() {
        assert_eq!(SkillChain::tier(), Tier::T2Composite);
    }

    #[test]
    fn skill_parallel_is_t2c() {
        assert_eq!(SkillParallel::tier(), Tier::T2Composite);
    }

    #[test]
    fn skill_error_is_t2p() {
        assert_eq!(SkillError::tier(), Tier::T2Primitive);
    }

    // Dominant primitive tests

    #[test]
    fn compliance_level_dominant_is_comparison() {
        let comp = ComplianceLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn skill_metadata_dominant_is_state() {
        let comp = SkillMetadata::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn skill_output_dominant_is_mapping() {
        let comp = SkillOutput::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn skill_chain_dominant_is_sequence() {
        let comp = SkillChain::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn skill_parallel_dominant_is_sum() {
        let comp = SkillParallel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn skill_error_dominant_is_boundary() {
        let comp = SkillError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }
}
