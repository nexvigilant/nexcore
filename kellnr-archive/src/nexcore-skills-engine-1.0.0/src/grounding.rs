//! # GroundsTo implementations for nexcore-skills-engine types
//!
//! Skill registry, routing, validation, taxonomy, and codegen types grounded
//! to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - Registry/routing types: Mapping (mu) dominant -- name -> skill lookup.
//! - Validation/taxonomy types: Comparison (kappa) dominant -- classification.
//! - Builder types: Mapping (mu) dominant -- config -> output.
//! - SQI types: Quantity (N) dominant -- numeric scoring.
//! - Error types: Boundary (partial) dominant -- error boundaries.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::assist_index::{SkillKnowledgeEntry, SkillKnowledgeIndex, SkillSearchResult};
use crate::builder::{BatchSummary, BuildError, BuildOptions, BuildReport, StructureCheck};
use crate::dtree_router::{DtreeRouter, DtreeRoutingResult, TaskCharacteristics};
use crate::ksb_verify::{CheckResult as KsbCheckResult, KsbBatchSummary, KsbError, KsbValidation};
use crate::registry::{SkillInfo as RegistrySkillInfo, SkillRegistry as EngineSkillRegistry};
use crate::routing::{RoutingEngine, RoutingResult, RoutingStrategy};
use crate::smst_v2::{ComponentScores, SmstV2Error, SmstV2Result};
use crate::sqi::{DimensionScore, EcosystemSqiResult, SqiDimension, SqiError, SqiGrade, SqiResult};
use crate::taxonomy::{
    ComplianceLevel as TaxComplianceLevel, NodeType, SkillCategory, SmstComponent,
    TaxonomyListResult, TaxonomyQueryResult,
};
use crate::validation::DiamondValidation;

// ---------------------------------------------------------------------------
// Registry -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// SkillRegistry (engine): T2-P (mu + varsigma + sigma), dominant mu
impl GroundsTo for EngineSkillRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- name -> skill info
            LexPrimitiva::State,    // varsigma -- mutable collection
            LexPrimitiva::Sequence, // sigma -- list ordering
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SkillInfo (engine registry): T2-P (varsigma + lambda + sigma), dominant varsigma
impl GroundsTo for RegistrySkillInfo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- skill metadata state
            LexPrimitiva::Location, // lambda -- skill path
            LexPrimitiva::Sequence, // sigma -- triggers list
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Routing -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// RoutingEngine: T2-P (mu + kappa + sigma), dominant mu
impl GroundsTo for RoutingEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- input -> skill route
            LexPrimitiva::Comparison, // kappa -- strategy selection
            LexPrimitiva::Sequence,   // sigma -- strategy priority
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// RoutingResult: T2-P (mu + N + kappa), dominant mu
impl GroundsTo for RoutingResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- route resolution
            LexPrimitiva::Quantity,   // N -- confidence score
            LexPrimitiva::Comparison, // kappa -- strategy used
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// RoutingStrategy: T1-Universal (kappa), dominant kappa
impl GroundsTo for RoutingStrategy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ---------------------------------------------------------------------------
// Validation -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// DiamondValidation: T2-C (kappa + partial + sigma + N), dominant kappa
impl GroundsTo for DiamondValidation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- compliance check
            LexPrimitiva::Boundary,   // partial -- pass/fail boundary
            LexPrimitiva::Sequence,   // sigma -- ordered checks
            LexPrimitiva::Quantity,   // N -- score
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// SQI -- N (Quantity) dominant
// ---------------------------------------------------------------------------

