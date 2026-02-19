//! # Meta-Level Grounded Types
//!
//! Types that represent meta-level concepts intrinsic to the lex-primitiva system:
//! capabilities, tools, and the building blocks that operate ON the primitive fabric
//! itself.
//!
//! These ground in the foundation crate because they describe the infrastructure
//! through which primitives are invoked and composed — not domain-specific behaviour
//! that belongs in an upper layer.
//!
//! ## Tier: T2-C (multi-domain composite)

#![forbid(unsafe_code)]

use crate::grounding::GroundsTo;
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// TOOL — μ (Mapping) dominant T2-C
// ═══════════════════════════════════════════════════════════════════════════════

/// A generic capability tool — maps input arguments to an observable output effect.
///
/// Grounds the MCP tool concept to its T1 primitive composition:
///
/// | Primitive | Role |
/// |-----------|------|
/// | μ (Mapping, dominant) | transforms inputs to outputs |
/// | → (Causality) | invocation causes a causal effect |
/// | ∂ (Boundary) | input schema + output contract delimit the tool |
/// | ∃ (Existence) | tool must be registered/present to be called |
/// | σ (Sequence) | tools compose sequentially in workflows |
///
/// ## Tier
///
/// **T2-C** — multi-domain composite. Tools span every domain (PV, Brain, Guardian,
/// FAERS, Vigilance), making this a cross-domain concept that transfers broadly
/// but not universally.
///
/// ## Coherence
///
/// ~0.32 — five distinct primitives with moderate coupling. The dominant μ is
/// clear, but the secondary role of → vs ∂ vs ∃ vs σ creates legitimate primitive
/// competition that lowers coherence below the T2-P baseline.
///
/// ## Domain Mappings
///
/// - **MCP**: a `#[tool]` function registered in `nexcore-mcp`
/// - **Brain**: an artifact-resolution skill invoked with structured arguments
/// - **Guardian**: a homeostasis actuator action dispatched on a threat signal
/// - **FAERS**: a parameterised query against the adverse event database
/// - **Vigil**: a capability exposed to the orchestrator's decision engine
///
/// ## Examples
///
/// ```rust
/// use nexcore_lex_primitiva::meta_types::Tool;
/// use nexcore_lex_primitiva::grounding::GroundsTo;
/// use nexcore_lex_primitiva::primitiva::LexPrimitiva;
///
/// let t = Tool::new("sha256", "foundation");
/// assert_eq!(t.qualified_name(), "foundation__sha256");
/// assert_eq!(Tool::dominant_primitive(), Some(LexPrimitiva::Mapping));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tool {
    /// Short name of the tool (e.g., `"sha256"`, `"signal_complete"`).
    pub name: String,
    /// Namespace / domain this tool belongs to (e.g., `"foundation"`, `"pv"`, `"faers"`).
    pub namespace: String,
}

impl Tool {
    /// Creates a new `Tool` with the given name and namespace.
    #[must_use]
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: namespace.into(),
        }
    }

    /// Returns the fully qualified tool identifier (used in MCP dispatch tables).
    ///
    /// Format: `{namespace}__{name}`
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}__{}", self.namespace, self.name)
    }
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.namespace, self.name)
    }
}

impl GroundsTo for Tool {
    fn primitive_composition() -> PrimitiveComposition {
        // μ (Mapping, dominant 0.80): tool transforms input → output
        // → (Causality): invocation causes effects in the system
        // ∂ (Boundary): input schema + output contract bound the tool
        // ∃ (Existence): tool must be registered to be callable
        // σ (Sequence): tools compose sequentially in workflows
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // primary: I/O transformation
            LexPrimitiva::Causality,  // invocation causes downstream effects
            LexPrimitiva::Boundary,   // contract defines valid inputs/outputs
            LexPrimitiva::Existence,  // registration precondition
            LexPrimitiva::Sequence,   // workflow composability
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::tier::Tier;

    #[test]
    fn tool_grounding_dominant_is_mapping() {
        let comp = Tool::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn tool_grounding_has_five_primitives() {
        let comp = Tool::primitive_composition();
        assert_eq!(comp.primitives.len(), 5);
    }

    #[test]
    fn tool_grounding_contains_required_primitives() {
        let comp = Tool::primitive_composition();
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Mapping));
        assert!(unique.contains(&LexPrimitiva::Causality));
        assert!(unique.contains(&LexPrimitiva::Boundary));
        assert!(unique.contains(&LexPrimitiva::Existence));
        assert!(unique.contains(&LexPrimitiva::Sequence));
    }

    #[test]
    fn tool_is_t2_composite() {
        assert_eq!(Tool::tier(), Tier::T2Composite);
    }

    #[test]
    fn tool_qualified_name() {
        let t = Tool::new("sha256", "foundation");
        assert_eq!(t.qualified_name(), "foundation__sha256");
    }

    #[test]
    fn tool_display() {
        let t = Tool::new("signal_complete", "pv");
        assert_eq!(format!("{t}"), "pv::signal_complete");
    }

    #[test]
    fn tool_confidence_is_0_80() {
        let comp = Tool::primitive_composition();
        assert!(
            (comp.confidence - 0.80).abs() < f64::EPSILON,
            "expected confidence 0.80, got {}",
            comp.confidence
        );
    }

    #[test]
    fn tool_no_state_mode() {
        let comp = Tool::primitive_composition();
        assert!(comp.state_mode.is_none());
    }
}
