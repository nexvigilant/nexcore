//! Hill curve response ceiling math — enforces Law 3: every response has a ceiling.
//!
//! The Hill equation `R = Rmax * (S^n / (K^n + S^n))` is the same mathematical
//! model used for enzyme kinetics and receptor-ligand binding in biology.
//! It guarantees `response < Rmax` for any finite signal, preventing infinite
//! amplification.

/// The Hill equation for saturating responses.
///
/// ```
/// use nexcore_homeostasis_primitives::hill::HillCurve;
///
/// let curve = HillCurve::new(100.0, 50.0, 2.0);
/// assert!(curve.calculate(0.0) < f64::EPSILON);   // zero signal → zero response
/// assert!((curve.calculate(50.0) - 50.0).abs() < 0.1); // K_half → half max
/// assert!(curve.calculate(1_000_000.0) < 100.0);  // never exceeds max
/// ```
#[derive(Clone, Debug)]
pub struct HillCurve {
    /// Rmax — the absolute response ceiling.
    pub max_response: f64,
    /// K — signal strength at half-maximal response.
    pub k_half: f64,
    /// n — Hill coefficient controlling steepness (1 = gradual, 4 = switch-like).
    pub hill_coefficient: f64,
}

impl HillCurve {
    /// Create a new Hill curve.
    pub fn new(max_response: f64, k_half: f64, hill_coefficient: f64) -> Self {
        Self { max_response, k_half, hill_coefficient }
    }

    /// Calculate the response for a given signal strength.
    ///
    /// The response is always strictly less than `max_response`.
    /// This is the mathematical guarantee of the ceiling.
    pub fn calculate(&self, signal_strength: f64) -> f64 {
        if signal_strength <= 0.0 {
            return 0.0;
        }
        let n = self.hill_coefficient;
        let k = self.k_half;
        let s = signal_strength;

        let s_n = s.powf(n);
        let k_n = k.powf(n);
        self.max_response * s_n / (k_n + s_n)
    }

    /// Calculate the signal strength required to produce a given response.
    ///
    /// Returns `f64::INFINITY` if `desired_response >= max_response`.
    pub fn inverse(&self, desired_response: f64) -> f64 {
        if desired_response <= 0.0 {
            return 0.0;
        }
        if desired_response >= self.max_response {
            return f64::INFINITY;
        }
        let n = self.hill_coefficient;
        let k = self.k_half;
        let r = desired_response;
        let r_max = self.max_response;

        // Solve: S^n = (R * K^n) / (Rmax - R)
        let ratio = (r * k.powf(n)) / (r_max - r);
        ratio.powf(1.0 / n)
    }

    /// Derivative of the Hill equation at a given signal — how sensitive is the
    /// response to a small change in signal?
    ///
    /// Peaks at `K_half`, approaches zero at both extremes.
    pub fn sensitivity(&self, signal_strength: f64) -> f64 {
        if signal_strength <= 0.0 {
            return 0.0;
        }
        let n = self.hill_coefficient;
        let k = self.k_half;
        let s = signal_strength;
        let r_max = self.max_response;

        let numerator = n * r_max * k.powf(n) * s.powf(n - 1.0);
        let denom_inner = k.powf(n) + s.powf(n);
        numerator / (denom_inner * denom_inner)
    }

    /// Returns `true` if the response is above `threshold * max_response`.
    ///
    /// Default threshold is 0.9 (90% saturation).
    pub fn is_saturated(&self, signal_strength: f64, threshold: f64) -> bool {
        self.calculate(signal_strength) > threshold * self.max_response
    }

    /// The signal range over which the response is most sensitive.
    ///
    /// Returns `(low, high)` where response goes from 10% to 90% of max.
    pub fn effective_range(&self) -> (f64, f64) {
        let n = self.hill_coefficient;
        let k = self.k_half;
        let low = k * (0.1_f64 / 0.9).powf(1.0 / n);
        let high = k * (0.9_f64 / 0.1).powf(1.0 / n);
        (low, high)
    }
}

