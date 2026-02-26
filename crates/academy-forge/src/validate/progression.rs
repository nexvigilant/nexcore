//! Difficulty progression validation rules (R20-R23).
//!
//! ## Implementation Status
//!
//! | Rule | Status | Description |
//! |------|--------|-------------|
//! | R20  | Not yet implemented | Bloom level increases across stages |
//! | R21  | Implemented | Passing score non-decreasing across stages |
//! | R22  | Not yet implemented | Activity count increases across stages |
//! | R23  | Implemented | Stage duration non-decreasing (after stage 2) |

use crate::validate::{Severity, ValidationFinding};

/// Run progression validation rules R20-R23.
///
/// Currently implements R21 and R23.  R20 and R22 are reserved but not yet
/// implemented — they return no findings until implemented.
pub fn validate_progression(content: &serde_json::Value) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();

    validate_passing_score_progression(content, &mut findings);
    validate_duration_progression(content, &mut findings);

    findings
}

/// R21: Passing score progression — later stages have equal or higher passing scores.
fn validate_passing_score_progression(
    content: &serde_json::Value,
    findings: &mut Vec<ValidationFinding>,
) {
    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        let mut prev_score: Option<u64> = None;
        for (si, stage) in stages.iter().enumerate() {
            if let Some(passing) = stage.get("passingScore").and_then(|p| p.as_u64()) {
                if let Some(prev) = prev_score {
                    if passing < prev {
                        findings.push(ValidationFinding {
                            rule: "R21".to_string(),
                            severity: Severity::Advisory,
                            message: format!(
                                "Passing score drops from {prev} to {passing} at stage {si}"
                            ),
                            field_path: Some(format!("stages[{si}].passingScore")),
                        });
                    }
                }
                prev_score = Some(passing);
            }
        }
    }
}

/// R23: Duration progression — later stages are equal or longer.
fn validate_duration_progression(
    content: &serde_json::Value,
    findings: &mut Vec<ValidationFinding>,
) {
    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        let mut prev_duration: Option<u64> = None;
        for (si, stage) in stages.iter().enumerate() {
            let stage_duration: u64 = stage
                .get("activities")
                .and_then(|a| a.as_array())
                .map(|activities| {
                    activities
                        .iter()
                        .filter_map(|a| a.get("estimatedDuration").and_then(|d| d.as_u64()))
                        .sum()
                })
                .unwrap_or(0);

            if stage_duration > 0 {
                if let Some(prev) = prev_duration {
                    if stage_duration < prev && si > 2 {
                        // Only flag after early stages
                        findings.push(ValidationFinding {
                            rule: "R23".to_string(),
                            severity: Severity::Advisory,
                            message: format!(
                                "Duration drops from {prev}min to {stage_duration}min at stage {si}"
                            ),
                            field_path: Some(format!("stages[{si}]")),
                        });
                    }
                }
                prev_duration = Some(stage_duration);
            }
        }
    }
}
