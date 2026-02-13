// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Compilation Point — a position in the 7-dimensional compilation space.
//!
//! Every artifact (source file, token stream, AST, binary, etc.)
//! occupies a point in this space. Transforms move between points.

use crate::axis::{
    AbstractionLevel, Axis, BranchConfig, Dimensionality, EvalState, LanguageId, ReflectionDepth,
    TemporalCoord,
};
use serde::{Deserialize, Serialize};

/// A position in the 7-dimensional compilation space.
///
/// Tier: T2-C (σ + μ + ν + ∂ + ρ + Σ — composed from all axis primitives)
///
/// Every code artifact — from a human's spoken intent to a running binary —
/// occupies a specific point in this space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationPoint {
    /// Position on the abstraction axis.
    pub abstraction: AbstractionLevel,
    /// Position on the language axis.
    pub language: LanguageId,
    /// Position on the temporal axis.
    pub time: TemporalCoord,
    /// Position on the evaluation axis.
    pub evaluation: EvalState,
    /// Position on the reflection axis.
    pub reflection: ReflectionDepth,
    /// Position on the projection axis.
    pub dimensionality: Dimensionality,
    /// Position on the branching axis.
    pub branch: BranchConfig,
}

impl Default for CompilationPoint {
    fn default() -> Self {
        Self {
            abstraction: AbstractionLevel::Source,
            language: LanguageId::default(),
            time: TemporalCoord::default(),
            evaluation: EvalState::Symbolic,
            reflection: ReflectionDepth::GROUND,
            dimensionality: Dimensionality::Linear,
            branch: BranchConfig::default(),
        }
    }
}

impl CompilationPoint {
    /// Create a point at a specific abstraction level and language.
    ///
    /// Other axes default to their natural starting positions.
    pub fn new(abstraction: AbstractionLevel, language: LanguageId) -> Self {
        Self {
            abstraction,
            language,
            ..Self::default()
        }
    }

    /// Create a point representing human intent (natural language).
    pub fn intent() -> Self {
        Self {
            abstraction: AbstractionLevel::Intent,
            language: LanguageId::natural(),
            dimensionality: Dimensionality::Linear,
            ..Self::default()
        }
    }

    /// Create a point representing source code.
    pub fn source(language: LanguageId) -> Self {
        Self {
            abstraction: AbstractionLevel::Source,
            language,
            dimensionality: Dimensionality::Linear,
            ..Self::default()
        }
    }

    /// Create a point representing a token stream.
    pub fn tokens(language: LanguageId) -> Self {
        Self {
            abstraction: AbstractionLevel::Token,
            language,
            dimensionality: Dimensionality::Linear,
            ..Self::default()
        }
    }

    /// Create a point representing an AST.
    pub fn ast(language: LanguageId) -> Self {
        Self {
            abstraction: AbstractionLevel::Ast,
            language,
            dimensionality: Dimensionality::Tree,
            ..Self::default()
        }
    }

    /// Create a point representing intermediate representation.
    pub fn ir(language: LanguageId) -> Self {
        Self {
            abstraction: AbstractionLevel::Ir,
            language,
            dimensionality: Dimensionality::Graph,
            ..Self::default()
        }
    }

    /// Create a point representing a binary.
    pub fn binary() -> Self {
        Self {
            abstraction: AbstractionLevel::Binary,
            language: LanguageId::new("machine"),
            dimensionality: Dimensionality::Linear,
            evaluation: EvalState::Concrete,
            ..Self::default()
        }
    }

    /// Create a point representing runtime execution.
    pub fn execution() -> Self {
        Self {
            abstraction: AbstractionLevel::Execution,
            language: LanguageId::new("machine"),
            dimensionality: Dimensionality::Scalar,
            evaluation: EvalState::Concrete,
            ..Self::default()
        }
    }

    // ── Builder methods ──

    /// Set the temporal coordinate.
    pub fn at_time(mut self, time: TemporalCoord) -> Self {
        self.time = time;
        self
    }

    /// Set the evaluation state.
    pub fn with_eval(mut self, eval: EvalState) -> Self {
        self.evaluation = eval;
        self
    }

    /// Set the reflection depth.
    pub fn with_reflection(mut self, depth: ReflectionDepth) -> Self {
        self.reflection = depth;
        self
    }

    /// Set the branch configuration.
    pub fn with_branch(mut self, branch: BranchConfig) -> Self {
        self.branch = branch;
        self
    }

    /// Set the dimensionality.
    pub fn with_dimensionality(mut self, dim: Dimensionality) -> Self {
        self.dimensionality = dim;
        self
    }

    // ── Axis queries ──

