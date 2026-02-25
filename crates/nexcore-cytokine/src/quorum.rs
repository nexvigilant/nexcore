//! Quorum Sensing - collective threshold detection.
//!
//! ## T1 Grounding
//!
//! - `SignalDensity` → ν (frequency) - signal count per unit
//! - `QuorumThreshold` → κ (comparison) - density vs threshold
//! - `PopulationHealth` → N (quantity) - fraction of healthy responders
//!
//! ## Biological Analog
//!
//! Bacteria use quorum sensing to coordinate behavior based on population density:
//! 1. Each cell emits autoinducer molecules at a constant rate
//! 2. When population density exceeds threshold, concentration triggers gene expression
//! 3. The collective switches behavior (biofilm formation, virulence, etc.)
//!
//! ## Claude Code Analog
//!
//! Collective decision-making based on signal density:
//! - **Signal counting**: How many hooks/skills/sensors report the same issue?
//! - **Threshold**: When >50% of components flag a problem, escalate
//! - **Collective response**: Trigger system-wide action only at quorum

use serde::{Deserialize, Serialize};

/// A signal vote from one member of the population.
///
/// # Tier: T2-P
/// Grounds to: ν (frequency — one signal pulse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalVote {
    /// Identifier of the voting member
    pub member_id: String,
    /// The signal being voted on (topic)
    pub topic: String,
    /// Vote direction: true = signal present, false = not detected
    pub detected: bool,
    /// Confidence in the detection \[0.0, 1.0\]
    pub confidence: f64,
}

impl SignalVote {
    /// Create a positive detection vote.
    pub fn detected(
        member_id: impl Into<String>,
        topic: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            member_id: member_id.into(),
            topic: topic.into(),
            detected: true,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Create a negative detection vote.
    pub fn not_detected(member_id: impl Into<String>, topic: impl Into<String>) -> Self {
        Self {
            member_id: member_id.into(),
            topic: topic.into(),
            detected: false,
            confidence: 1.0,
        }
    }
}

/// Result of a quorum check.
///
/// # Tier: T2-P
/// Grounds to: κ (comparison — threshold met or not)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuorumResult {
    /// The topic being sensed
    pub topic: String,
    /// Whether quorum was reached
    pub quorum_reached: bool,
    /// Signal density (fraction of population detecting)
    pub density: f64,
    /// Required threshold for quorum
    pub threshold: f64,
    /// Weighted confidence across positive votes
    pub weighted_confidence: f64,
    /// Total population size polled
    pub population_size: usize,
    /// Number of positive detections
    pub positive_count: usize,
}

/// A quorum sensor that tracks votes and determines collective decisions.
///
/// # Tier: T2-C (Composite)
/// Grounds to: ν (frequency) + κ (comparison against threshold)
#[derive(Debug)]
pub struct QuorumSensor {
    /// Topic being sensed
    topic: String,
    /// Threshold fraction \[0.0, 1.0\] for quorum (default: 0.5)
    threshold: f64,
    /// Minimum confidence for a vote to count
    min_confidence: f64,
    /// Collected votes
    votes: Vec<SignalVote>,
}

impl QuorumSensor {
    /// Create a new quorum sensor for a topic.
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            threshold: 0.5,
            min_confidence: 0.0,
            votes: Vec::new(),
        }
    }

    /// Set the quorum threshold (fraction of population needed).
    #[must_use]
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set minimum confidence for a vote to count.
    #[must_use]
    pub fn with_min_confidence(mut self, min_confidence: f64) -> Self {
        self.min_confidence = min_confidence.clamp(0.0, 1.0);
        self
    }

    /// Submit a vote.
    pub fn vote(&mut self, vote: SignalVote) {
        self.votes.push(vote);
    }

    /// Submit a positive detection.
    pub fn signal_detected(&mut self, member_id: impl Into<String>, confidence: f64) {
        self.votes.push(SignalVote::detected(
            member_id,
            self.topic.clone(),
            confidence,
        ));
    }

    /// Submit a negative detection.
    pub fn signal_absent(&mut self, member_id: impl Into<String>) {
        self.votes
            .push(SignalVote::not_detected(member_id, self.topic.clone()));
    }

    /// Evaluate whether quorum has been reached.
    pub fn evaluate(&self) -> QuorumResult {
        let qualifying_votes: Vec<_> = self
            .votes
            .iter()
            .filter(|v| v.confidence >= self.min_confidence)
            .collect();

        let population = qualifying_votes.len();
        let positive: Vec<_> = qualifying_votes.iter().filter(|v| v.detected).collect();

        // Precision loss acceptable: vote counts are small
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        let density = if population > 0 {
            positive.len() as f64 / population as f64
        } else {
            0.0
        };

        // Precision loss acceptable: vote counts are small
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        let weighted_confidence = if positive.is_empty() {
            0.0
        } else {
            positive.iter().map(|v| v.confidence).sum::<f64>() / positive.len() as f64
        };

        QuorumResult {
            topic: self.topic.clone(),
            quorum_reached: density >= self.threshold,
            density,
            threshold: self.threshold,
            weighted_confidence,
            population_size: population,
            positive_count: positive.len(),
        }
    }

    /// Reset votes for a new sensing round.
    pub fn reset(&mut self) {
        self.votes.clear();
    }

    /// Get the current vote count.
    pub fn vote_count(&self) -> usize {
        self.votes.len()
    }
}

