//! Session health monitoring via the χ (chi) branching ratio.
//!
//! ## Physical basis
//!
//! The χ metric is a cross-domain transfer from T Tauri stellar physics
//! (Serna et al. 2024, ApJ 968 68). In that model a young star mediates
//! accretion disc torques through three coupled terms:
//!
//! ```text
//! τ_acc + τ_wind + τ_disk = 0          (Eq. 9, Serna et al.)
//! ```
//!
//! The branching ratio χ = Ṁ_wind / Ṁ_acc (their Eq. 7) characterises
//! whether the system is spinning up (low χ), in equilibrium (0.02–0.27),
//! depleting (0.27–0.60), or past the Accelerating Propeller / Slow Wind
//! (APSW) thermodynamic limit (χ > 0.60).
//!
//! ## Terminal mapping
//!
//! | Stellar quantity | Terminal analog |
//! |-----------------|-----------------|
//! | Ṁ_acc (accretion rate) | `input_events` — commands, keystrokes, API calls arriving |
//! | Ṁ_wind (wind rate) | `output_events` — lines rendered, responses emitted |
//! | τ_disk (disc coupling) | `backpressure` — renderer / PTY write-buffer backpressure |
//!
//! A healthy terminal session sits in the equilibrium band (χ ∈ [0.02, 0.27]).
//! Persistent deviation indicates either a stalled renderer (χ ≈ 0) or an
//! output storm that will exhaust buffers (χ > 0.60).
//!
//! ## Example
//!
//! ```rust
//! use nexcore_terminal::health::{SessionHealth, HealthBand};
//!
//! let health = SessionHealth::new(100, 8, 0.05);
//! assert_eq!(health.band, HealthBand::Healthy);
//! assert!(health.chi > 0.0);
//! ```

use serde::{Deserialize, Serialize};

/// Equilibrium threshold for |τ_total| = |τ_acc + τ_wind + τ_disk|.
///
/// Below this value the three-torque sum is considered balanced.
const EQUILIBRIUM_THRESHOLD: f64 = 0.1;

/// χ thresholds derived from Table 2 of Serna et al. 2024, ApJ 968 68.
const CHI_SPIN_UP_MAX: f64 = 0.02;
const CHI_HEALTHY_MAX: f64 = 0.27;
const CHI_DEPLETING_MAX: f64 = 0.60;

/// Health band classification derived from the T Tauri χ equilibrium
/// boundaries (Serna et al. 2024, ApJ 968 68, Table 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthBand {
    /// χ < 0.02 — output is far below input; the session is consuming
    /// events faster than it can render them (stellar analog: spinning up).
    SpinningUp,
    /// 0.02 ≤ χ ≤ 0.27 — input/output rates are balanced (equilibrium band).
    Healthy,
    /// 0.27 < χ ≤ 0.60 — outputting more events than the sustainable
    /// rate; buffer depletion risk (stellar analog: strong wind phase).
    Depleting,
    /// χ > 0.60 — past the APSW thermodynamic limit; session is at risk
    /// of render-storm or buffer exhaustion (stellar analog: propeller phase).
    Critical,
}

impl HealthBand {
    /// Classify a χ value into a health band.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_terminal::health::HealthBand;
    ///
    /// assert_eq!(HealthBand::from_chi(0.01), HealthBand::SpinningUp);
    /// assert_eq!(HealthBand::from_chi(0.04), HealthBand::Healthy);
    /// assert_eq!(HealthBand::from_chi(0.40), HealthBand::Depleting);
    /// assert_eq!(HealthBand::from_chi(0.70), HealthBand::Critical);
    /// ```
    #[must_use]
    pub fn from_chi(chi: f64) -> Self {
        if chi < CHI_SPIN_UP_MAX {
            Self::SpinningUp
        } else if chi <= CHI_HEALTHY_MAX {
            Self::Healthy
        } else if chi <= CHI_DEPLETING_MAX {
            Self::Depleting
        } else {
            Self::Critical
        }
    }

    /// Human-readable label for the band.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::SpinningUp => "spinning_up",
            Self::Healthy => "healthy",
            Self::Depleting => "depleting",
            Self::Critical => "critical",
        }
    }
}

