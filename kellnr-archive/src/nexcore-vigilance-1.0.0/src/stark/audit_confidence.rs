//! # Dead File Audit Confidence Calculator
//!
//! Calculates deletion confidence using Bayesian evidence weighting
//! and Wilson score confidence intervals.
//!
//! ## Usage
//!
//! ```bash
//! # All evidence confirmed
//! audit-confidence --evidence D G B E A
//!
//! # Some partial
//! audit-confidence --evidence D G B --partial E A
//!
//! # Interactive mode
//! audit-confidence --interactive
//! ```

use std::io::{self, Write};

/// Evidence check for deletion safety assessment.
#[derive(Debug, Clone)]
pub struct EvidenceCheck {
    /// Short code (D, G, B, E, A)
    pub code: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// Description of what this evidence checks
    pub description: &'static str,
    /// Weight in final calculation (0.0 - 1.0)
    pub weight: f64,
    /// Score: 0.0 = not checked, 0.5 = partial, 1.0 = confirmed
    pub score: f64,
}

impl EvidenceCheck {
    /// Calculate weighted contribution to confidence.
    pub fn contribution(&self) -> f64 {
        self.weight * self.score
    }
}

/// Standard evidence categories for deletion confidence.
pub fn standard_evidence() -> Vec<EvidenceCheck> {
    vec![
        EvidenceCheck {
            code: "D",
            name: "No Dependencies",
            description: "Nothing imports this code externally",
            weight: 0.25,
            score: 0.0,
        },
        EvidenceCheck {
            code: "G",
            name: "Git Committed",
            description: "Source is version-controlled with clean working tree",
            weight: 0.2,
            score: 0.0,
        },
        EvidenceCheck {
            code: "B",
            name: "Build Artifacts",
            description: "Target contains only reproducible output",
            weight: 0.2,
            score: 0.0,
        },
        EvidenceCheck {
            code: "E",
            name: "Patterns Extracted",
            description: "Valuable code moved to permanent location",
            weight: 0.2,
            score: 0.0,
        },
        EvidenceCheck {
            code: "A",
            name: "No Recent Activity",
            description: "Last modified >14 days ago",
            weight: 0.15,
            score: 0.0,
        },
    ]
}

/// Result of Wilson score confidence interval calculation.
#[derive(Debug, Clone, Copy)]
pub struct WilsonInterval {
    /// Lower bound of confidence interval
    pub lower: f64,
    /// Upper bound of confidence interval
    pub upper: f64,
    /// Point estimate (successes / total)
    pub point: f64,
    /// Number of successes
    pub successes: usize,
    /// Total checks
    pub total: usize,
}

/// Calculate Wilson score confidence interval for binomial proportion.
///
/// This gives a statistically valid confidence interval for the "true"
/// probability of safe deletion, given the evidence checks performed.
///
/// # Arguments
///
/// * `successes` - Number of positive evidence checks (score >= 0.5)
/// * `total` - Total evidence checks performed
/// * `confidence_level` - Desired confidence level (0.90, 0.95, or 0.99)
///
/// # Returns
///
/// `WilsonInterval` containing lower bound, upper bound, and point estimate
pub fn wilson_confidence_interval(
    successes: usize,
    total: usize,
    confidence_level: f64,
) -> WilsonInterval {
    if total == 0 {
        return WilsonInterval {
            lower: 0.0,
            upper: 0.0,
            point: 0.0,
            successes,
            total,
        };
    }

    // Z-score for confidence level
    let z: f64 = match confidence_level {
        x if (x - 0.9).abs() < 0.001 => 1.645,
        x if (x - 0.99).abs() < 0.001 => 2.576,
        _ => 1.96, // Default to 95%
    };

    let n = total as f64;
    let p = (successes as f64) / n;

    let denominator = 1.0 + z.powi(2) / n;
    let center = (p + z.powi(2) / (2.0 * n)) / denominator;
    let spread = (z * ((p * (1.0 - p) + z.powi(2) / (4.0 * n)) / n).sqrt()) / denominator;

    WilsonInterval {
        lower: (center - spread).max(0.0),
        upper: (center + spread).min(1.0),
        point: p,
        successes,
        total,
    }
}

/// Calculate weighted confidence score from evidence checks.
pub fn calculate_weighted_confidence(evidence: &[EvidenceCheck]) -> f64 {
    let total_weight: f64 = evidence.iter().map(|e| e.weight).sum();
    if total_weight == 0.0 {
        return 0.0;
    }

    let weighted_sum: f64 = evidence.iter().map(|e| e.contribution()).sum();
    weighted_sum / total_weight
}

