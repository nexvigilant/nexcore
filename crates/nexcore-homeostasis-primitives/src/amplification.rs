//! Paired amplification system — enforces Law 1: every amplifier needs a paired attenuator.
//!
//! ## Design rule
//!
//! A system with an amplifier and no attenuator *will* eventually storm.  This
//! module makes that architectural mistake impossible: [`Amplifier::amplify`]
//! returns an [`AmplificationViolation`] error at call time unless the amplifier
//! was registered via [`PairedAmplificationSystem::register_pair`], and
//! registration itself validates that the attenuator gain is at least as large as
//! the amplifier gain.
//!
//! ## Biological analogy
//!
//! | Biology | Software |
//! |---------|----------|
//! | IL-6 (amplifier) | Retry logic |
//! | IL-10 (attenuator) | Exponential back-off + circuit breaker |
//! | Complement regulatory proteins | Rate limiters |
//!
//! ## Example
//!
//! ```
//! use nexcore_homeostasis_primitives::amplification::{
//!     Amplifier, Attenuator, PairedAmplificationSystem, create_standard_pair,
//! };
//!
//! let (amp, att) = create_standard_pair("retry", 2.0, 2.5, 100.0);
//! let mut sys = PairedAmplificationSystem::new();
//! sys.register_pair(amp, att).unwrap();
//!
//! let amplified = sys.amplify("retry_amplifier", 20.0).unwrap();
//! assert!(amplified <= 100.0);
//! ```

use nexcore_error::Error;
use std::collections::HashMap;

// =============================================================================
// Error type
// =============================================================================

/// Raised when amplification design rules are violated.
///
/// This indicates a *design* error, not a transient runtime failure.  If you
/// see this error, the system architecture is missing a required attenuator or
/// the attenuator gain is too weak relative to its paired amplifier.
#[derive(Debug, Error)]
pub enum AmplificationViolation {
    /// An amplifier was used without being registered through [`PairedAmplificationSystem`].
    #[error(
        "Amplifier '{name}' has no paired attenuator. \
         Register it through PairedAmplificationSystem."
    )]
    NoPairedAttenuator {
        /// Name of the offending amplifier.
        name: String,
    },

    /// The attenuator gain is weaker than its paired amplifier gain.
    #[error(
        "Attenuator '{att_name}' (gain={att_gain}) cannot keep pace with \
         amplifier '{amp_name}' (gain={amp_gain}). \
         Attenuator gain MUST be >= amplifier gain — a storm is guaranteed otherwise."
    )]
    AttenuatorTooWeak {
        /// Name of the amplifier.
        amp_name: String,
        /// Gain of the amplifier.
        amp_gain: f64,
        /// Name of the attenuator.
        att_name: String,
        /// Gain of the attenuator.
        att_gain: f64,
    },

    /// No component with the given name was found.
    #[error("No {component} named '{name}' registered in this system.")]
    NotFound {
        /// `"amplifier"` or `"attenuator"`.
        component: &'static str,
        /// The requested name.
        name: String,
    },
}

// =============================================================================
// Amplifier
// =============================================================================

/// An amplifier that increases system response.
///
/// Amplifiers are essential — without them the system cannot mount a
/// proportional response to growing threats.  They are dangerous without a
/// paired [`Attenuator`]: calling [`amplify`](Self::amplify) before the
/// amplifier is registered through [`PairedAmplificationSystem`] returns an
/// [`AmplificationViolation`] error.
///
/// Three ceilings are applied in order:
/// 1. `max_rate` — maximum increase per call (rate limiting)
/// 2. `ceiling` — absolute maximum output
#[derive(Clone, Debug)]
pub struct Amplifier {
    /// Unique name for this amplifier.
    pub name: String,
    /// Multiplication factor applied to the input value.
    pub gain: f64,
    /// Maximum increase in output allowed per [`amplify`](Self::amplify) call.
    pub max_rate: f64,
    /// Absolute maximum output value.
    pub ceiling: f64,
    /// Current output level (updated on each call).
    pub current_output: f64,
    /// Cumulative total amplified across all calls.
    pub total_amplified: f64,
    /// Whether this amplifier has been paired (set by [`PairedAmplificationSystem`]).
    pub(crate) is_paired: bool,
}

