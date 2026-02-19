//! # SPRT (Sequential Probability Ratio Test)
//!
//! SPRT is a sequential hypothesis testing method that allows for early
//! stopping in drug safety monitoring. Unlike fixed-sample methods (PRR, ROR),
//! SPRT updates after each case and decides whether to:
//!
//! 1. **Accept H₀** (no signal) - stop monitoring
//! 2. **Reject H₀** (signal detected) - trigger alert
//! 3. **Continue** - wait for more data
//!
//! ## Algorithm
//!
//! SPRT computes the likelihood ratio between two hypotheses:
//!
//! ```text
//! H₀: RR = 1 (no elevated risk)
//! H₁: RR = RR₁ (elevated risk, e.g., RR₁ = 2)
//!
//! LR(n) = Π P(xᵢ | H₁) / P(xᵢ | H₀)
//! log_LR(n) = Σ log(P(xᵢ | H₁) / P(xᵢ | H₀))
//! ```
//!
//! ## Decision Boundaries
//!
//! - If log_LR ≥ log(B), reject H₀ (signal detected)
//! - If log_LR ≤ log(A), accept H₀ (no signal)
//! - Otherwise, continue monitoring
//!
//! Where A = β/(1-α) and B = (1-β)/α for type I error α and type II error β.
//!
//! ## Complexity
//!
//! - TIME: O(1) per update, O(n) for n cumulative updates
//! - SPACE: O(1) - only maintains running log-likelihood
//!
//! ## Use Case
//!
//! SPRT is ideal for:
//! - Real-time safety monitoring
//! - Surveillance of newly approved drugs
//! - Post-marketing commitment studies
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::pv::signals::sequential::sprt::{SprtMonitor, SprtDecision, SprtConfig};
//!
//! // Configure SPRT with RR=2 as the alternative hypothesis
//! let config = SprtConfig::default();
//! let mut monitor = SprtMonitor::new(config);
//!
//! // Update with each new case
//! monitor.update(true);   // Drug + Event case
//! monitor.update(false);  // Drug + No Event case
//!
//! match monitor.decision() {
//!     SprtDecision::Signal => println!("Signal detected!"),
//!     SprtDecision::NoSignal => println!("No signal, stop monitoring"),
//!     SprtDecision::Continue => println!("Continue monitoring"),
//! }
//! ```
//!
//! ## References
//!
//! - Wald A (1945). "Sequential tests of statistical hypotheses." The Annals of
//!   Mathematical Statistics 16(2):117-186.
//!   DOI: [10.1214/aoms/1177731118](https://doi.org/10.1214/aoms/1177731118)
//!
//! - Wald A (1947). Sequential Analysis. John Wiley & Sons, New York.
//!   ISBN: 978-0-486-43912-1
//!
//! - Kulldorff M, Davis RL, Kolczak M, et al. (2011). "A maximized sequential probability
//!   ratio test for drug and vaccine safety surveillance." Sequential Analysis 30(1):58-78.
//!   DOI: [10.1080/07474946.2011.539924](https://doi.org/10.1080/07474946.2011.539924)

use serde::{Deserialize, Serialize};

/// SPRT decision outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SprtDecision {
    /// Signal detected - reject H₀
    Signal,
    /// No signal - accept H₀
    NoSignal,
    /// Insufficient evidence - continue monitoring
    Continue,
}

/// Configuration for SPRT monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SprtConfig {
    /// Type I error rate (false positive)
    pub alpha: f64,
    /// Type II error rate (false negative)
    pub beta: f64,
    /// Relative risk under alternative hypothesis (H₁)
    pub rr_alternative: f64,
    /// Expected event rate under null hypothesis
    pub expected_rate: f64,
}

impl Default for SprtConfig {
    /// Default configuration:
    /// - α = 0.05 (5% false positive rate)
    /// - β = 0.20 (20% false negative rate, 80% power)
    /// - RR₁ = 2.0 (detect doubling of risk)
    /// - Expected rate = 0.01 (1% background rate)
    fn default() -> Self {
        Self {
            alpha: 0.05,
            beta: 0.20,
            rr_alternative: 2.0,
            expected_rate: 0.01,
        }
    }
}

impl SprtConfig {
    /// Create a new SPRT configuration.
    ///
    /// # Arguments
    ///
    /// * `alpha` - Type I error rate (e.g., 0.05)
    /// * `beta` - Type II error rate (e.g., 0.20)
    /// * `rr_alternative` - Relative risk to detect (e.g., 2.0)
    /// * `expected_rate` - Background event rate (e.g., 0.01)
    #[must_use]
    pub fn new(alpha: f64, beta: f64, rr_alternative: f64, expected_rate: f64) -> Self {
        Self {
            alpha,
            beta,
            rr_alternative,
            expected_rate,
        }
    }

    /// Create a sensitive configuration for early signal detection.
    #[must_use]
    pub fn sensitive() -> Self {
        Self {
            alpha: 0.10,         // 10% false positive (more sensitive)
            beta: 0.10,          // 10% false negative (higher power)
            rr_alternative: 1.5, // Detect 50% increase
            expected_rate: 0.01,
        }
    }

