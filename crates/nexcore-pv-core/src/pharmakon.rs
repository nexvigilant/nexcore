//! # The Pharmakon Principle (ToV §34)
//!
//! The Greek word **pharmakon** (φάρμακον) carries deliberate, irreducible ambiguity:
//! remedy, poison, and ritual charm simultaneously.
//!
//! > **"Power cannot be directionally pure. What acts, acts in all directions."**
//!
//! Any intervention powerful enough to create intended change is powerful enough
//! to create unintended change. Efficacy and danger are inseparable properties
//! of potency itself.
//!
//! # Core Theory (§34.2)
//!
//! - **Disruption Principle**: To heal is to intervene - efficacy and danger inseparable
//! - **Balance/Equilibrium**: Pharmakon counters imbalance, but can overcorrect
//! - **Boundary Transgression**: Crossing thresholds carries inherent risk
//! - **Harm as Property**: Adverse effects are the shadow of action, not defects
//! - **Precision is Asymptotic**: Can narrow scatter, never eliminate
//! - **Dose as Ratio**: Remedy and poison coexist at every dose
//! - **Context Determines Expression**: Same intervention can flip based on θ
//!
//! # ToV Connections
//!
//! | Pharmakon Concept | ToV Connection |
//! |-------------------|----------------|
//! | Disruption | Definition 1.4 (Perturbation) |
//! | Equilibrium | §5 Safety Manifold int(M) |
//! | Boundary | Definition 5.3 (Harm Boundary) |
//! | Dose-as-ratio | §6.4 Continuous P(m,c,b,t,θ) |
//! | Context | Definition 4.1 (θ parameter) |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::pv::pharmakon::{
//!     PharmakónFace, TherapeuticWindow, BenefitRiskRatio, PotencyMagnitude,
//! };
//!
//! // A drug has a therapeutic window
//! let window = TherapeuticWindow::new(10.0, 100.0);
//! assert!(window.therapeutic_index() > 1.0);
//!
//! // Benefit-risk ratio shifts with context
//! let ratio = BenefitRiskRatio::from_components(0.8, 0.2);
//! assert!(ratio.favors_benefit());
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// PHARMAKON FACES (T2-P)
// ============================================================================

/// The three faces of pharmakon (§34.1).
///
/// # Tier: T2-P
///
/// The Greek word pharmakon carries deliberate ambiguity - these three
/// meanings coexist in every intervention.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PharmakónFace {
    /// Remedy / Medicine / Cure - the intended beneficial effect.
    Remedy = 0,
    /// Poison / Toxin - the harmful effect (same mechanism, different context).
    Poison = 1,
    /// Charm / Spell / Ritual - the psychological/social effect.
    Ritual = 2,
}

impl PharmakónFace {
    /// Get all three faces.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Remedy, Self::Poison, Self::Ritual]
    }

    /// Get the opposite face (Remedy ↔ Poison, Ritual unchanged).
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Remedy => Self::Poison,
            Self::Poison => Self::Remedy,
            Self::Ritual => Self::Ritual,
        }
    }
}

impl std::fmt::Display for PharmakónFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Remedy => write!(f, "Remedy (φάρμακον)"),
            Self::Poison => write!(f, "Poison (φάρμακον)"),
            Self::Ritual => write!(f, "Ritual (φάρμακον)"),
        }
    }
}

// ============================================================================
// POTENCY MAGNITUDE (T2-P)
// ============================================================================

/// Intervention potency magnitude (§34.2).
///
/// # Tier: T2-P
///
/// Newtype over f64 representing the strength of an intervention.
/// Higher potency means greater capacity for both benefit AND harm.
///
/// > "Any substance powerful enough to alter a disease state is,
/// > by definition, powerful enough to cause harm."
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PotencyMagnitude(f64);

impl PotencyMagnitude {
    /// Zero potency (no intervention).
    pub const ZERO: Self = Self(0.0);

    /// Create from raw value, clamping to non-negative.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if potency is above therapeutic threshold.
    ///
    /// Potency below this level is unlikely to have therapeutic effect.
    #[must_use]
    pub fn is_therapeutic(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }

    /// Check if potency is above toxic threshold.
    ///
    /// Potency above this level is likely to cause harm.
    #[must_use]
    pub fn is_toxic(&self, threshold: f64) -> bool {
        self.0 >= threshold
    }
}

impl Default for PotencyMagnitude {
    fn default() -> Self {
        Self::ZERO
    }
}

// ============================================================================
// BENEFIT-RISK RATIO (T2-P)
// ============================================================================

