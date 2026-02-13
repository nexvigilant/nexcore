//! Capability definition and tracking for CTVP validation.
//!
//! A capability represents a measurable effect that a system should produce.
//! This module provides builders and trackers for defining and measuring
//! capability achievement.

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A capability that a system should provide.
///
/// Capabilities are the fundamental unit of validation in CTVP.
/// Instead of asking "do tests pass?", we ask "is the capability achieved?"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    /// Unique identifier (e.g., "CAP-001")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Detailed description
    pub description: String,

    /// The desired effect this capability produces
    pub desired_effect: DesiredEffect,

    /// How to measure the capability
    pub measurement: Measurement,

    /// Success threshold
    pub threshold: Threshold,

    /// Validation status by phase
    #[serde(default)]
    pub validation_phases: HashMap<ValidationPhase, ValidationResult>,
}

impl Capability {
    /// Creates a new capability builder
    pub fn builder() -> CapabilityBuilder {
        CapabilityBuilder::default()
    }

    /// Returns the highest phase with validated evidence
    pub fn get_highest_validated_phase(&self) -> Option<ValidationPhase> {
        ValidationPhase::get_all()
            .into_iter()
            .rev()
            .find(|p| self.has_validated_evidence(p))
    }

    fn has_validated_evidence(&self, phase: &ValidationPhase) -> bool {
        self.validation_phases
            .get(phase)
            .map(|r| r.outcome.is_validated())
            .unwrap_or(false)
    }

    /// Returns true if capability is validated at the given phase
    pub fn is_validated_at(&self, phase: ValidationPhase) -> bool {
        self.validation_phases
            .get(&phase)
            .map(|r| r.outcome.is_validated())
            .unwrap_or(false)
    }

    /// Returns the evidence quality for a phase
    pub fn get_evidence_quality_at(&self, phase: ValidationPhase) -> EvidenceQuality {
        self.validation_phases
            .get(&phase)
            .map(|r| r.evidence_quality)
            .unwrap_or(EvidenceQuality::None)
    }
}

/// Builder for creating capabilities.
#[derive(Debug, Default)]
pub struct CapabilityBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    desired_effect: Option<DesiredEffect>,
    measurement: Option<Measurement>,
    threshold: Option<Threshold>,
}

impl CapabilityBuilder {
    /// Sets the capability ID
    pub fn set_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the capability ID (convenience alias)
    pub fn id(self, id: impl Into<String>) -> Self {
        self.set_id(id)
    }

    /// Sets the capability name
    pub fn set_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the capability name (convenience alias)
    pub fn name(self, name: impl Into<String>) -> Self {
        self.set_name(name)
    }

    /// Sets the capability description
    pub fn set_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the capability description (convenience alias)
    pub fn description(self, description: impl Into<String>) -> Self {
        self.set_description(description)
    }

    /// Sets the desired effect (shorthand)
    pub fn set_desired_effect(mut self, outcome: impl Into<String>) -> Self {
        self.desired_effect = Some(DesiredEffect {
            outcome: outcome.into(),
            beneficiary: String::new(),
            conditions: Vec::new(),
        });
        self
    }

    /// Sets the desired effect (convenience alias)
    pub fn desired_effect(self, outcome: impl Into<String>) -> Self {
        self.set_desired_effect(outcome)
    }

    /// Sets the full desired effect
    pub fn set_desired_effect_full(mut self, effect: DesiredEffect) -> Self {
        self.desired_effect = Some(effect);
        self
    }

    /// Sets the measurement metric (shorthand)
    pub fn set_measurement(mut self, metric: impl Into<String>) -> Self {
        self.measurement = Some(Measurement {
            metric: metric.into(),
            calculation: String::new(),
            unit: String::new(),
        });
        self
    }

    /// Sets the measurement metric (convenience alias)
    pub fn measurement(self, metric: impl Into<String>) -> Self {
        self.set_measurement(metric)
    }

    /// Sets the full measurement specification
    pub fn set_measurement_full(mut self, measurement: Measurement) -> Self {
        self.measurement = Some(measurement);
        self
    }

    /// Sets the success threshold
    pub fn set_threshold(mut self, threshold: Threshold) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Sets the success threshold (convenience alias)
    pub fn threshold(self, threshold: Threshold) -> Self {
        self.set_threshold(threshold)
    }

