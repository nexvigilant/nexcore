// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Root Constants
//!
//! All computation grounds to two constants: `0` and `1`.
//!
//! ## Mathematical Foundation
//!
//! ```text
//! 0 = absence, false, zero, ∅
//! 1 = existence, true, one, ∃
//! ```
//!
//! Every value in Prima can be traced through its primitive composition
//! back to these root constants. This module provides the machinery
//! for that tracing.
//!
//! ## Grounding Chain
//!
//! ```text
//! Value → Composition → Primitives → Constants → {0, 1}
//! ```
//!
//! ## Tier: T1-Universal (pure foundation)

use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};

/// The two root constants of all computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RootConstant {
    /// `0` — absence, false, zero, ∅
    Zero,
    /// `1` — existence, true, one, ∃
    One,
}

impl RootConstant {
    /// Symbol representation.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Zero => "0",
            Self::One => "1",
        }
    }

    /// Mathematical meaning.
    #[must_use]
    pub const fn meaning(&self) -> &'static str {
        match self {
            Self::Zero => "absence",
            Self::One => "existence",
        }
    }

    /// Boolean interpretation.
    #[must_use]
    pub const fn as_bool(&self) -> bool {
        match self {
            Self::Zero => false,
            Self::One => true,
        }
    }

    /// Integer interpretation.
    #[must_use]
    pub const fn as_i64(&self) -> i64 {
        match self {
            Self::Zero => 0,
            Self::One => 1,
        }
    }
}

impl std::fmt::Display for RootConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

/// Grounding trace showing how a value reaches root constants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingTrace {
    /// The primitive composition.
    pub composition: PrimitiveComposition,
    /// Steps tracing each primitive to constants.
    pub steps: Vec<PrimitiveGrounding>,
    /// Final root constants this value grounds to.
    pub roots: Vec<RootConstant>,
}

/// How a single primitive grounds to constants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveGrounding {
    /// The primitive being grounded.
    pub primitive: LexPrimitiva,
    /// The constant it grounds to.
    pub grounds_to: RootConstant,
    /// Explanation of the grounding.
    pub explanation: String,
}

impl GroundingTrace {
    /// Create a new trace from a composition.
    #[must_use]
    pub fn from_composition(composition: &PrimitiveComposition) -> Self {
        let steps: Vec<PrimitiveGrounding> = composition
            .primitives
            .iter()
            .map(|p| ground_primitive(*p))
            .collect();

        let roots = collect_roots(&steps);

        Self {
            composition: composition.clone(),
            steps,
            roots,
        }
    }

    /// Check if this trace reaches the specified root.
    #[must_use]
    pub fn reaches(&self, root: RootConstant) -> bool {
        self.roots.contains(&root)
    }

    /// Check if fully grounded (reaches at least one root).
    #[must_use]
    pub fn is_grounded(&self) -> bool {
        !self.roots.is_empty()
    }

    /// Format as a grounding chain string.
    #[must_use]
    pub fn format_chain(&self) -> String {
        let prims = format_primitives(&self.composition);
        let roots = format_roots(&self.roots);
        format!("{prims} → {roots}")
    }
}

/// Collect unique roots from grounding steps.
fn collect_roots(steps: &[PrimitiveGrounding]) -> Vec<RootConstant> {
    let mut roots = Vec::new();
    for step in steps {
        if !roots.contains(&step.grounds_to) {
            roots.push(step.grounds_to);
        }
    }
    roots
}

