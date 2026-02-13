/// Patient safety integration for the trust engine.
///
/// Bridges clinical pharmacovigilance scoring systems into Bayesian trust:
/// - **ICH E2A** seriousness criteria map to harm severity levels
/// - **WHO-UMC** causality categories weight evidence confidence
/// - **Naranjo** ADR probability scores convert to causality assessments
/// - **P0 Priority**: Patient safety violations amplify negative evidence
///   proportional to harm severity and causality confidence
///
/// # Mathematical Model
///
/// Harm-adjusted evidence weight:
/// ```text
/// effective_weight = base_weight * severity_multiplier * causality_weight
/// ```
///
/// Where `severity_multiplier` ranges from 1.0 (non-serious) to 5.0 (death)
/// and `causality_weight` ranges from 0.1 (unassessable) to 1.0 (certain).
///
/// # Conservation Principle
///
/// From the STEM anti-harm framework: harm evidence creates "memory" that
/// resists temporal decay. More serious harms create stronger cementation,
/// ensuring the system never forgets a lethal event as quickly as a minor one.
///
/// Tier: T3 (∂ Boundary + → Causality + κ Comparison + N Quantity + ∝ Irreversibility + ς State)
use crate::engine::{TrustConfig, TrustEngine};
use crate::evidence::Evidence;
use crate::level::LevelThresholds;

/// ICH E2A seriousness categories for adverse events.
///
/// Ordered by increasing severity. Each level maps to a trust evidence
/// multiplier that amplifies the negative impact of harm-related violations.
///
/// The P0 Patient Safety priority requires that trust decisions involving
/// patient harm are always amplified — never discounted or ignored.
///
/// Tier: T2-P (∂ Boundary + κ Comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HarmSeverity {
    /// No serious criteria met. Standard trust impact.
    NonSerious,
    /// Medically significant but not meeting other serious criteria.
    OtherSerious,
    /// Results in persistent or significant disability/incapacity.
    Disability,
    /// Congenital anomaly/birth defect.
    CongenitalAnomaly,
    /// Requires or prolongs hospitalization.
    Hospitalization,
    /// Immediate risk of death.
    LifeThreatening,
    /// Results in death.
    Death,
}

impl HarmSeverity {
    /// All severity levels in ascending order.
    pub const ALL: [HarmSeverity; 7] = [
        HarmSeverity::NonSerious,
        HarmSeverity::OtherSerious,
        HarmSeverity::Disability,
        HarmSeverity::CongenitalAnomaly,
        HarmSeverity::Hospitalization,
        HarmSeverity::LifeThreatening,
        HarmSeverity::Death,
    ];

    /// Multiplier for negative trust evidence based on harm severity.
    ///
    /// P0 Patient Safety: higher harm = stronger trust impact.
    /// A death-related violation carries 5x the base trust impact,
    /// ensuring the system treats lethal outcomes with maximum weight.
    pub fn trust_multiplier(self) -> f64 {
        match self {
            Self::NonSerious => 1.0,
            Self::OtherSerious => 1.5,
            Self::Disability => 2.0,
            Self::CongenitalAnomaly => 2.5,
            Self::Hospitalization => 3.0,
            Self::LifeThreatening => 4.0,
            Self::Death => 5.0,
        }
    }

    /// Additional cement factor for harm-related violations.
    ///
    /// Serious harms create stronger "memory" that resists temporal decay.
    /// This implements the conservation principle: the system cannot forget
    /// serious safety events as quickly as routine interactions.
    ///
    /// Applied additively to the engine's base cement_factor.
    pub fn cement_bonus(self) -> f64 {
        match self {
            Self::NonSerious => 0.0,
            Self::OtherSerious => 0.02,
            Self::Disability => 0.05,
            Self::CongenitalAnomaly => 0.05,
            Self::Hospitalization => 0.08,
            Self::LifeThreatening => 0.12,
            Self::Death => 0.20,
        }
    }