    /// Builds the capability
    pub fn build(self) -> Result<Capability, crate::error::CtvpError> {
        let fields = self.validate_fields()?;

        Ok(Capability {
            id: fields.0,
            name: fields.1,
            description: self.description.unwrap_or_default(),
            desired_effect: fields.2,
            measurement: fields.3,
            threshold: fields.4,
            validation_phases: HashMap::new(),
        })
    }

    fn validate_fields(
        &self,
    ) -> Result<(String, String, DesiredEffect, Measurement, Threshold), crate::error::CtvpError>
    {
        use crate::error::CtvpError;

        let id = self
            .id
            .as_ref()
            .ok_or(CtvpError::BuilderMissingField("id"))?
            .clone();
        let name = self
            .name
            .as_ref()
            .ok_or(CtvpError::BuilderMissingField("name"))?
            .clone();
        let desired_effect = self
            .desired_effect
            .as_ref()
            .ok_or(CtvpError::BuilderMissingField("desired_effect"))?
            .clone();
        let measurement = self
            .measurement
            .as_ref()
            .ok_or(CtvpError::BuilderMissingField("measurement"))?
            .clone();
        let threshold = self
            .threshold
            .as_ref()
            .ok_or(CtvpError::BuilderMissingField("threshold"))?
            .clone();

        Ok((id, name, desired_effect, measurement, threshold))
    }
}

impl From<CapabilityBuilder> for Result<Capability, crate::error::CtvpError> {
    fn from(builder: CapabilityBuilder) -> Self {
        builder.build()
    }
}

/// The desired effect a capability should produce.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesiredEffect {
    /// What outcome should happen
    pub outcome: String,

    /// Who benefits from this outcome
    pub beneficiary: String,

    /// Conditions under which this applies
    pub conditions: Vec<String>,
}

impl DesiredEffect {
    /// Creates a new desired effect
    pub fn new(outcome: impl Into<String>) -> Self {
        Self {
            outcome: outcome.into(),
            beneficiary: String::new(),
            conditions: Vec::new(),
        }
    }

    /// Sets the beneficiary
    pub fn set_beneficiary(mut self, beneficiary: impl Into<String>) -> Self {
        self.beneficiary = beneficiary.into();
        self
    }

    /// Adds a condition
    pub fn add_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }
}

/// How to measure a capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    /// Metric name
    pub metric: String,

    /// How to calculate the metric
    pub calculation: String,

    /// Unit of measurement
    pub unit: String,
}

impl Measurement {
    /// Creates a new measurement
    pub fn new(metric: impl Into<String>) -> Self {
        Self {
            metric: metric.into(),
            calculation: String::new(),
            unit: String::new(),
        }
    }

    /// Sets the calculation method
    pub fn set_calculation_method(mut self, calculation: impl Into<String>) -> Self {
        self.calculation = calculation.into();
        self
    }

    /// Sets the unit
    pub fn set_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }
}

/// Tracks capability achievement over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAchievementTracker {
    /// Capability being tracked
    pub capability_id: String,

    /// Individual measurements
    pub measurements: Vec<AchievementMeasurement>,

    /// Success threshold
    pub threshold: Threshold,

    /// Baseline distribution (for drift detection)
    #[serde(default)]
    pub baseline: Option<Vec<f64>>,
}

/// A single achievement measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementMeasurement {
    /// When this was measured
    pub timestamp: DateTime<Utc>,

    /// Whether capability was achieved
    pub achieved: bool,

    /// The effect value measured
    pub effect_value: f64,

    /// Additional context
    #[serde(default)]
    pub context: HashMap<String, String>,
}

impl CapabilityAchievementTracker {
    /// Creates a new tracker
    pub fn new(capability_id: impl Into<String>, threshold: Threshold) -> Self {
        Self {
            capability_id: capability_id.into(),
            measurements: Vec::new(),
            threshold,
            baseline: None,
        }
    }

    /// Records a new measurement
    pub fn record(&mut self, achieved: bool, effect_value: f64) {
        self.add_measurement(achieved, effect_value, HashMap::new());
    }

