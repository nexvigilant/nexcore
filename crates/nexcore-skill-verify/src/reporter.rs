//! Report formatting.

use crate::CheckOutcome;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ReportSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

pub struct Report {
    outcomes: Vec<CheckOutcome>,
    format: ReportFormat,
}

impl Report {
    pub fn new(outcomes: Vec<CheckOutcome>, format: ReportFormat) -> Self {
        Self { outcomes, format }
    }

    pub fn render(&self) -> String {
        match self.format {
            ReportFormat::Text => self.render_text(),
            ReportFormat::Json => self.render_json(),
        }
    }

    pub fn summary(&self) -> ReportSummary {
        ReportSummary {
            passed: self
                .outcomes
                .iter()
                .filter(|o| o.result.is_passed())
                .count(),
            failed: self
                .outcomes
                .iter()
                .filter(|o| o.result.is_failed())
                .count(),
            skipped: self
                .outcomes
                .iter()
                .filter(|o| o.result.is_skipped())
                .count(),
        }
    }

    fn render_text(&self) -> String {
        let mut out = String::new();
        for o in &self.outcomes {
            let icon = if o.result.is_passed() {
                "✓"
            } else if o.result.is_failed() {
                "✗"
            } else {
                "○"
            };
            let msg = match &o.result {
                crate::CheckResult::Passed { message } => message.clone(),
                crate::CheckResult::Failed { message, .. } => message.clone(),
                crate::CheckResult::Skipped { reason } => reason.clone(),
            };
            out.push_str(&format!("{icon} {}: {msg}\n", o.name));
        }
        let s = self.summary();
        out.push_str(&format!(
            "\n{} passed, {} failed, {} skipped\n",
            s.passed, s.failed, s.skipped
        ));
        out
    }

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "outcomes": self.outcomes.iter().map(|o| {
                serde_json::json!({
                    "name": o.name,
                    "status": if o.result.is_passed() { "passed" } else if o.result.is_failed() { "failed" } else { "skipped" },
                    "duration_ms": o.duration.as_millis(),
                })
            }).collect::<Vec<_>>(),
            "summary": self.summary(),
        })).unwrap_or_default()
    }
}
