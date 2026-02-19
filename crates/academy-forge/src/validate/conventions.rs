//! Structural consistency validation rules (R15-R19).

use crate::validate::{Severity, ValidationFinding};

/// Run conventions validation rules R15-R19.
pub fn validate_conventions(content: &serde_json::Value) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();

    validate_consistent_point_values(content, &mut findings);
    validate_quiz_type_distribution(content, &mut findings);

    findings
}

/// R18: Consistent point values within each stage.
fn validate_consistent_point_values(
    content: &serde_json::Value,
    findings: &mut Vec<ValidationFinding>,
) {
    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        for (si, stage) in stages.iter().enumerate() {
            let mut point_values = Vec::new();
            if let Some(activities) = stage.get("activities").and_then(|a| a.as_array()) {
                for activity in activities {
                    if let Some(quiz) = activity.get("quiz") {
                        if let Some(questions) = quiz.get("questions").and_then(|q| q.as_array()) {
                            for question in questions {
                                if let Some(points) =
                                    question.get("points").and_then(|p| p.as_u64())
                                {
                                    point_values.push(points);
                                }
                            }
                        }
                    }
                }
            }

            if point_values.len() > 1 {
                let first = point_values[0];
                if point_values.iter().any(|&p| p != first) {
                    findings.push(ValidationFinding {
                        rule: "R18".to_string(),
                        severity: Severity::Advisory,
                        message: format!(
                            "Inconsistent point values in stage {si}: {:?}",
                            point_values
                        ),
                        field_path: Some(format!("stages[{si}]")),
                    });
                }
            }
        }
    }
}

/// R19: Quiz type distribution (balance of MC/TF/MS across stages).
fn validate_quiz_type_distribution(
    content: &serde_json::Value,
    findings: &mut Vec<ValidationFinding>,
) {
    let mut mc_count = 0u64;
    let mut tf_count = 0u64;
    let mut ms_count = 0u64;

    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        for stage in stages {
            if let Some(activities) = stage.get("activities").and_then(|a| a.as_array()) {
                for activity in activities {
                    if let Some(quiz) = activity.get("quiz") {
                        if let Some(questions) = quiz.get("questions").and_then(|q| q.as_array()) {
                            for question in questions {
                                match question.get("type").and_then(|t| t.as_str()) {
                                    Some("multiple-choice") => mc_count += 1,
                                    Some("true-false") => tf_count += 1,
                                    Some("multiple-select") => ms_count += 1,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let total = mc_count + tf_count + ms_count;
    if total > 0 && tf_count as f64 / total as f64 > 0.5 {
        findings.push(ValidationFinding {
            rule: "R19".to_string(),
            severity: Severity::Advisory,
            message: format!(
                "Heavy true-false bias: {tf_count}/{total} questions are true-false (>50%)"
            ),
            field_path: None,
        });
    }
}