    /// Whether this severity qualifies as "serious" per ICH E2A.
    ///
    /// Serious adverse events trigger enhanced reporting requirements,
    /// and in the trust context, activate harm memory cementation.
    pub fn is_serious(self) -> bool {
        !matches!(self, Self::NonSerious)
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::NonSerious => "Non-serious",
            Self::OtherSerious => "Other Serious",
            Self::Disability => "Disability",
            Self::CongenitalAnomaly => "Congenital Anomaly",
            Self::Hospitalization => "Hospitalization",
            Self::LifeThreatening => "Life-threatening",
            Self::Death => "Death",
        }
    }
}

impl core::fmt::Display for HarmSeverity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// WHO-UMC causality assessment categories.
///
/// Determines how confidently a causal link is established between
/// an entity's action and the observed outcome. Higher causality
/// confidence produces stronger evidence weight in the trust model.
///
/// Based on the WHO-UMC system used globally for pharmacovigilance
/// signal assessment, adapted for general trust evidence weighting.
///
/// Tier: T2-P (→ Causality + N Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CausalityAssessment {
    /// Insufficient information or contradictory data.
    /// Minimal evidence weight (0.1) — barely registers.
    Unassessable,
    /// Time relationship unlikely, or plausible alternative explanation.
    /// Low evidence weight (0.25).
    Unlikely,
    /// Reasonable time sequence, but alternative causes exist.
    /// Moderate evidence weight (0.5).
    Possible,
    /// Reasonable time sequence, unlikely from other causes.
    /// Dechallenge provides supportive information.
    /// High evidence weight (0.8).
    Probable,
    /// Plausible time relationship, cannot be explained by other causes.
    /// Rechallenge positive (when applicable).
    /// Full evidence weight (1.0).
    Certain,
}

impl CausalityAssessment {
    /// All categories in ascending confidence order.
    pub const ALL: [CausalityAssessment; 5] = [
        CausalityAssessment::Unassessable,
        CausalityAssessment::Unlikely,
        CausalityAssessment::Possible,
        CausalityAssessment::Probable,
        CausalityAssessment::Certain,
    ];

    /// Confidence weight for evidence based on causality assessment.
    ///
    /// Scales the evidence signal proportional to causal confidence.
    /// This prevents uncertain causal links from having the same trust
    /// impact as definitively established ones (addresses Base Rate Neglect).
    pub fn evidence_weight(self) -> f64 {
        match self {
            Self::Unassessable => 0.1,
            Self::Unlikely => 0.25,
            Self::Possible => 0.5,
            Self::Probable => 0.8,
            Self::Certain => 1.0,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unassessable => "Unassessable",
            Self::Unlikely => "Unlikely",
            Self::Possible => "Possible",
            Self::Probable => "Probable",
            Self::Certain => "Certain",
        }
    }
}

impl core::fmt::Display for CausalityAssessment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Safety-aware trust configuration preset.
///
/// Preconfigured for patient safety contexts with:
/// - **Strict thresholds**: Higher bar for "Trusted" (0.7 instead of 0.6)
/// - **High asymmetry** (5.0): Safety violations hit very hard
/// - **Score ceiling** (0.95): Never fully trust in safety contexts
/// - **Diminishing returns**: Prevent complacency from accumulated good history
/// - **Cement factor**: Baseline harm memory
/// - **Slow decay**: Longer memory for safety-relevant signals
/// - **Higher significance**: Need more evidence before trusting
///
/// This implements the P0 Patient Safety priority: the default
/// configuration is conservative, requiring strong evidence to trust
/// and preserving memory of safety-related violations.
pub fn safety_config() -> TrustConfig {
    TrustConfig {
        prior_alpha: 1.0,
        prior_beta: 1.0,
        asymmetry_factor: 5.0,
        decay_rate: 0.005,
        significance_threshold: 10.0,
        thresholds: LevelThresholds::strict(),
        score_floor: 0.05,
        score_ceiling: 0.95,
        diminishing_factor: 0.05,
        cement_factor: 0.05,
    }
}

