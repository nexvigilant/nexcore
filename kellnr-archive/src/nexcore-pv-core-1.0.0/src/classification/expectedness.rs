//! Expectedness Assessment (RSI Comparison)
//!
//! Determines whether an adverse event is "expected" (listed/labeled) or
//! "unexpected" (unlisted/unlabeled) based on Reference Safety Information (RSI).
//!
//! # Safety Axioms
//!
//! This module implements **Safety Axiom S2: Expectedness Classification** - the
//! determination of whether an adverse event was anticipated based on known
//! product safety information. This is critical for Conservation Law CL4:
//! "No unexpected harm shall go unreported within regulatory timelines."
//!
//! # Regulatory Importance
//!
//! The combination of seriousness and expectedness determines reporting:
//!
//! | Serious | Expected | Requirement |
//! |---------|----------|-------------|
//! | Yes | No (Unexpected) | **15-day expedited report** |
//! | Yes | Yes (Expected) | Periodic reporting |
//! | No | Either | Periodic reporting |
//!
//! # Expectedness Categories
//!
//! - `ExpectedListed`: Event in RSI with matching nature/severity/frequency
//! - `ExpectedClassEffect`: Not in RSI but known class effect
//! - `UnexpectedUnlisted`: Not in RSI, not a class effect
//! - `UnexpectedIncreasedSeverity`: Listed but more severe than labeled
//! - `UnexpectedIncreasedFrequency`: Listed but more frequent than labeled
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::pv::classification::expectedness::{
//!     ExpectednessInput, assess_expectedness, TermMatchType,
//! };
//!
//! let input = ExpectednessInput {
//!     event_term: "Headache".to_string(),
//!     product_name: "Drug X".to_string(),
//!     listed_in_rsi: true,
//!     rsi_version: Some("v3.0".to_string()),
//!     rsi_section: Some("Section 4.8".to_string()),
//!     term_match_type: Some(TermMatchType::Exact),
//!     severity_vs_listed: None,
//!     frequency_vs_listed: None,
//!     is_class_effect: false,
//!     class_effect_justification: None,
//! };
//!
//! let result = assess_expectedness(&input);
//! assert!(result.is_expected);
//! ```
//!
//! # Reference
//!
//! ICH E2A, EMA GVP Module VII

use serde::{Deserialize, Serialize};

/// How the reported term matches the RSI listing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TermMatchType {
    /// Exact term match in RSI
    Exact,
    /// Synonymous term (same meaning)
    Synonymous,
    /// Broader term (RSI is more general)
    Broader,
    /// Narrower term (RSI is more specific)
    Narrower,
    /// Not listed in RSI
    NotListed,
}

/// Comparison of observed severity vs RSI listing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SeverityComparison {
    /// Same severity as labeled
    #[default]
    Same,
    /// Less severe than labeled
    LessSevere,
    /// More severe than labeled (makes it unexpected)
    MoreSevere,
    /// Severity not specified in RSI
    NotSpecified,
}

/// Comparison of observed frequency vs RSI listing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FrequencyComparison {
    /// Same frequency as labeled
    #[default]
    Same,
    /// Less frequent than labeled
    LessFrequent,
    /// More frequent than labeled (makes it unexpected)
    MoreFrequent,
    /// Frequency not specified in RSI
    NotSpecified,
}

/// Expectedness category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectednessCategory {
    /// Event is listed in RSI with consistent nature/severity/frequency
    ExpectedListed,
    /// Event not in RSI but is a known class effect
    ExpectedClassEffect,
    /// Event not in RSI, not a class effect
    UnexpectedUnlisted,
    /// Event in RSI but observed severity exceeds label
    UnexpectedIncreasedSeverity,
    /// Event in RSI but observed frequency exceeds label
    UnexpectedIncreasedFrequency,
}

impl std::fmt::Display for ExpectednessCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedListed => write!(f, "Expected (Listed)"),
            Self::ExpectedClassEffect => write!(f, "Expected (Class Effect)"),
            Self::UnexpectedUnlisted => write!(f, "Unexpected (Unlisted)"),
            Self::UnexpectedIncreasedSeverity => write!(f, "Unexpected (Increased Severity)"),
            Self::UnexpectedIncreasedFrequency => write!(f, "Unexpected (Increased Frequency)"),
        }
    }
}

/// Input for expectedness assessment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpectednessInput {
    /// The adverse event term (verbatim or preferred)
    pub event_term: String,
    /// Product name
    pub product_name: String,
    /// Whether event is listed in Reference Safety Information
    pub listed_in_rsi: bool,
    /// RSI version consulted
    pub rsi_version: Option<String>,
    /// RSI section containing the listing
    pub rsi_section: Option<String>,
    /// How the term matches the RSI
    pub term_match_type: Option<TermMatchType>,
    /// Severity comparison vs labeled
    pub severity_vs_listed: Option<SeverityComparison>,
    /// Frequency comparison vs labeled
    pub frequency_vs_listed: Option<FrequencyComparison>,
    /// Whether this is a known class effect
    pub is_class_effect: bool,
    /// Justification for class effect determination
    pub class_effect_justification: Option<String>,
}