/// A stateful response generator wrapping a [`HillCurve`] with rate limiting
/// and budget tracking.
///
/// Applies three layers of saturation in sequence:
/// 1. Hill curve mathematical ceiling
/// 2. Hard ceiling (belt-and-suspenders)
/// 3. Rate-of-increase limiting
#[derive(Clone, Debug)]
pub struct SaturatingResponse {
    /// Absolute response ceiling.
    pub max_response: f64,
    /// Underlying Hill curve.
    pub response_curve: HillCurve,
    /// Hard ceiling applied after the curve (defaults to 1.5× `max_response`).
    pub hard_ceiling: f64,
    /// Maximum response increase per `generate()` call.
    pub max_rate_of_increase: f64,
    /// Current response level (updated on each `generate()` call).
    pub current_response: f64,
    /// Optional total budget; `None` means unlimited.
    pub total_budget: Option<f64>,
    /// How much of the budget has been consumed.
    pub budget_consumed: f64,
}

impl SaturatingResponse {
    /// Create a new `SaturatingResponse`.
    pub fn new(max_response: f64, response_curve: HillCurve) -> Self {
        Self {
            max_response,
            hard_ceiling: max_response * 1.5,
            response_curve,
            max_rate_of_increase: 10.0,
            current_response: 0.0,
            total_budget: None,
            budget_consumed: 0.0,
        }
    }

    /// Generate a bounded response for the given signal.
    pub fn generate(&mut self, signal_strength: f64) -> f64 {
        let mut ideal = self.response_curve.calculate(signal_strength);

        // Hard ceiling
        if ideal > self.hard_ceiling {
            ideal = self.hard_ceiling;
        }

        // Rate limiting
        let max_step = self.current_response + self.max_rate_of_increase;
        if ideal > max_step {
            ideal = max_step;
        }

        // Budget
        if let Some(budget) = self.total_budget {
            let remaining = budget - self.budget_consumed;
            if ideal > remaining {
                ideal = remaining.max(0.0);
            }
        }

        self.budget_consumed += ideal;
        self.current_response = ideal;
        ideal
    }

    /// Reset internal state.
    pub fn reset(&mut self) {
        self.current_response = 0.0;
        self.budget_consumed = 0.0;
    }

    /// Current response as a fraction of maximum.
    pub fn utilization(&self) -> f64 {
        if self.max_response > 0.0 {
            self.current_response / self.max_response
        } else {
            0.0
        }
    }

    /// Whether the response is at or near ceiling (≥ 95% of max).
    pub fn is_at_ceiling(&self) -> bool {
        self.current_response >= self.max_response * 0.95
    }
}

/// A general-purpose ceiling enforcer for any numeric value.
///
/// Applies elastic dampening between `soft_limit` and `hard_limit`, then
/// clamps absolutely at `hard_limit`.
///
/// ```
/// use nexcore_homeostasis_primitives::hill::ResponseCeiling;
///
/// let ceiling = ResponseCeiling::new(80.0, 100.0, 0.1);
/// assert_eq!(ceiling.enforce(50.0), 50.0);    // below soft limit — unchanged
/// assert_eq!(ceiling.enforce(150.0), 100.0);  // above hard limit — clamped
/// ```
#[derive(Clone, Debug)]
pub struct ResponseCeiling {
    /// Start dampening at this value.
    pub soft_limit: f64,
    /// Absolute maximum — never exceeded.
    pub hard_limit: f64,
    /// Fraction of the excess above soft limit that is allowed through.
    pub elastic_factor: f64,
}

impl ResponseCeiling {
    /// Create a new ceiling enforcer.
    pub fn new(soft_limit: f64, hard_limit: f64, elastic_factor: f64) -> Self {
        Self { soft_limit, hard_limit, elastic_factor }
    }

    /// Enforce the ceiling on `value`.
    ///
    /// - Below `soft_limit`: returned unchanged.
    /// - Between limits: elastic dampening applied.
    /// - Above `hard_limit`: clamped to `hard_limit`.
    pub fn enforce(&self, value: f64) -> f64 {
        if value <= self.soft_limit {
            return value;
        }
        if value >= self.hard_limit {
            return self.hard_limit;
        }
        // Elastic region
        let excess = value - self.soft_limit;
        let allowed = excess * self.elastic_factor;
        (self.soft_limit + allowed).min(self.hard_limit)
    }

    /// Whether `value` is in the critical zone (above soft limit).
    pub fn is_critical(&self, value: f64) -> bool {
        value > self.soft_limit
    }

    /// Remaining headroom before the hard limit.
    pub fn headroom(&self, current: f64) -> f64 {
        (self.hard_limit - current).max(0.0)
    }
}