/// SqiResult: T2-C (N + mu + kappa + sigma), dominant N
impl GroundsTo for SqiResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric score
            LexPrimitiva::Mapping,    // mu -- skill -> score
            LexPrimitiva::Comparison, // kappa -- grade classification
            LexPrimitiva::Sequence,   // sigma -- dimension scores
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// DimensionScore: T2-P (N + mu + kappa), dominant N
impl GroundsTo for DimensionScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- score, weight, weighted
            LexPrimitiva::Mapping,    // mu -- dimension -> score
            LexPrimitiva::Comparison, // kappa -- dimension classification
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// SqiGrade: T1-Universal (kappa), dominant kappa
impl GroundsTo for SqiGrade {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// SqiDimension: T1-Universal (kappa), dominant kappa
impl GroundsTo for SqiDimension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// EcosystemSqiResult: T2-C (Sigma + N + mu + sigma), dominant Sigma
impl GroundsTo for EcosystemSqiResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- aggregated ecosystem scores
            LexPrimitiva::Quantity, // N -- numeric scores
            LexPrimitiva::Mapping,  // mu -- skills -> aggregate
            LexPrimitiva::Sequence, // sigma -- ordered results
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// SqiError: T2-P (partial + kappa), dominant partial
impl GroundsTo for SqiError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- error variant discrimination
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Builder -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// BuildReport: T2-C (mu + varsigma + lambda + sigma), dominant mu
impl GroundsTo for BuildReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- config -> report
            LexPrimitiva::State,    // varsigma -- structure checks
            LexPrimitiva::Location, // lambda -- path
            LexPrimitiva::Sequence, // sigma -- ordered checks
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// BuildOptions: T2-P (varsigma + partial + N), dominant varsigma
impl GroundsTo for BuildOptions {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- option configuration
            LexPrimitiva::Boundary, // partial -- compliance threshold
            LexPrimitiva::Quantity, // N -- compliance level
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// StructureCheck: T2-P (kappa + exists + mu), dominant kappa
impl GroundsTo for StructureCheck {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- present/correct checks
            LexPrimitiva::Existence,  // exists -- file presence
            LexPrimitiva::Mapping,    // mu -- check -> result
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// BatchSummary: T2-C (Sigma + N + mu + sigma), dominant Sigma
impl GroundsTo for BatchSummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- multi-skill aggregation
            LexPrimitiva::Quantity, // N -- success/failure counts
            LexPrimitiva::Mapping,  // mu -- batch -> summary
            LexPrimitiva::Sequence, // sigma -- ordered reports
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// BuildError: T2-P (partial + kappa + Sigma), dominant partial
impl GroundsTo for BuildError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- variant discrimination
            LexPrimitiva::Sum,        // Sigma -- aggregated errors
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// SMST v2 -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// SmstV2Result: T2-C (mu + N + kappa + sigma), dominant mu
impl GroundsTo for SmstV2Result {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- skill -> SMST extraction
            LexPrimitiva::Quantity,   // N -- component scores
            LexPrimitiva::Comparison, // kappa -- compliance check
            LexPrimitiva::Sequence,   // sigma -- ordered components
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// ComponentScores: T2-P (N + mu + partial), dominant N
impl GroundsTo for ComponentScores {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric component scores
            LexPrimitiva::Mapping,  // mu -- text -> scores
            LexPrimitiva::Boundary, // partial -- scoring boundaries
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// SmstV2Error: T2-P (partial + kappa), dominant partial
impl GroundsTo for SmstV2Error {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- error variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Taxonomy -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// TaxComplianceLevel (taxonomy): T2-P (kappa + N + varsigma), dominant kappa
impl GroundsTo for TaxComplianceLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- level classification
            LexPrimitiva::Quantity,   // N -- numeric requirements
            LexPrimitiva::State,      // varsigma -- requirement state
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// SmstComponent: T2-P (kappa + N + mu), dominant kappa
impl GroundsTo for SmstComponent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- component classification
            LexPrimitiva::Quantity,   // N -- weight
            LexPrimitiva::Mapping,    // mu -- name -> component
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// SkillCategory: T2-P (kappa + mu + varsigma), dominant kappa
impl GroundsTo for SkillCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- category classification
            LexPrimitiva::Mapping,    // mu -- name -> category
            LexPrimitiva::State,      // varsigma -- category attributes
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// NodeType: T2-P (kappa + mu), dominant kappa
impl GroundsTo for NodeType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- node type discrimination
            LexPrimitiva::Mapping,    // mu -- name -> node type
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// TaxonomyQueryResult: T2-P (mu + kappa + exists), dominant mu
impl GroundsTo for TaxonomyQueryResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- query -> result
            LexPrimitiva::Comparison, // kappa -- found/not-found
            LexPrimitiva::Existence,  // exists -- optional result
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// TaxonomyListResult: T2-P (mu + sigma + N), dominant mu
impl GroundsTo for TaxonomyListResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- query -> list
            LexPrimitiva::Sequence, // sigma -- ordered entries
            LexPrimitiva::Quantity, // N -- entry count
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Assist index -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// SkillKnowledgeEntry: T2-P (varsigma + mu + sigma), dominant varsigma
impl GroundsTo for SkillKnowledgeEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- entry state
            LexPrimitiva::Mapping,  // mu -- SKILL.md -> entry
            LexPrimitiva::Sequence, // sigma -- triggers list
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// SkillKnowledgeIndex: T2-P (mu + varsigma + sigma), dominant mu
impl GroundsTo for SkillKnowledgeIndex {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- name -> entry
            LexPrimitiva::State,    // varsigma -- index state
            LexPrimitiva::Sequence, // sigma -- entry ordering
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SkillSearchResult: T2-P (mu + N + kappa), dominant mu
impl GroundsTo for SkillSearchResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- query -> result
            LexPrimitiva::Quantity,   // N -- score
            LexPrimitiva::Comparison, // kappa -- relevance ranking
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// KSB verify -- kappa (Comparison) dominant
// ---------------------------------------------------------------------------

/// KsbValidation: T2-C (kappa + partial + sigma + N), dominant kappa
impl GroundsTo for KsbValidation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- validation checks
            LexPrimitiva::Boundary,   // partial -- pass/fail boundary
            LexPrimitiva::Sequence,   // sigma -- ordered results
            LexPrimitiva::Quantity,   // N -- score
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// KsbCheckResult: T2-P (kappa + partial + mu), dominant kappa
impl GroundsTo for KsbCheckResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- pass/fail check
            LexPrimitiva::Boundary,   // partial -- check boundary
            LexPrimitiva::Mapping,    // mu -- check -> result
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// KsbBatchSummary: T2-C (Sigma + N + mu + sigma), dominant Sigma
impl GroundsTo for KsbBatchSummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- batch aggregation
            LexPrimitiva::Quantity, // N -- counts
            LexPrimitiva::Mapping,  // mu -- batch -> summary
            LexPrimitiva::Sequence, // sigma -- ordered validations
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// KsbError: T2-P (partial + kappa), dominant partial
impl GroundsTo for KsbError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- error boundary
            LexPrimitiva::Comparison, // kappa -- variant discrimination
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Dtree router -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// DtreeRouter: T2-P (mu + kappa + rho), dominant mu
impl GroundsTo for DtreeRouter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- task -> skill route
            LexPrimitiva::Comparison, // kappa -- decision tree branching
            LexPrimitiva::Recursion,  // rho -- tree traversal
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// DtreeRoutingResult: T2-P (mu + N + kappa), dominant mu
impl GroundsTo for DtreeRoutingResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- route resolution
            LexPrimitiva::Quantity,   // N -- confidence
            LexPrimitiva::Comparison, // kappa -- skill selection
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// TaskCharacteristics: T2-C (varsigma + kappa + N + mu), dominant varsigma
impl GroundsTo for TaskCharacteristics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- task state snapshot
            LexPrimitiva::Comparison, // kappa -- characteristic classification
            LexPrimitiva::Quantity,   // N -- numeric features
            LexPrimitiva::Mapping,    // mu -- task -> characteristics
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
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
    fn skill_registry_is_t2p() {
        assert_eq!(EngineSkillRegistry::tier(), Tier::T2Primitive);
    }