/// Create a safety-configured trust engine.
///
/// Shorthand for `TrustEngine::with_config(safety_config())`.
pub fn safety_engine() -> TrustEngine {
    TrustEngine::with_config(safety_config())
}

/// Compute harm-adjusted evidence weight.
///
/// Combines base weight, harm severity multiplier, and causality
/// assessment confidence into a single effective weight.
///
/// ```text
/// effective = base_weight * severity_multiplier * causality_weight
/// ```
///
/// This implements the P0 Patient Safety priority: trust decisions
/// involving patient harm are amplified proportionally to severity
/// and causality confidence. A certain death-related violation has
/// 5.0x the impact of a non-serious event.
///
/// # Examples
///
/// ```text
/// Non-serious + Possible:  1.0 * 1.0 * 0.5 = 0.5
/// Death + Certain:          1.0 * 5.0 * 1.0 = 5.0
/// Hospitalization + Probable: 1.0 * 3.0 * 0.8 = 2.4
/// ```
pub fn harm_adjusted_weight(
    base_weight: f64,
    severity: HarmSeverity,
    causality: CausalityAssessment,
) -> f64 {
    base_weight.max(0.0) * severity.trust_multiplier() * causality.evidence_weight()
}

/// Record harm-related negative evidence into a trust engine.
///
/// Combines harm severity amplification with causality assessment
/// to determine the effective negative evidence weight, then records
/// it as negative evidence in the engine.
///
/// For serious harms, the negative evidence is **always** recorded
/// regardless of causality — even "Unassessable" events with Death
/// severity produce non-zero negative evidence (5.0 * 0.1 = 0.5).
/// This is the P0 guarantee: no patient death is ignored.
pub fn record_harm_evidence(
    engine: &mut TrustEngine,
    base_weight: f64,
    severity: HarmSeverity,
    causality: CausalityAssessment,
) {
    let weight = harm_adjusted_weight(base_weight, severity, causality);
    if weight > 0.0 {
        engine.record(Evidence::negative_weighted(weight));
    }
}

/// Naranjo adverse drug reaction probability score to causality assessment.
///
/// Maps the 10-question Naranjo algorithm total score (-4 to +13)
/// to a [`CausalityAssessment`] category:
///
/// | Score Range | Category | Interpretation |
/// |-------------|----------|----------------|
/// | >= 9        | Certain  | Definite ADR   |
/// | 5 to 8      | Probable | Likely ADR     |
/// | 1 to 4      | Possible | Potential ADR  |
/// | <= 0        | Unlikely | Doubtful ADR   |
///
/// This bridges clinical pharmacovigilance scoring directly into
/// the trust evidence weighting system. Clinicians scoring a Naranjo
/// of 7 ("Probable") automatically produce 0.8x weighted evidence.
pub fn naranjo_to_causality(score: i32) -> CausalityAssessment {
    if score >= 9 {
        CausalityAssessment::Certain
    } else if score >= 5 {
        CausalityAssessment::Probable
    } else if score >= 1 {
        CausalityAssessment::Possible
    } else {
        CausalityAssessment::Unlikely
    }
}

/// WHO-UMC causality string to assessment category.
///
/// Accepts common WHO-UMC terminology (case-insensitive first match):
/// - "certain" → Certain
/// - "probable" / "likely" → Probable
/// - "possible" → Possible
/// - "unlikely" → Unlikely
/// - "conditional" / "unclassified" → Unassessable
/// - "unassessable" / "unclassifiable" → Unassessable
///
/// Returns `None` for unrecognized strings.
pub fn who_umc_to_causality(assessment: &str) -> Option<CausalityAssessment> {
    let lower = assessment.to_lowercase();
    if lower.starts_with("certain") {
        Some(CausalityAssessment::Certain)
    } else if lower.starts_with("probable") || lower.starts_with("likely") {
        Some(CausalityAssessment::Probable)
    } else if lower.starts_with("possible") {
        Some(CausalityAssessment::Possible)
    } else if lower.starts_with("unlikely") {
        Some(CausalityAssessment::Unlikely)
    } else if lower.starts_with("conditional")
        || lower.starts_with("unclassif")
        || lower.starts_with("unassess")
    {
        Some(CausalityAssessment::Unassessable)
    } else {
        None
    }
}

