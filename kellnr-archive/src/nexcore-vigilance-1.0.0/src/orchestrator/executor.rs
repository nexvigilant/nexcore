//! Async execution engine for skill chains.
//!
//! Provides context passing between skills, smart signal determination,
//! and parallel/sequential execution with Andon signal monitoring.

use super::models::{
    AndonSignal, Chain, ChainOperator, ExecutionContext, ExecutionResult, ExecutionStatus,
    SkillResult,
};
use futures::future::join_all;
use serde_json::json;
use std::time::{Duration, Instant};

/// Async executor engine with context passing support.
pub struct ExecutorEngine {
    /// Whether to halt on RED signal
    pub halt_on_red: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for ExecutorEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutorEngine {
    /// Create a new executor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            halt_on_red: true,
            max_retries: 2,
        }
    }

    /// Execute a chain asynchronously with context passing.
    pub async fn execute(&self, chain: &Chain) -> ExecutionResult {
        let start_time = Instant::now();
        let mut skill_results = Vec::new();
        let mut status = ExecutionStatus::Completed;
        let mut halt_reason = None;
        let mut context = ExecutionContext::new();

        let max_level = chain.nodes.iter().map(|n| n.level).max().unwrap_or(0);

        for level in 0..=max_level {
            let level_nodes: Vec<_> = chain.nodes.iter().filter(|n| n.level == level).collect();
            if level_nodes.is_empty() {
                continue;
            }

            let parallel_nodes: Vec<_> = level_nodes
                .iter()
                .filter(|n| n.operator == ChainOperator::Parallel)
                .copied()
                .collect();
            let sequential_nodes: Vec<_> = level_nodes
                .iter()
                .filter(|n| n.operator != ChainOperator::Parallel)
                .copied()
                .collect();

            // Execute parallel group
            if !parallel_nodes.is_empty() {
                let p_results = self.execute_parallel(&parallel_nodes, &context).await;

                // Merge parallel results into context
                for result in &p_results {
                    context.merge(&result.output);
                    for artifact in &result.artifacts {
                        context.add_artifact(artifact.clone());
                    }
                }

                skill_results.extend(p_results.clone());

                if self.halt_on_red && p_results.iter().any(|r| r.signal == AndonSignal::Red) {
                    status = ExecutionStatus::Halted;
                    halt_reason = Some("Parallel execution encountered RED signal".to_string());
                    break;
                }
            }

            // Execute sequential nodes
            for node in sequential_nodes {
                let result = self.execute_skill(node, &context).await;

                // Update context with outputs
                context.merge(&result.output);
                for artifact in &result.artifacts {
                    context.add_artifact(artifact.clone());
                }

                skill_results.push(result.clone());

                if result.signal == AndonSignal::Red {
                    if node.operator == ChainOperator::Fallback {
                        // Skip to next level (fallback path)
                        continue;
                    }
                    if self.halt_on_red {
                        status = ExecutionStatus::Halted;
                        halt_reason =
                            Some(format!("Skill {} returned RED signal", result.skill_name));
                        break;
                    }
                }
            }

            if status == ExecutionStatus::Halted {
                break;
            }
        }

        ExecutionResult {
            chain: chain.clone(),
            status,
            skill_results,
            total_duration_seconds: start_time.elapsed().as_secs_f64(),
            halt_reason,
            context_accumulated: Some(context),
        }
    }

    async fn execute_parallel(
        &self,
        nodes: &[&super::models::ChainNode],
        context: &ExecutionContext,
    ) -> Vec<SkillResult> {
        // Use join_all for parallel execution
        let futures: Vec<_> = nodes
            .iter()
            .map(|&n| self.execute_skill(n, context))
            .collect();
        join_all(futures).await
    }

    async fn execute_skill(
        &self,
        node: &super::models::ChainNode,
        _context: &ExecutionContext,
    ) -> SkillResult {
        let start = Instant::now();

        // Simulate skill execution
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Simulated result
        let output = json!({
            "status": "simulated_success",
            "skill": node.skill_name,
        });

        // Determine signal from result
        let signal = self.determine_signal(&output);

        SkillResult {
            skill_name: node.skill_name.clone(),
            status: ExecutionStatus::Completed,
            signal,
            output,
            artifacts: Vec::new(),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Determine Andon signal from skill result.
    fn determine_signal(&self, result: &serde_json::Value) -> AndonSignal {
        // Check for error indicators
        if result.get("error").is_some() {
            return AndonSignal::Red;
        }
        if let Some(status) = result.get("status").and_then(|s| s.as_str()) {
            if status == "failed" || status == "error" {
                return AndonSignal::Red;
            }
            if status == "warning" {
                return AndonSignal::Yellow;
            }
        }

        // Check for warning indicators
        if result.get("warning").is_some() || result.get("warnings").is_some() {
            return AndonSignal::Yellow;
        }

        AndonSignal::Green
    }

    /// Format results as an Andon board visualization.
    #[must_use]
    pub fn format_andon_board(&self, results: &[SkillResult]) -> String {
        let mut lines = vec![
            String::new(),
            "### Andon Board".to_string(),
            "| Module | Signal | Duration | Notes |".to_string(),
            "|--------|--------|----------|-------|".to_string(),
        ];

        for (i, result) in results.iter().enumerate() {
            let duration = format!("{:.1}ms", result.duration_ms);
            let signal_str = match result.signal {
                AndonSignal::Green => "🟢 GREEN",
                AndonSignal::Yellow => "🟡 YELLOW",
                AndonSignal::Red => "🔴 RED",
            };
            let notes = result.error.as_deref().unwrap_or("Success");
            let notes_truncated = if notes.len() > 30 {
                format!("{}...", &notes[..27])
            } else {
                notes.to_string()
            };

            lines.push(format!(
                "| M{}: {} | {} | {} | {} |",
                i + 1,
                result.skill_name,
                signal_str,
                duration,
                notes_truncated
            ));
        }

        lines.join("\n")
    }
}
