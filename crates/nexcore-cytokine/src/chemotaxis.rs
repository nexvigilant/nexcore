//! Chemotaxis - gradient-following signal routing.
//!
//! ## T1 Grounding
//!
//! - `Gradient` → N (quantity) - concentration strength at a point
//! - `GradientField` → λ (location) - spatial distribution of concentrations
//! - `ChemotacticAgent` → → (causality) - gradient causes directed movement
//! - `Tropism` → κ (comparison) - positive (toward) vs negative (away)
//!
//! ## Biological Analog
//!
//! Cells navigate by sensing chemical gradients in their environment.
//! - **Positive chemotaxis**: Move toward higher concentration (attractant)
//! - **Negative chemotaxis**: Move away from source (repellent)
//! - **Gradient sensing**: Compare concentration at multiple points to compute direction
//!
//! ## Claude Code Analog
//!
//! Route processing toward highest-value signal sources.
//! - **Signal gradient**: Multiple signals from different sources create a concentration field
//! - **Directed routing**: Dispatch resources (attention, processing) toward strongest gradient
//! - **Adaptive navigation**: As signals shift, routing adapts dynamically

use crate::{Cytokine, CytokineFamily, ThreatLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Direction of chemotactic movement relative to gradient.
///
/// # Tier: T2-P
/// Grounds to: κ (comparison — toward vs away)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tropism {
    /// Move toward increasing concentration (attracted)
    Positive,
    /// Move away from concentration source (repelled)
    Negative,
}

impl Tropism {
    /// Get the direction multiplier (+1.0 or -1.0)
    pub fn multiplier(&self) -> f64 {
        match self {
            Self::Positive => 1.0,
            Self::Negative => -1.0,
        }
    }
}

/// A concentration gradient sample at a specific location.
///
/// # Tier: T2-P
/// Grounds to: N (quantity) + λ (location)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    /// Source identifier (which component emitted the signal)
    pub source: String,

    /// Concentration level \[0.0, 1.0\] (normalized signal strength)
    pub concentration: f64,

    /// Distance from the sensing agent (abstract units)
    pub distance: f64,

    /// Cytokine family creating this gradient
    pub family: CytokineFamily,

    /// Whether this gradient attracts or repels
    pub tropism: Tropism,
}

impl Gradient {
    /// Create a new gradient sample.
    pub fn new(
        source: impl Into<String>,
        family: CytokineFamily,
        concentration: f64,
        distance: f64,
    ) -> Self {
        Self {
            source: source.into(),
            concentration: concentration.clamp(0.0, 1.0),
            distance: distance.max(0.0),
            family,
            tropism: Tropism::Positive,
        }
    }

    /// Set tropism (attractant vs repellent).
    #[must_use]
    pub fn with_tropism(mut self, tropism: Tropism) -> Self {
        self.tropism = tropism;
        self
    }

    /// Compute effective signal strength at the agent's position.
    ///
    /// Concentration decays with inverse-square of distance:
    /// `effective = concentration / (1 + distance^2)`
    ///
    /// This models how real chemical signals diffuse through media.
    pub fn effective_strength(&self) -> f64 {
        if self.distance <= 0.0 {
            return self.concentration;
        }
        self.concentration / self.distance.mul_add(self.distance, 1.0)
    }

    /// Compute the directional pull (strength * tropism direction).
    pub fn directional_pull(&self) -> f64 {
        self.effective_strength() * self.tropism.multiplier()
    }
}

/// A field of concentration gradients from multiple sources.
///
/// Models the chemical environment an agent senses to navigate.
///
/// # Tier: T2-C (Composite)
/// Grounds to: λ (location, spatial field) + Σ (aggregation of samples)
#[derive(Debug, Clone, Default)]
pub struct GradientField {
    /// All gradient samples in the field
    samples: Vec<Gradient>,
}

impl GradientField {
    /// Create an empty gradient field.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a gradient sample to the field.
    pub fn add(&mut self, gradient: Gradient) {
        self.samples.push(gradient);
    }

    /// Add a gradient sample (builder pattern).
    #[must_use]
    pub fn with_gradient(mut self, gradient: Gradient) -> Self {
        self.samples.push(gradient);
        self
    }

    /// Get all samples.
    pub fn samples(&self) -> &[Gradient] {
        &self.samples
    }