    /// Which axes differ between this point and another.
    pub fn differing_axes(&self, other: &Self) -> Vec<Axis> {
        let mut axes = Vec::new();
        if self.abstraction != other.abstraction {
            axes.push(Axis::Abstraction);
        }
        if self.language != other.language {
            axes.push(Axis::Language);
        }
        if self.time != other.time {
            axes.push(Axis::Time);
        }
        if self.evaluation != other.evaluation {
            axes.push(Axis::Evaluation);
        }
        if self.reflection != other.reflection {
            axes.push(Axis::Reflection);
        }
        if self.dimensionality != other.dimensionality {
            axes.push(Axis::Projection);
        }
        if self.branch != other.branch {
            axes.push(Axis::Branching);
        }
        axes
    }

    /// How many axes differ between this point and another.
    pub fn axis_distance(&self, other: &Self) -> usize {
        self.differing_axes(other).len()
    }

    /// Whether two points occupy the same position.
    pub fn same_position(&self, other: &Self) -> bool {
        self.differing_axes(other).is_empty()
    }

    /// Short description of this point.
    pub fn summary(&self) -> String {
        format!(
            "{}({}, {}, r{}, {})",
            self.abstraction.label(),
            self.language.as_str(),
            self.evaluation.label(),
            self.reflection.depth(),
            self.dimensionality.label(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_point() {
        let p = CompilationPoint::default();
        assert_eq!(p.abstraction, AbstractionLevel::Source);
        assert_eq!(p.evaluation, EvalState::Symbolic);
        assert!(p.reflection.is_ground());
    }

    #[test]
    fn intent_point() {
        let p = CompilationPoint::intent();
        assert_eq!(p.abstraction, AbstractionLevel::Intent);
        assert_eq!(p.language, LanguageId::natural());
    }

    #[test]
    fn source_point() {
        let p = CompilationPoint::source(LanguageId::rust());
        assert_eq!(p.abstraction, AbstractionLevel::Source);
        assert_eq!(p.language, LanguageId::rust());
        assert_eq!(p.dimensionality, Dimensionality::Linear);
    }

    #[test]
    fn ast_point_is_tree() {
        let p = CompilationPoint::ast(LanguageId::rust());
        assert_eq!(p.dimensionality, Dimensionality::Tree);
    }

    #[test]
    fn ir_point_is_graph() {
        let p = CompilationPoint::ir(LanguageId::llvm_ir());
        assert_eq!(p.dimensionality, Dimensionality::Graph);
    }

    #[test]
    fn binary_is_concrete() {
        let p = CompilationPoint::binary();
        assert_eq!(p.evaluation, EvalState::Concrete);
    }

    #[test]
    fn execution_is_scalar() {
        let p = CompilationPoint::execution();
        assert_eq!(p.dimensionality, Dimensionality::Scalar);
        assert_eq!(p.evaluation, EvalState::Concrete);
    }

    #[test]
    fn builder_chain() {
        let p = CompilationPoint::source(LanguageId::rust())
            .at_time(TemporalCoord::with_label(3, "v1.0"))
            .with_eval(EvalState::Partial)
            .with_reflection(ReflectionDepth::META)
            .with_branch(BranchConfig::new().with_feature("async"));

        assert_eq!(p.time.revision, 3);
        assert_eq!(p.evaluation, EvalState::Partial);
        assert!(p.reflection.is_meta());
        assert!(p.branch.has_feature("async"));
    }

    #[test]
    fn differing_axes_same_point() {
        let a = CompilationPoint::source(LanguageId::rust());
        let b = CompilationPoint::source(LanguageId::rust());
        assert!(a.differing_axes(&b).is_empty());
        assert!(a.same_position(&b));
    }

    #[test]
    fn differing_axes_abstraction_only() {
        let source = CompilationPoint::source(LanguageId::rust());
        let tokens = CompilationPoint::tokens(LanguageId::rust());
        let diff = source.differing_axes(&tokens);
        // Abstraction changes; dimensionality stays Linear for both
        assert!(diff.contains(&Axis::Abstraction));
    }

    #[test]
    fn differing_axes_language_lateral() {
        let rust = CompilationPoint::source(LanguageId::rust());
        let js = CompilationPoint::source(LanguageId::javascript());
        let diff = rust.differing_axes(&js);
        assert_eq!(diff, vec![Axis::Language]);
    }

    #[test]
    fn differing_axes_multiple() {
        let a = CompilationPoint::source(LanguageId::rust());
        let b = CompilationPoint::ast(LanguageId::javascript());
        let diff = a.differing_axes(&b);
        assert!(diff.contains(&Axis::Abstraction));
        assert!(diff.contains(&Axis::Language));
        assert!(diff.contains(&Axis::Projection)); // Linear → Tree
    }

    #[test]
    fn axis_distance() {
        let a = CompilationPoint::intent();
        let b = CompilationPoint::binary();
        assert!(a.axis_distance(&b) >= 2); // abstraction + evaluation at minimum
    }

    #[test]
    fn summary_format() {
        let p = CompilationPoint::source(LanguageId::rust());
        let s = p.summary();
        assert!(s.contains("Source"));
        assert!(s.contains("rust"));
    }
}
