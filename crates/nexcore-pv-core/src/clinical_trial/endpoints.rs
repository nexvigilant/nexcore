//! FDA Endpoint Hierarchy
//!
//! Per FDA Multiple Endpoints Guidance:
//! - Primary endpoints: Required for approval, demonstrate substantial evidence
//! - Secondary endpoints: Support primary or show additional benefits
//! - Exploratory endpoints: Hypothesis-generating only

use super::types::{ConfidenceInterval, EffectSize, PValue, StatisticalSignificance};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Endpoint classification tier (FDA hierarchy)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EndpointTier {
    /// Required for approval - substantial evidence of effectiveness
    Primary,
    /// Supports primary or demonstrates additional clinical effects
    Secondary,
    /// Hypothesis-generating, not for approval claims
    Exploratory,
}

impl EndpointTier {
    /// FDA regulatory weight for endpoint tier
    #[must_use]
    pub const fn regulatory_weight(&self) -> f64 {
        match self {
            Self::Primary => 1.0,
            Self::Secondary => 0.5,
            Self::Exploratory => 0.1,
        }
    }

    /// Alpha allocation recommendation
    #[must_use]
    pub const fn recommended_alpha(&self) -> f64 {
        match self {
            Self::Primary => 0.05,
            Self::Secondary => 0.025, // Bonferroni-adjusted
            Self::Exploratory => 0.10,
        }
    }
}

impl fmt::Display for EndpointTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primary => write!(f, "Primary"),
            Self::Secondary => write!(f, "Secondary"),
            Self::Exploratory => write!(f, "Exploratory"),
        }
    }
}

/// Clinical trial endpoint result
/// T2-C: Composed from T2-P primitives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointResult {
    /// Effect size measurement
    pub effect_size: EffectSize,
    /// P-value from statistical test
    pub p_value: PValue,
    /// Confidence interval
    pub confidence_interval: ConfidenceInterval,
    /// Sample size
    pub n: usize,
}

impl EndpointResult {
    /// Create a new endpoint result
    #[must_use]
    pub const fn new(
        effect_size: EffectSize,
        p_value: PValue,
        confidence_interval: ConfidenceInterval,
        n: usize,
    ) -> Self {
        Self {
            effect_size,
            p_value,
            confidence_interval,
            n,
        }
    }

    /// Check if result meets FDA effectiveness standard
    /// Requires: p < 0.05 AND CI excludes null AND clinically meaningful effect
    #[must_use]
    pub fn meets_effectiveness_standard(&self) -> bool {
        self.p_value.is_significant_05()
            && self.confidence_interval.excludes_one()
            && self.effect_size.is_clinically_meaningful(0.5)
    }

    /// Get overall assessment
    #[must_use]
    pub fn assessment(&self) -> EndpointAssessment {
        if self.meets_effectiveness_standard() {
            EndpointAssessment::EffectivenessDemonstrated
        } else if self.p_value.is_significant_05() {
            EndpointAssessment::StatisticallySignificant
        } else if self.p_value.significance() == StatisticalSignificance::Marginal {
            EndpointAssessment::Marginal
        } else {
            EndpointAssessment::NotDemonstrated
        }
    }
}

impl fmt::Display for EndpointResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, n={}",
            self.effect_size, self.p_value, self.confidence_interval, self.n
        )
    }
}

/// Endpoint effectiveness assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndpointAssessment {
    /// Full FDA effectiveness standard met
    EffectivenessDemonstrated,
    /// Statistically significant but may lack clinical significance
    StatisticallySignificant,
    /// p-value between 0.05 and 0.10
    Marginal,
    /// No significant effect demonstrated
    NotDemonstrated,
}

/// Generic endpoint with tier and result
/// T2-C: Endpoint with classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Endpoint name/description
    pub name: String,
    /// Classification tier
    pub tier: EndpointTier,
    /// Statistical result (if evaluated)
    pub result: Option<EndpointResult>,
    /// Pre-specified in protocol
    pub pre_specified: bool,
}

impl Endpoint {
    /// Create a new endpoint definition
    #[must_use]
    pub fn new(name: impl Into<String>, tier: EndpointTier, pre_specified: bool) -> Self {
        Self {
            name: name.into(),
            tier,
            result: None,
            pre_specified,
        }
    }

    /// Set the result
    #[must_use]
    pub fn with_result(mut self, result: EndpointResult) -> Self {
        self.result = Some(result);
        self
    }

    /// Check if endpoint met its success criterion
    #[must_use]
    pub fn is_successful(&self) -> Option<bool> {
        self.result.as_ref().map(|r| {
            let alpha = self.tier.recommended_alpha();
            r.p_value.is_significant(alpha)
        })
    }
}

/// Primary endpoint (convenience type)
pub type PrimaryEndpoint = Endpoint;

/// Secondary endpoint (convenience type)
pub type SecondaryEndpoint = Endpoint;