impl Amplifier {
    /// Create an unpaired amplifier.
    ///
    /// You must register it via [`PairedAmplificationSystem::register_pair`]
    /// before calling [`amplify`](Self::amplify).
    pub fn new(name: impl Into<String>, gain: f64, ceiling: f64) -> Self {
        Self {
            name: name.into(),
            gain,
            max_rate: 10.0,
            ceiling,
            current_output: 0.0,
            total_amplified: 0.0,
            is_paired: false,
        }
    }

    /// Builder: set `max_rate` (maximum increase per call).
    pub fn with_max_rate(mut self, max_rate: f64) -> Self {
        self.max_rate = max_rate;
        self
    }

    /// Amplify `input_value`, subject to rate limiting and ceiling.
    ///
    /// Returns [`AmplificationViolation::NoPairedAttenuator`] if the amplifier
    /// has not been registered with a [`PairedAmplificationSystem`].
    pub fn amplify(&mut self, input_value: f64) -> Result<f64, AmplificationViolation> {
        if !self.is_paired {
            return Err(AmplificationViolation::NoPairedAttenuator {
                name: self.name.clone(),
            });
        }

        // Desired output from gain.
        let mut desired = input_value * self.gain;

        // Rate limiting: cap the increase per call.
        let max_step = self.current_output + self.max_rate;
        if desired > max_step {
            desired = max_step;
        }

        // Hard ceiling.
        if desired > self.ceiling {
            tracing::warn!(
                name = %self.name,
                ceiling = self.ceiling,
                "amplifier hit ceiling"
            );
            desired = self.ceiling;
        }

        self.current_output = desired;
        self.total_amplified += desired;
        Ok(desired)
    }

    /// Whether the amplifier is at (≥ 99 %) of its ceiling.
    pub fn is_at_ceiling(&self) -> bool {
        self.current_output >= self.ceiling * 0.99
    }

    /// Current output as a fraction of ceiling (0–1).
    pub fn utilization(&self) -> f64 {
        if self.ceiling > 0.0 {
            self.current_output / self.ceiling
        } else {
            0.0
        }
    }

    /// Reset internal state (current output and total).
    pub fn reset(&mut self) {
        self.current_output = 0.0;
        self.total_amplified = 0.0;
    }
}

// =============================================================================
// Attenuator
// =============================================================================

/// An attenuator that decreases system response — the brakes of the system.
///
/// The attenuation formula is: `output = input × dampening_factor` where
/// `dampening_factor = 1 / gain`.  For `gain = 2.5` a signal of 100 becomes 40.
///
/// A minimum dampening rate (`min_rate`) ensures that in emergency situations
/// *at least* `min_rate` units are removed per call, even if the dampening
/// factor alone would remove less.
#[derive(Clone, Debug)]
pub struct Attenuator {
    /// Unique name for this attenuator.
    pub name: String,
    /// Attenuation gain.  Must be >= its paired amplifier's gain.
    pub gain: f64,
    /// Minimum dampening applied per [`attenuate`](Self::attenuate) call.
    pub min_rate: f64,
    /// Minimum output value (floor — usually 0.0).
    pub floor: f64,
    /// Current output level (updated on each call).
    pub current_output: f64,
    /// Cumulative total dampened (sum of `input − output` per call).
    pub total_attenuated: f64,
}

impl Attenuator {
    /// Create an attenuator.
    pub fn new(name: impl Into<String>, gain: f64) -> Self {
        Self {
            name: name.into(),
            gain,
            min_rate: 5.0,
            floor: 0.0,
            current_output: 0.0,
            total_attenuated: 0.0,
        }
    }

    /// Builder: set `min_rate`.
    pub fn with_min_rate(mut self, min_rate: f64) -> Self {
        self.min_rate = min_rate;
        self
    }

