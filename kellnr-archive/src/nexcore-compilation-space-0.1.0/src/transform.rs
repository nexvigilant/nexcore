// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Transforms — movements between points in the compilation space.
//!
//! A transform maps one `CompilationPoint` to another, traversing
//! one or more axes. Transforms compose into chains.

use crate::axis::{Axis, Direction};
use crate::point::CompilationPoint;
use serde::{Deserialize, Serialize};

/// A transformation between two compilation points.
///
/// Tier: T2-C (μ + → + σ — mapped causal sequence)
///
/// Every compiler tool, transpiler, linter, AI assistant, and code
/// generator is a transform. The compilation space makes this explicit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    /// Name of this transform (e.g., "lex", "transpile", "intent_compile").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Source point in the compilation space.
    pub source: CompilationPoint,
    /// Target point in the compilation space.
    pub target: CompilationPoint,
    /// Whether the transform preserves all information (bijective).
    ///
    /// Lex is information-preserving (you can reconstruct source from tokens).
    /// Pretty-print is NOT (comments may be lost).
    pub preserves_information: bool,
    /// Whether the transform is reversible (an inverse transform exists).
    ///
    /// Lex is reversible (reverse-lex = pretty-print, roughly).
    /// Compilation is generally NOT reversible (many sources → same binary).
    pub reversible: bool,
}

impl Transform {
    /// Create a new transform.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        source: CompilationPoint,
        target: CompilationPoint,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            source,
            target,
            preserves_information: false,
            reversible: false,
        }
    }

    /// Mark this transform as information-preserving.
    pub fn information_preserving(mut self) -> Self {
        self.preserves_information = true;
        self
    }

    /// Mark this transform as reversible.
    pub fn reversible(mut self) -> Self {
        self.reversible = true;
        self
    }

    /// Which axes this transform traverses (which dimensions change).
    pub fn axes_traversed(&self) -> Vec<Axis> {
        self.source.differing_axes(&self.target)
    }

    /// How many axes change.
    pub fn axis_count(&self) -> usize {
        self.axes_traversed().len()
    }

    /// Whether this transform moves along a specific axis.
    pub fn traverses(&self, axis: Axis) -> bool {
        self.axes_traversed().contains(&axis)
    }

    /// Whether this is purely an abstraction change (vertical movement).
    pub fn is_vertical(&self) -> bool {
        let axes = self.axes_traversed();
        axes.len() == 1 && axes[0] == Axis::Abstraction
    }

    /// Whether this is purely a language change (lateral movement).
    pub fn is_lateral(&self) -> bool {
        let axes = self.axes_traversed();
        // Language change, possibly with projection change (AST shapes differ)
        axes.contains(&Axis::Language)
            && !axes.contains(&Axis::Abstraction)
            && !axes.contains(&Axis::Time)
    }

    /// Whether this is a temporal change (version movement).
    pub fn is_temporal(&self) -> bool {
        self.traverses(Axis::Time)
    }

    /// Primary direction of movement.
    pub fn direction(&self) -> Direction {
        if self.source.abstraction > self.target.abstraction {
            Direction::Down // Compiling: higher → lower abstraction
        } else if self.source.abstraction < self.target.abstraction {
            Direction::Up // Decompiling: lower → higher abstraction
        } else {
            Direction::Lateral
        }
    }

    /// Whether this is a lowering (compilation direction).
    pub fn is_lowering(&self) -> bool {
        self.direction() == Direction::Down
    }

    /// Whether this is a raising (decompilation direction).
    pub fn is_raising(&self) -> bool {
        self.direction() == Direction::Up
    }

    /// Short description: name + axes.
    pub fn summary(&self) -> String {
        let axes: Vec<&str> = self.axes_traversed().iter().map(|a| a.label()).collect();
        format!(
            "{}: {} → {} [{}]",
            self.name,
            self.source.summary(),
            self.target.summary(),
            axes.join(", "),
        )
    }
}

/// A chain of transforms — a path through the compilation space.
///
/// Tier: T2-C (σ + → — sequenced causal chain)
///
/// The standard compilation pipeline (lex → parse → lower → codegen)
/// is a `TransformChain`. So is any multi-step refactoring, migration,
/// or AI-assisted code transformation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformChain {
    /// The ordered sequence of transforms.
    pub steps: Vec<Transform>,
}

/// Error in a transform chain.
///
/// Tier: T2-P (∂ Boundary — constraint violation)
#[derive(Debug, Clone)]
pub struct ChainError {
    /// Index of the problematic step.
    pub step_index: usize,
    /// Description of the error.
    pub message: String,
}

impl TransformChain {
    /// Create an empty chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a transform to the chain.
    pub fn push(&mut self, transform: Transform) {
        self.steps.push(transform);
    }

    /// Number of steps.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The starting point (source of first transform).
    pub fn start(&self) -> Option<&CompilationPoint> {
        self.steps.first().map(|t| &t.source)
    }