/// Session health snapshot derived from the T Tauri three-torque model.
///
/// Torque balance: τ_acc + τ_wind + τ_disk = 0 (Eq. 9, Serna et al. 2024).
///
/// All three torques are normalised to the range [-1, 1] so that the
/// equilibrium condition |τ_total| < [`EQUILIBRIUM_THRESHOLD`] is
/// dimensionally consistent regardless of raw event counts.
///
/// # Examples
///
/// ```rust
/// use nexcore_terminal::health::{SessionHealth, HealthBand};
///
/// let h = SessionHealth::new(200, 20, 0.02);
/// assert_eq!(h.band, HealthBand::Healthy);
/// assert!((h.chi - 0.1).abs() < f64::EPSILON);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHealth {
    /// Branching ratio χ = output_events / input_events.
    ///
    /// Stellar analog: Ṁ_wind / Ṁ_acc (Eq. 7, Serna et al. 2024).
    /// Zero when `input_events == 0` (avoids divide-by-zero).
    pub chi: f64,

    /// Input torque τ_acc (accretion) — normalised event input rate.
    ///
    /// Positive when events are arriving; computed as `1.0 - chi` so that
    /// accretion dominates when χ is small.
    pub tau_acc: f64,

    /// Output torque τ_wind (wind) — normalised event output rate.
    ///
    /// Equals `chi` in the normalised frame.
    pub tau_wind: f64,

    /// Backpressure torque τ_disk (disc coupling) — renderer constraint.
    ///
    /// Provided directly as the write-buffer backpressure signal (0.0 = none,
    /// 1.0 = fully stalled). Applied with a negative sign (opposing wind)
    /// to maintain dimensional consistency with Eq. 9.
    pub tau_disk: f64,

    /// Whether the session is in torque equilibrium.
    ///
    /// True when |τ_acc + τ_wind + τ_disk| < `EQUILIBRIUM_THRESHOLD` (0.1).
    pub equilibrium: bool,

    /// Health band classification for this χ value.
    pub band: HealthBand,
}

impl SessionHealth {
    /// Build a health snapshot from raw event counters and backpressure.
    ///
    /// # Arguments
    ///
    /// * `input_count`  — total input events (keystrokes, API calls received)
    /// * `output_count` — total output events (lines rendered, responses sent)
    /// * `backpressure` — write-buffer pressure in [0.0, 1.0]; 0.0 = no pressure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_terminal::health::{SessionHealth, HealthBand};
    ///
    /// let h = SessionHealth::new(0, 0, 0.0);
    /// assert_eq!(h.band, HealthBand::SpinningUp);
    /// assert_eq!(h.chi, 0.0);
    /// ```
    #[must_use]
    pub fn new(input_count: u64, output_count: u64, backpressure: f64) -> Self {
        let chi = Self::chi_from_counts(input_count, output_count);

        // Normalised torques (Eq. 9 frame).
        // τ_acc = accretion dominance = 1 - χ  (positive when χ is small)
        // τ_wind = wind output       = χ        (positive when χ is large)
        // τ_disk = disc backpressure = -backpressure (opposes wind, clamp [0,1])
        let tau_acc = 1.0 - chi;
        let tau_wind = chi;
        let tau_disk = -(backpressure.clamp(0.0, 1.0));

        let tau_total = (tau_acc + tau_wind + tau_disk).abs();
        let equilibrium = tau_total < EQUILIBRIUM_THRESHOLD;
        let band = HealthBand::from_chi(chi);

        Self {
            chi,
            tau_acc,
            tau_wind,
            tau_disk,
            equilibrium,
            band,
        }
    }

    /// Compute χ = output / input, returning 0.0 when input is zero.
    ///
    /// This is the core branching ratio (Eq. 7, Serna et al. 2024). The
    /// zero-input case maps to `SpinningUp` by definition: no events have
    /// arrived so there is nothing to output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_terminal::health::SessionHealth;
    ///
    /// assert_eq!(SessionHealth::chi_from_counts(0, 0), 0.0);
    /// assert_eq!(SessionHealth::chi_from_counts(100, 10), 0.1);
    /// ```
    #[must_use]
    pub fn chi_from_counts(input: u64, output: u64) -> f64 {
        if input == 0 {
            return 0.0;
        }
        output as f64 / input as f64
    }

    /// Whether the session is in torque equilibrium.
    ///
    /// Returns `true` when |τ_acc + τ_wind + τ_disk| < 0.1, mirroring the
    /// equilibrium condition in Eq. 9 of Serna et al. 2024.
    ///
    /// This method is a convenience accessor; the same value is stored in the
    /// [`SessionHealth::equilibrium`] field.
    #[must_use]
    pub fn is_equilibrium(&self) -> bool {
        self.equilibrium
    }