/// Population health check — aggregate health status across components.
///
/// # Tier: T2-C
/// Grounds to: ν (frequency of health reports) + κ (comparison against threshold)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationHealth {
    /// Health reports from individual members
    pub reports: Vec<HealthReport>,
}

/// Individual health report from a population member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Member identifier
    pub member_id: String,
    /// Health score \[0.0, 1.0\]
    pub health: f64,
    /// Is the member responsive?
    pub responsive: bool,
}

impl PopulationHealth {
    /// Create a new population health tracker.
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    /// Add a health report.
    pub fn report(&mut self, member_id: impl Into<String>, health: f64, responsive: bool) {
        self.reports.push(HealthReport {
            member_id: member_id.into(),
            health: health.clamp(0.0, 1.0),
            responsive,
        });
    }

    /// Population size.
    pub fn size(&self) -> usize {
        self.reports.len()
    }

    /// Fraction of responsive members.
    pub fn availability(&self) -> f64 {
        if self.reports.is_empty() {
            return 0.0;
        }
        let responsive = self.reports.iter().filter(|r| r.responsive).count();
        // Precision loss acceptable: report counts are small
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        {
            responsive as f64 / self.reports.len() as f64
        }
    }

    /// Mean health across all responsive members.
    pub fn mean_health(&self) -> f64 {
        let responsive: Vec<_> = self.reports.iter().filter(|r| r.responsive).collect();

        if responsive.is_empty() {
            return 0.0;
        }

        // Precision loss acceptable: report counts are small
        #[allow(
            clippy::cast_precision_loss,
            reason = "Count-to-f64 conversion for bounded runtime metrics"
        )]
        {
            responsive.iter().map(|r| r.health).sum::<f64>() / responsive.len() as f64
        }
    }

    /// Check if population health is above a threshold.
    pub fn is_healthy(&self, threshold: f64) -> bool {
        self.availability() > 0.5 && self.mean_health() >= threshold
    }
}

impl Default for PopulationHealth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quorum_reached() {
        let mut sensor = QuorumSensor::new("error_spike").with_threshold(0.5);

        sensor.signal_detected("hook_a", 0.9);
        sensor.signal_detected("hook_b", 0.8);
        sensor.signal_absent("hook_c");

        let result = sensor.evaluate();
        assert!(result.quorum_reached);
        assert!((result.density - (2.0 / 3.0)).abs() < 0.01);
    }

    #[test]
    fn test_quorum_not_reached() {
        let mut sensor = QuorumSensor::new("rare_event").with_threshold(0.5);

        sensor.signal_detected("hook_a", 0.9);
        sensor.signal_absent("hook_b");
        sensor.signal_absent("hook_c");
        sensor.signal_absent("hook_d");

        let result = sensor.evaluate();
        assert!(!result.quorum_reached);
        assert_eq!(result.positive_count, 1);
    }

    #[test]
    fn test_quorum_empty() {
        let sensor = QuorumSensor::new("nothing");
        let result = sensor.evaluate();
        assert!(!result.quorum_reached);
        assert_eq!(result.population_size, 0);
    }

    #[test]
    fn test_quorum_confidence_filter() {
        let mut sensor = QuorumSensor::new("test")
            .with_threshold(0.5)
            .with_min_confidence(0.5);

        sensor.signal_detected("confident", 0.9);
        sensor.signal_detected("weak", 0.2); // Below min_confidence
        sensor.signal_absent("no");

        let result = sensor.evaluate();
        // Only 2 qualifying votes (confident + no), 1 positive
        assert_eq!(result.population_size, 2);
        assert_eq!(result.positive_count, 1);
        assert!((result.density - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quorum_weighted_confidence() {
        let mut sensor = QuorumSensor::new("test");
        sensor.signal_detected("a", 0.8);
        sensor.signal_detected("b", 0.6);

        let result = sensor.evaluate();
        assert!((result.weighted_confidence - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quorum_reset() {
        let mut sensor = QuorumSensor::new("test");
        sensor.signal_detected("a", 0.9);
        assert_eq!(sensor.vote_count(), 1);

        sensor.reset();
        assert_eq!(sensor.vote_count(), 0);
    }

    #[test]
    fn test_population_health_basic() {
        let mut health = PopulationHealth::new();
        health.report("agent_1", 0.9, true);
        health.report("agent_2", 0.8, true);
        health.report("agent_3", 0.0, false); // Unresponsive

        assert_eq!(health.size(), 3);
        assert!((health.availability() - (2.0 / 3.0)).abs() < 0.01);
        assert!((health.mean_health() - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_population_health_check() {
        let mut health = PopulationHealth::new();
        health.report("a", 0.9, true);
        health.report("b", 0.8, true);
        health.report("c", 0.7, true);

        assert!(health.is_healthy(0.7));
        assert!(!health.is_healthy(0.9));
    }

    #[test]
    fn test_population_health_empty() {
        let health = PopulationHealth::new();
        assert_eq!(health.size(), 0);
        assert!((health.availability() - 0.0).abs() < f64::EPSILON);
        assert!((health.mean_health() - 0.0).abs() < f64::EPSILON);
        assert!(!health.is_healthy(0.5));
    }

    #[test]
    fn test_signal_vote_creation() {
        let vote = SignalVote::detected("sensor_1", "high_cpu", 0.95);
        assert!(vote.detected);
        assert!((vote.confidence - 0.95).abs() < f64::EPSILON);

        let neg = SignalVote::not_detected("sensor_2", "high_cpu");
        assert!(!neg.detected);
    }
}