    /// Builder: set output `floor`.
    pub fn with_floor(mut self, floor: f64) -> Self {
        self.floor = floor;
        self
    }

    /// Dampening factor (`1 / gain`).
    pub fn dampening_factor(&self) -> f64 {
        if self.gain > 0.0 {
            1.0 / self.gain
        } else {
            1.0
        }
    }

    /// Attenuate `input_value`.
    ///
    /// Applies dampening factor, then guarantees at least `min_rate` of
    /// reduction (when input is large enough), then clamps to `floor`.
    pub fn attenuate(&mut self, input_value: f64) -> f64 {
        let mut desired = input_value * self.dampening_factor();

        // Enforce minimum dampening rate.
        if input_value > self.min_rate && (input_value - desired) < self.min_rate {
            desired = input_value - self.min_rate;
        }

        // Floor.
        desired = desired.max(self.floor);

        self.total_attenuated += input_value - desired;
        self.current_output = desired;
        desired
    }

    /// Emergency dampening: cut signal by `factor` immediately, bypassing
    /// the normal dampening-factor calculation.
    ///
    /// Used when a storm is detected or imminent.
    pub fn emergency_dampen(&mut self, input_value: f64, factor: f64) -> f64 {
        let result = (input_value * factor).max(self.floor);
        tracing::warn!(
            name = %self.name,
            input = input_value,
            output = result,
            "emergency dampening applied"
        );
        self.total_attenuated += input_value - result;
        self.current_output = result;
        result
    }

    /// Reset internal state.
    pub fn reset(&mut self) {
        self.current_output = 0.0;
        self.total_attenuated = 0.0;
    }
}

// =============================================================================
// PairStats
// =============================================================================

/// Per-pair statistics returned by [`PairedAmplificationSystem::get_statistics`].
#[derive(Clone, Debug)]
pub struct AmplifierStats {
    /// Amplifier gain.
    pub gain: f64,
    /// Ceiling value.
    pub ceiling: f64,
    /// Current output.
    pub current_output: f64,
    /// Utilization fraction (0–1).
    pub utilization: f64,
    /// Whether at ceiling.
    pub at_ceiling: bool,
    /// Total amplified.
    pub total_amplified: f64,
}

/// Per-attenuator statistics returned by [`PairedAmplificationSystem::get_statistics`].
#[derive(Clone, Debug)]
pub struct AttenuatorStats {
    /// Attenuator gain.
    pub gain: f64,
    /// Dampening factor (1/gain).
    pub dampening_factor: f64,
    /// Current output.
    pub current_output: f64,
    /// Total dampened.
    pub total_attenuated: f64,
}

/// System-wide statistics snapshot.
#[derive(Clone, Debug)]
pub struct SystemStats {
    /// Number of registered pairs.
    pub pairs: usize,
    /// Per-amplifier stats keyed by name.
    pub amplifiers: HashMap<String, AmplifierStats>,
    /// Per-attenuator stats keyed by name.
    pub attenuators: HashMap<String, AttenuatorStats>,
}

// =============================================================================
// PairedAmplificationSystem
// =============================================================================

/// Registry that enforces paired amplifier/attenuator registration.
///
/// This is the architectural enforcement point for Law 1.  You cannot register
/// an amplifier without its paired attenuator, and the attenuator's gain must be
/// >= the amplifier's gain.
///
/// ```
/// use nexcore_homeostasis_primitives::amplification::{
///     Amplifier, Attenuator, PairedAmplificationSystem,
/// };
///
/// let mut sys = PairedAmplificationSystem::new();
///
/// // Weaker attenuator is rejected.
/// let amp = Amplifier::new("scale_up", 3.0, 100.0);
/// let weak_att = Attenuator::new("scale_down", 2.0); // gain < amp gain
/// assert!(sys.register_pair(amp, weak_att).is_err());
///
/// // Correct pairing succeeds.
/// let amp2 = Amplifier::new("scale_up2", 2.0, 100.0);
/// let att2 = Attenuator::new("scale_down2", 2.5);
/// assert!(sys.register_pair(amp2, att2).is_ok());
/// ```
#[derive(Debug, Default)]
pub struct PairedAmplificationSystem {
    /// All registered pairs, keyed by `"<amp_name>:<att_name>"`.
    pairs: HashMap<String, (Amplifier, Attenuator)>,
}

