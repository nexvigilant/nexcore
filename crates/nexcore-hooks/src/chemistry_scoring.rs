//! Chemistry-based scoring for hook improvement actions.
//!
//! Applies real thermodynamic and kinetic equations to determine
//! whether capability improvements should proceed.
//!
//! # Equations Used
//!
//! | Equation | Application |
//! |----------|-------------|
//! | Arrhenius | Activation energy barrier for changes |
//! | Gibbs | Thermodynamic feasibility of improvement |
//! | Half-life | Staleness decay of capabilities |
//! | Michaelis-Menten | Processing capacity saturation |
//!
//! # Safety
//!
//! All inputs are clamped to valid ranges. Negative values are treated as zero.
//! Division by zero is guarded. All outputs are finite.
//!
//! # Observability
//!
//! All calculations are instrumented with `tracing` spans for debugging
//! and performance monitoring.

use serde::{Deserialize, Serialize};
use std::f64::consts::LN_2;
use tracing::{debug, instrument};

/// Gas constant (J/(mol·K)) - used for Arrhenius equation scaling
const R: f64 = 8.314;

/// Arrhenius rate calculation for improvement actions.
///
/// k = A × e^(-Ea/RT)
///
/// Higher temperature (urgency) lowers the effective activation barrier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrheniusRate {
    /// Pre-exponential factor (base rate)
    pub pre_factor: f64,
    /// Activation energy (complexity barrier)
    pub activation_energy: f64,
    /// Temperature (urgency/priority)
    pub temperature: f64,
    /// Calculated rate constant
    pub rate: f64,
}

impl ArrheniusRate {
    /// Calculate rate from complexity and urgency.
    ///
    /// # Arguments
    /// * `complexity` - Lines changed + dependency count (0-100)
    /// * `urgency` - Priority factor (1-10, higher = more urgent)
    ///
    /// # Chemistry Mapping
    /// - Activation energy scaled to kJ/mol range (complexity × 100)
    /// - Temperature scaled to Kelvin range (urgency × 30 + 250)
    /// - Low urgency (1) → ~280K (cold), High urgency (10) → ~550K (hot)
    #[must_use]
    #[instrument(level = "debug", skip_all, fields(complexity, urgency))]
    pub fn calculate(complexity: f64, urgency: f64) -> Self {
        let pre_factor = 1.0;
        // Scale complexity to realistic activation energy (kJ/mol equivalent)
        // Factor of 200 ensures complexity=50 at low urgency gives unfavorable rate
        let activation_energy = complexity.clamp(1.0, 100.0) * 200.0;
        // Scale urgency to temperature: low urgency = cold, high urgency = hot
        let temperature = (urgency * 30.0 + 250.0).clamp(280.0, 550.0);

        let rate = pre_factor * (-activation_energy / (R * temperature)).exp();

        debug!(
            activation_energy,
            temperature, rate, "Arrhenius rate calculated"
        );

        Self {
            pre_factor,
            activation_energy,
            temperature,
            rate,
        }
    }

    /// Check if rate exceeds threshold for action.
    #[must_use]
    pub fn is_favorable(&self, threshold: f64) -> bool {
        self.rate > threshold
    }

    /// Human-readable interpretation.
    #[must_use]
    pub fn interpretation(&self) -> &'static str {
        if self.rate > 0.5 {
            "Highly favorable - proceed immediately"
        } else if self.rate > 0.1 {
            "Favorable - recommend action"
        } else if self.rate > 0.01 {
            "Marginal - consider deferring"
        } else {
            "Unfavorable - high barrier"
        }
    }
}

/// Gibbs free energy calculation for improvement feasibility.
///
/// ΔG = ΔH - TΔS
///
/// Negative ΔG means improvement is thermodynamically spontaneous.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GibbsFeasibility {
    /// Enthalpy change (effort cost)
    pub delta_h: f64,
    /// Entropy change (disorder reduction = quality improvement)
    pub delta_s: f64,
    /// Temperature (priority multiplier)
    pub temperature: f64,
    /// Gibbs free energy
    pub delta_g: f64,
    /// Whether reaction is spontaneous
    pub spontaneous: bool,
}