    /// Create a conservative configuration for established drugs.
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            alpha: 0.01,         // 1% false positive (very specific)
            beta: 0.20,          // 20% false negative (standard power)
            rr_alternative: 3.0, // Only detect 3x increase
            expected_rate: 0.01,
        }
    }

    /// Lower decision boundary (accept H₀ threshold).
    ///
    /// log(A) where A = β / (1 - α)
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn lower_boundary(&self) -> f64 {
        (self.beta / (1.0 - self.alpha)).ln()
    }

    /// Upper decision boundary (reject H₀ threshold).
    ///
    /// log(B) where B = (1 - β) / α
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn upper_boundary(&self) -> f64 {
        ((1.0 - self.beta) / self.alpha).ln()
    }
}

/// Sequential Probability Ratio Test monitor.
///
/// Maintains state for sequential monitoring of drug-event pairs.
/// Call `update()` for each new observation and check `decision()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprtMonitor {
    /// Configuration
    config: SprtConfig,
    /// Cumulative log-likelihood ratio
    log_lr: f64,
    /// Number of events (cases)
    events: u32,
    /// Total observations
    total: u32,
}

impl SprtMonitor {
    /// Create a new SPRT monitor with the given configuration.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn new(config: SprtConfig) -> Self {
        Self {
            config,
            log_lr: 0.0,
            events: 0,
            total: 0,
        }
    }

    /// Create with default configuration.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(SprtConfig::default())
    }

    /// Update the monitor with a new observation.
    ///
    /// # Arguments
    ///
    /// * `is_event` - true if the observation is a drug-event case
    ///
    /// # Complexity: O(1)
    pub fn update(&mut self, is_event: bool) {
        self.total += 1;

        let p0 = self.config.expected_rate;
        let p1 = p0 * self.config.rr_alternative;

        // Clamp p1 to valid probability range
        let p1 = p1.min(1.0);

        if is_event {
            self.events += 1;
            // log(P(event|H₁) / P(event|H₀)) = log(p1/p0)
            if p0 > 0.0 && p1 > 0.0 {
                self.log_lr += (p1 / p0).ln();
            }
        } else {
            // log(P(no event|H₁) / P(no event|H₀)) = log((1-p1)/(1-p0))
            let q0 = 1.0 - p0;
            let q1 = 1.0 - p1;
            if q0 > 0.0 && q1 > 0.0 {
                self.log_lr += (q1 / q0).ln();
            }
        }
    }

    /// Update with multiple observations at once.
    ///
    /// # Arguments
    ///
    /// * `n_events` - Number of drug-event cases
    /// * `n_non_events` - Number of drug-non-event cases
    ///
    /// # Complexity: O(1)
    pub fn update_batch(&mut self, n_events: u32, n_non_events: u32) {
        let p0 = self.config.expected_rate;
        let p1 = (p0 * self.config.rr_alternative).min(1.0);

        self.events += n_events;
        self.total += n_events + n_non_events;

        // Batch update of log-likelihood
        if p0 > 0.0 && p1 > 0.0 {
            let log_event_ratio = (p1 / p0).ln();
            let log_non_event_ratio = ((1.0 - p1) / (1.0 - p0)).ln();

            self.log_lr += f64::from(n_events) * log_event_ratio
                + f64::from(n_non_events) * log_non_event_ratio;
        }
    }

    /// Get the current decision.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn decision(&self) -> SprtDecision {
        let lower = self.config.lower_boundary();
        let upper = self.config.upper_boundary();

        if self.log_lr >= upper {
            SprtDecision::Signal
        } else if self.log_lr <= lower {
            SprtDecision::NoSignal
        } else {
            SprtDecision::Continue
        }
    }

    /// Check if a signal has been detected.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn is_signal(&self) -> bool {
        self.decision() == SprtDecision::Signal
    }

    /// Check if monitoring should stop (either signal or no signal).
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn should_stop(&self) -> bool {
        self.decision() != SprtDecision::Continue
    }

    /// Get the current log-likelihood ratio.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn log_likelihood_ratio(&self) -> f64 {
        self.log_lr
    }

    /// Get the number of events observed.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn events(&self) -> u32 {
        self.events
    }

    /// Get the total number of observations.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn total(&self) -> u32 {
        self.total
    }

    /// Get the observed event rate.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn observed_rate(&self) -> f64 {
        if self.total > 0 {
            f64::from(self.events) / f64::from(self.total)
        } else {
            0.0
        }
    }

    /// Reset the monitor to initial state.
    ///
    /// # Complexity: O(1)
    pub fn reset(&mut self) {
        self.log_lr = 0.0;
        self.events = 0;
        self.total = 0;
    }

    /// Get the configuration.
    ///
    /// # Complexity: O(1)
    #[must_use]
    pub fn config(&self) -> &SprtConfig {
        &self.config
    }
}

