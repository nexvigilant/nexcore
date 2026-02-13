//! Combined Adverse Event Assessment
//!
//! Orchestrates causality + seriousness + expectedness into a single
//! regulatory recommendation. This is the sovereign replacement for
//! guardian-epa1's `services/assessment.rs`.
//!
//! # Safety Axioms
//!
//! Implements the **ICH E2A Decision Matrix**: the combination of seriousness
//! and expectedness determines reporting timeline. Conservation Law CL4
//! ("No unexpected harm shall go unreported") drives the expedited logic.
//!
//! # Reporting Matrix (ICH E2A / FDA 21 CFR 314.80)
//!
//! | Serious | Unexpected | Fatal | Deadline |
//! |---------|------------|-------|----------|
//! | Yes | Yes | Yes | 7 calendar days |
//! | Yes | Yes | No | 15 calendar days |
//! | Yes | No | - | Periodic (PSUR/PBRER) |
//! | No | - | - | Periodic (PSUR/PBRER) |

use serde::{Deserialize, Serialize};

use super::expectedness::{ExpectednessInput, ExpectednessResult, assess_expectedness};
use super::seriousness::{SeriousnessInput, SeriousnessResult, assess_seriousness};
use crate::causality::{WhoUmcCategory, WhoUmcResult, calculate_who_umc_quick};

/// Regulatory reporting deadline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportingDeadline {
    /// Fatal unexpected: 7 calendar days (FDA 21 CFR 314.80)
    SevenDays,
    /// Serious unexpected: 15 calendar days (ICH E2A)
    FifteenDays,
    /// All other: periodic safety reports (PSUR/PBRER)
    Periodic,
}

impl ReportingDeadline {
    /// Human-readable deadline string
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SevenDays => "7 calendar days",
            Self::FifteenDays => "15 calendar days",
            Self::Periodic => "Periodic reporting (PSUR/PBRER)",
        }
    }
}

impl std::fmt::Display for ReportingDeadline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Input for combined assessment
///
/// Wraps the three individual assessment inputs. The combined function
/// runs all three assessments and derives the regulatory recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedAssessmentInput {
    /// WHO-UMC quick causality factors
    pub temporal: i32,
    pub dechallenge: i32,
    pub rechallenge: i32,
    pub alternatives: i32,
    pub plausibility: i32,
    /// Seriousness criteria
    pub seriousness: SeriousnessInput,
    /// Expectedness criteria
    pub expectedness: ExpectednessInput,
}

/// Result of combined regulatory assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedAssessmentResult {
    /// WHO-UMC causality result
    pub causality: WhoUmcResult,
    /// ICH E2A seriousness result
    pub seriousness: SeriousnessResult,
    /// RSI-based expectedness result
    pub expectedness: ExpectednessResult,
    /// Whether expedited reporting is required
    pub requires_expedited: bool,
    /// Reporting deadline per ICH E2A / FDA 21 CFR 314.80
    pub deadline: ReportingDeadline,
    /// Human-readable regulatory rationale
    pub rationale: String,
}

/// Perform combined assessment: causality + seriousness + expectedness → regulatory recommendation
///
/// This function replaces guardian-epa1's cloud-only `assess_combined()` with a
/// sovereign, zero-dependency implementation using vigilance's existing algorithms.
#[must_use]
pub fn assess_combined(input: &CombinedAssessmentInput) -> CombinedAssessmentResult {
    // 1. Causality (WHO-UMC quick)
    let causality = calculate_who_umc_quick(
        input.temporal,
        input.dechallenge,
        input.rechallenge,
        input.alternatives,
        input.plausibility,
    );

    // 2. Seriousness (ICH E2A)
    let seriousness = assess_seriousness(&input.seriousness);

    // 3. Expectedness (RSI comparison)
    let expectedness = assess_expectedness(&input.expectedness);

    // 4. ICH E2A decision matrix
    let is_unexpected = !expectedness.is_expected;
    let is_fatal = input.seriousness.death;
    let requires_expedited = seriousness.is_serious && is_unexpected;

    let deadline = if is_fatal && is_unexpected {
        ReportingDeadline::SevenDays
    } else if seriousness.is_serious && is_unexpected {
        ReportingDeadline::FifteenDays
    } else {
        ReportingDeadline::Periodic
    };

    let rationale = derive_rationale(
        requires_expedited,
        seriousness.is_serious,
        is_fatal,
        &causality.category,
    );

    CombinedAssessmentResult {
        causality,
        seriousness,
        expectedness,
        requires_expedited,
        deadline,
        rationale,
    }
}