    /// Get number of gradient sources.
    pub fn source_count(&self) -> usize {
        self.samples.len()
    }

    /// Check if the field is empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Compute the dominant gradient direction.
    ///
    /// Returns the source with the strongest effective pull.
    /// Positive tropism adds to attractiveness; negative subtracts.
    pub fn dominant_source(&self) -> Option<&Gradient> {
        if self.samples.is_empty() {
            return None;
        }

        self.samples.iter().max_by(|a, b| {
            a.directional_pull()
                .partial_cmp(&b.directional_pull())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Compute net pull across all gradients.
    ///
    /// Positive value = net attraction; Negative = net repulsion.
    pub fn net_pull(&self) -> f64 {
        self.samples.iter().map(Gradient::directional_pull).sum()
    }

    /// Filter field to only include a specific cytokine family.
    #[must_use]
    pub fn filter_family(&self, family: CytokineFamily) -> Self {
        Self {
            samples: self
                .samples
                .iter()
                .filter(|g| g.family == family)
                .cloned()
                .collect(),
        }
    }

    /// Get per-source aggregated pull strengths.
    pub fn source_pulls(&self) -> HashMap<String, f64> {
        let mut pulls: HashMap<String, f64> = HashMap::new();
        for gradient in &self.samples {
            *pulls.entry(gradient.source.clone()).or_insert(0.0) += gradient.directional_pull();
        }
        pulls
    }

    /// Rank sources by total pull (descending).
    pub fn ranked_sources(&self) -> Vec<(String, f64)> {
        let mut pulls: Vec<_> = self.source_pulls().into_iter().collect();
        pulls.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        pulls
    }
}

/// A routing decision produced by chemotactic navigation.
///
/// # Tier: T2-C
/// Grounds to: → (causality) + λ (location)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemotacticDecision {
    /// Target source to route toward (or away from)
    pub target: String,

    /// Strength of the routing decision \[0.0, 1.0\]
    pub confidence: f64,

    /// Whether we're moving toward or away
    pub tropism: Tropism,

    /// The family that drove this decision
    pub primary_family: CytokineFamily,
}

/// Trait for components that navigate using signal gradients.
///
/// # Tier: T2-P (Cross-Domain Primitive)
/// Grounds to: → (causality — gradient causes movement)
pub trait ChemotacticAgent {
    /// Sense the current gradient field.
    ///
    /// Implementations read recent signals and construct a gradient field.
    fn sense_field(&self) -> GradientField;

    /// Compute routing decision from the gradient field.
    fn navigate(&self, field: &GradientField) -> Option<ChemotacticDecision> {
        let dominant = field.dominant_source()?;
        let pull = dominant.directional_pull();

        // Confidence is the fraction of net pull this source represents
        let net = field.net_pull().abs();
        let confidence = if net > 0.0 {
            (pull.abs() / net).min(1.0)
        } else {
            0.0
        };

        Some(ChemotacticDecision {
            target: dominant.source.clone(),
            confidence,
            tropism: dominant.tropism,
            primary_family: dominant.family,
        })
    }

    /// Sensitivity threshold — minimum effective strength to respond.
    ///
    /// Below this threshold, the agent ignores the gradient.
    fn sensitivity_threshold(&self) -> f64 {
        0.01
    }

    /// Filter the field to only actionable gradients.
    fn actionable_gradients(&self, field: &GradientField) -> GradientField {
        let threshold = self.sensitivity_threshold();
        GradientField {
            samples: field
                .samples()
                .iter()
                .filter(|g| g.effective_strength() >= threshold)
                .cloned()
                .collect(),
        }
    }
}

/// Build a gradient from a cytokine signal.
///
/// Converts signal severity to concentration and uses source as location.
pub fn gradient_from_signal(signal: &Cytokine, distance: f64) -> Gradient {
    let concentration = match signal.severity {
        ThreatLevel::Trace => 0.1,
        ThreatLevel::Low => 0.3,
        ThreatLevel::Medium => 0.5,
        ThreatLevel::High => 0.8,
        ThreatLevel::Critical => 1.0,
    };

    // Suppressing cytokines create repellent gradients
    let tropism = if signal.family.is_suppressing() {
        Tropism::Negative
    } else {
        Tropism::Positive
    };

    Gradient::new(
        signal.source.as_deref().unwrap_or("unknown"),
        signal.family,
        concentration,
        distance,
    )
    .with_tropism(tropism)
}

/// Build a gradient field from multiple signals with estimated distances.
///
/// Distances are derived from signal scope:
/// - Autocrine: 0.0 (same component)
/// - Paracrine: 1.0 (nearby)
/// - Endocrine: 5.0 (distant)
/// - Systemic: 10.0 (system-wide)
pub fn field_from_signals(signals: &[Cytokine]) -> GradientField {
    let mut field = GradientField::new();

    for signal in signals {
        let distance = match signal.scope {
            crate::Scope::Autocrine => 0.0,
            crate::Scope::Paracrine => 1.0,
            crate::Scope::Endocrine => 5.0,
            crate::Scope::Systemic => 10.0,
        };

        field.add(gradient_from_signal(signal, distance));
    }

    field
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::Scope;

    #[test]
    fn test_gradient_effective_strength_at_origin() {
        let g = Gradient::new("src", CytokineFamily::Il1, 0.8, 0.0);
        assert!((g.effective_strength() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gradient_inverse_square_decay() {
        let g = Gradient::new("src", CytokineFamily::Il1, 1.0, 2.0);
        // 1.0 / (1 + 4) = 0.2
        assert!((g.effective_strength() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gradient_directional_pull_positive() {
        let g = Gradient::new("src", CytokineFamily::Il1, 0.5, 0.0).with_tropism(Tropism::Positive);
        assert!((g.directional_pull() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gradient_directional_pull_negative() {
        let g =
            Gradient::new("src", CytokineFamily::Il10, 0.5, 0.0).with_tropism(Tropism::Negative);
        assert!((g.directional_pull() - (-0.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gradient_concentration_clamped() {
        let g = Gradient::new("src", CytokineFamily::Il1, 1.5, 0.0);
        assert!((g.concentration - 1.0).abs() < f64::EPSILON);

        let g2 = Gradient::new("src", CytokineFamily::Il1, -0.5, 0.0);
        assert!((g2.concentration - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gradient_field_dominant_source() {
        let field = GradientField::new()
            .with_gradient(Gradient::new("weak", CytokineFamily::Il1, 0.2, 5.0))
            .with_gradient(Gradient::new("strong", CytokineFamily::Il1, 1.0, 0.0))
            .with_gradient(Gradient::new("mid", CytokineFamily::Il1, 0.5, 1.0));

        let dominant = field.dominant_source();
        assert!(dominant.is_some());
        assert_eq!(dominant.map(|d| d.source.as_str()), Some("strong"));
    }

    #[test]
    fn test_gradient_field_empty() {
        let field = GradientField::new();
        assert!(field.is_empty());
        assert!(field.dominant_source().is_none());
        assert!((field.net_pull() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gradient_field_net_pull() {
        let field = GradientField::new()
            .with_gradient(
                Gradient::new("attract", CytokineFamily::Il1, 0.8, 0.0)
                    .with_tropism(Tropism::Positive),
            )
            .with_gradient(
                Gradient::new("repel", CytokineFamily::Il10, 0.3, 0.0)
                    .with_tropism(Tropism::Negative),
            );

        // 0.8 - 0.3 = 0.5
        assert!((field.net_pull() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gradient_field_ranked_sources() {
        let field = GradientField::new()
            .with_gradient(Gradient::new("b", CytokineFamily::Il1, 0.5, 0.0))
            .with_gradient(Gradient::new("a", CytokineFamily::Il1, 1.0, 0.0))
            .with_gradient(Gradient::new("c", CytokineFamily::Il1, 0.2, 0.0));

        let ranked = field.ranked_sources();
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].0, "a");
        assert_eq!(ranked[1].0, "b");
        assert_eq!(ranked[2].0, "c");
    }

    #[test]
    fn test_gradient_field_filter_family() {
        let field = GradientField::new()
            .with_gradient(Gradient::new("s1", CytokineFamily::Il1, 0.8, 0.0))
            .with_gradient(Gradient::new("s2", CytokineFamily::TnfAlpha, 0.5, 0.0))
            .with_gradient(Gradient::new("s3", CytokineFamily::Il1, 0.3, 0.0));

        let filtered = field.filter_family(CytokineFamily::Il1);
        assert_eq!(filtered.source_count(), 2);
    }

    #[test]
    fn test_gradient_from_signal() {
        let signal = Cytokine::new(CytokineFamily::Il1, "alarm")
            .with_severity(ThreatLevel::Critical)
            .with_source("guardian");

        let gradient = gradient_from_signal(&signal, 1.0);
        assert!((gradient.concentration - 1.0).abs() < f64::EPSILON);
        assert_eq!(gradient.tropism, Tropism::Positive);
        assert_eq!(gradient.source, "guardian");
    }

    #[test]
    fn test_gradient_from_suppressing_signal() {
        let signal = Cytokine::new(CytokineFamily::Il10, "cooldown")
            .with_severity(ThreatLevel::Medium)
            .with_source("regulator");

        let gradient = gradient_from_signal(&signal, 0.0);
        assert_eq!(gradient.tropism, Tropism::Negative);
        assert!((gradient.concentration - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_field_from_signals() {
        let signals = vec![
            Cytokine::new(CytokineFamily::Il1, "alarm")
                .with_severity(ThreatLevel::High)
                .with_scope(Scope::Autocrine)
                .with_source("self"),
            Cytokine::new(CytokineFamily::Il6, "acute")
                .with_severity(ThreatLevel::Medium)
                .with_scope(Scope::Endocrine)
                .with_source("distant"),
        ];

        let field = field_from_signals(&signals);
        assert_eq!(field.source_count(), 2);

        // Autocrine signal at distance 0 should have highest effective strength
        let dominant = field.dominant_source();
        assert!(dominant.is_some());
        assert_eq!(dominant.map(|d| d.source.as_str()), Some("self"));
    }

    #[test]
    fn test_chemotactic_navigation() {
        struct TestAgent;
        impl ChemotacticAgent for TestAgent {
            fn sense_field(&self) -> GradientField {
                GradientField::new()
                    .with_gradient(Gradient::new("api_a", CytokineFamily::Il1, 0.9, 1.0))
                    .with_gradient(Gradient::new("api_b", CytokineFamily::Il1, 0.3, 1.0))
            }
        }

        let agent = TestAgent;
        let field = agent.sense_field();
        let decision = agent.navigate(&field);

        assert!(decision.is_some());
        let d = decision.unwrap_or_else(|| ChemotacticDecision {
            target: String::new(),
            confidence: 0.0,
            tropism: Tropism::Positive,
            primary_family: CytokineFamily::Il1,
        });
        assert_eq!(d.target, "api_a");
        assert_eq!(d.tropism, Tropism::Positive);
        assert!(d.confidence > 0.5);
    }

    #[test]
    fn test_sensitivity_threshold_filtering() {
        struct SensitiveAgent;
        impl ChemotacticAgent for SensitiveAgent {
            fn sense_field(&self) -> GradientField {
                GradientField::new()
            }
            fn sensitivity_threshold(&self) -> f64 {
                0.1
            }
        }

        let agent = SensitiveAgent;
        let field = GradientField::new()
            .with_gradient(Gradient::new("strong", CytokineFamily::Il1, 0.8, 0.0))
            .with_gradient(Gradient::new("weak", CytokineFamily::Il1, 0.005, 0.0));

        let actionable = agent.actionable_gradients(&field);
        assert_eq!(actionable.source_count(), 1);
        assert_eq!(actionable.samples()[0].source, "strong");
    }

    #[test]
    fn test_tropism_multiplier() {
        assert!((Tropism::Positive.multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((Tropism::Negative.multiplier() - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_source_pulls_aggregation() {
        let field = GradientField::new()
            .with_gradient(Gradient::new("src_a", CytokineFamily::Il1, 0.5, 0.0))
            .with_gradient(Gradient::new("src_a", CytokineFamily::Il6, 0.3, 0.0))
            .with_gradient(Gradient::new("src_b", CytokineFamily::Il1, 0.2, 0.0));

        let pulls = field.source_pulls();
        assert_eq!(pulls.len(), 2);

        // src_a has 0.5 + 0.3 = 0.8
        let src_a_pull = pulls.get("src_a").copied().unwrap_or(0.0);
        assert!((src_a_pull - 0.8).abs() < f64::EPSILON);
    }
}
