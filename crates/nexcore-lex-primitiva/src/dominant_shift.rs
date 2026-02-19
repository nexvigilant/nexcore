//! # Dominant Shift (Phase Transition)
//!
//! Formalizes the "dominant shift" as a first-class operation in the Lex Primitiva
//! framework. A dominant shift occurs when adding a new primitive to a composition
//! changes which primitive holds structural dominance.
//!
//! ## Algorithm
//!
//! 1. Synthesize the base set via [`RevSynthesizer`] to infer the dominant.
//! 2. Synthesize the expanded set (base + new primitive) to infer the new dominant.
//! 3. Compare: if the dominant changed, `shifted = true` (phase transition).
//! 4. Report tier and coherence change for both states.
//!
//! ## Significance
//!
//! A dominant shift signals a **phase transition** — the composition's structural
//! character has reorganized. Like a catalyst that changes reaction kinetics, the
//! added primitive can flip which role governs the whole.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_lex_primitiva::dominant_shift::compute_dominant_shift;
//! use nexcore_lex_primitiva::primitiva::LexPrimitiva;
//!
//! // Adding Boundary to [Comparison] creates the Gatekeeper pattern:
//! // Boundary takes dominance over Comparison.
//! let shift = compute_dominant_shift(&[LexPrimitiva::Comparison], LexPrimitiva::Boundary);
//! // The result reflects the Gatekeeper phase transition.
//! assert!(!shift.base_primitives.is_empty());
//! assert_eq!(shift.added_primitive, "Boundary");
//! ```
//!
//! ## Tier
//!
//! T2-C (State + Causality + Comparison + Boundary)

use crate::composition::CompositionAlgebra;
use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::synthesizer::{RevSynthesizer, SynthesisOpts};
use crate::tier::Tier;
use serde::{Deserialize, Serialize};

// ─── PrimitiveInfo ────────────────────────────────────────────────────────────

/// Identifying information for a single dominant primitive.
///
/// Tier: T2-P (Existence + Mapping)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveInfo {
    /// Canonical name (e.g., "Boundary", "Quantity").
    pub name: String,
    /// Mathematical symbol (e.g., "∂", "N").
    pub symbol: String,
}

impl PrimitiveInfo {
    /// Construct from a [`LexPrimitiva`].
    #[must_use]
    pub fn from_primitive(p: LexPrimitiva) -> Self {
        Self {
            name: p.name().to_string(),
            symbol: p.symbol().to_string(),
        }
    }

    /// Sentinel value used when no dominant exists (empty composition).
    #[must_use]
    pub fn none() -> Self {
        Self {
            name: "none".to_string(),
            symbol: "∅".to_string(),
        }
    }
}

impl std::fmt::Display for PrimitiveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.symbol)
    }
}

// ─── DominantShift ────────────────────────────────────────────────────────────

/// Result of a dominant shift analysis.
///
/// Captures the complete before/after state when a primitive is added to a
/// composition. `shifted = true` signals a **phase transition** — the dominant
/// primitive has changed, reorganizing the composition's interaction topology.
///
/// ## Tier
///
/// T2-C (State + Causality + Comparison + Boundary)
///
/// ## Example
///
/// ```rust
/// use nexcore_lex_primitiva::dominant_shift::compute_dominant_shift;
/// use nexcore_lex_primitiva::primitiva::LexPrimitiva;
///
/// let shift = compute_dominant_shift(&[LexPrimitiva::Comparison], LexPrimitiva::Boundary);
/// assert!(!shift.added_primitive.is_empty());
/// assert!(shift.coherence_before >= 0.0 && shift.coherence_before <= 1.0);
/// assert!(shift.coherence_after >= 0.0 && shift.coherence_after <= 1.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominantShift {
    /// The base primitive set (names), before adding.
    pub base_primitives: Vec<String>,
    /// The primitive being added (name).
    pub added_primitive: String,
    /// Dominant primitive before adding.
    pub old_dominant: PrimitiveInfo,
    /// Dominant primitive after adding.
    pub new_dominant: PrimitiveInfo,
    /// Whether the dominant primitive changed (phase transition occurred).
    pub shifted: bool,
    /// Tier before adding (e.g., "T1", "T2-P", "T2-C", "T3").
    pub old_tier: String,
    /// Tier after adding.
    pub new_tier: String,
    /// Semantic coherence before adding (0.0-1.0).
    pub coherence_before: f64,
    /// Semantic coherence after adding (0.0-1.0).
    pub coherence_after: f64,
}