/// Derive regulatory rationale from assessment components
fn derive_rationale(
    requires_expedited: bool,
    is_serious: bool,
    is_fatal: bool,
    causality_category: &WhoUmcCategory,
) -> String {
    let causality_str = match causality_category {
        WhoUmcCategory::Certain => "certain",
        WhoUmcCategory::ProbableLikely => "probable",
        WhoUmcCategory::Possible => "possible",
        WhoUmcCategory::Unlikely => "unlikely",
        WhoUmcCategory::ConditionalUnclassified => "conditional/unclassified",
        WhoUmcCategory::UnassessableUnclassifiable => "unassessable",
    };

    if requires_expedited {
        if is_fatal {
            format!(
                "Fatal unexpected event with {causality_str} causality — \
                 requires 7-day expedited reporting per ICH E2A / FDA 21 CFR 314.80"
            )
        } else {
            format!(
                "Serious unexpected event with {causality_str} causality — \
                 requires 15-day expedited reporting per ICH E2A"
            )
        }
    } else if is_serious {
        format!(
            "Serious expected event with {causality_str} causality — \
             include in periodic safety reports (PSUR/PBRER)"
        )
    } else {
        format!(
            "Non-serious event with {causality_str} causality — \
             include in periodic safety reports"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(
        death: bool,
        life_threatening: bool,
        hospitalization: bool,
        listed_in_rsi: bool,
    ) -> CombinedAssessmentInput {
        CombinedAssessmentInput {
            temporal: 1,
            dechallenge: 1,
            rechallenge: 0,
            alternatives: 0,
            plausibility: 1,
            seriousness: SeriousnessInput {
                death,
                life_threatening,
                hospitalization,
                ..Default::default()
            },
            expectedness: ExpectednessInput {
                event_term: "Test Event".to_string(),
                product_name: "Drug X".to_string(),
                listed_in_rsi,
                ..Default::default()
            },
        }
    }

    #[test]
    fn combined_fatal_unexpected() {
        let input = make_input(true, false, false, false);
        let result = assess_combined(&input);

        assert!(result.requires_expedited);
        assert_eq!(result.deadline, ReportingDeadline::SevenDays);
        assert!(result.seriousness.is_serious);
        assert!(!result.expectedness.is_expected);
        assert!(result.rationale.contains("Fatal"));
        assert!(result.rationale.contains("7-day"));
    }

    #[test]
    fn combined_serious_unexpected() {
        let input = make_input(false, false, true, false);
        let result = assess_combined(&input);

        assert!(result.requires_expedited);
        assert_eq!(result.deadline, ReportingDeadline::FifteenDays);
        assert!(result.seriousness.is_serious);
        assert!(!result.expectedness.is_expected);
        assert!(result.rationale.contains("15-day"));
    }

    #[test]
    fn combined_serious_expected() {
        let input = make_input(false, false, true, true);
        let result = assess_combined(&input);

        assert!(!result.requires_expedited);
        assert_eq!(result.deadline, ReportingDeadline::Periodic);
        assert!(result.seriousness.is_serious);
        assert!(result.expectedness.is_expected);
        assert!(result.rationale.contains("periodic"));
    }

    #[test]
    fn combined_non_serious() {
        let input = make_input(false, false, false, true);
        let result = assess_combined(&input);

        assert!(!result.requires_expedited);
        assert_eq!(result.deadline, ReportingDeadline::Periodic);
        assert!(!result.seriousness.is_serious);
        assert!(result.rationale.contains("Non-serious"));
    }

    #[test]
    fn combined_life_threatening_unexpected() {
        let input = make_input(false, true, false, false);
        let result = assess_combined(&input);

        assert!(result.requires_expedited);
        // Life-threatening but not fatal → 15-day
        assert_eq!(result.deadline, ReportingDeadline::FifteenDays);
        assert!(result.seriousness.is_serious);
    }

    #[test]
    fn deadline_display() {
        assert_eq!(ReportingDeadline::SevenDays.to_string(), "7 calendar days");
        assert_eq!(
            ReportingDeadline::FifteenDays.to_string(),
            "15 calendar days"
        );
        assert_eq!(
            ReportingDeadline::Periodic.to_string(),
            "Periodic reporting (PSUR/PBRER)"
        );
    }
}
