#![allow(clippy::unwrap_used)] // Config deserialization — will add proper error handling

use serde::{Deserialize, Serialize};

/// A WebMCP tool annotation describing behavioral hints for agents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    /// Whether this tool only reads data (no side effects).
    pub read_only_hint: String,
    /// Whether calling this tool multiple times has the same effect.
    pub idempotent_hint: String,
    /// Whether this tool can destroy or modify data irreversibly.
    pub destructive_hint: String,
}

impl Default for ToolAnnotations {
    fn default() -> Self {
        Self {
            read_only_hint: "true".into(),
            idempotent_hint: "true".into(),
            destructive_hint: "false".into(),
        }
    }
}

/// A single execution step within a tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionStep {
    /// The action to perform (e.g., "navigate", "click", "fill").
    pub action: String,
    /// Target URL for navigate actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// CSS selector for interaction actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Value to fill for fill actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Execution metadata for a tool — how Playwright runs it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolExecution {
    /// Ordered steps the browser automation engine executes.
    pub steps: Vec<ExecutionStep>,
    /// Root selector for result extraction.
    #[serde(default = "default_selector")]
    pub selector: String,
    /// Whether to auto-submit forms.
    #[serde(default)]
    pub autosubmit: bool,
    /// How to extract results: "list", "text", "table", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_extract: Option<String>,
}

fn default_selector() -> String {
    "body".into()
}

/// A single tool within a WebMCP config.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebMcpTool {
    /// Kebab-case tool name (e.g., "browse-safety-alerts").
    pub name: String,
    /// Human-readable description of what this tool does.
    pub description: String,
    /// JSON Schema for tool input parameters.
    pub input_schema: serde_json::Value,
    /// Behavioral annotations for agents.
    #[serde(default)]
    pub annotations: ToolAnnotations,
    /// Execution metadata for browser automation.
    pub execution: ToolExecution,
}

/// A complete WebMCP config — one URL pattern on one domain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebMcpConfig {
    /// Unique config ID (UUID from hub, or locally generated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Target domain (e.g., "fda.gov").
    pub domain: String,
    /// URL pattern within the domain (e.g., "fda.gov/safety/medwatch").
    pub url_pattern: String,
    /// Human-readable title.
    pub title: String,
    /// Description including disclaimer.
    pub description: String,
    /// Discovery tags (max 10).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Tools available for this config.
    pub tools: Vec<WebMcpTool>,
}

impl WebMcpConfig {
    /// Validate this config against microgram quality standards.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.domain.is_empty() {
            issues.push("domain is empty".into());
        }
        if self.url_pattern.is_empty() {
            issues.push("url_pattern is empty".into());
        }
        if self.title.is_empty() {
            issues.push("title is empty".into());
        }
        if self.tags.len() > 10 {
            issues.push(format!("tags count {} exceeds max 10", self.tags.len()));
        }
        if self.tools.is_empty() {
            issues.push("config has no tools".into());
        }
        if !self.description.contains("DISCLAIMER") {
            issues.push("description missing DISCLAIMER".into());
        }

        for tool in &self.tools {
            let prefix = format!("tool '{}'", tool.name);

            if tool.name != tool.name.to_lowercase() || tool.name.contains(' ') {
                issues.push(format!("{prefix}: name not kebab-case"));
            }
            if tool.description.len() < 20 {
                issues.push(format!(
                    "{prefix}: description too short ({})",
                    tool.description.len()
                ));
            }
            if tool.execution.steps.is_empty() {
                issues.push(format!("{prefix}: no execution steps"));
            }
            if tool.annotations.read_only_hint == "true" && tool.execution.result_extract.is_none()
            {
                issues.push(format!("{prefix}: readOnly but no resultExtract"));
            }
            for (i, step) in tool.execution.steps.iter().enumerate() {
                if step.action.is_empty() {
                    issues.push(format!("{prefix}: step {i} missing action"));
                }
                if step.action == "navigate" && step.url.is_none() {
                    issues.push(format!("{prefix}: navigate step {i} missing url"));
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> WebMcpConfig {
        WebMcpConfig {
            id: None,
            domain: "fda.gov".into(),
            url_pattern: "fda.gov/safety/medwatch".into(),
            title: "FDA MedWatch — Safety Reporting by NexVigilant".into(),
            description: "Navigate FDA MedWatch. DISCLAIMER: This WebMCP configuration was developed by NexVigilant, LLC.".into(),
            tags: vec!["fda".into(), "medwatch".into(), "pharmacovigilance".into()],
            tools: vec![
                WebMcpTool {
                    name: "browse-safety-alerts".into(),
                    description: "Browse current FDA safety alerts, drug recalls, and MedWatch notifications".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {}}),
                    annotations: ToolAnnotations {
                        read_only_hint: "true".into(),
                        idempotent_hint: "true".into(),
                        destructive_hint: "false".into(),
                    },
                    execution: ToolExecution {
                        steps: vec![ExecutionStep {
                            action: "navigate".into(),
                            url: Some("https://www.fda.gov/safety/medwatch".into()),
                            selector: None,
                            value: None,
                        }],
                        selector: "body".into(),
                        autosubmit: false,
                        result_extract: Some("list".into()),
                    },
                },
            ],
        }
    }

    #[test]
    fn test_config_validates_clean() {
        let config = sample_config();
        let issues = config.validate();
        assert!(issues.is_empty(), "Expected no issues, got: {issues:?}");
    }

    #[test]
    fn test_config_catches_missing_disclaimer() {
        let mut config = sample_config();
        config.description = "No disclaimer here".into();
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("DISCLAIMER")));
    }

    #[test]
    fn test_config_catches_too_many_tags() {
        let mut config = sample_config();
        config.tags = (0..11).map(|i| format!("tag-{i}")).collect();
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("tags count")));
    }

    #[test]
    fn test_config_catches_empty_tools() {
        let mut config = sample_config();
        config.tools.clear();
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("no tools")));
    }

    #[test]
    fn test_config_catches_readonly_no_extract() {
        let mut config = sample_config();
        config.tools[0].execution.result_extract = None;
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("resultExtract")));
    }

    #[test]
    fn test_config_catches_navigate_no_url() {
        let mut config = sample_config();
        config.tools[0].execution.steps[0].url = None;
        let issues = config.validate();
        assert!(issues.iter().any(|i| i.contains("missing url")));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = sample_config();
        let json = serde_json::to_string(&config).expect("serialize");
        let back: WebMcpConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config, back);
    }

    #[test]
    fn test_tool_annotations_default() {
        let ann = ToolAnnotations::default();
        assert_eq!(ann.read_only_hint, "true");
        assert_eq!(ann.idempotent_hint, "true");
        assert_eq!(ann.destructive_hint, "false");
    }
}