/// Benefit-risk ratio (§34.3.3).
///
/// # Tier: T2-P
///
/// > "Both properties—remedy and poison—exist simultaneously at every dose.
/// > We are shifting a ratio between benefit and harm that was always present."
///
/// The ratio is stored as benefit probability, with risk = 1 - benefit.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct BenefitRiskRatio(f64);

impl BenefitRiskRatio {
    /// Pure benefit (theoretical limit, never achievable).
    pub const PURE_BENEFIT: Self = Self(1.0);
    /// Pure risk (theoretical limit).
    pub const PURE_RISK: Self = Self(0.0);
    /// Balanced (equal benefit and risk).
    pub const BALANCED: Self = Self(0.5);

    /// Create from benefit probability (0.0-1.0).
    #[must_use]
    pub fn new(benefit_probability: f64) -> Self {
        Self(benefit_probability.clamp(0.0, 1.0))
    }

    /// Create from separate benefit and risk components.
    ///
    /// Normalizes to ratio: benefit / (benefit + risk).
    #[must_use]
    pub fn from_components(benefit: f64, risk: f64) -> Self {
        let total = benefit.abs() + risk.abs();
        if total == 0.0 {
            Self::BALANCED
        } else {
            Self::new(benefit.abs() / total)
        }
    }

    /// Get benefit probability (0.0-1.0).
    #[must_use]
    pub const fn benefit(self) -> f64 {
        self.0
    }

    /// Get risk probability (0.0-1.0).
    #[must_use]
    pub fn risk(self) -> f64 {
        1.0 - self.0
    }

    /// Check if ratio favors benefit (> 0.5).
    #[must_use]
    pub fn favors_benefit(self) -> bool {
        self.0 > 0.5
    }

    /// Check if ratio favors risk (< 0.5).
    #[must_use]
    pub fn favors_risk(self) -> bool {
        self.0 < 0.5
    }

    /// Get the dominant face based on ratio.
    #[must_use]
    pub fn dominant_face(self) -> PharmakónFace {
        if self.0 > 0.5 {
            PharmakónFace::Remedy
        } else {
            PharmakónFace::Poison
        }
    }
}

impl Default for BenefitRiskRatio {
    fn default() -> Self {
        Self::BALANCED
    }
}

// ============================================================================
// THERAPEUTIC WINDOW (T2-C)
// ============================================================================

/// Therapeutic window (§34.3.3).
///
/// # Tier: T2-C
///
/// The range between minimum effective dose and minimum toxic dose.
/// This is the "safe zone" where the intervention acts as remedy,
/// not poison.
///
/// > "Sola dosis facit venenum" (the dose alone makes the poison)
/// > — Paracelsus
///
/// But the pharmakon principle goes deeper: both properties exist
/// at every dose; we're shifting a ratio.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TherapeuticWindow {
    /// Minimum effective dose (MED) - below this, no therapeutic effect.
    pub min_effective: f64,
    /// Minimum toxic dose (MTD) - above this, toxic effects dominate.
    pub min_toxic: f64,
}

impl TherapeuticWindow {
    /// Create a therapeutic window.
    ///
    /// # Panics
    ///
    /// Does not panic - if min_toxic < min_effective, creates an
    /// inverted window (no safe zone).
    #[must_use]
    pub fn new(min_effective: f64, min_toxic: f64) -> Self {
        Self {
            min_effective: min_effective.max(0.0),
            min_toxic: min_toxic.max(0.0),
        }
    }

    /// Therapeutic index (TI = MTD / MED).
    ///
    /// Higher values indicate safer drugs.
    /// - TI < 1: No safe therapeutic zone (always toxic before effective)
    /// - TI = 1: No margin (effective dose = toxic dose)
    /// - TI = 2: Narrow margin (doubling dose becomes toxic)
    /// - TI > 10: Wide margin (relatively safe)
    #[must_use]
    pub fn therapeutic_index(&self) -> f64 {
        if self.min_effective == 0.0 {
            f64::INFINITY
        } else {
            self.min_toxic / self.min_effective
        }
    }

    /// Check if a dose is in the therapeutic window.
    #[must_use]
    pub fn is_therapeutic(&self, dose: f64) -> bool {
        dose >= self.min_effective && dose < self.min_toxic
    }

    /// Check if a dose is sub-therapeutic (ineffective).
    #[must_use]
    pub fn is_subtherapeutic(&self, dose: f64) -> bool {
        dose < self.min_effective
    }

