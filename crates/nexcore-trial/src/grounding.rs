//! Lex Primitiva grounding for TRIAL framework types.
//!
//! Tier: T2-P (Cross-domain protocol design — clinical trial methodology universalized)
//!
//! Every domain type in nexcore-trial maps to a `PrimitiveComposition` declaring
//! which T1 Lex Primitiva it decomposes to. This enables:
//! - Stoichiometric balancing across trial pipelines
//! - Cross-domain transfer scoring via tier multipliers
//! - Primitive conservation auditing (no primitive created or destroyed)
//!
//! ## TRIAL Phase → Primitive Mapping
//!
//! | Phase    | Letter | Primitives      | Rationale                                         |
//! |----------|--------|-----------------|---------------------------------------------------|
//! | Target   | T      | →, ∃, N         | Hypothesis (→ cause), instantiation (∃), quantity (N) |
//! | Regiment | R      | σ, ∂, μ         | Sequence (σ), boundary (∂ blinding), mapping (μ)  |
//! | Interim  | I      | ν, κ, ∂         | Frequency (ν looks), comparison (κ), boundary (∂) |
//! | Assay    | A      | N, κ, →         | Quantity (N), comparison (κ), causality (→)       |
//! | Lifecycle| L      | π, ν, ∝         | Persistence (π), frequency (ν), irreversibility (∝)|

use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use LexPrimitiva::{
    Boundary, Causality, Comparison, Existence, Frequency, Irreversibility, Mapping, Persistence,
    Quantity, Sequence, State,
};

// ── Phase-level grounding ────────────────────────────────────────────────

/// T — TARGET phase: hypothesis definition, population, power analysis.
/// Dominant: → (Causality) — every experiment tests a causal claim.
#[must_use]
pub fn target_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Causality, Existence, Quantity])
        .with_dominant(Causality, 0.95)
}

/// R — REGIMENT phase: randomization, blinding, arm assignment.
/// Dominant: σ (Sequence) — randomization is ordered allocation.
#[must_use]
pub fn regiment_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Sequence, Boundary, Mapping])
        .with_dominant(Sequence, 0.90)
}

/// I — INTERIM phase: interim analysis, safety monitoring, adaptation triggers.
/// Dominant: ν (Frequency) — interim looks occur at pre-specified information fractions.
#[must_use]
pub fn interim_phase_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Frequency, Comparison, Boundary])
        .with_dominant(Frequency, 0.90)
}

/// A — ASSAY phase: endpoint evaluation, multiplicity correction, effect sizes.
/// Dominant: κ (Comparison) — the assay compares treatment vs control.
#[must_use]
pub fn assay_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Quantity, Comparison, Causality])
        .with_dominant(Comparison, 0.95)
}

/// L — LIFECYCLE phase: report generation, surveillance planning, decay analysis.
/// Dominant: π (Persistence) — the lifecycle preserves evidence for posterity.
#[must_use]
pub fn lifecycle_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Persistence, Frequency, Irreversibility])
        .with_dominant(Persistence, 0.90)
}

// ── Type-level grounding ─────────────────────────────────────────────────

/// Protocol — the immutable experiment definition.
/// T2-P: →+∃+N+σ+∂ (Causal hypothesis + instantiation + quantity + sequenced phases + boundaries)
#[must_use]
pub fn protocol_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Causality, Existence, Quantity, Sequence, Boundary])
        .with_dominant(Causality, 0.95)
}

/// Endpoint — a measurable outcome.
/// T2-P: κ+N+∃ (Comparison of quantities against existence thresholds)
#[must_use]
pub fn endpoint_type_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Comparison, Quantity, Existence])
        .with_dominant(Comparison, 0.90)
}

/// Arm — a treatment or control group.
/// T2-P: ∃+∂+μ (Exists as entity, bounded by eligibility, mapped to treatment)
#[must_use]
pub fn arm_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Existence, Boundary, Mapping])
        .with_dominant(Existence, 0.85)
}

/// SafetyRule — hard stop boundary condition.
/// T2-P: ∂+N (Boundary defined by quantitative threshold)
#[must_use]
pub fn safety_rule_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Boundary, Quantity])
        .with_dominant(Boundary, 0.95)
}

/// Adaptation — pre-specified mid-trial modification.
/// T2-P: ς+→+∂ (State change triggered by causality, bounded by pre-specification)
#[must_use]
pub fn adaptation_type_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![State, Causality, Boundary])
        .with_dominant(State, 0.90)
}