impl PairedAmplificationSystem {
    /// Create an empty system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an amplifier/attenuator pair.
    ///
    /// Validates that `attenuator.gain >= amplifier.gain`.  On success marks the
    /// amplifier as paired (so [`Amplifier::amplify`] will work) and returns the
    /// canonical pair ID `"<amp_name>:<att_name>"`.
    ///
    /// Returns [`AmplificationViolation::AttenuatorTooWeak`] if the gain check
    /// fails.
    pub fn register_pair(
        &mut self,
        mut amplifier: Amplifier,
        attenuator: Attenuator,
    ) -> Result<String, AmplificationViolation> {
        if attenuator.gain < amplifier.gain {
            return Err(AmplificationViolation::AttenuatorTooWeak {
                amp_name: amplifier.name.clone(),
                amp_gain: amplifier.gain,
                att_name: attenuator.name.clone(),
                att_gain: attenuator.gain,
            });
        }

        if (attenuator.gain - amplifier.gain).abs() < f64::EPSILON {
            tracing::warn!(
                amp = %amplifier.name,
                att = %attenuator.name,
                "attenuator and amplifier have equal gain — consider making attenuator slightly stronger"
            );
        }

        amplifier.is_paired = true;

        let pair_id = format!("{}:{}", amplifier.name, attenuator.name);
        tracing::info!(
            pair_id = %pair_id,
            amp_gain = amplifier.gain,
            att_gain = attenuator.gain,
            "registered amplifier/attenuator pair"
        );

        self.pairs.insert(pair_id.clone(), (amplifier, attenuator));
        Ok(pair_id)
    }

    /// Look up an amplifier by name.
    pub fn get_amplifier(&self, name: &str) -> Option<&Amplifier> {
        self.pairs
            .values()
            .find(|(amp, _)| amp.name == name)
            .map(|(amp, _)| amp)
    }

    /// Look up an attenuator by name.
    pub fn get_attenuator(&self, name: &str) -> Option<&Attenuator> {
        self.pairs
            .values()
            .find(|(_, att)| att.name == name)
            .map(|(_, att)| att)
    }

    /// Get a pair by its canonical `"<amp_name>:<att_name>"` ID.
    pub fn get_pair(&self, pair_id: &str) -> Option<(&Amplifier, &Attenuator)> {
        self.pairs.get(pair_id).map(|(a, b)| (a, b))
    }

    /// Amplify `value` using the named amplifier.
    ///
    /// Returns [`AmplificationViolation::NotFound`] if no amplifier with that
    /// name is registered.
    pub fn amplify(&mut self, amp_name: &str, value: f64) -> Result<f64, AmplificationViolation> {
        let pair = self
            .pairs
            .values_mut()
            .find(|(amp, _)| amp.name == amp_name);

        match pair {
            Some((amp, _)) => amp.amplify(value),
            None => Err(AmplificationViolation::NotFound {
                component: "amplifier",
                name: amp_name.to_string(),
            }),
        }
    }

    /// Attenuate `value` using the named attenuator.
    ///
    /// Returns [`AmplificationViolation::NotFound`] if no attenuator with that
    /// name is registered.
    pub fn attenuate(&mut self, att_name: &str, value: f64) -> Result<f64, AmplificationViolation> {
        let pair = self
            .pairs
            .values_mut()
            .find(|(_, att)| att.name == att_name);

        match pair {
            Some((_, att)) => Ok(att.attenuate(value)),
            None => Err(AmplificationViolation::NotFound {
                component: "attenuator",
                name: att_name.to_string(),
            }),
        }
    }