impl GibbsFeasibility {
    /// Calculate feasibility from effort and quality gain.
    ///
    /// # Arguments
    /// * `effort` - Work required (lines, complexity) - positive
    /// * `quality_gain` - Code quality improvement (0-1) - positive means improvement
    /// * `priority` - Importance multiplier (1-10)
    #[must_use]
    #[instrument(level = "debug", skip_all, fields(effort, quality_gain, priority))]
    pub fn calculate(effort: f64, quality_gain: f64, priority: f64) -> Self {
        let delta_h = effort.clamp(0.0, 100.0);
        let delta_s = quality_gain.clamp(-1.0, 1.0);
        let temperature = (priority * 10.0).clamp(10.0, 100.0);

        let delta_g = delta_h - (temperature * delta_s);
        let spontaneous = delta_g < 0.0;

        Self {
            delta_h,
            delta_s,
            temperature,
            delta_g,
            spontaneous,
        }
    }

    /// Human-readable interpretation.
    #[must_use]
    pub fn interpretation(&self) -> &'static str {
        if self.delta_g < -50.0 {
            "Highly spontaneous - strong driving force"
        } else if self.delta_g < 0.0 {
            "Spontaneous - will proceed naturally"
        } else if self.delta_g < 20.0 {
            "Near equilibrium - may need catalyst"
        } else {
            "Non-spontaneous - requires external energy"
        }
    }
}

/// Half-life decay for capability staleness.
///
/// N(t) = N₀ × e^(-kt) where k = ln(2) / t½
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessDecay {
    /// Days since last update
    pub days_elapsed: f64,
    /// Half-life in days (when relevance drops to 50%)
    pub half_life: f64,
    /// Decay constant
    pub decay_constant: f64,
    /// Current relevance (0-1)
    pub relevance: f64,
}

impl StalenessDecay {
    /// Calculate staleness from days since update.
    ///
    /// # Arguments
    /// * `days_elapsed` - Days since last modification (negative clamped to 0)
    /// * `half_life` - Days until 50% staleness (minimum: 1.0)
    ///
    /// # Safety
    /// - Negative `days_elapsed` clamped to 0 (relevance cannot exceed 1.0)
    /// - `half_life` minimum is 1.0 to prevent division issues
    #[must_use]
    #[instrument(level = "debug", skip_all, fields(days_elapsed, half_life))]
    pub fn calculate(days_elapsed: f64, half_life: f64) -> Self {
        // Clamp inputs to valid ranges
        let days_elapsed = days_elapsed.max(0.0);
        let half_life = half_life.max(1.0);

        // Use exact ln(2) constant instead of approximation
        let decay_constant = LN_2 / half_life;

        // Calculate relevance and clamp to [0, 1]
        let relevance = (-decay_constant * days_elapsed).exp().clamp(0.0, 1.0);

        Self {
            days_elapsed,
            half_life,
            decay_constant,
            relevance,
        }
    }

    /// Check if capability needs refresh.
    #[must_use]
    pub fn needs_refresh(&self, threshold: f64) -> bool {
        self.relevance < threshold
    }

    /// Human-readable interpretation.
    #[must_use]
    pub fn interpretation(&self) -> &'static str {
        if self.relevance > 0.9 {
            "Fresh - recently updated"
        } else if self.relevance > 0.7 {
            "Current - still relevant"
        } else if self.relevance > 0.5 {
            "Aging - consider refresh"
        } else if self.relevance > 0.25 {
            "Stale - needs update"
        } else {
            "Decayed - requires overhaul"
        }
    }
}

/// Michaelis-Menten saturation for processing capacity.
///
/// v = Vmax × [S] / (Km + [S])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacitySaturation {
    /// Pending actions (substrate concentration)
    pub pending_count: f64,
    /// Maximum processing rate
    pub vmax: f64,
    /// Half-saturation constant
    pub km: f64,
    /// Current processing rate
    pub rate: f64,
    /// Saturation fraction (0-1)
    pub saturation: f64,
}