/// Format primitives for display.
fn format_primitives(comp: &PrimitiveComposition) -> String {
    comp.primitives
        .iter()
        .map(|p| p.symbol())
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Format roots for display.
fn format_roots(roots: &[RootConstant]) -> String {
    if roots.is_empty() {
        return "∅".to_string();
    }
    let syms: Vec<_> = roots.iter().map(|r| r.symbol()).collect();
    format!("{{{}}}", syms.join(", "))
}

/// Ground a single primitive to its root constant.
fn ground_primitive(p: LexPrimitiva) -> PrimitiveGrounding {
    let (grounds_to, explanation) = primitive_grounding(p);
    PrimitiveGrounding {
        primitive: p,
        grounds_to,
        explanation,
    }
}

/// Determine how a primitive grounds to constants.
fn primitive_grounding(p: LexPrimitiva) -> (RootConstant, String) {
    match p {
        // Zero-grounded primitives (absence, negation, empty)
        LexPrimitiva::Void => (RootConstant::Zero, "∅ represents absence → 0".into()),
        LexPrimitiva::Boundary => (RootConstant::Zero, "∂ marks limit/halt → 0".into()),

        // One-grounded primitives (existence, presence)
        LexPrimitiva::Existence => (RootConstant::One, "∃ asserts existence → 1".into()),
        LexPrimitiva::Persistence => (RootConstant::One, "π endures → 1".into()),

        // Quantity grounds to both (can be 0 or 1 or any N)
        LexPrimitiva::Quantity => (
            RootConstant::One,
            "N contains {0,1} → 1 (existence of quantity)".into(),
        ),

        // Structural primitives ground through existence
        LexPrimitiva::Sequence => (RootConstant::One, "σ is ordered existence → 1".into()),
        LexPrimitiva::Mapping => (RootConstant::One, "μ transforms existence → 1".into()),
        LexPrimitiva::State => (
            RootConstant::One,
            "ς is data existing at a point → 1".into(),
        ),
        LexPrimitiva::Location => (
            RootConstant::One,
            "λ points to existing reference → 1".into(),
        ),

        // Sum grounds through existence (one of many exists)
        LexPrimitiva::Sum => (
            RootConstant::One,
            "Σ selects one existing variant → 1".into(),
        ),

        // Comparison grounds through both (result is 0 or 1)
        LexPrimitiva::Comparison => (
            RootConstant::One,
            "κ produces {0,1} → grounds to both".into(),
        ),

        // Causal primitives ground through existence (something happens)
        LexPrimitiva::Causality => (
            RootConstant::One,
            "→ produces effect (existence) → 1".into(),
        ),
        LexPrimitiva::Recursion => (RootConstant::One, "ρ self-references existence → 1".into()),

        // Irreversibility (one-way state transition)
        LexPrimitiva::Irreversibility => (RootConstant::One, "∝ one-way transition → 1".into()),

        // Frequency for time-based
        LexPrimitiva::Frequency => (RootConstant::One, "ν measures rate → 1".into()),

        // Product grounds through existence (conjunction of components)
        LexPrimitiva::Product => (
            RootConstant::One,
            "× combines existing components → 1".into(),
        ),
        _ => (RootConstant::One, "unknown primitive → 1".into()),
    }
}

// ============================================================================
// Constant Tracking for Values
// ============================================================================

/// Track how a value's constants flow through operations.
#[derive(Debug, Clone, Default)]
pub struct ConstantTracker {
    /// Traces accumulated during computation.
    traces: Vec<GroundingTrace>,
}

impl ConstantTracker {
    /// Create a new tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a composition.
    pub fn track(&mut self, composition: &PrimitiveComposition) {
        self.traces
            .push(GroundingTrace::from_composition(composition));
    }

    /// Get all traces.
    #[must_use]
    pub fn traces(&self) -> &[GroundingTrace] {
        &self.traces
    }

    /// Check if all tracked values are grounded.
    #[must_use]
    pub fn all_grounded(&self) -> bool {
        self.traces.iter().all(GroundingTrace::is_grounded)
    }

    /// Format a summary of tracked constants.
    #[must_use]
    pub fn summary(&self) -> String {
        let total = self.traces.len();
        let grounded = self.traces.iter().filter(|t| t.is_grounded()).count();
        format!("{grounded}/{total} values grounded to {{0, 1}}")
    }
}

// ============================================================================
// Literal to Constant Mapping
// ============================================================================

/// Map a literal integer to its root constant.
#[must_use]
pub fn int_to_constant(n: i64) -> RootConstant {
    if n == 0 {
        RootConstant::Zero
    } else {
        RootConstant::One // All non-zero integers ground to existence
    }
}

/// Map a boolean to its root constant.
#[must_use]
pub const fn bool_to_constant(b: bool) -> RootConstant {
    if b {
        RootConstant::One
    } else {
        RootConstant::Zero
    }
}

/// Map a float to its root constant (zero vs non-zero).
#[must_use]
pub fn float_to_constant(f: f64) -> RootConstant {
    if f == 0.0 {
        RootConstant::Zero
    } else {
        RootConstant::One
    }
}

/// Map a string to its root constant (empty vs non-empty).
#[must_use]
pub fn string_to_constant(s: &str) -> RootConstant {
    if s.is_empty() {
        RootConstant::Zero
    } else {
        RootConstant::One
    }
}

/// Map an Option to its root constant.
#[must_use]
pub fn option_to_constant<T>(opt: &Option<T>) -> RootConstant {
    if opt.is_some() {
        RootConstant::One
    } else {
        RootConstant::Zero
    }
}

/// Map a sequence length to root constant.
#[must_use]
pub fn len_to_constant(len: usize) -> RootConstant {
    if len == 0 {
        RootConstant::Zero
    } else {
        RootConstant::One
    }
}

// ============================================================================
// Display Helpers
// ============================================================================

/// Format a complete grounding report.
#[must_use]
pub fn format_grounding_report(trace: &GroundingTrace) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Composition: {}",
        format_primitives(&trace.composition)
    ));
    lines.push("Grounding steps:".to_string());
    for step in &trace.steps {
        lines.push(format!(
            "  {} → {} ({})",
            step.primitive.symbol(),
            step.grounds_to,
            step.explanation
        ));
    }
    lines.push(format!("Roots: {}", format_roots(&trace.roots)));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_constant_symbols() {
        assert_eq!(RootConstant::Zero.symbol(), "0");
        assert_eq!(RootConstant::One.symbol(), "1");
    }

    #[test]
    fn test_root_constant_meanings() {
        assert_eq!(RootConstant::Zero.meaning(), "absence");
        assert_eq!(RootConstant::One.meaning(), "existence");
    }

    #[test]
    fn test_root_constant_as_bool() {
        assert!(!RootConstant::Zero.as_bool());
        assert!(RootConstant::One.as_bool());
    }

    #[test]
    fn test_root_constant_as_i64() {
        assert_eq!(RootConstant::Zero.as_i64(), 0);
        assert_eq!(RootConstant::One.as_i64(), 1);
    }

    #[test]
    fn test_void_grounds_to_zero() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Void]);
        let trace = GroundingTrace::from_composition(&comp);
        assert!(trace.reaches(RootConstant::Zero));
    }

    #[test]
    fn test_existence_grounds_to_one() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Existence]);
        let trace = GroundingTrace::from_composition(&comp);
        assert!(trace.reaches(RootConstant::One));
    }

    #[test]
    fn test_quantity_is_grounded() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let trace = GroundingTrace::from_composition(&comp);
        assert!(trace.is_grounded());
    }

    #[test]
    fn test_mixed_composition_reaches_both() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Void, LexPrimitiva::Existence]);
        let trace = GroundingTrace::from_composition(&comp);
        assert!(trace.reaches(RootConstant::Zero));
        assert!(trace.reaches(RootConstant::One));
    }

    #[test]
    fn test_format_chain() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        let trace = GroundingTrace::from_composition(&comp);
        let chain = trace.format_chain();
        assert!(chain.contains("N"));
        assert!(chain.contains("1"));
    }

    #[test]
    fn test_constant_tracker() {
        let mut tracker = ConstantTracker::new();
        tracker.track(&PrimitiveComposition::new(vec![LexPrimitiva::Quantity]));
        tracker.track(&PrimitiveComposition::new(vec![LexPrimitiva::Void]));
        assert!(tracker.all_grounded());
        assert_eq!(tracker.traces().len(), 2);
    }

    #[test]
    fn test_int_to_constant() {
        assert_eq!(int_to_constant(0), RootConstant::Zero);
        assert_eq!(int_to_constant(1), RootConstant::One);
        assert_eq!(int_to_constant(42), RootConstant::One);
        assert_eq!(int_to_constant(-1), RootConstant::One);
    }

    #[test]
    fn test_bool_to_constant() {
        assert_eq!(bool_to_constant(false), RootConstant::Zero);
        assert_eq!(bool_to_constant(true), RootConstant::One);
    }

    #[test]
    fn test_float_to_constant() {
        assert_eq!(float_to_constant(0.0), RootConstant::Zero);
        assert_eq!(float_to_constant(1.0), RootConstant::One);
        assert_eq!(float_to_constant(3.14), RootConstant::One);
    }

    #[test]
    fn test_string_to_constant() {
        assert_eq!(string_to_constant(""), RootConstant::Zero);
        assert_eq!(string_to_constant("hello"), RootConstant::One);
    }

    #[test]
    fn test_option_to_constant() {
        let none: Option<i32> = None;
        let some: Option<i32> = Some(42);
        assert_eq!(option_to_constant(&none), RootConstant::Zero);
        assert_eq!(option_to_constant(&some), RootConstant::One);
    }

    #[test]
    fn test_len_to_constant() {
        assert_eq!(len_to_constant(0), RootConstant::Zero);
        assert_eq!(len_to_constant(1), RootConstant::One);
        assert_eq!(len_to_constant(100), RootConstant::One);
    }

    #[test]
    fn test_grounding_report_format() {
        let comp = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Quantity]);
        let trace = GroundingTrace::from_composition(&comp);
        let report = format_grounding_report(&trace);
        assert!(report.contains("Composition:"));
        assert!(report.contains("Grounding steps:"));
        assert!(report.contains("Roots:"));
    }

    #[test]
    fn test_tracker_summary() {
        let mut tracker = ConstantTracker::new();
        tracker.track(&PrimitiveComposition::new(vec![LexPrimitiva::Quantity]));
        let summary = tracker.summary();
        assert!(summary.contains("1/1"));
    }

    #[test]
    fn test_all_primitives_are_grounded() {
        // Every primitive should ground to something
        let primitives = [
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Recursion,
            LexPrimitiva::Void,
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
            LexPrimitiva::Existence,
            LexPrimitiva::Persistence,
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Sum,
        ];

        for prim in primitives {
            let comp = PrimitiveComposition::new(vec![prim]);
            let trace = GroundingTrace::from_composition(&comp);
            assert!(trace.is_grounded(), "{} should be grounded", prim.symbol());
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", RootConstant::Zero), "0");
        assert_eq!(format!("{}", RootConstant::One), "1");
    }
}