    /// Check if a dose is toxic.
    #[must_use]
    pub fn is_toxic(&self, dose: f64) -> bool {
        dose >= self.min_toxic
    }

    /// Get the width of the therapeutic window.
    #[must_use]
    pub fn width(&self) -> f64 {
        (self.min_toxic - self.min_effective).max(0.0)
    }

    /// Check if the window is valid (has positive width).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.min_toxic > self.min_effective
    }

    /// Get the midpoint dose (center of therapeutic window).
    #[must_use]
    pub fn optimal_dose(&self) -> f64 {
        (self.min_effective + self.min_toxic) / 2.0
    }

    /// Calculate benefit-risk ratio at a given dose.
    ///
    /// Uses sigmoidal approximation based on distance from window edges.
    #[must_use]
    pub fn benefit_risk_at(&self, dose: f64) -> BenefitRiskRatio {
        if dose < self.min_effective {
            // Sub-therapeutic: low benefit, low risk
            let efficacy = (dose / self.min_effective).clamp(0.0, 1.0);
            BenefitRiskRatio::new(efficacy * 0.3) // Max 30% benefit when sub-therapeutic
        } else if dose >= self.min_toxic {
            // Toxic: benefit exists but risk dominates
            let toxicity_factor = ((dose - self.min_toxic) / self.min_toxic).min(1.0);
            BenefitRiskRatio::new(0.5 - toxicity_factor * 0.4) // 50% → 10% as dose increases
        } else {
            // Therapeutic window: calculate position
            let position = (dose - self.min_effective) / self.width();
            // Peak benefit at optimal dose (middle of window)
            let distance_from_optimal = (position - 0.5).abs();
            let benefit = 0.9 - distance_from_optimal * 0.3; // 90% at optimal, 75% at edges
            BenefitRiskRatio::new(benefit)
        }
    }
}

// ============================================================================
// CONTEXTUAL EXPRESSION (T2-C)
// ============================================================================

/// Context that determines pharmakon expression (§34.3.4).
///
/// # Tier: T2-C
///
/// > "The same intervention in a different body, system, population,
/// > or timing can flip from therapeutic to toxic."
///
/// This corresponds to the θ parameter in ToV Definition 4.1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextualExpression {
    /// Individual susceptibility factor (0.0-1.0).
    ///
    /// Higher values indicate greater sensitivity to the intervention.
    pub susceptibility: f64,
    /// Timing factor (0.0-1.0).
    ///
    /// How appropriate is the timing of intervention?
    pub timing_appropriateness: f64,
    /// System state factor (0.0-1.0).
    ///
    /// How receptive is the system to this intervention?
    pub system_receptivity: f64,
}

impl ContextualExpression {
    /// Create a new contextual expression.
    #[must_use]
    pub fn new(susceptibility: f64, timing: f64, receptivity: f64) -> Self {
        Self {
            susceptibility: susceptibility.clamp(0.0, 1.0),
            timing_appropriateness: timing.clamp(0.0, 1.0),
            system_receptivity: receptivity.clamp(0.0, 1.0),
        }
    }

    /// Default/neutral context.
    #[must_use]
    pub fn neutral() -> Self {
        Self::new(0.5, 0.5, 0.5)
    }

    /// High-risk context (susceptible, poor timing, unreceptive).
    #[must_use]
    pub fn high_risk() -> Self {
        Self::new(0.9, 0.2, 0.2)
    }

    /// Optimal context (moderate susceptibility, good timing, receptive).
    #[must_use]
    pub fn optimal() -> Self {
        Self::new(0.5, 0.9, 0.9)
    }

    /// Calculate overall context favorability (0.0-1.0).
    ///
    /// Higher values indicate context more likely to express remedy face.
    #[must_use]
    pub fn favorability(&self) -> f64 {
        // Susceptibility is double-edged (affects both benefit and risk)
        // Timing and receptivity favor benefit
        let timing_weight = 0.4;
        let receptivity_weight = 0.4;
        let susceptibility_penalty = 0.2;

        self.timing_appropriateness * timing_weight
            + self.system_receptivity * receptivity_weight
            + (1.0 - self.susceptibility) * susceptibility_penalty
    }

    /// Predict which face will be expressed given this context.
    #[must_use]
    pub fn predicted_face(&self) -> PharmakónFace {
        if self.favorability() > 0.5 {
            PharmakónFace::Remedy
        } else {
            PharmakónFace::Poison
        }
    }

