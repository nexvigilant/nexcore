//! Evidence types for Minesweeper-PV belief updates.
//!
//! ## Conservation Laws
//!
//! Evidence computation follows Conservation Law #4 (Information Conservation):
//! - Likelihood ratios are computed as geometric means to preserve independence
//! - Propagated evidence is attenuated based on path confidence

use serde::{Deserialize, Serialize};

use crate::minesweeper::types::{EvidenceSource, MechanisticPlausibility, TemporalPatternStrength};

/// Evidence collected for a drug-event cell.
///
/// Evidence is used to update the belief state through likelihood ratio computation.
/// Multiple evidence components are combined using a geometric mean approach.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Proportional Reporting Ratio (PRR) - disproportionality measure
    pub prr: f64,
    /// Reporting Odds Ratio (ROR) - alternative disproportionality measure
    pub ror: f64,
    /// Chi-square statistic for significance testing
    pub chi2: f64,
    /// Number of observed cases
    pub count: u32,
    /// Temporal pattern strength assessment
    pub temporal_pattern: TemporalPatternStrength,
    /// Mechanistic plausibility assessment
    pub mechanism: MechanisticPlausibility,
    /// Whether positive dechallenge was observed
    pub positive_dechallenge: bool,
    /// Whether positive rechallenge was observed
    pub positive_rechallenge: bool,
    /// Case quality score (0.0 to 1.0)
    pub case_quality: f64,
    /// Source of the evidence
    pub source: EvidenceSource,
    /// Confidence factor for propagated evidence (0.0 to 1.0)
    pub confidence: f64,
}

impl Default for Evidence {
    fn default() -> Self {
        Self {
            prr: 1.0,
            ror: 1.0,
            chi2: 0.0,
            count: 0,
            temporal_pattern: TemporalPatternStrength::Unknown,
            mechanism: MechanisticPlausibility::Unknown,
            positive_dechallenge: false,
            positive_rechallenge: false,
            case_quality: 0.5,
            source: EvidenceSource::Direct,
            confidence: 1.0,
        }
    }
}

impl Evidence {
    /// Create a new evidence builder
    #[must_use]
    pub fn builder() -> EvidenceBuilder {
        EvidenceBuilder::default()
    }

    /// Compute combined likelihood ratio from evidence components.
    ///
    /// The likelihood ratio (LR) is computed as a geometric mean of four components:
    /// 1. Statistical LR from PRR/chi-square
    /// 2. Temporal pattern LR
    /// 3. Mechanistic plausibility LR
    /// 4. Challenge/rechallenge LR
    ///
    /// For propagated evidence, the LR is attenuated by the confidence factor.
    #[must_use]
    pub fn compute_likelihood_ratio(&self) -> f64 {
        // Statistical component: strong signal if PRR >= 2, chi2 >= 4, count >= 3
        let lr_stat = if self.prr >= 2.0 && self.chi2 >= 4.0 && self.count >= 3 {
            self.prr
        } else {
            // Attenuated contribution for weaker signals
            1.0 + (self.prr - 1.0) * 0.3
        };

        // Temporal pattern component
        let lr_temporal = self.temporal_pattern.likelihood_ratio();

        // Mechanistic plausibility component
        let lr_mechanism = self.mechanism.likelihood_ratio();

        // Challenge/rechallenge component (strong evidence of causality)
        let lr_challenge = if self.positive_rechallenge {
            10.0 // Positive rechallenge is very strong evidence
        } else if self.positive_dechallenge {
            3.0 // Positive dechallenge is moderately strong
        } else {
            1.0 // No challenge data
        };

        // Geometric mean of all four components
        let lr_combined = (lr_stat * lr_temporal * lr_mechanism * lr_challenge).powf(0.25);

        // Attenuate propagated evidence
        if self.source == EvidenceSource::Propagated {
            1.0 + (lr_combined - 1.0) * self.confidence
        } else {
            lr_combined
        }
    }

    /// Create propagated evidence with attenuated values
    #[must_use]
    pub fn propagate(&self, weight: f64) -> Self {
        Self {
            prr: 1.0 + (self.prr - 1.0) * weight,
            ror: 1.0 + (self.ror - 1.0) * weight,
            chi2: self.chi2 * weight * weight, // Chi-square scales quadratically
            count: 0,                          // Propagated evidence doesn't transfer case counts
            temporal_pattern: if weight > 0.5 {
                self.temporal_pattern
            } else {
                TemporalPatternStrength::Weak
            },
            mechanism: if weight > 0.7 {
                self.mechanism
            } else {
                MechanisticPlausibility::Speculative
            },
            positive_dechallenge: false, // Challenge data doesn't propagate
            positive_rechallenge: false,
            case_quality: self.case_quality * weight,
            source: EvidenceSource::Propagated,
            confidence: weight,
        }
    }
}

/// Builder for Evidence with fluent API
#[derive(Debug, Default)]
pub struct EvidenceBuilder {
    evidence: Evidence,
}

impl EvidenceBuilder {
    /// Set PRR value
    #[must_use]
    pub fn prr(mut self, prr: f64) -> Self {
        self.evidence.prr = prr;
        self
    }

    /// Set ROR value
    #[must_use]
    pub fn ror(mut self, ror: f64) -> Self {
        self.evidence.ror = ror;
        self
    }

    /// Set chi-square statistic
    #[must_use]
    pub fn chi2(mut self, chi2: f64) -> Self {
        self.evidence.chi2 = chi2;
        self
    }

    /// Set case count
    #[must_use]
    pub fn count(mut self, count: u32) -> Self {
        self.evidence.count = count;
        self
    }

