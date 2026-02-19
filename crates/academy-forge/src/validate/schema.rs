//! Schema validation rules (R1-R8).
//!
//! Validates structural correctness of academy content JSON.

use crate::validate::{Severity, ValidationFinding};

/// Run schema validation rules R1-R8 against academy content.
pub fn validate_schema(content: &serde_json::Value) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();

    validate_required_fields(content, &mut findings);
    validate_id_naming(content, &mut findings);
    validate_component_count(content, &mut findings);
    validate_estimated_duration(content, &mut findings);
    validate_correct_answer_bounds(content, &mut findings);
    validate_no_duplicate_ids(content, &mut findings);
    validate_true_false_no_options(content, &mut findings);
    validate_mc_has_options(content, &mut findings);

    findings
}

/// R1: Required fields (id, title, description) at all levels.
fn validate_required_fields(content: &serde_json::Value, findings: &mut Vec<ValidationFinding>) {
    let required = ["id", "title", "description"];
    for field in &required {
        if content.get(*field).is_none() {
            findings.push(ValidationFinding {
                rule: "R1".to_string(),
                severity: Severity::Error,
                message: format!("Missing required field: {field}"),
                field_path: Some(field.to_string()),
            });
        }
    }

    // Check stages
    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        for (i, stage) in stages.iter().enumerate() {
            for field in &required {
                if stage.get(*field).is_none() {
                    findings.push(ValidationFinding {
                        rule: "R1".to_string(),
                        severity: Severity::Error,
                        message: format!("Missing required field: {field} in stage {i}"),
                        field_path: Some(format!("stages[{i}].{field}")),
                    });
                }
            }
        }
    }
}

/// R2: ID naming convention regex: `^[a-z][-a-z]*-\d{2}(-\d{2})?(-[a-z][a-z0-9-]*)?$`
///
/// Accepts any lowercase prefix (e.g., `tov-01`, `pv-ed-01`, `sig-02-03-a01`).
fn validate_id_naming(content: &serde_json::Value, findings: &mut Vec<ValidationFinding>) {
    let id_pattern = regex::Regex::new(r"^[a-z][-a-z]*-\d{2}(-\d{2})?(-[a-z][a-z0-9-]*)?$");
    let Ok(re) = id_pattern else { return };

    // Collect all IDs
    let ids = collect_all_ids(content);
    for (path, id) in &ids {
        if !re.is_match(id) {
            findings.push(ValidationFinding {
                rule: "R2".to_string(),
                severity: Severity::Error,
                message: format!(
                    "ID '{id}' does not match naming convention prefix-NN(-NN)?(-name)?"
                ),
                field_path: Some(path.clone()),
            });
        }
    }
}

/// R3: `componentCount` == actual count of activities across all stages.
fn validate_component_count(content: &serde_json::Value, findings: &mut Vec<ValidationFinding>) {
    let declared = content
        .get("metadata")
        .and_then(|m| m.get("componentCount"))
        .and_then(|c| c.as_u64());

    if let Some(declared_count) = declared {
        let mut actual_count: u64 = 0;
        if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
            for stage in stages {
                if let Some(activities) = stage.get("activities").and_then(|a| a.as_array()) {
                    actual_count += activities.len() as u64;
                }
            }
        }

        if declared_count != actual_count {
            findings.push(ValidationFinding {
                rule: "R3".to_string(),
                severity: Severity::Error,
                message: format!(
                    "componentCount is {declared_count} but actual activity count is {actual_count}"
                ),
                field_path: Some("metadata.componentCount".to_string()),
            });
        }
    }
}

/// R4: `estimatedDuration` == sum of activity durations.
fn validate_estimated_duration(content: &serde_json::Value, findings: &mut Vec<ValidationFinding>) {
    let declared = content
        .get("metadata")
        .and_then(|m| m.get("estimatedDuration"))
        .and_then(|d| d.as_u64());

    if let Some(declared_duration) = declared {
        let mut actual_duration: u64 = 0;
        if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
            for stage in stages {
                if let Some(activities) = stage.get("activities").and_then(|a| a.as_array()) {
                    for activity in activities {
                        if let Some(dur) =
                            activity.get("estimatedDuration").and_then(|d| d.as_u64())
                        {
                            actual_duration += dur;
                        }
                    }
                }
            }
        }

        if declared_duration != actual_duration {
            findings.push(ValidationFinding {
                rule: "R4".to_string(),
                severity: Severity::Error,
                message: format!(
                    "estimatedDuration is {declared_duration} but sum of activities is {actual_duration}"
                ),
                field_path: Some("metadata.estimatedDuration".to_string()),
            });
        }
    }
}

/// R5: `correctAnswer` index < options.length for MC/MS questions.
fn validate_correct_answer_bounds(
    content: &serde_json::Value,
    findings: &mut Vec<ValidationFinding>,
) {
    visit_quizzes(content, |path, quiz| {
        if let Some(questions) = quiz.get("questions").and_then(|q| q.as_array()) {
            for (qi, question) in questions.iter().enumerate() {
                let q_type = question.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if q_type == "multiple-choice" || q_type == "multiple-select" {
                    if let Some(correct) = question.get("correctAnswer").and_then(|c| c.as_u64()) {
                        let options_len = question
                            .get("options")
                            .and_then(|o| o.as_array())
                            .map(|a| a.len() as u64)
                            .unwrap_or(0);

                        if correct >= options_len {
                            findings.push(ValidationFinding {
                                rule: "R5".to_string(),
                                severity: Severity::Error,
                                message: format!(
                                    "correctAnswer index {correct} >= options length {options_len}"
                                ),
                                field_path: Some(format!("{path}.questions[{qi}].correctAnswer")),
                            });
                        }
                    }
                }
            }
        }
    });
}