/// Calculate expected sample size for SPRT (approximate).
///
/// Returns the expected number of observations needed to reach a decision
/// under H₀ (no signal) and H₁ (signal present).
///
/// # Complexity: O(1)
#[must_use]
pub fn expected_sample_size(config: &SprtConfig) -> (f64, f64) {
    let p0 = config.expected_rate;
    let p1 = (p0 * config.rr_alternative).min(1.0);

    let lower = config.lower_boundary();
    let upper = config.upper_boundary();

    // Expected value of log-likelihood ratio per observation
    let e0 = p0 * (p1 / p0).ln() + (1.0 - p0) * ((1.0 - p1) / (1.0 - p0)).ln();
    let e1 = p1 * (p1 / p0).ln() + (1.0 - p1) * ((1.0 - p1) / (1.0 - p0)).ln();

    // Approximate expected sample sizes using Wald's equations
    let n_under_h0 = if e0.abs() > 1e-10 {
        ((1.0 - config.alpha) * lower + config.alpha * upper) / e0
    } else {
        f64::INFINITY
    };

    let n_under_h1 = if e1.abs() > 1e-10 {
        (config.beta * lower + (1.0 - config.beta) * upper) / e1
    } else {
        f64::INFINITY
    };

    (n_under_h0.abs(), n_under_h1.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprt_config_default() {
        let config = SprtConfig::default();
        assert!((config.alpha - 0.05).abs() < 0.001);
        assert!((config.beta - 0.20).abs() < 0.001);
        assert!((config.rr_alternative - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_sprt_boundaries() {
        let config = SprtConfig::default();

        let lower = config.lower_boundary();
        let upper = config.upper_boundary();

        // Lower boundary should be negative
        assert!(lower < 0.0);
        // Upper boundary should be positive
        assert!(upper > 0.0);
        // Upper > Lower
        assert!(upper > lower);
    }

    #[test]
    fn test_sprt_signal_detection() {
        let config = SprtConfig {
            alpha: 0.05,
            beta: 0.20,
            rr_alternative: 2.0,
            expected_rate: 0.10, // 10% expected rate
        };
        let mut monitor = SprtMonitor::new(config);

        // Simulate high event rate (many events)
        for _ in 0..20 {
            monitor.update(true); // Events
        }
        for _ in 0..30 {
            monitor.update(false); // Non-events
        }

        // With 40% observed rate vs 10% expected, should detect signal
        assert!(
            monitor.decision() == SprtDecision::Signal,
            "Should detect signal with elevated rate. LR: {}, events: {}/{}",
            monitor.log_likelihood_ratio(),
            monitor.events(),
            monitor.total()
        );
    }

    #[test]
    fn test_sprt_no_signal() {
        let config = SprtConfig {
            alpha: 0.05,
            beta: 0.20,
            rr_alternative: 2.0,
            expected_rate: 0.10,
        };
        let mut monitor = SprtMonitor::new(config);

        // Simulate expected rate (10% events)
        for _ in 0..10 {
            monitor.update(true);
        }
        for _ in 0..90 {
            monitor.update(false);
        }

        // With observed rate = expected rate, should not signal
        // May continue or conclude no signal
        assert!(
            monitor.decision() != SprtDecision::Signal,
            "Should not detect signal at expected rate"
        );
    }

    #[test]
    fn test_sprt_continue() {
        let config = SprtConfig::default();
        let mut monitor = SprtMonitor::new(config);

        // Single observation - insufficient data
        monitor.update(true);

        // Should continue monitoring
        assert_eq!(monitor.decision(), SprtDecision::Continue);
        assert!(!monitor.should_stop());
    }

    #[test]
    fn test_sprt_batch_update() {
        let config = SprtConfig::default();
        let mut monitor1 = SprtMonitor::new(config);
        let mut monitor2 = SprtMonitor::new(config);

        // Sequential updates
        for _ in 0..10 {
            monitor1.update(true);
        }
        for _ in 0..90 {
            monitor1.update(false);
        }

        // Batch update
        monitor2.update_batch(10, 90);

        // Should give same result
        assert!(
            (monitor1.log_likelihood_ratio() - monitor2.log_likelihood_ratio()).abs() < 0.001,
            "Batch and sequential should match"
        );
    }

    #[test]
    fn test_sprt_reset() {
        let config = SprtConfig::default();
        let mut monitor = SprtMonitor::new(config);

        monitor.update(true);
        monitor.update(true);
        assert!(monitor.events() > 0);

        monitor.reset();
        assert_eq!(monitor.events(), 0);
        assert_eq!(monitor.total(), 0);
        assert!((monitor.log_likelihood_ratio() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_expected_sample_size() {
        let config = SprtConfig::default();
        let (n0, n1) = expected_sample_size(&config);

        // Should return finite positive values
        assert!(n0 > 0.0 && n0.is_finite());
        assert!(n1 > 0.0 && n1.is_finite());

        // Expected sample under H₁ should generally be smaller
        // (easier to detect when there's actually a signal)
        // This isn't always true but is typical
    }

    #[test]
    fn test_observed_rate() {
        let config = SprtConfig::default();
        let mut monitor = SprtMonitor::new(config);

        monitor.update_batch(25, 75);

        assert!((monitor.observed_rate() - 0.25).abs() < 0.001);
    }
}