/// Action recommendation based on confidence score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recommendation {
    /// Confidence ≥ 95%: All evidence checks passed. Proceed with deletion.
    SafeToDelete,
    /// Confidence 80-95%: Most checks passed. Create backup before deletion.
    DeleteWithBackup,
    /// Confidence 60-80%: Some uncertainty. Extract valuable patterns first.
    ExtractFirst,
    /// Confidence < 60%: Insufficient evidence. Investigate further.
    DoNotDelete,
}

impl Recommendation {
    /// Get recommendation from confidence score.
    pub fn from_confidence(confidence: f64) -> Self {
        if confidence >= 0.95 {
            Self::SafeToDelete
        } else if confidence >= 0.8 {
            Self::DeleteWithBackup
        } else if confidence >= 0.6 {
            Self::ExtractFirst
        } else {
            Self::DoNotDelete
        }
    }

    /// Get emoji and label for display.
    pub fn display(&self) -> (&'static str, &'static str) {
        match self {
            Self::SafeToDelete => ("✅", "SAFE TO DELETE"),
            Self::DeleteWithBackup => ("⚠️", "DELETE WITH BACKUP"),
            Self::ExtractFirst => ("🔍", "EXTRACT FIRST"),
            Self::DoNotDelete => ("❌", "DO NOT DELETE"),
        }
    }

    /// Get explanation text.
    pub fn explanation(&self) -> &'static str {
        match self {
            Self::SafeToDelete => "All evidence checks passed. Proceed with deletion.",
            Self::DeleteWithBackup => "Most checks passed. Create backup before deletion.",
            Self::ExtractFirst => "Some uncertainty. Extract valuable patterns before deletion.",
            Self::DoNotDelete => "Insufficient evidence. Investigate further before any action.",
        }
    }
}

/// Full confidence assessment result.
#[derive(Debug)]
pub struct ConfidenceAssessment {
    /// Individual evidence checks with pass/fail status and weights.
    pub evidence: Vec<EvidenceCheck>,
    /// Weighted confidence score (0.0-1.0) combining all evidence.
    pub weighted_confidence: f64,
    /// Wilson score interval for statistical confidence bounds.
    pub wilson_interval: WilsonInterval,
    /// Action recommendation based on confidence thresholds.
    pub recommendation: Recommendation,
}

impl ConfidenceAssessment {
    /// Create assessment from evidence checks.
    pub fn assess(evidence: Vec<EvidenceCheck>) -> Self {
        let weighted_confidence = calculate_weighted_confidence(&evidence);
        let successes = evidence.iter().filter(|e| e.score >= 0.5).count();
        let wilson_interval = wilson_confidence_interval(successes, evidence.len(), 0.95);
        let recommendation = Recommendation::from_confidence(weighted_confidence);

        Self {
            evidence,
            weighted_confidence,
            wilson_interval,
            recommendation,
        }
    }

    /// Print formatted results to stdout.
    pub fn print_report(&self) {
        println!("\n{}", "-".repeat(60));
        println!("  EVIDENCE SUMMARY");
        println!("{}", "-".repeat(60));

        for e in &self.evidence {
            let score_str = if e.score >= 1.0 {
                "✓"
            } else if e.score >= 0.5 {
                "½"
            } else {
                "✗"
            };
            println!(
                "  [{}] {:20} (weight: {:.2}, score: {:.1})",
                score_str, e.name, e.weight, e.score
            );
        }

        println!("\n{}", "-".repeat(60));
        println!("  CONFIDENCE CALCULATION");
        println!("{}", "-".repeat(60));

        println!(
            "\n  Weighted Confidence:     {:.1}%",
            self.weighted_confidence * 100.0
        );
        println!(
            "  Point Estimate:          {:.1}% ({}/{} checks passed)",
            self.wilson_interval.point * 100.0,
            self.wilson_interval.successes,
            self.wilson_interval.total
        );
        println!(
            "  95% Confidence Interval: [{:.1}% - {:.1}%]",
            self.wilson_interval.lower * 100.0,
            self.wilson_interval.upper * 100.0
        );

        let (emoji, label) = self.recommendation.display();
        println!("\n{}", "-".repeat(60));
        println!("  RECOMMENDATION");
        println!("{}", "-".repeat(60));
        println!("\n  {} {}", emoji, label);
        println!("  {}", self.recommendation.explanation());

        println!("\n{}", "-".repeat(60));
        println!("  STATISTICAL NOTE");
        println!("{}", "-".repeat(60));
        println!(
            r#"
  The Wilson 95% CI [{:.1}% - {:.1}%] represents the range
  where the "true" deletion safety probability likely falls.
  
  With only {} evidence checks, even perfect scores yield wide intervals.
  To narrow the interval, add more independent evidence checks.
"#,
            self.wilson_interval.lower * 100.0,
            self.wilson_interval.upper * 100.0,
            self.wilson_interval.total
        );
    }
}