    /// Primary decision point: amplify, attenuate, or maintain based on
    /// the ratio of response to threat.
    ///
    /// - `proportionality > proportionality_threshold` → attenuate via first registered attenuator
    /// - `proportionality < 1.0` → amplify via first registered amplifier
    /// - otherwise → return `current_response` unchanged
    ///
    /// When no threat is present (`threat_level < 0.01`) any existing response
    /// is attenuated back toward zero.
    pub fn process_response(
        &mut self,
        current_response: f64,
        threat_level: f64,
        proportionality_threshold: f64,
    ) -> f64 {
        if threat_level < 0.01 {
            if current_response > 0.0 {
                // Find the first attenuator and dampen.
                if let Some((_, att)) = self.pairs.values_mut().next() {
                    return att.attenuate(current_response);
                }
            }
            return 0.0;
        }

        let proportionality = current_response / threat_level;

        if proportionality > proportionality_threshold {
            // Over-responding — attenuate.
            if let Some((_, att)) = self.pairs.values_mut().next() {
                return att.attenuate(current_response);
            }
        } else if proportionality < 1.0 {
            // Under-responding — amplify.
            let input = if current_response > 0.0 {
                current_response
            } else {
                threat_level
            };
            if let Some((amp, _)) = self.pairs.values_mut().next() {
                // Amplify returns Ok always when paired; fall back gracefully.
                return amp.amplify(input).unwrap_or(current_response);
            }
        }

        // Proportional — maintain.
        current_response
    }

    /// Emergency dampening through all registered attenuators in sequence.
    ///
    /// Each attenuator cuts the running value by `factor`.  Used when storm is
    /// detected.
    pub fn emergency_dampen_all(&mut self, current_response: f64, factor: f64) -> f64 {
        let mut result = current_response;
        for (_, att) in self.pairs.values_mut() {
            result = att.emergency_dampen(result, factor);
        }
        result
    }

    /// Snapshot of current amplification/attenuation statistics.
    pub fn get_statistics(&self) -> SystemStats {
        let mut amplifiers = HashMap::new();
        let mut attenuators = HashMap::new();

        for (amp, att) in self.pairs.values() {
            amplifiers.insert(
                amp.name.clone(),
                AmplifierStats {
                    gain: amp.gain,
                    ceiling: amp.ceiling,
                    current_output: amp.current_output,
                    utilization: amp.utilization(),
                    at_ceiling: amp.is_at_ceiling(),
                    total_amplified: amp.total_amplified,
                },
            );
            attenuators.insert(
                att.name.clone(),
                AttenuatorStats {
                    gain: att.gain,
                    dampening_factor: att.dampening_factor(),
                    current_output: att.current_output,
                    total_attenuated: att.total_attenuated,
                },
            );
        }

        SystemStats {
            pairs: self.pairs.len(),
            amplifiers,
            attenuators,
        }
    }

    /// Number of registered pairs.
    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }
}

// =============================================================================
// create_standard_pair
// =============================================================================

