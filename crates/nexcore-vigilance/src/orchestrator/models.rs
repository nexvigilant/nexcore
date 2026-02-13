//! Domain models for agent orchestration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

/// Skill metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub keywords: Vec<String>,
    pub triggers: Vec<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

/// Scored skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredSkill {
    pub skill: Skill,
    pub score: f64,
    pub match_reasons: Vec<String>,
}

/// Task analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalysis {
    pub intent: String,
    pub domain: String,
    pub complexity: Complexity,
    pub keywords_extracted: Vec<String>,
}

/// Execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Halted,
}

/// Andon signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AndonSignal {
    Green,
    Yellow,
    Red,
}

/// Skill result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub skill_name: String,
    pub status: ExecutionStatus,
    pub signal: AndonSignal,
    pub output: serde_json::Value,
    pub artifacts: Vec<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Chain operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainOperator {
    Sequential,
    Parallel,
    Fallback,
    Conditional,
    End,
}

/// Chain node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNode {
    pub skill_name: String,
    pub operator: ChainOperator,
    pub level: u32,
    pub dependencies: Vec<String>,
}

/// Execution chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub nodes: Vec<ChainNode>,
    pub analysis: Option<TaskAnalysis>,
    pub confidence: f64,
    /// Name of the preset this chain was expanded from (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_name: Option<String>,
    /// Safety Manifold for this chain
    #[serde(skip)]
    pub safety_manifold: Option<crate::SafetyManifold>,
}

/// Chain metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetrics {
    pub chain_expression: String,
    pub domain: String,
    pub success: bool,
    pub duration_seconds: f64,
    pub skills_used: Vec<String>,
    pub skill_success_rates: HashMap<String, f64>,
}

/// Research step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResearchStep {
    AskQuestion = 1,
    BackgroundResearch = 2,
    FormulateHypothesis = 3,
    DesignExperiment = 4,
    CollectData = 5,
    AnalyzeData = 6,
    DrawConclusions = 7,
    CommunicateResults = 8,
}

/// Research report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub question: String,
    pub hypothesis: String,
    pub methodology: String,
    pub analysis: String,
    pub conclusions: String,
}

/// Execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub chain: Chain,
    pub status: ExecutionStatus,
    pub skill_results: Vec<SkillResult>,
    pub total_duration_seconds: f64,
    pub halt_reason: Option<String>,
    /// Accumulated context from skill executions
    #[serde(default)]
    pub context_accumulated: Option<ExecutionContext>,
}

/// Accumulated context passed between skills during chain execution.
///
/// This allows skills to share data, artifacts, and messages as they execute.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Key-value data accumulated from skill outputs
    pub data: HashMap<String, serde_json::Value>,
    /// Artifact paths created during execution
    pub artifacts: Vec<String>,
    /// Execution messages/logs
    pub messages: Vec<String>,
}

impl ExecutionContext {
    /// Create a new empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge skill outputs into context.
    pub fn merge(&mut self, outputs: &serde_json::Value) {
        if let Some(obj) = outputs.as_object() {
            for (key, value) in obj {
                self.data.insert(key.clone(), value.clone());
            }
        }
    }

    /// Add an artifact path.
    pub fn add_artifact(&mut self, artifact: String) {
        self.artifacts.push(artifact);
    }

    /// Add an execution message.
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    /// Format context as prompt input for a skill.
    #[must_use]
    pub fn format_for_skill(&self) -> String {
        let mut parts = vec!["## Prior Context".to_string(), String::new()];

        if !self.data.is_empty() {
            parts.push("### Accumulated Data".to_string());
            for (key, value) in self.data.iter().take(10) {
                let value_str = value.to_string();
                let truncated = if value_str.len() > 200 {
                    format!("{}...", &value_str[..200])
                } else {
                    value_str
                };
                parts.push(format!("- **{key}**: {truncated}"));
            }
            parts.push(String::new());
        }

        if !self.artifacts.is_empty() {
            parts.push("### Artifacts Created".to_string());
            for artifact in self.artifacts.iter().rev().take(5) {
                parts.push(format!("- {artifact}"));
            }
            parts.push(String::new());
        }

        parts.join("\n")
    }
}