impl CapacitySaturation {
    /// Calculate saturation from pending action count.
    ///
    /// # Arguments
    /// * `pending` - Number of pending improvement actions
    /// * `vmax` - Maximum actions per session (minimum: 0.001)
    /// * `km` - Half-saturation point (minimum: 0.001)
    ///
    /// # Safety
    /// - `vmax` and `km` clamped to minimum 0.001 to prevent division by zero
    /// - All outputs are finite
    #[must_use]
    #[instrument(level = "debug", skip_all, fields(pending, vmax, km))]
    pub fn calculate(pending: usize, vmax: f64, km: f64) -> Self {
        let pending_count = pending as f64;
        // Guard against division by zero and invalid parameters
        let vmax = vmax.max(0.001);
        let km = km.max(0.001);

        let denominator = km + pending_count;
        let rate = vmax * pending_count / denominator;
        let saturation = (rate / vmax).clamp(0.0, 1.0);

        Self {
            pending_count,
            vmax,
            km,
            rate,
            saturation,
        }
    }

    /// Human-readable interpretation.
    #[must_use]
    pub fn interpretation(&self) -> &'static str {
        if self.saturation < 0.3 {
            "Low load - capacity available"
        } else if self.saturation < 0.6 {
            "Moderate load - normal processing"
        } else if self.saturation < 0.85 {
            "High load - approaching saturation"
        } else {
            "Saturated - at maximum capacity"
        }
    }
}

/// Combined chemistry score for improvement actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemistryScore {
    /// Arrhenius kinetic analysis
    pub kinetics: ArrheniusRate,
    /// Gibbs thermodynamic analysis
    pub thermodynamics: GibbsFeasibility,
    /// Staleness decay analysis
    pub staleness: StalenessDecay,
    /// Overall recommendation
    pub recommendation: Recommendation,
}

/// Action recommendation based on chemistry analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recommendation {
    /// Proceed immediately - favorable kinetics and thermodynamics
    Proceed,
    /// Schedule for later - favorable but not urgent
    Schedule,
    /// Defer - unfavorable conditions
    Defer,
    /// Skip - not worth the energy investment
    Skip,
}