/// Run interactive confidence assessment.
pub fn run_interactive() -> io::Result<ConfidenceAssessment> {
    println!("\n{}", "=".repeat(60));
    println!("  DEAD FILE AUDIT CONFIDENCE CALCULATOR");
    println!("{}\n", "=".repeat(60));

    println!("For each evidence category, enter a score:");
    println!("  1.0 = Fully confirmed");
    println!("  0.5 = Partially confirmed");
    println!("  0.0 = Not confirmed / Unknown");
    println!();

    let mut evidence = standard_evidence();

    for e in &mut evidence {
        loop {
            print!("  [{}] {}: ", e.code, e.name);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let trimmed = input.trim();
            if trimmed.is_empty() {
                e.score = 0.0;
                break;
            }

            match trimmed.parse::<f64>() {
                Ok(score) if (0.0..=1.0).contains(&score) => {
                    e.score = score;
                    break;
                }
                _ => {
                    println!("      Please enter a value between 0.0 and 1.0");
                }
            }
        }
    }

    Ok(ConfidenceAssessment::assess(evidence))
}

/// Parse command-line arguments and run assessment.
pub fn run_with_args(confirmed: &[&str], partial: &[&str]) -> ConfidenceAssessment {
    let mut evidence = standard_evidence();

    for e in &mut evidence {
        if confirmed.iter().any(|c| c.eq_ignore_ascii_case(e.code)) {
            e.score = 1.0;
        } else if partial.iter().any(|p| p.eq_ignore_ascii_case(e.code)) {
            e.score = 0.5;
        }
    }

    ConfidenceAssessment::assess(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wilson_interval_all_success() {
        let interval = wilson_confidence_interval(5, 5, 0.95);
        assert!((interval.point - 1.0).abs() < f64::EPSILON);
        assert!(interval.lower > 0.5);
        assert!((interval.upper - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_wilson_interval_all_failure() {
        let interval = wilson_confidence_interval(0, 5, 0.95);
        assert!(interval.point.abs() < f64::EPSILON);
        assert!(interval.lower.abs() < f64::EPSILON);
        assert!(interval.upper < 0.5);
    }

    #[test]
    fn test_wilson_interval_mixed() {
        let interval = wilson_confidence_interval(3, 5, 0.95);
        assert!((interval.point - 0.6).abs() < f64::EPSILON);
        assert!(interval.lower < 0.6);
        assert!(interval.upper > 0.6);
    }

    #[test]
    fn test_weighted_confidence_all_confirmed() {
        let mut evidence = standard_evidence();
        for e in &mut evidence {
            e.score = 1.0;
        }
        let conf = calculate_weighted_confidence(&evidence);
        assert!((conf - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_weighted_confidence_none_confirmed() {
        let evidence = standard_evidence();
        let conf = calculate_weighted_confidence(&evidence);
        assert!(conf.abs() < f64::EPSILON);
    }

    #[test]
    fn test_recommendation_thresholds() {
        assert_eq!(
            Recommendation::from_confidence(1.0),
            Recommendation::SafeToDelete
        );
        assert_eq!(
            Recommendation::from_confidence(0.95),
            Recommendation::SafeToDelete
        );
        assert_eq!(
            Recommendation::from_confidence(0.94),
            Recommendation::DeleteWithBackup
        );
        assert_eq!(
            Recommendation::from_confidence(0.8),
            Recommendation::DeleteWithBackup
        );
        assert_eq!(
            Recommendation::from_confidence(0.79),
            Recommendation::ExtractFirst
        );
        assert_eq!(
            Recommendation::from_confidence(0.6),
            Recommendation::ExtractFirst
        );
        assert_eq!(
            Recommendation::from_confidence(0.59),
            Recommendation::DoNotDelete
        );
        assert_eq!(
            Recommendation::from_confidence(0.0),
            Recommendation::DoNotDelete
        );
    }

    #[test]
    fn test_full_assessment() {
        let assessment = run_with_args(&["D", "G", "B", "E", "A"], &[]);
        assert!((assessment.weighted_confidence - 1.0).abs() < f64::EPSILON);
        assert_eq!(assessment.recommendation, Recommendation::SafeToDelete);
    }
}