impl PrimaryEndpoint {
    /// Create a primary endpoint
    #[must_use]
    pub fn primary(name: impl Into<String>) -> Self {
        Self::new(name, EndpointTier::Primary, true)
    }
}

impl SecondaryEndpoint {
    /// Create a secondary endpoint
    #[must_use]
    pub fn secondary(name: impl Into<String>, pre_specified: bool) -> Self {
        Self::new(name, EndpointTier::Secondary, pre_specified)
    }
}

/// Endpoint hierarchy for a clinical trial
/// T3: Full domain type with trial-level logic
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndpointHierarchy {
    /// Primary endpoints (typically 1-2)
    pub primary: Vec<Endpoint>,
    /// Secondary endpoints
    pub secondary: Vec<Endpoint>,
    /// Exploratory endpoints
    pub exploratory: Vec<Endpoint>,
    /// Multiplicity adjustment method
    pub multiplicity_adjustment: Option<MultiplicityMethod>,
}

impl EndpointHierarchy {
    /// Create a new endpoint hierarchy
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a primary endpoint
    #[must_use]
    pub fn with_primary(mut self, endpoint: Endpoint) -> Self {
        self.primary.push(endpoint);
        self
    }

    /// Add a secondary endpoint
    #[must_use]
    pub fn with_secondary(mut self, endpoint: Endpoint) -> Self {
        self.secondary.push(endpoint);
        self
    }

    /// Add an exploratory endpoint
    #[must_use]
    pub fn with_exploratory(mut self, endpoint: Endpoint) -> Self {
        self.exploratory.push(endpoint);
        self
    }

    /// Set multiplicity adjustment method
    #[must_use]
    pub fn with_multiplicity(mut self, method: MultiplicityMethod) -> Self {
        self.multiplicity_adjustment = Some(method);
        self
    }

    /// Total endpoint count
    #[must_use]
    pub fn total_endpoints(&self) -> usize {
        self.primary.len() + self.secondary.len() + self.exploratory.len()
    }

    /// Check if all primary endpoints met
    #[must_use]
    pub fn primary_success(&self) -> bool {
        !self.primary.is_empty() && self.primary.iter().all(|e| e.is_successful() == Some(true))
    }

    /// Count successful secondary endpoints
    #[must_use]
    pub fn secondary_success_count(&self) -> usize {
        self.secondary
            .iter()
            .filter(|e| e.is_successful() == Some(true))
            .count()
    }

    /// FDA substantial evidence assessment
    #[must_use]
    pub fn demonstrates_substantial_evidence(&self) -> bool {
        // FDA requires primary endpoint success
        // Secondary endpoints provide supporting evidence
        self.primary_success()
    }
}

/// Multiplicity adjustment methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiplicityMethod {
    /// Bonferroni correction
    Bonferroni,
    /// Holm-Bonferroni step-down
    HolmBonferroni,
    /// Hochberg step-up
    Hochberg,
    /// Fixed-sequence (hierarchical)
    FixedSequence,
    /// Graphical approach (Bretz-Maurer-Hommel)
    Graphical,
    /// Fallback (Wiens)
    Fallback,
    /// No adjustment (single primary endpoint)
    None,
}

impl fmt::Display for MultiplicityMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bonferroni => write!(f, "Bonferroni"),
            Self::HolmBonferroni => write!(f, "Holm-Bonferroni"),
            Self::Hochberg => write!(f, "Hochberg"),
            Self::FixedSequence => write!(f, "Fixed-sequence"),
            Self::Graphical => write!(f, "Graphical"),
            Self::Fallback => write!(f, "Fallback"),
            Self::None => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_hierarchy() {
        let result = EndpointResult::new(
            EffectSize::new(0.7),
            PValue::new_unchecked(0.01),
            ConfidenceInterval::ci95(1.2, 2.1),
            200,
        );

        let hierarchy = EndpointHierarchy::new()
            .with_primary(PrimaryEndpoint::primary("Overall Survival").with_result(result.clone()))
            .with_secondary(SecondaryEndpoint::secondary("PFS", true).with_result(result))
            .with_multiplicity(MultiplicityMethod::FixedSequence);

        assert!(hierarchy.primary_success());
        assert!(hierarchy.demonstrates_substantial_evidence());
        assert_eq!(hierarchy.secondary_success_count(), 1);
    }

    #[test]
    fn test_effectiveness_standard() {
        let effective = EndpointResult::new(
            EffectSize::new(0.8),
            PValue::new_unchecked(0.01),
            ConfidenceInterval::ci95(1.5, 3.0),
            500,
        );
        assert!(effective.meets_effectiveness_standard());

        let not_clinical = EndpointResult::new(
            EffectSize::new(0.2), // Too small
            PValue::new_unchecked(0.01),
            ConfidenceInterval::ci95(1.1, 1.3),
            500,
        );
        assert!(!not_clinical.meets_effectiveness_standard());
    }
}
