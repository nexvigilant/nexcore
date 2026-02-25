//! Engine configuration types.
//!
//! Types for engine configuration, workflow definitions, and registry structure.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Default timeout in milliseconds.
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Default retry count.
pub const DEFAULT_RETRIES: u32 = 3;

/// Engine endpoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEndpoints {
    /// Trigger endpoint path.
    #[serde(default = "default_trigger")]
    pub trigger: String,
    /// Status endpoint path.
    #[serde(default = "default_status")]
    pub status: String,
    /// Health check endpoint path.
    #[serde(default = "default_health")]
    pub health: String,
    /// Additional custom endpoints.
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

fn default_trigger() -> String {
    "/".to_string()
}

fn default_status() -> String {
    "/status".to_string()
}

fn default_health() -> String {
    "/health".to_string()
}

impl Default for EngineEndpoints {
    fn default() -> Self {
        Self {
            trigger: default_trigger(),
            status: default_status(),
            health: default_health(),
            custom: HashMap::new(),
        }
    }
}

/// Configuration for a single engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfig {
    /// Engine URL.
    pub url: String,
    /// Audience for authentication (usually same as URL).
    pub audience: String,
    /// Timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Number of retries.
    #[serde(default = "default_retries")]
    pub retries: u32,
    /// Human-readable description.
    pub description: String,
    /// Endpoint paths.
    #[serde(default)]
    pub endpoints: EngineEndpoints,
    /// Engine capabilities.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT_MS
}

fn default_retries() -> u32 {
    DEFAULT_RETRIES
}

impl EngineConfig {
    /// Create a new engine configuration.
    #[must_use]
    pub fn new(url: impl Into<String>, description: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            audience: url.clone(),
            url,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            retries: DEFAULT_RETRIES,
            description: description.into(),
            endpoints: EngineEndpoints::default(),
            capabilities: Vec::new(),
        }
    }

    /// Set the timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set the retry count.
    #[must_use]
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Add a capability.
    #[must_use]
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Check if the engine has a specific capability.
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Get the endpoint URL for an action.
    #[must_use]
    pub fn endpoint_url(&self, action: &str) -> String {
        let path = self
            .endpoints
            .custom
            .get(action)
            .map(String::as_str)
            .unwrap_or_else(|| match action {
                "trigger" | "" => &self.endpoints.trigger,
                "status" => &self.endpoints.status,
                "health" => &self.endpoints.health,
                _ => &self.endpoints.trigger,
            });

        format!("{}{}", self.url, path)
    }
}

/// A single step in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    /// Name of the engine to execute.
    pub engine: String,
    /// Action to perform.
    pub action: String,
    /// Description of this step.
    pub description: String,
    /// Step-specific timeout (overrides engine timeout).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Step-specific retries (overrides engine retries).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    /// Whether this step can run in parallel with others.
    #[serde(default)]
    pub parallel: bool,
    /// Whether this step is optional (won't fail workflow if it fails).
    #[serde(default)]
    pub optional: bool,
}

impl WorkflowStep {
    /// Create a new workflow step.
    #[must_use]
    pub fn new(
        engine: impl Into<String>,
        action: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            engine: engine.into(),
            action: action.into(),
            description: description.into(),
            timeout: None,
            retries: None,
            parallel: false,
            optional: false,
        }
    }

    /// Make this step parallel.
    #[must_use]
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Make this step optional.
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set step-specific timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }
}

/// Workflow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowConfig {
    /// Human-readable description.
    pub description: String,
    /// Steps in the workflow.
    pub steps: Vec<WorkflowStep>,
    /// Cron schedule expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    /// Timezone for schedule.
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Whether the workflow is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Maximum concurrent executions.
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: u32,
    /// Workflow-level timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

fn default_enabled() -> bool {
    true
}

fn default_max_concurrency() -> u32 {
    1
}

impl WorkflowConfig {
    /// Create a new workflow configuration.
    #[must_use]
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            steps: Vec::new(),
            schedule: None,
            timezone: default_timezone(),
            enabled: true,
            max_concurrency: 1,
            timeout: None,
        }
    }

    /// Add a step to the workflow.
    #[must_use]
    pub fn with_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Set the cron schedule.
    #[must_use]
    pub fn with_schedule(mut self, schedule: impl Into<String>) -> Self {
        self.schedule = Some(schedule.into());
        self
    }

    /// Set enabled status.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Complete registry containing engines and workflows.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    /// Registered engines.
    pub engines: HashMap<String, EngineConfig>,
    /// Registered workflows.
    pub workflows: HashMap<String, WorkflowConfig>,
}