/// InterimData — snapshot of accumulating evidence.
/// T2-P: N+ν+κ (Quantities at a frequency point for comparison)
#[must_use]
pub fn interim_data_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Quantity, Frequency, Comparison])
        .with_dominant(Quantity, 0.85)
}

/// InterimDecision — the decision from an interim look.
/// T2-P: ς+∂+→ (State transition at boundary, caused by evidence)
#[must_use]
pub fn interim_decision_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![State, Boundary, Causality])
        .with_dominant(State, 0.90)
}

/// EndpointResult — outcome of statistical testing.
/// T2-P: N+κ+→ (Quantitative comparison yielding causal inference)
#[must_use]
pub fn endpoint_result_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Quantity, Comparison, Causality])
        .with_dominant(Comparison, 0.95)
}

/// ArmAssignment — subject-to-arm randomization mapping.
/// T2-P: μ+σ+∂ (Mapping in sequence with blinding boundary)
#[must_use]
pub fn arm_assignment_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Mapping, Sequence, Boundary])
        .with_dominant(Mapping, 0.90)
}

/// AdjustedResult — multiplicity-corrected test result.
/// T2-P: κ+N+∂ (Comparison of quantities with adjusted boundary)
#[must_use]
pub fn adjusted_result_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Comparison, Quantity, Boundary])
        .with_dominant(Comparison, 0.90)
}

/// TrialVerdict — final determination: Positive / Negative / Inconclusive.
/// T2-P: ∝+κ+→ (Irreversible conclusion from comparison establishing causality)
#[must_use]
pub fn verdict_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Irreversibility, Comparison, Causality])
        .with_dominant(Irreversibility, 0.95)
}

/// BlindingReport — assessment of blinding integrity.
/// T2-P: ∂+κ+N (Boundary integrity measured by quantitative comparison)
#[must_use]
pub fn blinding_report_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![Boundary, Comparison, Quantity])
        .with_dominant(Boundary, 0.90)
}

// ── Composite pipeline grounding ─────────────────────────────────────────

/// Full TRIAL pipeline composition — the union of all five phases.
/// This is the "molecular formula" for a complete experiment.
///
/// T2-P: →+∃+N+σ+∂+μ+ν+κ+π+∝+ς (11 of 15 operational primitives)
///
/// Missing primitives (by design):
/// - ρ (Recursion): trials are acyclic by protocol
/// - ∅ (Void): trials must produce a verdict, never void
/// - λ (Location): trials are location-independent
/// - Σ (Sum): aggregation is internal to phases, not a top-level primitive
#[must_use]
pub fn trial_pipeline_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![
        Causality,
        Existence,
        Quantity,
        Sequence,
        Boundary,
        Mapping,
        Frequency,
        Comparison,
        Persistence,
        Irreversibility,
        State,
    ])
    .with_dominant(Causality, 0.95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_phase_has_three_primitives() {
        let comp = target_composition();
        assert_eq!(comp.primitives.len(), 3);
        assert_eq!(comp.dominant, Some(Causality));
    }

    #[test]
    fn test_all_phases_have_distinct_dominants() {
        let phases = [
            target_composition(),
            regiment_composition(),
            interim_phase_composition(),
            assay_composition(),
            lifecycle_composition(),
        ];
        let dominants: Vec<_> = phases.iter().filter_map(|p| p.dominant).collect();
        let unique: std::collections::HashSet<_> = dominants.iter().collect();
        assert_eq!(
            unique.len(),
            5,
            "Each TRIAL phase must have a unique dominant primitive"
        );
    }

    #[test]
    fn test_pipeline_covers_eleven_primitives() {
        let comp = trial_pipeline_composition();
        assert_eq!(comp.primitives.len(), 11);
        assert!(comp.confidence > 0.90);
    }

    #[test]
    fn test_safety_rule_dominant_is_boundary() {
        let comp = safety_rule_composition();
        assert_eq!(comp.dominant, Some(Boundary));
    }

    #[test]
    fn test_verdict_is_irreversible() {
        let comp = verdict_composition();
        assert_eq!(comp.dominant, Some(Irreversibility));
    }

    #[test]
    fn test_protocol_composition_includes_causality_and_existence() {
        let comp = protocol_composition();
        assert!(comp.primitives.contains(&Causality));
        assert!(comp.primitives.contains(&Existence));
        assert!(comp.primitives.contains(&Quantity));
    }
}
