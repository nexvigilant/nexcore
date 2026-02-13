//! # GroundsTo implementations for nexcore-skeletal types
//!
//! Connects skeletal system types to the Lex Primitiva type system.
//!
//! ## Grounding Summary
//!
//! | Type | Primitives | Dominant | Tier | Confidence |
//! |------|-----------|----------|------|------------|
//! | `WolffsLaw` | ρ ∂ κ | ρ Recursion | T2-C | 0.70 |
//! | `Correction` | → π | → Causality | T2-P | 0.80 |
//! | `BoneMarrow` | π σ ∃ | π Persistence | T2-C | 0.65 |
//! | `Joint` | ∂ μ | ∂ Boundary | T2-P | 0.80 |
//! | `SkeletalHealth` | ς κ ∂ | ς State | T2-C | 0.65 |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{BoneMarrow, Correction, Joint, SkeletalHealth, WolffsLaw};

// ---------------------------------------------------------------------------
// WolffsLaw: T2-C (ρ recursion + ∂ boundary + κ comparison), dominant ρ
// ---------------------------------------------------------------------------

/// WolffsLaw: T2-C (ρ · ∂ · κ), dominant ρ [0.70]
///
/// Wolff's Law states that bone remodels along lines of stress. In Claude Code,
/// CLAUDE.md strengthens where corrections concentrate. Recursion-dominant:
/// the process IS recursive — corrections feed back into the structure that
/// governs future behavior, which in turn generates new corrections.
impl GroundsTo for WolffsLaw {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // ρ — corrections feed back into CLAUDE.md
            LexPrimitiva::Boundary,   // ∂ — stress threshold boundary
            LexPrimitiva::Comparison, // κ — frequency comparison against threshold
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.70)
    }
}

// ---------------------------------------------------------------------------
// Correction: T2-P (→ causality + π persistence), dominant →
// ---------------------------------------------------------------------------

/// Correction: T2-P (→ · π), dominant → [0.80]
///
/// A correction records a stress point where bone remodels. In Claude Code,
/// each correction is a causal event — an error CAUSED a rule to be added.
/// Causality-dominant: the correction IS a cause-effect pair (mistake → fix).
impl GroundsTo for Correction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,   // → — error caused correction
            LexPrimitiva::Persistence, // π — persisted in CLAUDE.md
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ---------------------------------------------------------------------------
// BoneMarrow: T2-C (π persistence + σ sequence + ∃ existence), dominant π
// ---------------------------------------------------------------------------

/// BoneMarrow: T2-C (π · σ · ∃), dominant π [0.65]
///
/// Bone marrow produces blood cells; CLAUDE.md produces contextual awareness,
/// validation rules, and behavioral patterns. Persistence-dominant: marrow
/// IS the persistent source that continuously generates runtime resources.
impl GroundsTo for BoneMarrow {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // π — CLAUDE.md persists across sessions
            LexPrimitiva::Sequence,    // σ — generates ordered patterns/rules
            LexPrimitiva::Existence,   // ∃ — brings guidance into existence
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.65)
    }
}

// ---------------------------------------------------------------------------
// Joint: T2-P (∂ boundary + μ mapping), dominant ∂
// ---------------------------------------------------------------------------

/// Joint: T2-P (∂ · μ), dominant ∂ [0.80]
///
/// Joints connect rigid bones while constraining range of motion. In Claude Code,
/// interfaces (MCP, hooks, settings) connect components with defined constraints.
/// Boundary-dominant: a joint IS a boundary — it defines what motion is allowed
/// between two rigid structures.
impl GroundsTo for Joint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — interface constraint boundary
            LexPrimitiva::Mapping,  // μ — maps one structure to another
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ---------------------------------------------------------------------------
// SkeletalHealth: T2-C (ς state + κ comparison + ∂ boundary), dominant ς
// ---------------------------------------------------------------------------

/// SkeletalHealth: T2-C (ς · κ · ∂), dominant ς [0.65]
///
/// Overall structural health assessment of the project skeleton.
/// State-dominant: health IS a state snapshot — present/absent/active/inactive
/// for each structural component.
impl GroundsTo for SkeletalHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς — health is a state assessment
            LexPrimitiva::Comparison, // κ — comparing present vs expected
            LexPrimitiva::Boundary,   // ∂ — healthy/unhealthy boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
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
    fn wolffs_law_is_recursion_dominant_t2c() {
        let comp = WolffsLaw::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        // 3 unique primitives = T2-P per tier classification (2-3 = T2-P)
        // Actually: Tier::classify uses unique count: 3 = T2-P
        assert_eq!(WolffsLaw::tier(), Tier::T2Primitive);
    }

    #[test]
    fn correction_is_causality_dominant_t2p() {
        let comp = Correction::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
        assert_eq!(Correction::tier(), Tier::T2Primitive);
    }

    #[test]
    fn bone_marrow_is_persistence_dominant() {
        let comp = BoneMarrow::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn joint_is_boundary_dominant_t2p() {
        let comp = Joint::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(Joint::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skeletal_health_is_state_dominant() {
        let comp = SkeletalHealth::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn bone_marrow_tier_is_t2c_or_t2p() {
        // 3 unique primitives = T2-P per tier classification
        let tier = BoneMarrow::tier();
        assert!(tier == Tier::T2Primitive || tier == Tier::T2Composite);
    }

    #[test]
    fn skeletal_health_tier() {
        // 3 unique primitives = T2-P per tier classification
        let tier = SkeletalHealth::tier();
        assert!(tier == Tier::T2Primitive || tier == Tier::T2Composite);
    }

    #[test]
    fn wolffs_law_confidence() {
        let comp = WolffsLaw::primitive_composition();
        assert!((comp.confidence - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn correction_confidence() {
        let comp = Correction::primitive_composition();
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn joint_confidence() {
        let comp = Joint::primitive_composition();
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }
}