    /// One-line human-readable summary of the session health state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_terminal::health::SessionHealth;
    ///
    /// let h = SessionHealth::new(100, 5, 0.0);
    /// let s = h.summary();
    /// assert!(s.contains("healthy"));
    /// assert!(s.contains("χ="));
    /// ```
    #[must_use]
    pub fn summary(&self) -> String {
        let eq_marker = if self.equilibrium { "eq" } else { "!eq" };
        format!(
            "band={} χ={:.4} τ_acc={:.3} τ_wind={:.3} τ_disk={:.3} [{}]",
            self.band.label(),
            self.chi,
            self.tau_acc,
            self.tau_wind,
            self.tau_disk,
            eq_marker,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- HealthBand::from_chi ---

    #[test]
    fn test_healthy_band() {
        // χ = 0.04 sits in the equilibrium range [0.02, 0.27]
        assert_eq!(HealthBand::from_chi(0.04), HealthBand::Healthy);
    }

    #[test]
    fn test_healthy_band_boundary_low() {
        // χ = 0.02 is the lower boundary (inclusive)
        assert_eq!(HealthBand::from_chi(0.02), HealthBand::Healthy);
    }

    #[test]
    fn test_healthy_band_boundary_high() {
        // χ = 0.27 is the upper boundary (inclusive)
        assert_eq!(HealthBand::from_chi(0.27), HealthBand::Healthy);
    }

    #[test]
    fn test_spinning_up() {
        // χ = 0.01 < 0.02 → SpinningUp
        assert_eq!(HealthBand::from_chi(0.01), HealthBand::SpinningUp);
    }

    #[test]
    fn test_depleting() {
        // χ = 0.40 ∈ (0.27, 0.60] → Depleting
        assert_eq!(HealthBand::from_chi(0.40), HealthBand::Depleting);
    }

    #[test]
    fn test_critical() {
        // χ = 0.70 > 0.60 → Critical
        assert_eq!(HealthBand::from_chi(0.70), HealthBand::Critical);
    }

    // --- SessionHealth::chi_from_counts ---

    #[test]
    fn test_zero_input() {
        // Zero input must not panic and must return χ = 0.0 → SpinningUp
        let chi = SessionHealth::chi_from_counts(0, 0);
        assert_eq!(chi, 0.0);
        assert_eq!(HealthBand::from_chi(chi), HealthBand::SpinningUp);
    }

    #[test]
    fn test_zero_input_with_output() {
        // Even if output is non-zero, zero input → χ = 0.0
        let chi = SessionHealth::chi_from_counts(0, 99);
        assert_eq!(chi, 0.0);
    }

    #[test]
    fn test_chi_ratio_precision() {
        // 10 output / 100 input = 0.1
        let chi = SessionHealth::chi_from_counts(100, 10);
        assert!((chi - 0.1).abs() < f64::EPSILON);
    }

    // --- SessionHealth::new ---

    #[test]
    fn test_equilibrium() {
        // τ_acc = 1 - 0.1 = 0.9, τ_wind = 0.1, τ_disk = -0.0 = 0.0
        // |τ_total| = |0.9 + 0.1 + 0.0| = 1.0  — NOT equilibrium
        //
        // Use backpressure = 0.9 to bring the sum close to zero:
        // τ_acc = 0.9, τ_wind = 0.1, τ_disk = -0.9 → sum = 0.1 → NOT eq (= threshold)
        //
        // Instead calibrate: input=100, output=5 → χ=0.05
        // τ_acc=0.95, τ_wind=0.05, τ_disk=-backpressure
        // For eq: |0.95 + 0.05 - bp| < 0.1  → |1.0 - bp| < 0.1 → bp ∈ (0.9, 1.1)
        // Use bp = 0.95: |1.0 - 0.95| = 0.05 < 0.1 → equilibrium = true
        let h = SessionHealth::new(100, 5, 0.95);
        assert!(h.equilibrium, "expected equilibrium with bp=0.95, χ=0.05");
        assert_eq!(h.band, HealthBand::Healthy);
    }

    #[test]
    fn test_not_equilibrium_no_backpressure() {
        // No backpressure: τ_acc=0.9, τ_wind=0.1, τ_disk=0.0
        // |τ_total| = 1.0 > 0.1 → NOT equilibrium
        let h = SessionHealth::new(100, 10, 0.0);
        assert!(!h.equilibrium);
    }

    #[test]
    fn test_zero_input_session_health() {
        // Zero input → χ = 0.0, SpinningUp, no panic
        let h = SessionHealth::new(0, 0, 0.0);
        assert_eq!(h.chi, 0.0);
        assert_eq!(h.band, HealthBand::SpinningUp);
    }

    #[test]
    fn test_summary_contains_band_and_chi() {
        let h = SessionHealth::new(100, 5, 0.0);
        let s = h.summary();
        assert!(s.contains("healthy"), "summary should contain band label");
        assert!(s.contains("χ="), "summary should contain chi marker");
    }

    #[test]
    fn test_is_equilibrium_matches_field() {
        let h = SessionHealth::new(100, 5, 0.95);
        assert_eq!(h.is_equilibrium(), h.equilibrium);
    }

    #[test]
    fn test_chi_exactly_at_spin_up_boundary() {
        // χ = 0.02 → boundary between SpinningUp and Healthy → Healthy (inclusive)
        let h = SessionHealth::new(100, 2, 0.0);
        assert_eq!(h.band, HealthBand::Healthy);
        assert!((h.chi - 0.02).abs() < f64::EPSILON);
    }

    #[test]
    fn test_critical_band_torques() {
        // 70 output / 100 input → χ = 0.70 → Critical
        let h = SessionHealth::new(100, 70, 0.0);
        assert_eq!(h.band, HealthBand::Critical);
        assert!((h.chi - 0.70).abs() < f64::EPSILON);
        // τ_wind > τ_acc in this regime
        assert!(h.tau_wind > h.tau_acc);
    }
}
