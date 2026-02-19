//! Drift Detection for Phase 4 Surveillance

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Direction of trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving over time
    Improving,
    /// Stable over time
    Stable,
    /// Degrading over time
    Degrading,
}

impl TrendDirection {
    /// Returns emoji for trend
    ///
    /// # Returns
    /// Emoji string
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Improving => "📈",
            Self::Degrading => "📉",
            Self::Stable => "➡️",
        }
    }
}

/// A drift alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAlert {
    /// Alert message
    pub message: String,
    /// Current value
    pub current_value: f64,
    /// Target threshold
    pub target: f64,
    /// Alert threshold
    pub alert_threshold: f64,
    /// Timestamp
    pub timestamp: f64,
}

/// Detects drift from baseline behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetector {
    /// Target threshold (Phase 2 target)
    pub target: f64,
    /// Alert threshold (Phase 4 alert)
    pub alert_threshold: f64,
    /// Historical values
    values: VecDeque<f64>,
    /// Window size for trend analysis
    window_size: usize,
    /// Baseline values (from validated state)
    baseline: Option<Vec<f64>>,
}

impl DriftDetector {
    /// Creates new drift detector
    ///
    /// # Arguments
    /// * `target` - Target threshold (Phase 2)
    /// * `alert_threshold` - Alert threshold (Phase 4)
    ///
    /// # Returns
    /// New DriftDetector
    pub fn new(target: f64, alert_threshold: f64) -> Self {
        Self {
            target,
            alert_threshold,
            values: VecDeque::new(),
            window_size: 100,
            baseline: None,
        }
    }

    /// Sets window size
    ///
    /// # Arguments
    /// * `size` - Window size
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_window(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Sets baseline from current values
    pub fn set_baseline(&mut self) {
        self.baseline = Some(self.values.iter().copied().collect());
    }

    /// Records a value
    ///
    /// # Arguments
    /// * `value` - The value to record
    pub fn record(&mut self, value: f64) {
        self.values.push_front(value);
        if self.values.len() > self.window_size {
            self.values.pop_back();
        }
    }

    /// Returns current value (most recent)
    ///
    /// # Returns
    /// Most recent value or 0.0
    pub fn current(&self) -> f64 {
        self.values.front().copied().unwrap_or(0.0)
    }

    /// Returns average of recent values
    ///
    /// # Returns
    /// Mean of values in window
    pub fn mean(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.values.iter().sum();
        sum / self.values.len() as f64
    }

    /// Returns standard deviation
    ///
    /// # Returns
    /// Standard deviation of values
    pub fn std_dev(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let variance: f64 = self.values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
            / (self.values.len() - 1) as f64;
        variance.sqrt()
    }

    /// Detects trend direction
    ///
    /// # Returns
    /// Trend direction based on recent values
    pub fn trend(&self) -> TrendDirection {
        if self.values.len() < 5 {
            return TrendDirection::Stable;
        }

        // Compare first half to second half
        let mid = self.values.len() / 2;
        let recent: f64 = self.values.iter().take(mid).sum::<f64>() / mid as f64;
        let older: f64 =
            self.values.iter().skip(mid).sum::<f64>() / (self.values.len() - mid) as f64;

        let diff = recent - older;
        let threshold = 0.05; // 5% change threshold

        if diff > threshold {
            TrendDirection::Improving
        } else if diff < -threshold {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        }
    }

    /// Checks if alerting
    ///
    /// # Returns
    /// True if current value below alert threshold
    pub fn is_alerting(&self) -> bool {
        !self.values.is_empty() && self.current() < self.alert_threshold
    }

    /// Checks and returns alert if needed
    ///
    /// # Arguments
    /// * `value` - Current value to check
    ///
    /// # Returns
    /// DriftAlert if alerting, None otherwise
    pub fn check_and_alert(&mut self, value: f64) -> Option<DriftAlert> {
        self.record(value);

        if value < self.alert_threshold {
            Some(DriftAlert {
                message: format!(
                    "🚨 DRIFT ALERT: CAR {:.1}% below threshold {:.1}%",
                    value * 100.0,
                    self.alert_threshold * 100.0
                ),
                current_value: value,
                target: self.target,
                alert_threshold: self.alert_threshold,
                timestamp: super::now(),
            })
        } else {
            None
        }
    }

    /// Calculates drift score (difference from baseline)
    ///
    /// # Returns
    /// Drift score (0.0 = no drift, higher = more drift)
    pub fn drift_score(&self) -> f64 {
        match &self.baseline {
            None => 0.0,
            Some(baseline) if baseline.is_empty() => 0.0,
            Some(baseline) => {
                let baseline_mean: f64 = baseline.iter().sum::<f64>() / baseline.len() as f64;
                (self.mean() - baseline_mean).abs()
            }
        }
    }

    /// Generates status report
    ///
    /// # Returns
    /// Formatted status string
    pub fn report(&self) -> String {
        let trend = self.trend();
        let status = if self.is_alerting() {
            "🚨 ALERTING"
        } else if self.current() >= self.target {
            "✅ Healthy"
        } else {
            "🟡 Below target"
        };

        format!(
            "Drift: current={:.1}% mean={:.1}% σ={:.3} {} {} (target={:.1}%, alert={:.1}%)",
            self.current() * 100.0,
            self.mean() * 100.0,
            self.std_dev(),
            trend.emoji(),
            status,
            self.target * 100.0,
            self.alert_threshold * 100.0
        )
    }

    /// Returns observation count
    ///
    /// # Returns
    /// Number of recorded values
    pub fn observation_count(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_detector() {
        let mut d = DriftDetector::new(0.80, 0.70);
        d.record(0.85);
        d.record(0.82);
        d.record(0.78);
        assert!((d.mean() - 0.8167).abs() < 0.01);
    }

    #[test]
    fn test_alerting() {
        let mut d = DriftDetector::new(0.80, 0.70);
        d.record(0.65);
        assert!(d.is_alerting());
    }

    #[test]
    fn test_trend() {
        let mut d = DriftDetector::new(0.80, 0.70);
        // Record improving trend
        for v in [0.70, 0.72, 0.75, 0.78, 0.80, 0.82, 0.85, 0.87, 0.88, 0.90] {
            d.record(v);
        }
        // Most recent values (0.90, 0.88, ...) are higher than older ones
        assert_eq!(d.trend(), TrendDirection::Improving);
    }
}