    /// Set temporal pattern strength from string
    #[must_use]
    pub fn temporal_pattern(mut self, pattern: &str) -> Self {
        self.evidence.temporal_pattern = match pattern.to_lowercase().as_str() {
            "strong" => TemporalPatternStrength::Strong,
            "moderate" => TemporalPatternStrength::Moderate,
            "weak" => TemporalPatternStrength::Weak,
            "inconsistent" => TemporalPatternStrength::Inconsistent,
            _ => TemporalPatternStrength::Unknown,
        };
        self
    }

    /// Set temporal pattern strength directly
    #[must_use]
    pub fn temporal_pattern_strength(mut self, strength: TemporalPatternStrength) -> Self {
        self.evidence.temporal_pattern = strength;
        self
    }

    /// Set mechanistic plausibility from string
    #[must_use]
    pub fn mechanism(mut self, mechanism: &str) -> Self {
        self.evidence.mechanism = match mechanism.to_lowercase().as_str() {
            "established" => MechanisticPlausibility::Established,
            "plausible" => MechanisticPlausibility::Plausible,
            "speculative" => MechanisticPlausibility::Speculative,
            "implausible" => MechanisticPlausibility::Implausible,
            _ => MechanisticPlausibility::Unknown,
        };
        self
    }

    /// Set mechanistic plausibility directly
    #[must_use]
    pub fn mechanistic_plausibility(mut self, plausibility: MechanisticPlausibility) -> Self {
        self.evidence.mechanism = plausibility;
        self
    }

    /// Set positive dechallenge
    #[must_use]
    pub fn positive_dechallenge(mut self, value: bool) -> Self {
        self.evidence.positive_dechallenge = value;
        self
    }

    /// Set positive rechallenge
    #[must_use]
    pub fn positive_rechallenge(mut self, value: bool) -> Self {
        self.evidence.positive_rechallenge = value;
        self
    }

    /// Set case quality score
    #[must_use]
    pub fn case_quality(mut self, quality: f64) -> Self {
        self.evidence.case_quality = quality.clamp(0.0, 1.0);
        self
    }

    /// Set evidence source
    #[must_use]
    pub fn source(mut self, source: EvidenceSource) -> Self {
        self.evidence.source = source;
        self
    }

    /// Set confidence (for propagated evidence)
    #[must_use]
    pub fn confidence(mut self, confidence: f64) -> Self {
        self.evidence.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Build the Evidence instance
    #[must_use]
    pub fn build(self) -> Evidence {
        self.evidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_default() {
        let ev = Evidence::default();
        assert!((ev.prr - 1.0).abs() < f64::EPSILON);
        assert_eq!(ev.source, EvidenceSource::Direct);
    }

    #[test]
    fn test_evidence_builder() {
        let ev = Evidence::builder()
            .prr(2.5)
            .chi2(5.0)
            .count(10)
            .temporal_pattern("strong")
            .mechanism("plausible")
            .build();

        assert!((ev.prr - 2.5).abs() < f64::EPSILON);
        assert_eq!(ev.count, 10);
        assert_eq!(ev.temporal_pattern, TemporalPatternStrength::Strong);
        assert_eq!(ev.mechanism, MechanisticPlausibility::Plausible);
    }

    #[test]
    fn test_likelihood_ratio_strong_signal() {
        let ev = Evidence::builder()
            .prr(3.0)
            .chi2(8.0)
            .count(5)
            .temporal_pattern("strong")
            .mechanism("established")
            .build();

        let lr = ev.compute_likelihood_ratio();
        // Strong signal should have LR > 1
        assert!(lr > 1.0, "LR = {}", lr);
    }

    #[test]
    fn test_likelihood_ratio_weak_signal() {
        let ev = Evidence::builder()
            .prr(1.2)
            .chi2(1.0)
            .count(1)
            .temporal_pattern("weak")
            .mechanism("speculative")
            .build();

        let lr = ev.compute_likelihood_ratio();
        // Weak signal should be closer to 1
        assert!(lr < 2.0, "LR = {}", lr);
    }

    #[test]
    fn test_rechallenge_boost() {
        let ev_no_rechallenge = Evidence::builder().prr(2.0).chi2(4.0).count(3).build();

        let ev_rechallenge = Evidence::builder()
            .prr(2.0)
            .chi2(4.0)
            .count(3)
            .positive_rechallenge(true)
            .build();

        assert!(
            ev_rechallenge.compute_likelihood_ratio()
                > ev_no_rechallenge.compute_likelihood_ratio()
        );
    }

    #[test]
    fn test_evidence_propagation() {
        let ev = Evidence::builder()
            .prr(3.0)
            .temporal_pattern("strong")
            .mechanism("established")
            .build();

        let propagated = ev.propagate(0.5);

        assert_eq!(propagated.source, EvidenceSource::Propagated);
        assert!((propagated.confidence - 0.5).abs() < f64::EPSILON);
        assert!(propagated.prr < ev.prr); // Attenuated
        assert!(!propagated.positive_dechallenge); // Challenge data doesn't propagate
    }

    #[test]
    fn test_propagated_evidence_attenuation() {
        let ev = Evidence::builder()
            .prr(4.0)
            .chi2(10.0)
            .count(10)
            .temporal_pattern("strong")
            .mechanism("established")
            .build();

        let direct_lr = ev.compute_likelihood_ratio();
        let propagated = ev.propagate(0.3);
        let propagated_lr = propagated.compute_likelihood_ratio();

        // Propagated evidence should have lower LR
        assert!(propagated_lr < direct_lr);
        // But still > 1 since original was strong
        assert!(propagated_lr > 1.0);
    }
}