    /// Modify benefit-risk ratio based on context.
    #[must_use]
    pub fn modify_ratio(&self, base_ratio: BenefitRiskRatio) -> BenefitRiskRatio {
        let favorability = self.favorability();
        // Shift ratio toward benefit if favorable, toward risk if unfavorable
        let shift = (favorability - 0.5) * 0.4; // Max ±20% shift
        BenefitRiskRatio::new(base_ratio.benefit() + shift)
    }
}

impl Default for ContextualExpression {
    fn default() -> Self {
        Self::neutral()
    }
}

// ============================================================================
// PHARMAKON ANALYSIS (T3)
// ============================================================================

/// Complete pharmakon analysis for an intervention (§34).
///
/// # Tier: T3
///
/// Domain-specific composite combining all pharmakon concepts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PharmakonAnalysis {
    /// Intervention potency.
    pub potency: PotencyMagnitude,
    /// Therapeutic window (if applicable).
    pub therapeutic_window: Option<TherapeuticWindow>,
    /// Base benefit-risk ratio (without context).
    pub base_ratio: BenefitRiskRatio,
    /// Current context.
    pub context: ContextualExpression,
    /// Actual dose/magnitude applied.
    pub applied_dose: f64,
}

impl PharmakonAnalysis {
    /// Create a new pharmakon analysis.
    #[must_use]
    pub fn new(potency: PotencyMagnitude, applied_dose: f64) -> Self {
        Self {
            potency,
            therapeutic_window: None,
            base_ratio: BenefitRiskRatio::BALANCED,
            context: ContextualExpression::neutral(),
            applied_dose,
        }
    }

    /// Set therapeutic window.
    #[must_use]
    pub fn with_window(mut self, window: TherapeuticWindow) -> Self {
        self.therapeutic_window = Some(window);
        self
    }

    /// Set base benefit-risk ratio.
    #[must_use]
    pub fn with_base_ratio(mut self, ratio: BenefitRiskRatio) -> Self {
        self.base_ratio = ratio;
        self
    }

    /// Set context.
    #[must_use]
    pub fn with_context(mut self, context: ContextualExpression) -> Self {
        self.context = context;
        self
    }

    /// Calculate effective benefit-risk ratio (combining all factors).
    #[must_use]
    pub fn effective_ratio(&self) -> BenefitRiskRatio {
        // Start with window-based ratio if available
        let dose_ratio = self
            .therapeutic_window
            .map(|w| w.benefit_risk_at(self.applied_dose))
            .unwrap_or(self.base_ratio);

        // Modify by context
        self.context.modify_ratio(dose_ratio)
    }

    /// Predict which pharmakon face will manifest.
    #[must_use]
    pub fn predicted_face(&self) -> PharmakónFace {
        self.effective_ratio().dominant_face()
    }

    /// Check if intervention is within therapeutic window.
    #[must_use]
    pub fn is_in_therapeutic_window(&self) -> Option<bool> {
        self.therapeutic_window
            .map(|w| w.is_therapeutic(self.applied_dose))
    }

    /// Get therapeutic index if window is defined.
    #[must_use]
    pub fn therapeutic_index(&self) -> Option<f64> {
        self.therapeutic_window.map(|w| w.therapeutic_index())
    }

    /// Generate a summary of the analysis.
    #[must_use]
    pub fn summary(&self) -> PharmakonSummary {
        let effective = self.effective_ratio();
        PharmakonSummary {
            predicted_face: self.predicted_face(),
            effective_benefit_probability: effective.benefit(),
            effective_risk_probability: effective.risk(),
            in_therapeutic_window: self.is_in_therapeutic_window(),
            therapeutic_index: self.therapeutic_index(),
            context_favorability: self.context.favorability(),
        }
    }
}