/// Minimum impact floor for serious harm events.
///
/// Ensures that any event meeting ICH E2A "serious" criteria
/// produces at least this much negative evidence weight,
/// regardless of how low the causality confidence is.
///
/// This is the P0 safety guarantee: serious harms are never
/// completely discounted even when causality is uncertain.
///
/// Returns the max of the harm-adjusted weight and the floor.
pub fn serious_harm_floor(
    base_weight: f64,
    severity: HarmSeverity,
    causality: CausalityAssessment,
    floor: f64,
) -> f64 {
    let adjusted = harm_adjusted_weight(base_weight, severity, causality);
    if severity.is_serious() {
        adjusted.max(floor.max(0.0))
    } else {
        adjusted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    // --- HarmSeverity ---

    #[test]
    fn severity_ordering() {
        assert!(HarmSeverity::NonSerious < HarmSeverity::OtherSerious);
        assert!(HarmSeverity::OtherSerious < HarmSeverity::Disability);
        assert!(HarmSeverity::Disability < HarmSeverity::CongenitalAnomaly);
        assert!(HarmSeverity::CongenitalAnomaly < HarmSeverity::Hospitalization);
        assert!(HarmSeverity::Hospitalization < HarmSeverity::LifeThreatening);
        assert!(HarmSeverity::LifeThreatening < HarmSeverity::Death);
    }

    #[test]
    fn severity_multipliers_monotonically_increase() {
        let mults: Vec<f64> = HarmSeverity::ALL
            .iter()
            .map(|s| s.trust_multiplier())
            .collect();
        for pair in mults.windows(2) {
            assert!(
                pair[1] > pair[0],
                "multiplier should increase: {} -> {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn severity_cement_bonus_non_negative() {
        for sev in HarmSeverity::ALL {
            assert!(
                sev.cement_bonus() >= 0.0,
                "{sev} cement bonus should be non-negative"
            );
        }
    }

    #[test]
    fn severity_serious_classification() {
        assert!(!HarmSeverity::NonSerious.is_serious());
        assert!(HarmSeverity::OtherSerious.is_serious());
        assert!(HarmSeverity::Death.is_serious());
        assert!(HarmSeverity::Hospitalization.is_serious());
    }

    #[test]
    fn death_has_maximum_multiplier() {
        let max_mult = HarmSeverity::ALL
            .iter()
            .map(|s| s.trust_multiplier())
            .fold(0.0_f64, f64::max);
        assert!(
            (max_mult - HarmSeverity::Death.trust_multiplier()).abs() < EPS,
            "Death should have highest multiplier"
        );
    }

    // --- CausalityAssessment ---

    #[test]
    fn causality_ordering() {
        assert!(CausalityAssessment::Unassessable < CausalityAssessment::Unlikely);
        assert!(CausalityAssessment::Unlikely < CausalityAssessment::Possible);
        assert!(CausalityAssessment::Possible < CausalityAssessment::Probable);
        assert!(CausalityAssessment::Probable < CausalityAssessment::Certain);
    }

    #[test]
    fn causality_weights_monotonically_increase() {
        let weights: Vec<f64> = CausalityAssessment::ALL
            .iter()
            .map(|c| c.evidence_weight())
            .collect();
        for pair in weights.windows(2) {
            assert!(
                pair[1] > pair[0],
                "weight should increase: {} -> {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn certain_causality_has_full_weight() {
        assert!(
            (CausalityAssessment::Certain.evidence_weight() - 1.0).abs() < EPS,
            "Certain should have weight 1.0"
        );
    }

    #[test]
    fn unassessable_has_minimal_but_nonzero_weight() {
        let w = CausalityAssessment::Unassessable.evidence_weight();
        assert!(w > 0.0, "Unassessable should have nonzero weight");
        assert!(w < 0.2, "Unassessable should have minimal weight");
    }

    // --- harm_adjusted_weight ---

    #[test]
    fn harm_weight_death_certain() {
        let w = harm_adjusted_weight(1.0, HarmSeverity::Death, CausalityAssessment::Certain);
        assert!(
            (w - 5.0).abs() < EPS,
            "Death + Certain should be 5.0, got {w:.2}"
        );
    }

    #[test]
    fn harm_weight_nonserious_possible() {
        let w = harm_adjusted_weight(1.0, HarmSeverity::NonSerious, CausalityAssessment::Possible);
        assert!(
            (w - 0.5).abs() < EPS,
            "NonSerious + Possible should be 0.5, got {w:.2}"
        );
    }

    #[test]
    fn harm_weight_scales_with_base() {
        let w1 = harm_adjusted_weight(
            1.0,
            HarmSeverity::Hospitalization,
            CausalityAssessment::Probable,
        );
        let w2 = harm_adjusted_weight(
            2.0,
            HarmSeverity::Hospitalization,
            CausalityAssessment::Probable,
        );
        assert!(
            (w2 - 2.0 * w1).abs() < EPS,
            "doubling base should double weight"
        );
    }

    #[test]
    fn harm_weight_negative_base_clamped() {
        let w = harm_adjusted_weight(-5.0, HarmSeverity::Death, CausalityAssessment::Certain);
        assert!(w.abs() < EPS, "negative base should clamp to 0, got {w:.2}");
    }

    // --- safety_config ---

    #[test]
    fn safety_config_is_stricter_than_default() {
        let safe = safety_config();
        let default = TrustConfig::default();

        assert!(
            safe.asymmetry_factor > default.asymmetry_factor,
            "safety asymmetry should be higher"
        );
        assert!(
            safe.significance_threshold > default.significance_threshold,
            "safety should require more evidence"
        );
        assert!(
            safe.score_ceiling < default.score_ceiling,
            "safety should cap overconfidence"
        );
        assert!(
            safe.decay_rate < default.decay_rate,
            "safety should have longer memory"
        );
    }

    #[test]
    fn safety_config_has_strict_thresholds() {
        let safe = safety_config();
        let strict = LevelThresholds::strict();
        assert!(
            (safe.thresholds.trusted - strict.trusted).abs() < EPS,
            "safety should use strict thresholds"
        );
    }

    #[test]
    fn safety_engine_starts_neutral() {
        let engine = safety_engine();
        assert!((engine.score() - 0.5).abs() < EPS);
    }

    // --- record_harm_evidence ---

    #[test]
    fn harm_evidence_reduces_trust() {
        let mut engine = safety_engine();
        let before = engine.score();
        record_harm_evidence(
            &mut engine,
            1.0,
            HarmSeverity::Hospitalization,
            CausalityAssessment::Probable,
        );
        assert!(engine.score() < before, "harm evidence should reduce trust");
    }

    #[test]
    fn death_drops_trust_more_than_nonserious() {
        let mut death_engine = safety_engine();
        record_harm_evidence(
            &mut death_engine,
            1.0,
            HarmSeverity::Death,
            CausalityAssessment::Certain,
        );
        let death_drop = 0.5 - death_engine.score();

        let mut minor_engine = safety_engine();
        record_harm_evidence(
            &mut minor_engine,
            1.0,
            HarmSeverity::NonSerious,
            CausalityAssessment::Certain,
        );
        let minor_drop = 0.5 - minor_engine.score();

        assert!(
            death_drop > minor_drop,
            "death ({death_drop:.4}) should drop trust more than non-serious ({minor_drop:.4})"
        );
    }

    #[test]
    fn uncertain_causality_reduces_impact() {
        let mut certain_engine = safety_engine();
        record_harm_evidence(
            &mut certain_engine,
            1.0,
            HarmSeverity::Death,
            CausalityAssessment::Certain,
        );
        let certain_drop = 0.5 - certain_engine.score();

        let mut uncertain_engine = safety_engine();
        record_harm_evidence(
            &mut uncertain_engine,
            1.0,
            HarmSeverity::Death,
            CausalityAssessment::Unlikely,
        );
        let uncertain_drop = 0.5 - uncertain_engine.score();

        assert!(
            certain_drop > uncertain_drop,
            "certain causality ({certain_drop:.4}) should drop more than unlikely ({uncertain_drop:.4})"
        );
    }

    #[test]
    fn p0_guarantee_death_always_has_impact() {
        // Even with lowest causality, Death severity produces nonzero impact
        let w = harm_adjusted_weight(1.0, HarmSeverity::Death, CausalityAssessment::Unassessable);
        assert!(
            w > 0.0,
            "Death + Unassessable must still produce positive weight: {w:.4}"
        );
        // Specifically: 1.0 * 5.0 * 0.1 = 0.5
        assert!((w - 0.5).abs() < EPS, "expected 0.5, got {w:.4}");
    }

    // --- Naranjo mapping ---

    #[test]
    fn naranjo_score_boundaries() {
        assert_eq!(naranjo_to_causality(-4), CausalityAssessment::Unlikely);
        assert_eq!(naranjo_to_causality(0), CausalityAssessment::Unlikely);
        assert_eq!(naranjo_to_causality(1), CausalityAssessment::Possible);
        assert_eq!(naranjo_to_causality(4), CausalityAssessment::Possible);
        assert_eq!(naranjo_to_causality(5), CausalityAssessment::Probable);
        assert_eq!(naranjo_to_causality(8), CausalityAssessment::Probable);
        assert_eq!(naranjo_to_causality(9), CausalityAssessment::Certain);
        assert_eq!(naranjo_to_causality(13), CausalityAssessment::Certain);
    }

    #[test]
    fn naranjo_feeds_into_trust() {
        let mut engine = safety_engine();
        let naranjo_score = 7; // Probable ADR
        let causality = naranjo_to_causality(naranjo_score);
        assert_eq!(causality, CausalityAssessment::Probable);

        record_harm_evidence(&mut engine, 1.0, HarmSeverity::Hospitalization, causality);
        // Should have meaningful impact: 1.0 * 3.0 * 0.8 = 2.4 (before asymmetry)
        assert!(
            engine.score() < 0.3,
            "Naranjo 7 + Hospitalization should be significant"
        );
    }

    // --- WHO-UMC mapping ---

    #[test]
    fn who_umc_string_mapping() {
        assert_eq!(
            who_umc_to_causality("Certain"),
            Some(CausalityAssessment::Certain)
        );
        assert_eq!(
            who_umc_to_causality("probable"),
            Some(CausalityAssessment::Probable)
        );
        assert_eq!(
            who_umc_to_causality("Likely"),
            Some(CausalityAssessment::Probable)
        );
        assert_eq!(
            who_umc_to_causality("possible"),
            Some(CausalityAssessment::Possible)
        );
        assert_eq!(
            who_umc_to_causality("Unlikely"),
            Some(CausalityAssessment::Unlikely)
        );
        assert_eq!(
            who_umc_to_causality("Conditional"),
            Some(CausalityAssessment::Unassessable)
        );
        assert_eq!(
            who_umc_to_causality("Unclassifiable"),
            Some(CausalityAssessment::Unassessable)
        );
        assert_eq!(
            who_umc_to_causality("Unassessable"),
            Some(CausalityAssessment::Unassessable)
        );
        assert_eq!(who_umc_to_causality("garbage"), None);
    }

    // --- serious_harm_floor ---

    #[test]
    fn serious_harm_floor_enforces_minimum() {
        // Unassessable + OtherSerious = 1.0 * 1.5 * 0.1 = 0.15
        // But floor is 0.5
        let w = serious_harm_floor(
            1.0,
            HarmSeverity::OtherSerious,
            CausalityAssessment::Unassessable,
            0.5,
        );
        assert!(
            (w - 0.5).abs() < EPS,
            "floor should enforce minimum: got {w:.4}"
        );
    }

    #[test]
    fn serious_harm_floor_does_not_reduce() {
        // Death + Certain = 5.0, floor = 0.5 — should not reduce
        let w = serious_harm_floor(1.0, HarmSeverity::Death, CausalityAssessment::Certain, 0.5);
        assert!(
            (w - 5.0).abs() < EPS,
            "floor should not reduce high-severity weight: got {w:.4}"
        );
    }

    #[test]
    fn nonserious_ignores_floor() {
        // NonSerious + Unassessable = 0.1, floor = 0.5 — not applied
        let w = serious_harm_floor(
            1.0,
            HarmSeverity::NonSerious,
            CausalityAssessment::Unassessable,
            0.5,
        );
        assert!(
            (w - 0.1).abs() < EPS,
            "non-serious should ignore floor: got {w:.4}"
        );
    }

    // --- Integration scenario ---

    #[test]
    fn safety_pipeline_full_scenario() {
        let mut engine = safety_engine();

        // Phase 1: Build trust with positive history
        for _ in 0..20 {
            engine.record(Evidence::positive());
        }
        let trusted_score = engine.score();
        assert!(
            trusted_score > 0.7,
            "should build trust: {trusted_score:.3}"
        );

        // Phase 2: Serious harm event (Naranjo 6 = Probable, Hospitalization)
        let causality = naranjo_to_causality(6);
        record_harm_evidence(&mut engine, 1.0, HarmSeverity::Hospitalization, causality);
        let after_harm = engine.score();
        assert!(
            after_harm < trusted_score,
            "harm should reduce trust: {trusted_score:.3} -> {after_harm:.3}"
        );

        // Phase 3: Recovery requires many positive interactions
        let mut recovery_count = 0u32;
        while engine.score() < trusted_score && recovery_count < 1000 {
            engine.record(Evidence::positive());
            recovery_count += 1;
        }
        assert!(
            recovery_count > 5,
            "recovery from safety event should take significant effort: {recovery_count}"
        );

        // Phase 4: Death event — devastating relative to current trust
        let before_death = engine.score();
        record_harm_evidence(
            &mut engine,
            1.0,
            HarmSeverity::Death,
            CausalityAssessment::Certain,
        );
        let after_death = engine.score();
        let death_drop = before_death - after_death;
        assert!(
            death_drop > 0.05,
            "death event should cause significant trust drop: {before_death:.3} -> {after_death:.3}"
        );

        // Phase 5: Fresh engine — death event on minimal trust is devastating
        let mut fresh = safety_engine();
        for _ in 0..5 {
            fresh.record(Evidence::positive());
        }
        record_harm_evidence(
            &mut fresh,
            1.0,
            HarmSeverity::Death,
            CausalityAssessment::Certain,
        );
        assert!(
            fresh.score() < 0.3,
            "death on fresh engine should devastate trust: {:.3}",
            fresh.score()
        );
    }

    // --- Display ---

    #[test]
    fn display_formatting() {
        assert_eq!(format!("{}", HarmSeverity::Death), "Death");
        assert_eq!(format!("{}", HarmSeverity::NonSerious), "Non-serious");
        assert_eq!(format!("{}", CausalityAssessment::Probable), "Probable");
        assert_eq!(
            format!("{}", CausalityAssessment::Unassessable),
            "Unassessable"
        );
    }
}