impl ChemistryScore {
    /// Calculate comprehensive chemistry score.
    ///
    /// # Arguments
    /// * `complexity` - Change complexity (0-100)
    /// * `urgency` - Priority (1-10)
    /// * `effort` - Work required
    /// * `quality_gain` - Expected quality improvement (0-1)
    /// * `days_since_update` - Staleness in days
    #[must_use]
    #[instrument(
        level = "debug",
        skip_all,
        fields(complexity, urgency, effort, quality_gain, days_since_update)
    )]
    pub fn calculate(
        complexity: f64,
        urgency: f64,
        effort: f64,
        quality_gain: f64,
        days_since_update: f64,
    ) -> Self {
        let kinetics = ArrheniusRate::calculate(complexity, urgency);
        let thermodynamics = GibbsFeasibility::calculate(effort, quality_gain, urgency);
        let staleness = StalenessDecay::calculate(days_since_update, 30.0);

        // Determine recommendation
        let recommendation = if thermodynamics.spontaneous && kinetics.rate > 0.1 {
            if staleness.relevance < 0.5 {
                Recommendation::Proceed // Stale + favorable = urgent
            } else {
                Recommendation::Schedule // Fresh + favorable = can wait
            }
        } else if thermodynamics.spontaneous {
            Recommendation::Schedule // Favorable but slow kinetics
        } else if kinetics.rate > 0.3 {
            Recommendation::Defer // Fast but unfavorable thermodynamics
        } else {
            Recommendation::Skip // Neither favorable
        };

        Self {
            kinetics,
            thermodynamics,
            staleness,
            recommendation,
        }
    }

    /// Get activation energy in the 0-100 scale used by hooks.
    #[must_use]
    pub fn activation_energy_score(&self) -> u32 {
        // Invert: low barrier = low score (easy), high barrier = high score (hard)
        let normalized = (1.0 - self.kinetics.rate.min(1.0)) * 100.0;
        normalized as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Behavioral Tests (Phase 0)
    // =========================================================================

    #[test]
    fn test_arrhenius_high_urgency() {
        let rate = ArrheniusRate::calculate(50.0, 10.0); // High urgency
        assert!(rate.rate > 0.1, "High urgency should give favorable rate");
    }

    #[test]
    fn test_arrhenius_low_urgency() {
        let rate = ArrheniusRate::calculate(50.0, 1.0); // Low urgency
        assert!(rate.rate < 0.1, "Low urgency should give unfavorable rate");
    }

    #[test]
    fn test_gibbs_spontaneous() {
        // High quality gain with high priority = spontaneous
        let gibbs = GibbsFeasibility::calculate(10.0, 0.8, 8.0);
        assert!(gibbs.spontaneous, "High quality gain should be spontaneous");
        assert!(gibbs.delta_g < 0.0);
    }

    #[test]
    fn test_gibbs_non_spontaneous() {
        // High effort, low quality gain = non-spontaneous
        let gibbs = GibbsFeasibility::calculate(80.0, 0.1, 2.0);
        assert!(
            !gibbs.spontaneous,
            "High effort, low gain should be non-spontaneous"
        );
    }

    #[test]
    fn test_staleness_fresh() {
        let decay = StalenessDecay::calculate(5.0, 30.0);
        assert!(decay.relevance > 0.8, "5 days should still be fresh");
    }

    #[test]
    fn test_staleness_half_life() {
        let decay = StalenessDecay::calculate(30.0, 30.0);
        assert!(
            (decay.relevance - 0.5).abs() < 0.01,
            "At half-life, relevance should be ~0.5"
        );
    }

    #[test]
    fn test_saturation() {
        let sat = CapacitySaturation::calculate(10, 5.0, 10.0);
        assert!(
            (sat.rate - 2.5).abs() < 0.01,
            "At Km, rate should be Vmax/2"
        );
    }

    #[test]
    fn test_chemistry_score_proceed() {
        // Stale, easy change, high quality gain
        let score = ChemistryScore::calculate(20.0, 8.0, 15.0, 0.7, 60.0);
        assert_eq!(score.recommendation, Recommendation::Proceed);
    }

    #[test]
    fn test_chemistry_score_skip() {
        // Complex, low urgency, low quality gain
        let score = ChemistryScore::calculate(90.0, 1.0, 80.0, 0.1, 5.0);
        assert_eq!(score.recommendation, Recommendation::Skip);
    }

    // =========================================================================
    // Golden Value Tests (Phase 2 - Efficacy)
    // Validate equations produce correct chemistry values
    // =========================================================================

    #[test]
    fn test_arrhenius_golden_value() {
        // At complexity=50 (Ea=10000), urgency=1 (T=280):
        // k = exp(-10000 / (8.314 * 280)) = exp(-4.29) ≈ 0.0136
        let rate = ArrheniusRate::calculate(50.0, 1.0);
        assert!(
            (rate.rate - 0.0136).abs() < 0.002,
            "Arrhenius golden value: expected ~0.0136, got {}",
            rate.rate
        );
    }

    #[test]
    fn test_arrhenius_high_temp_golden() {
        // At complexity=50 (Ea=10000), urgency=10 (T=550):
        // k = exp(-10000 / (8.314 * 550)) = exp(-2.19) ≈ 0.112
        let rate = ArrheniusRate::calculate(50.0, 10.0);
        assert!(
            (rate.rate - 0.112).abs() < 0.01,
            "Arrhenius high-T golden: expected ~0.112, got {}",
            rate.rate
        );
    }

    #[test]
    fn test_gibbs_golden_value() {
        // ΔG = ΔH - TΔS = 10 - (80 * 0.8) = 10 - 64 = -54
        let gibbs = GibbsFeasibility::calculate(10.0, 0.8, 8.0);
        assert!(
            (gibbs.delta_g - (-54.0)).abs() < 0.1,
            "Gibbs golden value: expected -54.0, got {}",
            gibbs.delta_g
        );
    }

    #[test]
    fn test_staleness_uses_exact_ln2() {
        // With exact LN_2 (0.693147...), at t=half_life: relevance = 0.5 exactly
        let decay = StalenessDecay::calculate(30.0, 30.0);
        // Should be within 0.001 of 0.5 (better precision than 0.693 approximation)
        assert!(
            (decay.relevance - 0.5).abs() < 0.001,
            "Exact LN_2 should give relevance=0.5, got {}",
            decay.relevance
        );
    }

    #[test]
    fn test_michaelis_menten_golden() {
        // v = Vmax * S / (Km + S) = 5 * 10 / (10 + 10) = 50/20 = 2.5
        let sat = CapacitySaturation::calculate(10, 5.0, 10.0);
        assert!(
            (sat.rate - 2.5).abs() < 0.001,
            "MM golden: expected 2.5, got {}",
            sat.rate
        );
        assert!(
            (sat.saturation - 0.5).abs() < 0.001,
            "MM saturation: expected 0.5, got {}",
            sat.saturation
        );
    }

    // =========================================================================
    // Edge Case Tests (Phase 1 - Safety)
    // Validate inputs are properly guarded
    // =========================================================================

    #[test]
    fn test_staleness_negative_days_clamped() {
        // Negative days should be clamped to 0, giving relevance = 1.0
        let decay = StalenessDecay::calculate(-30.0, 30.0);
        assert!(
            decay.relevance <= 1.0,
            "Negative days must not produce relevance > 1.0, got {}",
            decay.relevance
        );
        assert!(
            (decay.relevance - 1.0).abs() < 0.001,
            "Negative days should give relevance=1.0, got {}",
            decay.relevance
        );
    }

    #[test]
    fn test_staleness_zero_half_life_guarded() {
        // Zero half-life should be clamped to 1.0, not divide by zero
        let decay = StalenessDecay::calculate(10.0, 0.0);
        assert!(
            decay.relevance.is_finite(),
            "Zero half-life must not produce NaN/Inf"
        );
        assert!(
            decay.decay_constant.is_finite(),
            "Decay constant must be finite"
        );
    }

    #[test]
    fn test_saturation_zero_km_guarded() {
        // Zero Km should be clamped, not divide by zero
        let sat = CapacitySaturation::calculate(10, 5.0, 0.0);
        assert!(sat.rate.is_finite(), "Zero Km must not produce NaN/Inf");
        assert!(sat.saturation.is_finite(), "Saturation must be finite");
    }

    #[test]
    fn test_saturation_negative_vmax_guarded() {
        // Negative vmax should be clamped to minimum
        let sat = CapacitySaturation::calculate(10, -5.0, 10.0);
        assert!(
            sat.rate.is_finite(),
            "Negative vmax must produce finite rate"
        );
        assert!(sat.rate >= 0.0, "Rate cannot be negative");
    }

    #[test]
    fn test_arrhenius_negative_inputs_safe() {
        // Negative inputs should be clamped, producing finite results
        let rate = ArrheniusRate::calculate(-50.0, -10.0);
        assert!(
            rate.rate.is_finite(),
            "Negative inputs must produce finite rate"
        );
        assert!(rate.rate >= 0.0, "Rate cannot be negative");
    }

    #[test]
    fn test_gibbs_extreme_values_safe() {
        // Extreme values should produce finite results
        let gibbs = GibbsFeasibility::calculate(1000.0, 100.0, 1000.0);
        assert!(
            gibbs.delta_g.is_finite(),
            "Extreme inputs must produce finite ΔG"
        );
    }

    #[test]
    fn test_all_outputs_finite() {
        // Comprehensive check that no calculation produces NaN or Inf
        let score = ChemistryScore::calculate(50.0, 5.0, 30.0, 0.5, 15.0);

        assert!(score.kinetics.rate.is_finite());
        assert!(score.kinetics.activation_energy.is_finite());
        assert!(score.kinetics.temperature.is_finite());

        assert!(score.thermodynamics.delta_g.is_finite());
        assert!(score.thermodynamics.delta_h.is_finite());
        assert!(score.thermodynamics.delta_s.is_finite());

        assert!(score.staleness.relevance.is_finite());
        assert!(score.staleness.decay_constant.is_finite());
    }
}