/// R6: No duplicate IDs.
fn validate_no_duplicate_ids(content: &serde_json::Value, findings: &mut Vec<ValidationFinding>) {
    let ids = collect_all_ids(content);
    let mut seen = std::collections::HashMap::new();
    for (path, id) in &ids {
        if let Some(first_path) = seen.get(id) {
            findings.push(ValidationFinding {
                rule: "R6".to_string(),
                severity: Severity::Error,
                message: format!("Duplicate ID '{id}' (first at {first_path})"),
                field_path: Some(path.clone()),
            });
        } else {
            seen.insert(id.clone(), path.clone());
        }
    }
}

/// R7: `true-false` questions must NOT have `options` field.
fn validate_true_false_no_options(
    content: &serde_json::Value,
    findings: &mut Vec<ValidationFinding>,
) {
    visit_quizzes(content, |path, quiz| {
        if let Some(questions) = quiz.get("questions").and_then(|q| q.as_array()) {
            for (qi, question) in questions.iter().enumerate() {
                let q_type = question.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if q_type == "true-false" && question.get("options").is_some() {
                    findings.push(ValidationFinding {
                        rule: "R7".to_string(),
                        severity: Severity::Error,
                        message: "true-false question must NOT have options field".to_string(),
                        field_path: Some(format!("{path}.questions[{qi}].options")),
                    });
                }
            }
        }
    });
}

/// R8: `multiple-choice` questions MUST have `options` field.
fn validate_mc_has_options(content: &serde_json::Value, findings: &mut Vec<ValidationFinding>) {
    visit_quizzes(content, |path, quiz| {
        if let Some(questions) = quiz.get("questions").and_then(|q| q.as_array()) {
            for (qi, question) in questions.iter().enumerate() {
                let q_type = question.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if q_type == "multiple-choice" && question.get("options").is_none() {
                    findings.push(ValidationFinding {
                        rule: "R8".to_string(),
                        severity: Severity::Error,
                        message: "multiple-choice question MUST have options field".to_string(),
                        field_path: Some(format!("{path}.questions[{qi}].options")),
                    });
                }
            }
        }
    });
}

/// Collect all `id` fields from the content tree.
fn collect_all_ids(content: &serde_json::Value) -> Vec<(String, String)> {
    let mut ids = Vec::new();

    if let Some(id) = content.get("id").and_then(|v| v.as_str()) {
        ids.push(("id".to_string(), id.to_string()));
    }

    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        for (si, stage) in stages.iter().enumerate() {
            if let Some(id) = stage.get("id").and_then(|v| v.as_str()) {
                ids.push((format!("stages[{si}].id"), id.to_string()));
            }
            if let Some(activities) = stage.get("activities").and_then(|a| a.as_array()) {
                for (ai, activity) in activities.iter().enumerate() {
                    if let Some(id) = activity.get("id").and_then(|v| v.as_str()) {
                        ids.push((format!("stages[{si}].activities[{ai}].id"), id.to_string()));
                    }
                }
            }
        }
    }

    ids
}

/// Visit all quiz objects in the content tree.
fn visit_quizzes(content: &serde_json::Value, mut visitor: impl FnMut(&str, &serde_json::Value)) {
    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        for (si, stage) in stages.iter().enumerate() {
            if let Some(activities) = stage.get("activities").and_then(|a| a.as_array()) {
                for (ai, activity) in activities.iter().enumerate() {
                    if let Some(quiz) = activity.get("quiz") {
                        let path = format!("stages[{si}].activities[{ai}].quiz");
                        visitor(&path, quiz);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_r1_missing_required_fields() {
        let content = json!({});
        let findings = validate_schema(&content);
        let r1s: Vec<_> = findings.iter().filter(|f| f.rule == "R1").collect();
        assert_eq!(r1s.len(), 3); // id, title, description
    }

    #[test]
    fn test_r6_duplicate_ids() {
        let content = json!({
            "id": "tov-01",
            "title": "Test",
            "description": "Test",
            "stages": [
                { "id": "tov-01", "title": "Stage", "description": "Dup" }
            ]
        });
        let findings = validate_schema(&content);
        let r6s: Vec<_> = findings.iter().filter(|f| f.rule == "R6").collect();
        assert_eq!(r6s.len(), 1);
    }

    #[test]
    fn test_r7_true_false_with_options() {
        let content = json!({
            "id": "tov-01",
            "title": "Test",
            "description": "Test",
            "stages": [{
                "id": "tov-01-01",
                "title": "Stage",
                "description": "Stage",
                "activities": [{
                    "id": "tov-01-01-act",
                    "quiz": {
                        "questions": [{
                            "type": "true-false",
                            "options": ["True", "False"],
                            "correctAnswer": 0
                        }]
                    }
                }]
            }]
        });
        let findings = validate_schema(&content);
        let r7s: Vec<_> = findings.iter().filter(|f| f.rule == "R7").collect();
        assert_eq!(r7s.len(), 1);
    }
}