/// Regulatory impact of expectedness determination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectednessRegulatoryImpact {
    /// Whether expedited reporting is required (if also serious)
    pub requires_expedited: bool,
    /// Regulatory justification
    pub reporting_justification: String,
}

/// Result of expectedness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectednessResult {
    /// Whether the event is expected
    pub is_expected: bool,
    /// Specific expectedness category
    pub category: ExpectednessCategory,
    /// Assessment confidence (0.0-1.0)
    pub confidence: f64,
    /// RSI reference if available
    pub rsi_reference: Option<String>,
    /// Regulatory reporting impact
    pub regulatory_impact: ExpectednessRegulatoryImpact,
}

/// Assess expectedness based on RSI comparison
///
/// Implements Safety Axiom S2 and Conservation Law CL4 by ensuring
/// proper classification of expected vs unexpected events.
#[must_use]
pub fn assess_expectedness(input: &ExpectednessInput) -> ExpectednessResult {
    // Case 1: Event is listed in RSI
    if input.listed_in_rsi {
        // Check if severity is increased beyond label
        if matches!(
            input.severity_vs_listed,
            Some(SeverityComparison::MoreSevere)
        ) {
            return ExpectednessResult {
                is_expected: false,
                category: ExpectednessCategory::UnexpectedIncreasedSeverity,
                confidence: 0.9,
                rsi_reference: input.rsi_section.clone(),
                regulatory_impact: ExpectednessRegulatoryImpact {
                    requires_expedited: true,
                    reporting_justification:
                        "Event listed in RSI but observed severity exceeds labeled information. \
                         Classifies as unexpected for regulatory purposes."
                            .to_string(),
                },
            };
        }

        // Check if frequency is increased beyond label
        if matches!(
            input.frequency_vs_listed,
            Some(FrequencyComparison::MoreFrequent)
        ) {
            return ExpectednessResult {
                is_expected: false,
                category: ExpectednessCategory::UnexpectedIncreasedFrequency,
                confidence: 0.85,
                rsi_reference: input.rsi_section.clone(),
                regulatory_impact: ExpectednessRegulatoryImpact {
                    requires_expedited: true,
                    reporting_justification:
                        "Event listed in RSI but observed frequency suggests increase beyond \
                         labeled information. May require expedited reporting."
                            .to_string(),
                },
            };
        }

        // Listed with consistent characteristics = expected
        return ExpectednessResult {
            is_expected: true,
            category: ExpectednessCategory::ExpectedListed,
            confidence: 0.95,
            rsi_reference: input.rsi_section.clone(),
            regulatory_impact: ExpectednessRegulatoryImpact {
                requires_expedited: false,
                reporting_justification:
                    "Expected events follow periodic reporting schedules unless other factors \
                     indicate expedited reporting is needed."
                        .to_string(),
            },
        };
    }

    // Case 2: Not in RSI but is a known class effect
    if input.is_class_effect {
        return ExpectednessResult {
            is_expected: true,
            category: ExpectednessCategory::ExpectedClassEffect,
            confidence: 0.8, // Lower confidence for class effect determination
            rsi_reference: None,
            regulatory_impact: ExpectednessRegulatoryImpact {
                requires_expedited: false,
                reporting_justification:
                    "Event is a known class effect. May be considered expected for regulatory \
                     purposes, though company policies may vary."
                        .to_string(),
            },
        };
    }

    // Case 3: Not in RSI, not a class effect = unexpected
    ExpectednessResult {
        is_expected: false,
        category: ExpectednessCategory::UnexpectedUnlisted,
        confidence: 0.95,
        rsi_reference: None,
        regulatory_impact: ExpectednessRegulatoryImpact {
            requires_expedited: true,
            reporting_justification:
                "Unexpected (unlabeled) events require expedited reporting when serious. \
                 The combination of serious + unexpected mandates 15-day submission."
                    .to_string(),
        },
    }
}