    /// Records a measurement with context
    pub fn record_with_context(
        &mut self,
        achieved: bool,
        effect_value: f64,
        context: HashMap<String, String>,
    ) {
        self.add_measurement(achieved, effect_value, context);
    }

    fn add_measurement(
        &mut self,
        achieved: bool,
        effect_value: f64,
        context: HashMap<String, String>,
    ) {
        self.measurements.push(AchievementMeasurement {
            timestamp: Utc::now(),
            achieved,
            effect_value,
            context,
        });
    }

    /// Calculates the Capability Achievement Rate (CAR)
    pub fn get_capability_achievement_rate(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }
        let ok = self.measurements.iter().filter(|m| m.achieved).count();
        ok as f64 / self.measurements.len() as f64
    }

    /// Returns the effect value at a given percentile
    pub fn get_effect_percentile(&self, p: f64) -> Option<f64> {
        let v = self.get_sorted_effect_values()?;
        let idx = ((p / 100.0) * (v.len() - 1) as f64).floor() as usize;
        v.get(idx).copied()
    }

    fn get_sorted_effect_values(&self) -> Option<Vec<f64>> {
        if self.measurements.is_empty() {
            return None;
        }
        let mut v: Vec<f64> = self.measurements.iter().map(|m| m.effect_value).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Some(v)
    }

    /// Checks if the threshold is currently met
    pub fn check_meets_threshold(&self) -> bool {
        let car = self.get_capability_achievement_rate();
        self.threshold.is_met(car)
    }

    /// Sets the baseline for drift detection
    pub fn set_baseline(&mut self, baseline: Vec<f64>) {
        self.baseline = Some(baseline);
    }

    /// Sets baseline from current measurements
    pub fn set_baseline_from_current(&mut self) {
        let values: Vec<f64> = self.measurements.iter().map(|m| m.effect_value).collect();
        self.baseline = Some(values);
    }

    /// Returns measurements from a time window
    pub fn get_measurements_since(&self, since: DateTime<Utc>) -> Vec<&AchievementMeasurement> {
        self.measurements
            .iter()
            .filter(|m| m.timestamp >= since)
            .collect()
    }

    /// Returns the most recent N measurements
    pub fn get_recent_measurements(&self, n: usize) -> Vec<&AchievementMeasurement> {
        self.measurements.iter().rev().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CtvpResult;

    #[test]
    fn test_capability_builder() -> CtvpResult<()> {
        let cap = Capability::builder()
            .set_id("CAP-001")
            .set_name("Test Capability")
            .set_desired_effect("Users can do something")
            .set_measurement("success_rate")
            .set_threshold(Threshold::gte(0.99))
            .build()?;

        assert_eq!(cap.id, "CAP-001");
        assert_eq!(cap.name, "Test Capability");
        Ok(())
    }

    #[test]
    fn test_capability_builder_missing_field() {
        let result = Capability::builder().set_id("CAP-001").build();

        assert!(result.is_err());
    }

    #[test]
    fn test_achievement_tracker() -> CtvpResult<()> {
        let mut tracker = CapabilityAchievementTracker::new("CAP-001", Threshold::gte(0.90));

        // Record 9 successes and 1 failure
        for _ in 0..9 {
            tracker.record(true, 1.0);
        }
        tracker.record(false, 0.0);

        assert!((tracker.get_capability_achievement_rate() - 0.9).abs() < f64::EPSILON);
        assert!(tracker.check_meets_threshold());
        Ok(())
    }

    #[test]
    fn test_effect_percentile() -> CtvpResult<()> {
        let mut tracker = CapabilityAchievementTracker::new("CAP-001", Threshold::gte(0.90));

        // Record values 1-100
        for i in 1..=100 {
            tracker.record(true, i as f64);
        }

        // p50 should be around 50
        let p50 = tracker
            .get_effect_percentile(50.0)
            .ok_or_else(|| crate::error::CtvpError::Analysis("No p50".into()))?;
        assert!(p50 >= 49.0 && p50 <= 51.0);

        // p95 should be around 95
        let p95 = tracker
            .get_effect_percentile(95.0)
            .ok_or_else(|| crate::error::CtvpError::Analysis("No p95".into()))?;
        assert!(p95 >= 94.0 && p95 <= 96.0);
        Ok(())
    }
}
