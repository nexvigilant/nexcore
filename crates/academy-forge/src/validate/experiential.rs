//! Experiential learning validation rules (R24-R27).
//!
//! ## Implementation Status
//!
//! | Rule | Status | Description |
//! |------|--------|-------------|
//! | R24  | Not yet implemented | Reflection activity present per stage |
//! | R25  | Implemented | Quantitative stages need visualization metadata |
//! | R26  | Implemented | Middle+ stages need interactive activities |
//! | R27  | Not yet implemented | Hook activity opens each stage |

use crate::validate::{Severity, ValidationFinding};

/// Run experiential validation rules R24-R27.
///
/// Currently implements R25 and R26.  R24 and R27 are reserved but not yet
/// implemented — they return no findings until implemented.
pub fn validate_experiential(content: &serde_json::Value) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();

    validate_visualization_presence(content, &mut findings);
    validate_sandbox_presence(content, &mut findings);

    findings
}

/// R25: Stages with quantitative content should have visualization metadata.
fn validate_visualization_presence(
    content: &serde_json::Value,
    findings: &mut Vec<ValidationFinding>,
) {
    let quantitative_keywords = [
        "signal",
        "PRR",
        "ROR",
        "threshold",
        "conservation",
        "manifold",
        "compute",
        "calculate",
        "formula",
    ];

    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        for (si, stage) in stages.iter().enumerate() {
            let stage_text = stage.to_string();
            let is_quantitative = quantitative_keywords
                .iter()
                .any(|kw| stage_text.contains(kw));

            if is_quantitative {
                let has_viz = stage
                    .get("activities")
                    .and_then(|a| a.as_array())
                    .map(|activities| {
                        activities.iter().any(|a| {
                            a.get("observatory").is_some() || a.get("visualization").is_some()
                        })
                    })
                    .unwrap_or(false);

                if !has_viz {
                    findings.push(ValidationFinding {
                        rule: "R25".to_string(),
                        severity: Severity::Advisory,
                        message: format!(
                            "Stage {si} has quantitative content but no visualization metadata"
                        ),
                        field_path: Some(format!("stages[{si}]")),
                    });
                }
            }
        }
    }
}

/// R26: Middle+ stages should include interactive activities.
fn validate_sandbox_presence(content: &serde_json::Value, findings: &mut Vec<ValidationFinding>) {
    if let Some(stages) = content.get("stages").and_then(|s| s.as_array()) {
        for (si, stage) in stages.iter().enumerate() {
            if si < 3 {
                continue; // Skip early stages
            }

            let has_interactive = stage
                .get("activities")
                .and_then(|a| a.as_array())
                .map(|activities| {
                    activities.iter().any(|a| {
                        let activity_type = a.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        matches!(
                            activity_type,
                            "sandbox" | "simulation" | "interactive" | "practice"
                        )
                    })
                })
                .unwrap_or(false);

            if !has_interactive {
                findings.push(ValidationFinding {
                    rule: "R26".to_string(),
                    severity: Severity::Advisory,
                    message: format!("Stage {si} (middle+) has no interactive/sandbox activities"),
                    field_path: Some(format!("stages[{si}]")),
                });
            }
        }
    }
}