/// Create a [`HillCurve`] with biologically-inspired parameters.
///
/// | Sensitivity | K_half | n | Behaviour |
/// |-------------|--------|---|-----------|
/// | `"low"`     | 80% max | 1 | Gradual, high threshold |
/// | `"medium"`  | 50% max | 2 | Balanced (default) |
/// | `"high"`    | 20% max | 4 | Switch-like, low threshold |
///
/// ```
/// use nexcore_homeostasis_primitives::hill::create_biological_response_curve;
///
/// let curve = create_biological_response_curve(100.0, "medium");
/// assert!((curve.calculate(50.0) - 50.0).abs() < 0.1);
/// ```
pub fn create_biological_response_curve(max_response: f64, sensitivity: &str) -> HillCurve {
    let (k_half, n) = match sensitivity {
        "low" => (max_response * 0.8, 1.0),
        "high" => (max_response * 0.2, 4.0),
        _ => (max_response * 0.5, 2.0), // "medium" is default
    };
    HillCurve::new(max_response, k_half, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hill_curve_zero_signal_returns_zero() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        assert!(curve.calculate(0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hill_curve_at_k_half_returns_half_max() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        assert!((curve.calculate(50.0) - 50.0).abs() < 0.01);
    }

    #[test]
    fn hill_curve_never_exceeds_max() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        assert!(curve.calculate(1_000_000.0) < 100.0);
    }

    #[test]
    fn hill_curve_is_monotonic() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        let mut prev = 0.0;
        for i in 0..=200 {
            let r = curve.calculate(i as f64);
            assert!(r >= prev, "response decreased at signal {i}: {r} < {prev}");
            prev = r;
        }
    }

    #[test]
    fn hill_inverse_roundtrip() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        let response = curve.calculate(75.0);
        let signal_back = curve.inverse(response);
        assert!((signal_back - 75.0).abs() < 0.01, "inverse failed: got {signal_back}");
    }

    #[test]
    fn hill_inverse_at_max_returns_inf() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        assert_eq!(curve.inverse(100.0), f64::INFINITY);
    }

    #[test]
    fn response_ceiling_below_soft_unchanged() {
        let ceiling = ResponseCeiling::new(80.0, 100.0, 0.1);
        assert_eq!(ceiling.enforce(50.0), 50.0);
    }

    #[test]
    fn response_ceiling_above_hard_clamped() {
        let ceiling = ResponseCeiling::new(80.0, 100.0, 0.1);
        assert_eq!(ceiling.enforce(150.0), 100.0);
    }

    #[test]
    fn response_ceiling_elastic_region() {
        let ceiling = ResponseCeiling::new(80.0, 100.0, 0.1);
        // value = 90, excess = 10, allowed = 1 → result = 81
        let result = ceiling.enforce(90.0);
        assert!((result - 81.0).abs() < f64::EPSILON, "got {result}");
    }

    #[test]
    fn response_ceiling_headroom() {
        let ceiling = ResponseCeiling::new(80.0, 100.0, 0.1);
        assert_eq!(ceiling.headroom(70.0), 30.0);
        assert_eq!(ceiling.headroom(110.0), 0.0); // clamped at 0
    }

    #[test]
    fn saturating_response_rate_limits() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        let mut sr = SaturatingResponse::new(100.0, curve);
        sr.max_rate_of_increase = 10.0;
        // First call: ideal > 10, gets clamped to 0 + 10 = 10
        let r = sr.generate(1000.0);
        assert!(r <= 10.0, "rate limit exceeded: {r}");
    }

    #[test]
    fn saturating_response_reset() {
        let curve = HillCurve::new(100.0, 50.0, 2.0);
        let mut sr = SaturatingResponse::new(100.0, curve);
        sr.generate(50.0);
        sr.reset();
        assert_eq!(sr.current_response, 0.0);
        assert_eq!(sr.budget_consumed, 0.0);
    }

    #[test]
    fn create_biological_response_curve_medium() {
        let curve = create_biological_response_curve(100.0, "medium");
        assert!((curve.calculate(50.0) - 50.0).abs() < 0.1);
    }

    #[test]
    fn create_biological_response_curve_high_sensitivity() {
        let medium = create_biological_response_curve(100.0, "medium");
        let high = create_biological_response_curve(100.0, "high");
        // High sensitivity should respond more strongly to the same weak signal
        assert!(high.calculate(10.0) > medium.calculate(10.0));
    }
}
