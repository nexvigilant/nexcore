//! WorkflowExecutor: Executes skill chains from activated workflow specs.
//!
//! Uses nexcore-skill-exec for direct, high-performance skill execution
//! instead of spawning Claude CLI processes.

use crate::errors::VigilError;
use crate::models::{ExecutorResult, ExecutorType};
use crate::sources::workflow::{ChainLink, WorkflowSpec};
use nexcore_skill_exec::{CompositeExecutor, ExecutionRequest};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Executor for workflow spec skill chains.
pub struct WorkflowExecutor {
    /// Rust-based skill executor (faster than CLI)
    executor: Arc<CompositeExecutor>,
}

impl WorkflowExecutor {
    /// Create a new WorkflowExecutor with CompositeExecutor.
    pub fn new() -> Self {
        Self {
            executor: Arc::new(CompositeExecutor::new()),
        }
    }

    /// Execute a workflow spec's skill chain in DAG order.
    pub async fn execute(&self, workflow: &WorkflowSpec, context: &str) -> ExecutorResult {
        info!(
            workflow = %workflow.name,
            chain_length = workflow.chain.len(),
            "workflow_execution_starting"
        );

        let mut outputs: HashMap<String, String> = HashMap::new();
        let mut all_outputs: Vec<String> = Vec::new();

        // Execute skills in order (respecting DAG dependencies)
        let ordered_skills = self.topological_order(&workflow.chain, &workflow.dag);

        for skill_name in ordered_skills {
            let link = workflow.chain.iter().find(|c| c.skill == skill_name);
            if let Some(link) = link {
                match self.execute_skill(link, context, &outputs).await {
                    Ok(output) => {
                        info!(skill = %link.skill, role = %link.role, "skill_executed");
                        outputs.insert(link.skill.clone(), output.clone());
                        all_outputs.push(format!("[{}] {}", link.skill, output));
                    }
                    Err(e) => {
                        error!(skill = %link.skill, error = %e, "skill_execution_failed");
                        return ExecutorResult {
                            executor: ExecutorType::Mcp,
                            success: false,
                            output: None,
                            error: Some(format!("Skill {} failed: {}", link.skill, e)),
                            metadata: HashMap::new(),
                        };
                    }
                }
            }
        }

        ExecutorResult {
            executor: ExecutorType::Mcp,
            success: true,
            output: Some(all_outputs.join("\n---\n")),
            error: None,
            metadata: {
                let mut m: HashMap<String, serde_json::Value> = HashMap::new();
                m.insert(
                    "workflow".to_string(),
                    serde_json::Value::String(workflow.name.clone()),
                );
                m.insert(
                    "skills_executed".to_string(),
                    serde_json::Value::Number(workflow.chain.len().into()),
                );
                m.insert(
                    "executor".to_string(),
                    serde_json::Value::String("nexcore-skill-exec".to_string()),
                );
                m
            },
        }
    }

    /// Execute a single skill via nexcore-skill-exec (Rust-based, direct).
    async fn execute_skill(
        &self,
        link: &ChainLink,
        context: &str,
        prior_outputs: &HashMap<String, String>,
    ) -> Result<String, VigilError> {
        // Build context from prior outputs
        let mut full_context = context.to_string();
        for input in &link.inputs {
            if let Some(output) = prior_outputs.get(input) {
                full_context.push_str(&format!("\n\n[From {}]:\n{}", input, output));
            }
        }

        // Try to discover skill
        match self.executor.discover_skill(&link.skill) {
            Ok(_skill_info) => {
                // Build execution request with parameters
                let params = serde_json::json!({
                    "context": full_context,
                    "role": link.role,
                });

                let request = ExecutionRequest::new(&link.skill, params)
                    .with_timeout(Duration::from_secs(120));

                // Execute via Rust executor
                let result =
                    self.executor
                        .execute(&request)
                        .await
                        .map_err(|e| VigilError::Executor {
                            executor: ExecutorType::Mcp,
                            message: format!("Skill execution failed: {}", e),
                        })?;

                if result.is_success() {
                    Ok(result.output.to_string())
                } else {
                    Err(VigilError::Executor {
                        executor: ExecutorType::Mcp,
                        message: result.error.unwrap_or_else(|| "Unknown error".to_string()),
                    })
                }
            }
            Err(_) => {
                // Skill not found or no executor - fall back to CLI
                warn!(skill = %link.skill, "skill_not_found_falling_back_to_cli");
                self.execute_skill_cli(link, &full_context).await
            }
        }
    }

    /// Fallback: Execute skill via Claude CLI (for skills without shell scripts).
    async fn execute_skill_cli(
        &self,
        link: &ChainLink,
        context: &str,
    ) -> Result<String, VigilError> {
        use std::process::Command;

        let prompt = format!(
            "/{} {}",
            link.skill,
            context.chars().take(2000).collect::<String>()
        );

        let output = Command::new("claude")
            .args(["--print", "--dangerously-skip-permissions", "-p", &prompt])
            .output()
            .map_err(|e| VigilError::Executor {
                executor: ExecutorType::Mcp,
                message: format!("Failed to invoke claude: {}", e),
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(VigilError::Executor {
                executor: ExecutorType::Mcp,
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    /// Order skills by DAG dependencies (topological sort).
    fn topological_order(
        &self,
        chain: &[ChainLink],
        dag: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        // Simple order-based sort if DAG is empty
        if dag.is_empty() {
            let mut ordered: Vec<_> = chain.iter().collect();
            ordered.sort_by_key(|c| c.order);
            return ordered.iter().map(|c| c.skill.clone()).collect();
        }

        // Kahn's algorithm for topological sort
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for link in chain {
            in_degree.entry(link.skill.clone()).or_insert(0);
            adj.entry(link.skill.clone()).or_default();
        }

        for (skill, deps) in dag {
            for dep in deps {
                adj.entry(dep.clone()).or_default().push(skill.clone());
                *in_degree.entry(skill.clone()).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|&(_, d)| *d == 0)
            .map(|(k, _)| k.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(skill) = queue.pop() {
            result.push(skill.clone());
            if let Some(neighbors) = adj.get(&skill) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        result
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}