    /// The ending point (target of last transform).
    pub fn end(&self) -> Option<&CompilationPoint> {
        self.steps.last().map(|t| &t.target)
    }

    /// All unique axes traversed across the entire chain.
    pub fn total_axes(&self) -> Vec<Axis> {
        let mut axes: Vec<Axis> = self.steps.iter().flat_map(|t| t.axes_traversed()).collect();
        axes.sort_by_key(|a| *a as u8);
        axes.dedup();
        axes
    }

    /// Number of unique axes traversed.
    pub fn total_axis_count(&self) -> usize {
        self.total_axes().len()
    }

    /// Whether the chain is a round trip (ends where it started).
    pub fn is_round_trip(&self) -> bool {
        match (self.start(), self.end()) {
            (Some(start), Some(end)) => start.same_position(end),
            _ => false,
        }
    }

    /// Whether the entire chain preserves information.
    ///
    /// A chain preserves information only if every step does.
    pub fn preserves_information(&self) -> bool {
        self.steps.iter().all(|t| t.preserves_information)
    }

    /// Whether the entire chain is reversible.
    ///
    /// A chain is reversible only if every step is.
    pub fn is_reversible(&self) -> bool {
        self.steps.iter().all(|t| t.reversible)
    }

    /// Validate the chain — check that consecutive transforms connect.
    ///
    /// Returns errors for steps whose source doesn't match the
    /// previous step's target (on axes that should be continuous).
    pub fn validate(&self) -> Vec<ChainError> {
        let mut errors = Vec::new();
        for i in 1..self.steps.len() {
            let prev_target = &self.steps[i - 1].target;
            let curr_source = &self.steps[i].source;

            // Check that the previous output is compatible with current input
            // We check abstraction and language as the primary continuity axes
            if prev_target.abstraction != curr_source.abstraction {
                errors.push(ChainError {
                    step_index: i,
                    message: format!(
                        "Abstraction mismatch: step {} outputs {} but step {} expects {}",
                        i - 1,
                        prev_target.abstraction.label(),
                        i,
                        curr_source.abstraction.label(),
                    ),
                });
            }
            if prev_target.language != curr_source.language {
                errors.push(ChainError {
                    step_index: i,
                    message: format!(
                        "Language mismatch: step {} outputs {} but step {} expects {}",
                        i - 1,
                        prev_target.language.as_str(),
                        i,
                        curr_source.language.as_str(),
                    ),
                });
            }
        }
        errors
    }

    /// Summary of the chain.
    pub fn summary(&self) -> String {
        if self.steps.is_empty() {
            return "Empty chain".to_string();
        }
        let names: Vec<&str> = self.steps.iter().map(|t| t.name.as_str()).collect();
        let axes = self.total_axes();
        let axis_labels: Vec<&str> = axes.iter().map(|a| a.label()).collect();
        format!(
            "{} steps: {} | axes: {}",
            self.steps.len(),
            names.join(" → "),
            axis_labels.join(", "),
        )
    }
}

/// Sort Axis by discriminant for dedup.
impl Axis {
    fn as_u8(self) -> u8 {
        match self {
            Self::Abstraction => 0,
            Self::Language => 1,
            Self::Time => 2,
            Self::Evaluation => 3,
            Self::Reflection => 4,
            Self::Projection => 5,
            Self::Branching => 6,
        }
    }
}

impl PartialOrd for Axis {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Axis {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::{AbstractionLevel, LanguageId};

    // ── Transform tests ──

    #[test]
    fn transform_creation() {
        let t = Transform::new(
            "lex",
            "Tokenize source code",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        );
        assert_eq!(t.name, "lex");
        assert!(!t.preserves_information);
        assert!(!t.reversible);
    }

    #[test]
    fn transform_builders() {
        let t = Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        )
        .information_preserving()
        .reversible();

        assert!(t.preserves_information);
        assert!(t.reversible);
    }