/// Create a standard amplifier/attenuator pair ready for registration.
///
/// The attenuation gain defaults to 25 % above the amplification gain when
/// an equal or weaker value is supplied, ensuring the system can always brake
/// faster than it accelerates.
///
/// ```
/// use nexcore_homeostasis_primitives::amplification::{
///     PairedAmplificationSystem, create_standard_pair,
/// };
///
/// let (amp, att) = create_standard_pair("autoscale", 2.0, 2.5, 100.0);
/// assert_eq!(amp.name, "autoscale_amplifier");
/// assert_eq!(att.name, "autoscale_attenuator");
/// assert!(att.gain >= amp.gain);
///
/// let mut sys = PairedAmplificationSystem::new();
/// sys.register_pair(amp, att).unwrap();
/// assert_eq!(sys.pair_count(), 1);
/// ```
pub fn create_standard_pair(
    name: &str,
    amplification_gain: f64,
    attenuation_gain: f64,
    ceiling: f64,
) -> (Amplifier, Attenuator) {
    let safe_att_gain = if attenuation_gain < amplification_gain {
        let adjusted = amplification_gain * 1.25;
        tracing::warn!(
            adjusted,
            amplification_gain,
            "attenuation_gain too low — adjusted to maintain safety margin"
        );
        adjusted
    } else {
        attenuation_gain
    };

    let amplifier = Amplifier::new(format!("{name}_amplifier"), amplification_gain, ceiling);
    let attenuator = Attenuator::new(format!("{name}_attenuator"), safe_att_gain);

    (amplifier, attenuator)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn paired_system(amp_gain: f64, att_gain: f64) -> PairedAmplificationSystem {
        let (amp, att) = create_standard_pair("test", amp_gain, att_gain, 200.0);
        let mut sys = PairedAmplificationSystem::new();
        sys.register_pair(amp, att).unwrap();
        sys
    }

    // ── Amplifier ─────────────────────────────────────────────────────────────

    #[test]
    fn amplifier_unpaired_returns_error() {
        let mut amp = Amplifier::new("solo", 2.0, 100.0);
        let result = amp.amplify(10.0);
        assert!(matches!(
            result,
            Err(AmplificationViolation::NoPairedAttenuator { .. })
        ));
    }

    #[test]
    fn amplifier_applies_gain() {
        let mut sys = paired_system(2.0, 2.5);
        let result = sys.amplify("test_amplifier", 20.0).unwrap();
        // 20 × 2.0 = 40, well under ceiling and rate limit from 0 → 0 + 10.
        // Rate limit: current=0, max_step = 0 + 10 = 10; desired=40 → clamped to 10.
        assert_eq!(result, 10.0);
    }

    #[test]
    fn amplifier_respects_ceiling() {
        let (mut amp, att) = create_standard_pair("big", 2.0, 2.5, 50.0);
        amp.is_paired = true;
        // After several calls the current_output rises toward ceiling.
        for _ in 0..20 {
            let _ = amp.amplify(1000.0);
        }
        assert!(amp.current_output <= 50.0);
        assert!(amp.is_at_ceiling());
    }

    #[test]
    fn amplifier_rate_limits_increase() {
        let mut sys = paired_system(2.0, 2.5);
        // First call from current=0 → capped at max_rate=10.
        let r1 = sys.amplify("test_amplifier", 500.0).unwrap();
        assert_eq!(r1, 10.0, "first call rate-limited to 10");
        // Second call: current=10, max_step=20.
        let r2 = sys.amplify("test_amplifier", 500.0).unwrap();
        assert_eq!(r2, 20.0, "second call rate-limited to 20");
    }

    // ── Attenuator ────────────────────────────────────────────────────────────

    #[test]
    fn attenuator_dampening_factor() {
        let att = Attenuator::new("brake", 2.5);
        assert!((att.dampening_factor() - 0.4).abs() < 1e-10);
    }

    #[test]
    fn attenuator_applies_dampening() {
        let mut att = Attenuator::new("brake", 2.5);
        // 100 × 0.4 = 40 (reduction of 60, which is > min_rate=5).
        let result = att.attenuate(100.0);
        assert!((result - 40.0).abs() < 0.01, "attenuated: {result}");
        assert!((att.total_attenuated - 60.0).abs() < 0.01);
    }

    #[test]
    fn attenuator_enforces_minimum_rate() {
        // gain=1.01 → dampening_factor ≈ 0.99 → would only remove ≈1% per call.
        // min_rate=5 should ensure at least 5 is removed.
        let mut att = Attenuator::new("slow_brake", 1.01).with_min_rate(5.0);
        let result = att.attenuate(100.0);
        assert!(100.0 - result >= 5.0, "min_rate not enforced: {result}");
    }

    #[test]
    fn attenuator_floor_respected() {
        let mut att = Attenuator::new("floored", 100.0).with_floor(1.0);
        // gain=100 → dampening_factor=0.01 → 10 × 0.01 = 0.1 < floor=1.
        let result = att.attenuate(10.0);
        assert!(result >= 1.0, "floor not respected: {result}");
    }

    #[test]
    fn attenuator_emergency_dampen() {
        let mut att = Attenuator::new("brake", 2.5);
        let result = att.emergency_dampen(100.0, 0.5);
        assert!((result - 50.0).abs() < 0.01);
    }

    // ── PairedAmplificationSystem ─────────────────────────────────────────────

    #[test]
    fn register_pair_rejects_weak_attenuator() {
        let amp = Amplifier::new("amp", 3.0, 100.0);
        let weak_att = Attenuator::new("att", 2.0); // 2.0 < 3.0
        let mut sys = PairedAmplificationSystem::new();
        let result = sys.register_pair(amp, weak_att);
        assert!(matches!(
            result,
            Err(AmplificationViolation::AttenuatorTooWeak { .. })
        ));
    }

    #[test]
    fn register_pair_accepts_equal_gain() {
        let amp = Amplifier::new("amp", 2.0, 100.0);
        let att = Attenuator::new("att", 2.0); // equal is allowed
        let mut sys = PairedAmplificationSystem::new();
        assert!(sys.register_pair(amp, att).is_ok());
    }

    #[test]
    fn process_response_attenuates_when_over_proportional() {
        let mut sys = paired_system(2.0, 2.5);
        // response=100, threat=10 → proportionality=10 > threshold=3.
        let new_response = sys.process_response(100.0, 10.0, 3.0);
        assert!(
            new_response < 100.0,
            "should have attenuated: {new_response}"
        );
    }

    #[test]
    fn process_response_amplifies_when_under_responding() {
        let mut sys = paired_system(2.0, 2.5);
        // response=1, threat=100 → proportionality=0.01 < 1.0.
        let new_response = sys.process_response(1.0, 100.0, 3.0);
        assert!(
            new_response >= 1.0,
            "should have amplified or maintained: {new_response}"
        );
    }

    #[test]
    fn process_response_maintains_when_proportional() {
        let mut sys = paired_system(2.0, 2.5);
        // Set current_output so next amplify call isn't purely rate-limited from 0.
        // response=20, threat=10 → proportionality=2.0 < threshold=3 and > 1.0 → maintain.
        let new_response = sys.process_response(20.0, 10.0, 3.0);
        assert_eq!(new_response, 20.0);
    }

    #[test]
    fn emergency_dampen_all_halves_response() {
        let mut sys = paired_system(2.0, 2.5);
        let result = sys.emergency_dampen_all(100.0, 0.5);
        assert!(result < 100.0, "emergency dampen did nothing: {result}");
    }

    #[test]
    fn get_statistics_reflects_state() {
        let mut sys = paired_system(2.0, 2.5);
        let _ = sys.amplify("test_amplifier", 5.0);
        let stats = sys.get_statistics();
        assert_eq!(stats.pairs, 1);
        assert!(stats.amplifiers.contains_key("test_amplifier"));
        assert!(stats.attenuators.contains_key("test_attenuator"));
    }

    // ── create_standard_pair ──────────────────────────────────────────────────

    #[test]
    fn create_standard_pair_names() {
        let (amp, att) = create_standard_pair("retry", 2.0, 2.5, 100.0);
        assert_eq!(amp.name, "retry_amplifier");
        assert_eq!(att.name, "retry_attenuator");
    }

    #[test]
    fn create_standard_pair_adjusts_weak_attenuation() {
        // Pass att_gain < amp_gain → should be auto-corrected.
        let (amp, att) = create_standard_pair("x", 3.0, 1.0, 100.0);
        assert!(
            att.gain >= amp.gain,
            "att.gain={} amp.gain={}",
            att.gain,
            amp.gain
        );
    }

    #[test]
    fn create_standard_pair_full_registration() {
        let (amp, att) = create_standard_pair("autoscale", 2.0, 2.5, 100.0);
        let mut sys = PairedAmplificationSystem::new();
        sys.register_pair(amp, att).unwrap();
        assert_eq!(sys.pair_count(), 1);
    }
}