/// Generate human-readable rationale for expectedness determination
#[must_use]
pub fn generate_expectedness_rationale(
    result: &ExpectednessResult,
    input: &ExpectednessInput,
) -> String {
    let mut lines = Vec::new();

    match result.category {
        ExpectednessCategory::ExpectedListed => {
            lines.push("**EXPECTED (LABELED) EVENT**".to_string());
            lines.push(String::new());
            lines.push(format!(
                "The adverse event \"{}\" is listed in the Reference Safety Information (RSI) for {}.",
                input.event_term, input.product_name
            ));
            if let Some(ref section) = input.rsi_section {
                lines.push(format!("RSI Section: {section}"));
            }
            if let Some(ref version) = input.rsi_version {
                lines.push(format!("RSI Version: {version}"));
            }
            if let Some(match_type) = input.term_match_type {
                lines.push(format!("Match Type: {match_type:?}"));
            }
            lines.push(String::new());
            lines.push(
                "The nature, severity, and frequency are consistent with labeling.".to_string(),
            );
        }

        ExpectednessCategory::ExpectedClassEffect => {
            lines.push("**EXPECTED (CLASS EFFECT)**".to_string());
            lines.push(String::new());
            lines.push(format!(
                "The adverse event \"{}\" is not specifically listed in the RSI for {}, \
                 but is a known class effect.",
                input.event_term, input.product_name
            ));
            if let Some(ref justification) = input.class_effect_justification {
                lines.push(String::new());
                lines.push(format!("Justification: {justification}"));
            }
        }

        ExpectednessCategory::UnexpectedUnlisted => {
            lines.push("**UNEXPECTED (UNLABELED) EVENT**".to_string());
            lines.push(String::new());
            lines.push(format!(
                "The adverse event \"{}\" is NOT listed in the RSI for {}.",
                input.event_term, input.product_name
            ));
            if let Some(ref version) = input.rsi_version {
                lines.push(format!("RSI Version Reviewed: {version}"));
            }
            lines.push(String::new());
            lines.push("If serious, expedited reporting is required.".to_string());
        }

        ExpectednessCategory::UnexpectedIncreasedSeverity => {
            lines.push("**UNEXPECTED (INCREASED SEVERITY)**".to_string());
            lines.push(String::new());
            lines.push(format!(
                "While \"{}\" is listed in the RSI for {}, the observed severity is \
                 GREATER than described in labeling.",
                input.event_term, input.product_name
            ));
            lines.push(String::new());
            lines.push("Classifies as UNEXPECTED for regulatory purposes.".to_string());
        }

        ExpectednessCategory::UnexpectedIncreasedFrequency => {
            lines.push("**UNEXPECTED (INCREASED FREQUENCY)**".to_string());
            lines.push(String::new());
            lines.push(format!(
                "While \"{}\" is listed in the RSI for {}, the observed frequency suggests \
                 an increase beyond labeled information.",
                input.event_term, input.product_name
            ));
            lines.push(String::new());
            lines.push("May require expedited reporting.".to_string());
        }
    }

    lines.push(String::new());
    lines.push(format!("Confidence: {:.0}%", result.confidence * 100.0));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_listed() {
        let input = ExpectednessInput {
            event_term: "Headache".to_string(),
            product_name: "Drug X".to_string(),
            listed_in_rsi: true,
            rsi_section: Some("Section 4.8".to_string()),
            term_match_type: Some(TermMatchType::Exact),
            ..Default::default()
        };
        let result = assess_expectedness(&input);

        assert!(result.is_expected);
        assert_eq!(result.category, ExpectednessCategory::ExpectedListed);
        assert!(!result.regulatory_impact.requires_expedited);
    }

    #[test]
    fn test_unexpected_unlisted() {
        let input = ExpectednessInput {
            event_term: "Novel Syndrome".to_string(),
            product_name: "Drug Y".to_string(),
            listed_in_rsi: false,
            is_class_effect: false,
            ..Default::default()
        };
        let result = assess_expectedness(&input);

        assert!(!result.is_expected);
        assert_eq!(result.category, ExpectednessCategory::UnexpectedUnlisted);
        assert!(result.regulatory_impact.requires_expedited);
    }

    #[test]
    fn test_expected_class_effect() {
        let input = ExpectednessInput {
            event_term: "Nausea".to_string(),
            product_name: "Drug Z".to_string(),
            listed_in_rsi: false,
            is_class_effect: true,
            class_effect_justification: Some("Common to all opioids".to_string()),
            ..Default::default()
        };
        let result = assess_expectedness(&input);

        assert!(result.is_expected);
        assert_eq!(result.category, ExpectednessCategory::ExpectedClassEffect);
        assert!((result.confidence - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unexpected_increased_severity() {
        let input = ExpectednessInput {
            event_term: "Rash".to_string(),
            product_name: "Drug A".to_string(),
            listed_in_rsi: true,
            severity_vs_listed: Some(SeverityComparison::MoreSevere),
            ..Default::default()
        };
        let result = assess_expectedness(&input);

        assert!(!result.is_expected);
        assert_eq!(
            result.category,
            ExpectednessCategory::UnexpectedIncreasedSeverity
        );
        assert!(result.regulatory_impact.requires_expedited);
    }

    #[test]
    fn test_unexpected_increased_frequency() {
        let input = ExpectednessInput {
            event_term: "Dizziness".to_string(),
            product_name: "Drug B".to_string(),
            listed_in_rsi: true,
            frequency_vs_listed: Some(FrequencyComparison::MoreFrequent),
            ..Default::default()
        };
        let result = assess_expectedness(&input);

        assert!(!result.is_expected);
        assert_eq!(
            result.category,
            ExpectednessCategory::UnexpectedIncreasedFrequency
        );
    }

    #[test]
    fn test_rationale_generation() {
        let input = ExpectednessInput {
            event_term: "Headache".to_string(),
            product_name: "Drug X".to_string(),
            listed_in_rsi: true,
            rsi_version: Some("v3.0".to_string()),
            term_match_type: Some(TermMatchType::Exact),
            ..Default::default()
        };
        let result = assess_expectedness(&input);
        let rationale = generate_expectedness_rationale(&result, &input);

        assert!(rationale.contains("EXPECTED"));
        assert!(rationale.contains("Headache"));
        assert!(rationale.contains("v3.0"));
    }
}