impl Registry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an engine.
    pub fn register_engine(&mut self, name: impl Into<String>, config: EngineConfig) {
        self.engines.insert(name.into(), config);
    }

    /// Register a workflow.
    pub fn register_workflow(&mut self, name: impl Into<String>, config: WorkflowConfig) {
        self.workflows.insert(name.into(), config);
    }

    /// Get an engine by name.
    #[must_use]
    pub fn get_engine(&self, name: &str) -> Option<&EngineConfig> {
        self.engines.get(name)
    }

    /// Get a workflow by name.
    #[must_use]
    pub fn get_workflow(&self, name: &str) -> Option<&WorkflowConfig> {
        self.workflows.get(name)
    }

    /// Get all engine names.
    #[must_use]
    pub fn engine_names(&self) -> Vec<&str> {
        self.engines.keys().map(String::as_str).collect()
    }

    /// Get all workflow names.
    #[must_use]
    pub fn workflow_names(&self) -> Vec<&str> {
        self.workflows.keys().map(String::as_str).collect()
    }

    /// Get only enabled workflows.
    #[must_use]
    pub fn enabled_workflows(&self) -> HashMap<&str, &WorkflowConfig> {
        self.workflows
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, config)| (name.as_str(), config))
            .collect()
    }

    /// Find engines by capability.
    #[must_use]
    pub fn engines_with_capability(&self, capability: &str) -> Vec<&str> {
        self.engines
            .iter()
            .filter(|(_, config)| config.has_capability(capability))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Check if an engine exists.
    #[must_use]
    pub fn has_engine(&self, name: &str) -> bool {
        self.engines.contains_key(name)
    }

    /// Get registry statistics.
    #[must_use]
    pub fn stats(&self) -> RegistryStats {
        let enabled_workflows = self.workflows.values().filter(|w| w.enabled).count();
        let total_steps: usize = self.workflows.values().map(|w| w.steps.len()).sum();

        RegistryStats {
            engine_count: self.engines.len(),
            workflow_count: self.workflows.len(),
            enabled_workflow_count: enabled_workflows,
            total_step_count: total_steps,
        }
    }
}

/// Validate a workflow against available engines.
///
/// Returns a list of validation issues, empty if valid.
#[must_use]
pub fn validate_workflow(workflow: &WorkflowConfig, engine_set: &HashSet<&str>) -> Vec<String> {
    workflow
        .steps
        .iter()
        .enumerate()
        .filter_map(|(i, step)| {
            if engine_set.contains(step.engine.as_str()) {
                None
            } else {
                Some(format!(
                    "Step {} references unknown engine '{}'",
                    i, step.engine
                ))
            }
        })
        .collect()
}

/// Registry statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Number of registered engines.
    pub engine_count: usize,
    /// Number of registered workflows.
    pub workflow_count: usize,
    /// Number of enabled workflows.
    pub enabled_workflow_count: usize,
    /// Total number of steps across all workflows.
    pub total_step_count: usize,
}