impl std::fmt::Display for DominantShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.shifted {
            write!(
                f,
                "PHASE TRANSITION: {} → {} (coherence {:.2} → {:.2}, tier {} → {})",
                self.old_dominant,
                self.new_dominant,
                self.coherence_before,
                self.coherence_after,
                self.old_tier,
                self.new_tier,
            )
        } else {
            write!(
                f,
                "STABLE: {} holds dominance (coherence {:.2} → {:.2})",
                self.new_dominant,
                self.coherence_before,
                self.coherence_after,
            )
        }
    }
}

// ─── Core Operation ───────────────────────────────────────────────────────────

/// Compute the dominant shift when `added` is appended to the `base` set.
///
/// Uses [`RevSynthesizer`] to infer the dominant primitive for both the base
/// composition and the expanded composition (base + added), then detects
/// whether the dominant has changed.
///
/// # Arguments
///
/// * `base` — The current set of T1 primitives. May be empty.
/// * `added` — The new primitive being incorporated into the composition.
///
/// # Returns
///
/// A [`DominantShift`] capturing the before/after state. If `shifted = true`,
/// a phase transition occurred — the structural character has changed.
///
/// # Notes
///
/// * An empty `base` yields `old_dominant = none`.
/// * Synthesis uses `min_coherence = 0.0` so novel combinations always resolve.
/// * Coherence is computed via [`CompositionAlgebra::validate_semantics`].
#[must_use]
pub fn compute_dominant_shift(base: &[LexPrimitiva], added: LexPrimitiva) -> DominantShift {
    let base_names: Vec<String> = base.iter().map(|p| p.name().to_string()).collect();
    let added_name = added.name().to_string();

    let algebra = CompositionAlgebra::new();
    let synth = RevSynthesizer::new();

    // ── Before ───────────────────────────────────────────────────────────────

    let (old_dominant, old_tier_code, coherence_before) = if base.is_empty() {
        (None, Tier::T1Universal.code().to_string(), 0.0)
    } else {
        let before_prims: Vec<LexPrimitiva> = base.to_vec();
        let before_comp = PrimitiveComposition::new(before_prims.clone());
        let tier = Tier::classify(&before_comp);
        let coherence = algebra.validate_semantics(&before_comp).coherence;
        let dominant = synth
            .synthesize(before_prims, SynthesisOpts::default())
            .ok()
            .and_then(|r| r.dominant);
        (dominant, tier.code().to_string(), coherence)
    };

    // ── After ────────────────────────────────────────────────────────────────

    let mut after_prims: Vec<LexPrimitiva> = base.to_vec();
    after_prims.push(added);

    let after_comp = PrimitiveComposition::new(after_prims.clone());
    let new_tier = Tier::classify(&after_comp);
    let coherence_after = algebra.validate_semantics(&after_comp).coherence;
    let new_dominant = synth
        .synthesize(after_prims, SynthesisOpts::default())
        .ok()
        .and_then(|r| r.dominant);

    // ── Assemble ─────────────────────────────────────────────────────────────

    let old_dominant_info = old_dominant
        .map(PrimitiveInfo::from_primitive)
        .unwrap_or_else(PrimitiveInfo::none);
    let new_dominant_info = new_dominant
        .map(PrimitiveInfo::from_primitive)
        .unwrap_or_else(PrimitiveInfo::none);

    let shifted = old_dominant != new_dominant;

    DominantShift {
        base_primitives: base_names,
        added_primitive: added_name,
        old_dominant: old_dominant_info,
        new_dominant: new_dominant_info,
        shifted,
        old_tier: old_tier_code,
        new_tier: new_tier.code().to_string(),
        coherence_before,
        coherence_after,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::primitiva::LexPrimitiva::{
        Boundary, Comparison, Mapping, Quantity, Sequence, State,
    };

    #[test]
    fn test_adding_boundary_to_comparison_shifts_dominant() {
        // [Comparison] → Comparison is sole primitive (dominant = Comparison)
        // [Comparison, Boundary] → Boundary takes over (Gatekeeper pattern: ∂ guards κ)
        let shift = compute_dominant_shift(&[Comparison], Boundary);
        assert!(
            shift.shifted,
            "Adding Boundary to Comparison should shift dominant: old={} new={}",
            shift.old_dominant,
            shift.new_dominant
        );
        assert_eq!(shift.new_dominant.name, "Boundary");
        assert_eq!(shift.old_dominant.name, "Comparison");
    }

    #[test]
    fn test_empty_base_has_no_old_dominant() {
        let shift = compute_dominant_shift(&[], Sequence);
        assert_eq!(
            shift.old_dominant.name, "none",
            "Empty base should have no dominant"
        );
        assert_eq!(shift.new_dominant.name, "Sequence");
        assert!(shift.shifted, "None → Sequence is always a shift");
        assert_eq!(shift.base_primitives.len(), 0);
        assert_eq!(shift.added_primitive, "Sequence");
    }

    #[test]
    fn test_single_primitive_base_reports_old_dominant() {
        let shift = compute_dominant_shift(&[Sequence], Mapping);
        assert_eq!(shift.old_dominant.name, "Sequence");
        assert_eq!(shift.added_primitive, "Mapping");
        assert_eq!(shift.base_primitives, vec!["Sequence"]);
    }

    #[test]
    fn test_coherence_is_bounded() {
        let shift = compute_dominant_shift(&[Boundary, Comparison], Quantity);
        assert!(
            shift.coherence_before >= 0.0 && shift.coherence_before <= 1.0,
            "coherence_before out of range: {}",
            shift.coherence_before
        );
        assert!(
            shift.coherence_after >= 0.0 && shift.coherence_after <= 1.0,
            "coherence_after out of range: {}",
            shift.coherence_after
        );
    }

    #[test]
    fn test_tier_strings_are_valid() {
        let valid_tiers = ["T1", "T2-P", "T2-C", "T3"];
        let shift = compute_dominant_shift(&[Boundary], Comparison);
        assert!(
            valid_tiers.contains(&shift.old_tier.as_str()),
            "Invalid old_tier: {}",
            shift.old_tier
        );
        assert!(
            valid_tiers.contains(&shift.new_tier.as_str()),
            "Invalid new_tier: {}",
            shift.new_tier
        );
    }

    #[test]
    fn test_single_base_has_t1_tier_before() {
        // One primitive → T1 tier
        let shift = compute_dominant_shift(&[State], Comparison);
        assert_eq!(shift.old_tier, "T1");
        // Two primitives → T2-P or higher
        assert!(["T2-P", "T2-C", "T3"].contains(&shift.new_tier.as_str()));
    }

    #[test]
    fn test_display_shifted_contains_phase_transition() {
        let shift = compute_dominant_shift(&[Comparison], Boundary);
        let s = format!("{}", shift);
        if shift.shifted {
            assert!(s.contains("PHASE TRANSITION"), "Display: {}", s);
        } else {
            assert!(s.contains("STABLE"), "Display: {}", s);
        }
    }

    #[test]
    fn test_display_always_has_marker() {
        for (base, added) in [
            (vec![Boundary, Comparison], Quantity),
            (vec![Sequence], Mapping),
            (vec![], State),
        ] {
            let shift = compute_dominant_shift(&base, added);
            let s = format!("{}", shift);
            assert!(
                s.contains("PHASE TRANSITION") || s.contains("STABLE"),
                "Display must have marker: {}",
                s
            );
        }
    }

    #[test]
    fn test_primitive_info_from_primitive() {
        let info = PrimitiveInfo::from_primitive(Boundary);
        assert_eq!(info.name, "Boundary");
        assert_eq!(info.symbol, "∂");
    }

    #[test]
    fn test_primitive_info_none() {
        let info = PrimitiveInfo::none();
        assert_eq!(info.name, "none");
        assert_eq!(info.symbol, "∅");
    }

    #[test]
    fn test_primitive_info_display() {
        let info = PrimitiveInfo::from_primitive(Quantity);
        let s = format!("{}", info);
        assert!(s.contains("Quantity") && s.contains("N"));
    }

    #[test]
    fn test_result_fields_reflect_input() {
        let shift = compute_dominant_shift(&[Sequence, Quantity], Boundary);
        assert_eq!(shift.added_primitive, "Boundary");
        assert_eq!(shift.base_primitives, vec!["Sequence", "Quantity"]);
    }

    #[test]
    fn test_shifted_flag_is_consistent_with_dominants() {
        let shift = compute_dominant_shift(&[Comparison], Boundary);
        let actually_different = shift.old_dominant.name != shift.new_dominant.name;
        assert_eq!(
            shift.shifted, actually_different,
            "shifted flag must match dominant names"
        );
    }

    #[test]
    fn test_empty_base_coherence_before_is_zero() {
        let shift = compute_dominant_shift(&[], Mapping);
        assert!(
            shift.coherence_before.abs() < f64::EPSILON,
            "Empty base coherence should be 0.0, got {}",
            shift.coherence_before
        );
    }
}