/// Summary of pharmakon analysis.
///
/// # Tier: T3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PharmakonSummary {
    /// Which face is predicted to manifest.
    pub predicted_face: PharmakónFace,
    /// Effective benefit probability after all adjustments.
    pub effective_benefit_probability: f64,
    /// Effective risk probability after all adjustments.
    pub effective_risk_probability: f64,
    /// Whether dose is in therapeutic window (if applicable).
    pub in_therapeutic_window: Option<bool>,
    /// Therapeutic index (if applicable).
    pub therapeutic_index: Option<f64>,
    /// Context favorability score.
    pub context_favorability: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pharmakon_faces() {
        assert_eq!(PharmakónFace::Remedy.opposite(), PharmakónFace::Poison);
        assert_eq!(PharmakónFace::Poison.opposite(), PharmakónFace::Remedy);
        assert_eq!(PharmakónFace::Ritual.opposite(), PharmakónFace::Ritual);
        assert_eq!(PharmakónFace::all().len(), 3);
    }

    #[test]
    fn test_potency_magnitude() {
        let p = PotencyMagnitude::new(50.0);
        assert_eq!(p.value(), 50.0);
        assert!(p.is_therapeutic(40.0));
        assert!(!p.is_therapeutic(60.0));

        // Negative clamped to zero
        assert_eq!(PotencyMagnitude::new(-10.0).value(), 0.0);
    }

    #[test]
    fn test_benefit_risk_ratio() {
        let balanced = BenefitRiskRatio::BALANCED;
        assert!(!balanced.favors_benefit());
        assert!(!balanced.favors_risk());

        let favorable = BenefitRiskRatio::new(0.8);
        assert!(favorable.favors_benefit());
        assert_eq!(favorable.dominant_face(), PharmakónFace::Remedy);

        let risky = BenefitRiskRatio::new(0.3);
        assert!(risky.favors_risk());
        assert_eq!(risky.dominant_face(), PharmakónFace::Poison);

        // From components
        let ratio = BenefitRiskRatio::from_components(80.0, 20.0);
        assert!((ratio.benefit() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_therapeutic_window() {
        let window = TherapeuticWindow::new(10.0, 100.0);

        assert!(window.is_valid());
        assert_eq!(window.therapeutic_index(), 10.0);
        assert_eq!(window.width(), 90.0);
        assert_eq!(window.optimal_dose(), 55.0);

        assert!(window.is_subtherapeutic(5.0));
        assert!(window.is_therapeutic(50.0));
        assert!(window.is_toxic(150.0));

        // Inverted window
        let inverted = TherapeuticWindow::new(100.0, 10.0);
        assert!(!inverted.is_valid());
        assert!(inverted.therapeutic_index() < 1.0);
    }

    #[test]
    fn test_benefit_risk_at_dose() {
        let window = TherapeuticWindow::new(10.0, 100.0);

        // Sub-therapeutic
        let sub = window.benefit_risk_at(5.0);
        assert!(sub.benefit() < 0.5);

        // Optimal (middle of window)
        let optimal = window.benefit_risk_at(55.0);
        assert!(optimal.favors_benefit());

        // Toxic
        let toxic = window.benefit_risk_at(150.0);
        assert!(toxic.favors_risk());
    }

    #[test]
    fn test_contextual_expression() {
        let neutral = ContextualExpression::neutral();
        assert!((neutral.favorability() - 0.5).abs() < 0.1);

        let optimal = ContextualExpression::optimal();
        assert!(optimal.favorability() > 0.7);
        assert_eq!(optimal.predicted_face(), PharmakónFace::Remedy);

        let risky = ContextualExpression::high_risk();
        assert!(risky.favorability() < 0.3);
        assert_eq!(risky.predicted_face(), PharmakónFace::Poison);
    }

    #[test]
    fn test_context_modifies_ratio() {
        let base = BenefitRiskRatio::BALANCED;

        let optimal_context = ContextualExpression::optimal();
        let modified = optimal_context.modify_ratio(base);
        assert!(modified.benefit() > base.benefit());

        let risky_context = ContextualExpression::high_risk();
        let modified = risky_context.modify_ratio(base);
        assert!(modified.benefit() < base.benefit());
    }

    #[test]
    fn test_pharmakon_analysis() {
        let window = TherapeuticWindow::new(10.0, 100.0);
        let analysis = PharmakonAnalysis::new(PotencyMagnitude::new(50.0), 55.0)
            .with_window(window)
            .with_context(ContextualExpression::optimal());

        assert!(analysis.is_in_therapeutic_window() == Some(true));
        assert!(analysis.therapeutic_index() == Some(10.0));

        let summary = analysis.summary();
        assert_eq!(summary.predicted_face, PharmakónFace::Remedy);
        assert!(summary.effective_benefit_probability > 0.7);
    }

    #[test]
    fn test_dose_makes_poison() {
        // Paracelsus: "Sola dosis facit venenum"
        let window = TherapeuticWindow::new(10.0, 100.0);

        // Same drug at different doses
        let therapeutic =
            PharmakonAnalysis::new(PotencyMagnitude::new(50.0), 50.0).with_window(window);
        let toxic = PharmakonAnalysis::new(PotencyMagnitude::new(50.0), 150.0).with_window(window);

        assert_eq!(therapeutic.predicted_face(), PharmakónFace::Remedy);
        assert_eq!(toxic.predicted_face(), PharmakónFace::Poison);
    }
}