/// Request payload sent to engines.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineRequest {
    /// Unique request identifier.
    pub request_id: String,
    /// Step/action to perform.
    pub step: String,
    /// Payload data.
    pub payload: serde_json::Value,
    /// ISO timestamp.
    pub timestamp: String,
    /// Correlation ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EngineRequest {
    /// Create a new engine request.
    #[must_use]
    pub fn new(request_id: String, step: String, payload: serde_json::Value) -> Self {
        Self {
            request_id,
            step,
            payload,
            timestamp: nexcore_chrono::DateTime::now().to_rfc3339(),
            correlation_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the correlation ID.
    #[must_use]
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

/// Options for engine calls.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCallOptions {
    /// Timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Number of retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    /// Custom headers.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Idempotency key for deduplication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    /// Correlation ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

/// Result of an engine call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCallResult {
    /// Whether the call succeeded.
    pub success: bool,
    /// Response from the engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<super::super::workflow::EngineResponse>,
    /// Error message if the call failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Duration of the call in milliseconds.
    pub duration_ms: u64,
    /// Number of retry attempts.
    pub retry_count: u32,
    /// Whether the circuit breaker was open.
    pub circuit_breaker_open: bool,
}

impl EngineCallResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(response: super::super::workflow::EngineResponse, duration_ms: u64) -> Self {
        Self {
            success: true,
            response: Some(response),
            error: None,
            duration_ms,
            retry_count: 0,
            circuit_breaker_open: false,
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failure(error: String, duration_ms: u64) -> Self {
        Self {
            success: false,
            response: None,
            error: Some(error),
            duration_ms,
            retry_count: 0,
            circuit_breaker_open: false,
        }
    }

    /// Create a circuit breaker open result.
    #[must_use]
    pub fn circuit_breaker_open(duration_ms: u64) -> Self {
        Self {
            success: false,
            response: None,
            error: Some("Circuit breaker is open".to_string()),
            duration_ms,
            retry_count: 0,
            circuit_breaker_open: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::workflow::{EngineResponse, EngineStatus};
    use super::*;

    #[test]
    fn test_engine_config() {
        let config = EngineConfig::new("https://api.example.com", "Test engine")
            .with_timeout(60_000)
            .with_retries(5)
            .with_capability("content-creation");

        assert_eq!(config.url, "https://api.example.com");
        assert_eq!(config.timeout_ms, 60_000);
        assert_eq!(config.retries, 5);
        assert!(config.has_capability("content-creation"));
        assert!(!config.has_capability("unknown"));
    }

    #[test]
    fn test_engine_endpoint_url() {
        let config = EngineConfig::new("https://api.example.com", "Test engine");

        assert_eq!(config.endpoint_url("trigger"), "https://api.example.com/");
        assert_eq!(
            config.endpoint_url("health"),
            "https://api.example.com/health"
        );
        assert_eq!(
            config.endpoint_url("status"),
            "https://api.example.com/status"
        );
    }

    #[test]
    fn test_workflow_step() {
        let step = WorkflowStep::new("content-engine", "create", "Create content")
            .parallel()
            .optional()
            .with_timeout(120_000);

        assert_eq!(step.engine, "content-engine");
        assert!(step.parallel);
        assert!(step.optional);
        assert_eq!(step.timeout, Some(120_000));
    }

    #[test]
    fn test_workflow_config() {
        let workflow = WorkflowConfig::new("Daily content workflow")
            .with_step(WorkflowStep::new("engine-1", "create", "Create"))
            .with_step(WorkflowStep::new("engine-2", "distribute", "Distribute"))
            .with_schedule("0 8 * * 1-5");

        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.schedule, Some("0 8 * * 1-5".to_string()));
    }

    #[test]
    fn test_registry() {
        let mut registry = Registry::new();

        registry.register_engine(
            "content-engine",
            EngineConfig::new("https://content.example.com", "Content creation"),
        );
        registry.register_engine(
            "distribution-engine",
            EngineConfig::new("https://dist.example.com", "Distribution"),
        );

        registry.register_workflow(
            "daily-content",
            WorkflowConfig::new("Daily content")
                .with_step(WorkflowStep::new("content-engine", "create", "Create"))
                .with_step(WorkflowStep::new(
                    "distribution-engine",
                    "distribute",
                    "Dist",
                )),
        );

        assert_eq!(registry.engine_names().len(), 2);
        assert_eq!(registry.workflow_names().len(), 1);
        assert!(registry.has_engine("content-engine"));
        assert!(!registry.has_engine("unknown"));
    }

    #[test]
    fn test_validate_workflow() {
        let workflow = WorkflowConfig::new("Test workflow")
            .with_step(WorkflowStep::new("engine-1", "action", "Step 1"))
            .with_step(WorkflowStep::new("unknown-engine", "action", "Step 2"));

        let engines: HashSet<&str> = ["engine-1", "engine-2"].into_iter().collect();
        let issues = validate_workflow(&workflow, &engines);

        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("unknown-engine"));
    }

    #[test]
    fn test_engine_call_result() {
        let success = EngineCallResult::success(
            EngineResponse {
                status: EngineStatus::Ok,
                output: Some(serde_json::json!({"result": "ok"})),
                error: None,
                next_cursor: None,
                metadata: std::collections::HashMap::new(),
                execution_time_ms: Some(100),
            },
            150,
        );

        assert!(success.success);
        assert!(!success.circuit_breaker_open);

        let cb_open = EngineCallResult::circuit_breaker_open(0);
        assert!(!cb_open.success);
        assert!(cb_open.circuit_breaker_open);
    }
}