    #[test]
    fn routing_strategy_is_t1() {
        assert_eq!(RoutingStrategy::tier(), Tier::T1Universal);
    }

    #[test]
    fn diamond_validation_is_t2c() {
        assert_eq!(DiamondValidation::tier(), Tier::T2Composite);
    }

    #[test]
    fn sqi_result_is_t2c() {
        assert_eq!(SqiResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn sqi_grade_is_t1() {
        assert_eq!(SqiGrade::tier(), Tier::T1Universal);
    }

    #[test]
    fn build_report_is_t2c() {
        assert_eq!(BuildReport::tier(), Tier::T2Composite);
    }

    #[test]
    fn smst_v2_result_is_t2c() {
        assert_eq!(SmstV2Result::tier(), Tier::T2Composite);
    }

    #[test]
    fn routing_engine_dominant_is_mapping() {
        let comp = RoutingEngine::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn diamond_validation_dominant_is_comparison() {
        let comp = DiamondValidation::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn sqi_result_dominant_is_quantity() {
        let comp = SqiResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn ecosystem_sqi_dominant_is_sum() {
        let comp = EcosystemSqiResult::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn dtree_router_dominant_is_mapping() {
        let comp = DtreeRouter::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn ksb_validation_dominant_is_comparison() {
        let comp = KsbValidation::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn task_characteristics_is_t2c() {
        assert_eq!(TaskCharacteristics::tier(), Tier::T2Composite);
    }
}