    #[test]
    fn transform_axes_vertical() {
        let t = Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        );
        let axes = t.axes_traversed();
        assert!(axes.contains(&Axis::Abstraction));
    }

    #[test]
    fn transform_lateral() {
        let t = Transform::new(
            "transpile",
            "Rust AST → JS AST",
            CompilationPoint::ast(LanguageId::rust()),
            CompilationPoint::ast(LanguageId::javascript()),
        );
        assert!(t.is_lateral());
        assert!(!t.is_vertical());
        assert!(t.traverses(Axis::Language));
    }

    #[test]
    fn transform_lowering() {
        let t = Transform::new(
            "compile",
            "Source → Binary",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::binary(),
        );
        assert!(t.is_lowering());
        assert!(!t.is_raising());
        assert_eq!(t.direction(), Direction::Down);
    }

    #[test]
    fn transform_raising() {
        let t = Transform::new(
            "decompile",
            "Binary → Source",
            CompilationPoint::binary(),
            CompilationPoint::source(LanguageId::c_lang()),
        );
        assert!(t.is_raising());
        assert_eq!(t.direction(), Direction::Up);
    }

    #[test]
    fn transform_multi_axis() {
        let t = Transform::new(
            "cross_compile",
            "Rust source → WASM binary",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::new(AbstractionLevel::Binary, LanguageId::wasm()),
        );
        assert!(t.traverses(Axis::Abstraction));
        assert!(t.traverses(Axis::Language));
        assert!(t.axis_count() >= 2);
    }

    #[test]
    fn transform_summary() {
        let t = Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        );
        let s = t.summary();
        assert!(s.contains("lex"));
        assert!(s.contains("Abstraction"));
    }

    // ── TransformChain tests ──

    #[test]
    fn empty_chain() {
        let chain = TransformChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
        assert!(chain.start().is_none());
        assert!(chain.end().is_none());
    }

    #[test]
    fn chain_push() {
        let mut chain = TransformChain::new();
        chain.push(Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        ));
        chain.push(Transform::new(
            "parse",
            "Parse tokens to AST",
            CompilationPoint::tokens(LanguageId::rust()),
            CompilationPoint::ast(LanguageId::rust()),
        ));
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn chain_start_end() {
        let mut chain = TransformChain::new();
        chain.push(Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        ));
        chain.push(Transform::new(
            "parse",
            "Parse",
            CompilationPoint::tokens(LanguageId::rust()),
            CompilationPoint::ast(LanguageId::rust()),
        ));

        assert_eq!(
            chain.start().map(|p| p.abstraction),
            Some(AbstractionLevel::Source)
        );
        assert_eq!(
            chain.end().map(|p| p.abstraction),
            Some(AbstractionLevel::Ast)
        );
    }

    #[test]
    fn chain_total_axes() {
        let mut chain = TransformChain::new();
        // Vertical step
        chain.push(Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        ));
        // Vertical + projection step (tokens are linear, AST is tree)
        chain.push(Transform::new(
            "parse",
            "Parse",
            CompilationPoint::tokens(LanguageId::rust()),
            CompilationPoint::ast(LanguageId::rust()),
        ));

        let axes = chain.total_axes();
        assert!(axes.contains(&Axis::Abstraction));
    }

    #[test]
    fn chain_preserves_information() {
        let mut chain = TransformChain::new();
        chain.push(
            Transform::new(
                "lex",
                "Tokenize",
                CompilationPoint::source(LanguageId::rust()),
                CompilationPoint::tokens(LanguageId::rust()),
            )
            .information_preserving(),
        );
        assert!(chain.preserves_information());

        chain.push(Transform::new(
            "lower",
            "Lower AST",
            CompilationPoint::tokens(LanguageId::rust()),
            CompilationPoint::ir(LanguageId::rust()),
        ));
        assert!(!chain.preserves_information());
    }

    #[test]
    fn chain_validate_connected() {
        let mut chain = TransformChain::new();
        chain.push(Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        ));
        chain.push(Transform::new(
            "parse",
            "Parse",
            CompilationPoint::tokens(LanguageId::rust()),
            CompilationPoint::ast(LanguageId::rust()),
        ));
        assert!(chain.validate().is_empty());
    }

    #[test]
    fn chain_validate_disconnected() {
        let mut chain = TransformChain::new();
        chain.push(Transform::new(
            "lex",
            "Tokenize",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        ));
        // Mismatch: expects AST input but previous outputs tokens
        chain.push(Transform::new(
            "lower",
            "Lower",
            CompilationPoint::ast(LanguageId::rust()),
            CompilationPoint::ir(LanguageId::rust()),
        ));
        let errors = chain.validate();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].step_index, 1);
    }

    #[test]
    fn chain_round_trip() {
        let rust_src = CompilationPoint::source(LanguageId::rust());
        let mut chain = TransformChain::new();
        chain.push(Transform::new(
            "lex",
            "Tokenize",
            rust_src.clone(),
            CompilationPoint::tokens(LanguageId::rust()),
        ));
        chain.push(Transform::new(
            "pretty_print",
            "Format",
            CompilationPoint::tokens(LanguageId::rust()),
            rust_src,
        ));
        assert!(chain.is_round_trip());
    }

    #[test]
    fn chain_not_round_trip() {
        let mut chain = TransformChain::new();
        chain.push(Transform::new(
            "compile",
            "Compile",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::binary(),
        ));
        assert!(!chain.is_round_trip());
    }

    #[test]
    fn chain_summary() {
        let mut chain = TransformChain::new();
        chain.push(Transform::new(
            "lex",
            "",
            CompilationPoint::source(LanguageId::rust()),
            CompilationPoint::tokens(LanguageId::rust()),
        ));
        chain.push(Transform::new(
            "parse",
            "",
            CompilationPoint::tokens(LanguageId::rust()),
            CompilationPoint::ast(LanguageId::rust()),
        ));
        let s = chain.summary();
        assert!(s.contains("lex → parse"));
        assert!(s.contains("2 steps"));
    }
}
