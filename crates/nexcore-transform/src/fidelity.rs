//! Fidelity scoring: structural preservation measurement.
//!
//! Scores how well a transformation preserves the source structure:
//! paragraph count match, concept coverage, and mapping confidence.
//!
//! Formula: `score = paragraph_match * 0.3 + mean_coverage * 0.4 + aggregate_confidence * 0.3`

use crate::annotation::ConceptAnnotation;
use crate::plan::TransformationPlan;
use serde::{Deserialize, Serialize};

/// Fidelity report for a completed transformation.
///
/// Tier: T2-C | Dominant: kappa (Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityReport {
    /// Plan ID this report is for.
    pub plan_id: String,
    /// Source paragraph count.
    pub source_paragraphs: usize,
    /// Output paragraph count.
    pub output_paragraphs: usize,
    /// Paragraph count match score (1.0 if equal, decays with difference).
    pub paragraph_match: f64,
    /// Per-paragraph concept coverage ratios.
    pub coverage_per_paragraph: Vec<f64>,
    /// Mean concept coverage across paragraphs.
    pub mean_coverage: f64,
    /// Aggregate mapping confidence from the plan.
    pub aggregate_confidence: f64,
    /// Final fidelity score (0.0..=1.0).
    pub fidelity_score: f64,
}

/// Score the fidelity of a transformation output.
///
/// # Arguments
/// - `plan`: The transformation plan that was executed.
/// - `output_paragraph_count`: Number of paragraphs in the output.
/// - `concept_hits`: Number of successfully mapped concepts found in output
///   (per paragraph, parallel to plan paragraphs; use empty vec if unknown).
pub fn score_fidelity(
    plan: &TransformationPlan,
    output_paragraph_count: usize,
    concept_hits: &[usize],
) -> FidelityReport {
    let source_count = plan.source.paragraphs.len();

    // Paragraph match: 1.0 if equal, decays with difference
    let paragraph_match = if source_count == 0 && output_paragraph_count == 0 {
        1.0
    } else if source_count == 0 || output_paragraph_count == 0 {
        0.0
    } else {
        let ratio = output_paragraph_count as f64 / source_count as f64;
        // Symmetric around 1.0: both expansion and contraction reduce score
        if ratio >= 1.0 { 1.0 / ratio } else { ratio }
    };

    // Coverage per paragraph
    let coverage_per_paragraph: Vec<f64> = plan
        .annotations
        .iter()
        .enumerate()
        .map(|(i, ann)| compute_coverage(ann, concept_hits.get(i).copied()))
        .collect();

    let mean_coverage = if coverage_per_paragraph.is_empty() {
        0.0
    } else {
        let sum: f64 = coverage_per_paragraph.iter().sum();
        sum / coverage_per_paragraph.len() as f64
    };

    let aggregate_confidence = plan.mapping_table.aggregate_confidence;

    // Weighted score
    let fidelity_score = paragraph_match * 0.3 + mean_coverage * 0.4 + aggregate_confidence * 0.3;

    FidelityReport {
        plan_id: plan.plan_id.clone(),
        source_paragraphs: source_count,
        output_paragraphs: output_paragraph_count,
        paragraph_match,
        coverage_per_paragraph,
        mean_coverage,
        aggregate_confidence,
        fidelity_score,
    }
}

/// Compute concept coverage for a single paragraph annotation.
///
/// If `hits` is provided, uses hits/total_concepts.
/// Otherwise falls back to the annotation's own coverage metric.
fn compute_coverage(ann: &ConceptAnnotation, hits: Option<usize>) -> f64 {
    if ann.concepts.is_empty() {
        return 1.0; // No concepts = nothing to miss = full coverage
    }
    match hits {
        Some(h) => {
            let total = ann.concepts.len();
            (h.min(total) as f64) / total as f64
        }
        None => ann.coverage(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::compile_plan;
    use crate::profile::builtin_pharmacovigilance;

    const SAMPLE: &str =
        "The citizen faces danger.\n\nVigilance is our duty.\n\nThe union must act.";

    #[test]
    fn test_perfect_fidelity() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);
        let source_count = plan.source.paragraphs.len();

        // Perfect: same paragraph count, all concepts hit
        let hits: Vec<usize> = plan.annotations.iter().map(|a| a.concepts.len()).collect();
        let report = score_fidelity(&plan, source_count, &hits);

        assert_eq!(report.paragraph_match, 1.0);
        assert!(report.fidelity_score > 0.5);
    }

    #[test]
    fn test_paragraph_mismatch_reduces_score() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);

        // Mismatch: output has 6 paragraphs, source has 3
        let report = score_fidelity(&plan, 6, &[]);
        assert!(report.paragraph_match < 1.0);
        assert!((report.paragraph_match - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_empty_plan_fidelity() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Empty", "", "none", &pv);
        let report = score_fidelity(&plan, 0, &[]);
        assert_eq!(report.paragraph_match, 1.0);
        assert_eq!(report.source_paragraphs, 0);
    }

    #[test]
    fn test_fidelity_score_in_range() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);
        let report = score_fidelity(&plan, 3, &[]);
        assert!(
            (0.0..=1.0).contains(&report.fidelity_score),
            "score {} out of range",
            report.fidelity_score
        );
    }

    #[test]
    fn test_no_concepts_full_coverage() {
        let ann = ConceptAnnotation {
            paragraph_index: 0,
            concepts: vec![],
        };
        assert_eq!(compute_coverage(&ann, None), 1.0);
    }

    #[test]
    fn test_coverage_with_hits() {
        let ann = ConceptAnnotation {
            paragraph_index: 0,
            concepts: vec![
                crate::annotation::ConceptOccurrence {
                    term: "a".into(),
                    has_bridge: true,
                },
                crate::annotation::ConceptOccurrence {
                    term: "b".into(),
                    has_bridge: true,
                },
            ],
        };
        assert!((compute_coverage(&ann, Some(1)) - 0.5).abs() < 0.001);
        assert!((compute_coverage(&ann, Some(2)) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_hits_capped_at_total() {
        let ann = ConceptAnnotation {
            paragraph_index: 0,
            concepts: vec![crate::annotation::ConceptOccurrence {
                term: "a".into(),
                has_bridge: true,
            }],
        };
        // Passing more hits than concepts should cap at 1.0
        assert!((compute_coverage(&ann, Some(5)) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_symmetric_paragraph_scoring() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE, "politics", &pv);
        let source_count = plan.source.paragraphs.len(); // 3

        // 2 output paragraphs (contracted)
        let report_fewer = score_fidelity(&plan, source_count - 1, &[]);
        // 4 output paragraphs (expanded)
        let report_more = score_fidelity(&plan, source_count + 1, &[]);

        // Both should reduce paragraph_match below 1.0
        assert!(report_fewer.paragraph_match < 1.0);
        assert!(report_more.paragraph_match < 1.0);
    }
}
