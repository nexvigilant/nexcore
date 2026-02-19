/// Trust velocity and change detection.
///
/// Tracks the exponentially weighted moving average (EWMA) of trust score
/// changes to detect anomalous behavioral shifts (regime changes).
///
/// Fixes: Fallacy #12 (Slippery Slope Blindness), Gap #8 (Volatility Detection).
///
/// Tier: T2-P (Frequency v + Sequence s)

/// Exponentially weighted moving average tracker for trust score velocity.
///
/// Velocity is the rate of trust score change per interaction.
/// Positive velocity = trust improving. Negative = trust degrading.
/// Near-zero velocity = stable trust.
#[derive(Debug, Clone)]
pub struct TrustVelocity {
    /// Current EWMA of score deltas
    ewma: f64,
    /// Smoothing factor in (0, 1). Higher = more responsive to recent changes.
    smoothing: f64,
    /// Peak absolute velocity observed
    peak_magnitude: f64,
    /// Number of updates recorded
    update_count: u64,
    /// Whether the tracker has been initialized
    initialized: bool,
}

impl TrustVelocity {
    /// Create a new velocity tracker with the given smoothing factor.
    ///
    /// `smoothing` controls responsiveness: 0.3 = balanced, 0.1 = slow/stable,
    /// 0.7 = fast/reactive. Clamped to (0.01, 0.99).
    pub fn new(smoothing: f64) -> Self {
        Self {
            ewma: 0.0,
            smoothing: smoothing.clamp(0.01, 0.99),
            peak_magnitude: 0.0,
            update_count: 0,
            initialized: false,
        }
    }

    /// Update the velocity with a new score delta.
    ///
    /// Call this after each trust update with `score_after - score_before`.
    pub fn update(&mut self, score_delta: f64) {
        if !self.initialized {
            self.ewma = score_delta;
            self.initialized = true;
        } else {
            self.ewma = self.smoothing * score_delta + (1.0 - self.smoothing) * self.ewma;
        }
        let abs_ewma = self.ewma.abs();
        if abs_ewma > self.peak_magnitude {
            self.peak_magnitude = abs_ewma;
        }
        self.update_count += 1;
    }

    /// Current trust velocity (EWMA of score deltas).
    ///
    /// Positive = trust improving, Negative = trust degrading, ~0 = stable.
    pub fn velocity(&self) -> f64 {
        self.ewma
    }

    /// Whether the current velocity magnitude exceeds the given threshold.
    ///
    /// Use this to detect anomalous trust changes. Suggested thresholds:
    /// - 0.01 = sensitive (detects gradual drift)
    /// - 0.05 = moderate (detects meaningful shifts)
    /// - 0.10 = aggressive (only flags dramatic changes)
    pub fn is_anomalous(&self, threshold: f64) -> bool {
        self.initialized && self.ewma.abs() > threshold
    }

    /// Direction of trust change.
    pub fn direction(&self) -> TrustDirection {
        if !self.initialized {
            return TrustDirection::Stable;
        }
        if self.ewma > 0.005 {
            TrustDirection::Improving
        } else if self.ewma < -0.005 {
            TrustDirection::Degrading
        } else {
            TrustDirection::Stable
        }
    }

    /// Peak absolute velocity ever observed.
    pub fn peak_magnitude(&self) -> f64 {
        self.peak_magnitude
    }

    /// Number of updates recorded.
    pub fn update_count(&self) -> u64 {
        self.update_count
    }

    /// Reset the tracker to initial state.
    pub fn reset(&mut self) {
        self.ewma = 0.0;
        self.peak_magnitude = 0.0;
        self.update_count = 0;
        self.initialized = false;
    }
}

impl Default for TrustVelocity {
    fn default() -> Self {
        Self::new(0.3)
    }
}

/// Direction of trust score movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustDirection {
    /// Trust is increasing (velocity > +0.005)
    Improving,
    /// Trust is stable (velocity within +/- 0.005)
    Stable,
    /// Trust is decreasing (velocity < -0.005)
    Degrading,
}

impl core::fmt::Display for TrustDirection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Improving => write!(f, "Improving"),
            Self::Stable => write!(f, "Stable"),
            Self::Degrading => write!(f, "Degrading"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let v = TrustVelocity::default();
        assert!((v.velocity()).abs() < f64::EPSILON);
        assert!(!v.is_anomalous(0.01));
        assert_eq!(v.direction(), TrustDirection::Stable);
        assert_eq!(v.update_count(), 0);
    }

    #[test]
    fn positive_deltas_give_positive_velocity() {
        let mut v = TrustVelocity::default();
        for _ in 0..10 {
            v.update(0.05);
        }
        assert!(v.velocity() > 0.0);
        assert_eq!(v.direction(), TrustDirection::Improving);
    }

    #[test]
    fn negative_deltas_give_negative_velocity() {
        let mut v = TrustVelocity::default();
        for _ in 0..10 {
            v.update(-0.05);
        }
        assert!(v.velocity() < 0.0);
        assert_eq!(v.direction(), TrustDirection::Degrading);
    }

    #[test]
    fn detects_direction_reversal() {
        let mut v = TrustVelocity::new(0.5); // High smoothing = responsive

        // Build positive velocity
        for _ in 0..5 {
            v.update(0.05);
        }
        assert_eq!(v.direction(), TrustDirection::Improving);

        // Sudden reversal
        for _ in 0..10 {
            v.update(-0.08);
        }
        assert_eq!(v.direction(), TrustDirection::Degrading);
    }

    #[test]
    fn anomaly_detection() {
        let mut v = TrustVelocity::default();

        // Small changes — not anomalous
        v.update(0.001);
        assert!(!v.is_anomalous(0.05));

        // Large sudden change
        v.update(0.2);
        assert!(v.is_anomalous(0.05));
    }

    #[test]
    fn peak_magnitude_tracked() {
        let mut v = TrustVelocity::default();
        v.update(0.1);
        v.update(-0.001);
        v.update(0.001);
        // Peak should be near 0.1 (first update sets EWMA directly)
        assert!(v.peak_magnitude() > 0.05);
    }

    #[test]
    fn reset_clears_state() {
        let mut v = TrustVelocity::default();
        v.update(0.1);
        v.update(0.1);
        v.reset();
        assert!((v.velocity()).abs() < f64::EPSILON);
        assert_eq!(v.update_count(), 0);
    }

    #[test]
    fn smoothing_clamped() {
        let v = TrustVelocity::new(-5.0);
        assert!((v.smoothing - 0.01).abs() < f64::EPSILON);
        let v = TrustVelocity::new(100.0);
        assert!((v.smoothing - 0.99).abs() < f64::EPSILON);
    }
}
