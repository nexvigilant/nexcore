//! Core workflow types.
//!
//! Types representing workflow execution state, results, and events.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow execution options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExecutionOptions {
    /// Unique request identifier.
    pub request_id: String,
    /// Correlation ID for tracing across services.
    pub correlation_id: String,
    /// Timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Number of retries for failed steps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    /// Whether to execute steps in parallel where possible.
    #[serde(default)]
    pub parallel: bool,
    /// Whether to continue execution on step errors.
    #[serde(default)]
    pub continue_on_error: bool,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for WorkflowExecutionOptions {
    fn default() -> Self {
        Self {
            request_id: NexId::v4().to_string(),
            correlation_id: NexId::v4().to_string(),
            timeout: None,
            retries: None,
            parallel: false,
            continue_on_error: false,
            metadata: HashMap::new(),
        }
    }
}

/// Result of a single workflow step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStepResult {
    /// Index of the step in the workflow.
    pub step_index: usize,
    /// Name of the engine that executed this step.
    pub engine: String,
    /// Action performed by the engine.
    pub action: String,
    /// Whether the step succeeded.
    pub success: bool,
    /// Response from the engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<EngineResponse>,
    /// Error message if the step failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Duration of the step in milliseconds.
    pub duration_ms: u64,
    /// Number of retry attempts.
    pub retry_count: u32,
    /// When the step started.
    pub start_time: DateTime,
    /// When the step ended.
    pub end_time: DateTime,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response from an engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineResponse {
    /// Status of the response.
    pub status: EngineStatus,
    /// Output data from the engine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// Error message if status is error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>,
}

/// Engine response status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineStatus {
    /// Operation completed successfully.
    #[default]
    Ok,
    /// Operation failed with an error.
    Error,
}

/// Result of a complete workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExecutionResult {
    /// Unique execution identifier.
    pub execution_id: String,
    /// Name of the workflow that was executed.
    pub workflow_name: String,
    /// Whether the workflow completed successfully.
    pub success: bool,
    /// Results from each step.
    pub results: Vec<WorkflowStepResult>,
    /// Total duration in milliseconds.
    pub total_duration_ms: u64,
    /// When the workflow started.
    pub start_time: DateTime,
    /// When the workflow ended.
    pub end_time: DateTime,
    /// Request ID from options.
    pub request_id: String,
    /// Correlation ID from options.
    pub correlation_id: String,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Error message if the workflow failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Execution context for a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExecutionContext {
    /// Unique execution identifier.
    pub execution_id: String,
    /// Name of the workflow.
    pub workflow_name: String,
    /// Request ID.
    pub request_id: String,
    /// Correlation ID.
    pub correlation_id: String,
    /// When execution started.
    pub start_time: DateTime,
    /// Execution options.
    pub options: WorkflowExecutionOptions,
    /// Results from completed steps.
    pub step_results: Vec<WorkflowStepResult>,
    /// Index of the current step being executed.
    pub current_step_index: usize,
    /// Input payload for the workflow.
    pub payload: serde_json::Value,
    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Execution context for a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepExecutionContext {
    /// Index of the step.
    pub step_index: usize,
    /// Name of the engine.
    pub engine: String,
    /// Action to perform.
    pub action: String,
    /// Input payload for the step.
    pub payload: serde_json::Value,
    /// Results from previous steps.
    pub previous_results: Vec<WorkflowStepResult>,
    /// Execution ID from workflow context.
    pub execution_id: String,
    /// Request ID from workflow context.
    pub request_id: String,
    /// Correlation ID from workflow context.
    pub correlation_id: String,
    /// Step-specific timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Step-specific retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Workflow execution events for observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowExecutionEvent {
    /// Workflow execution started.
    WorkflowStarted {
        /// Execution ID.
        execution_id: String,
        /// Workflow name.
        workflow_name: String,
        /// When the event occurred.
        timestamp: DateTime,
    },
    /// Workflow execution completed.
    WorkflowCompleted {
        /// Execution ID.
        execution_id: String,
        /// Workflow name.
        workflow_name: String,
        /// Whether the workflow succeeded.
        success: bool,
        /// Total duration in milliseconds.
        duration_ms: u64,
        /// When the event occurred.
        timestamp: DateTime,
    },
    /// Step execution started.
    StepStarted {
        /// Execution ID.
        execution_id: String,
        /// Step index.
        step_index: usize,
        /// Engine name.
        engine: String,
        /// Action name.
        action: String,
        /// When the event occurred.
        timestamp: DateTime,
    },
    /// Step execution completed.
    StepCompleted {
        /// Execution ID.
        execution_id: String,
        /// Step index.
        step_index: usize,
        /// Engine name.
        engine: String,
        /// Action name.
        action: String,
        /// Whether the step succeeded.
        success: bool,
        /// Duration in milliseconds.
        duration_ms: u64,
        /// When the event occurred.
        timestamp: DateTime,
    },
    /// Step was retried.
    StepRetried {
        /// Execution ID.
        execution_id: String,
        /// Step index.
        step_index: usize,
        /// Engine name.
        engine: String,
        /// Retry count.
        retry_count: u32,
        /// Error that caused the retry.
        error: String,
        /// When the event occurred.
        timestamp: DateTime,
    },
    /// Workflow execution failed.
    WorkflowFailed {
        /// Execution ID.
        execution_id: String,
        /// Workflow name.
        workflow_name: String,
        /// Error message.
        error: String,
        /// Step index where the failure occurred.
        #[serde(skip_serializing_if = "Option::is_none")]
        step_index: Option<usize>,
        /// When the event occurred.
        timestamp: DateTime,
    },
}

impl WorkflowExecutionEvent {
    /// Create a workflow started event.
    #[must_use]
    pub fn workflow_started(execution_id: String, workflow_name: String) -> Self {
        Self::WorkflowStarted {
            execution_id,
            workflow_name,
            timestamp: DateTime::now(),
        }
    }

    /// Create a workflow completed event.
    #[must_use]
    pub fn workflow_completed(
        execution_id: String,
        workflow_name: String,
        success: bool,
        duration_ms: u64,
    ) -> Self {
        Self::WorkflowCompleted {
            execution_id,
            workflow_name,
            success,
            duration_ms,
            timestamp: DateTime::now(),
        }
    }

    /// Create a step started event.
    #[must_use]
    pub fn step_started(
        execution_id: String,
        step_index: usize,
        engine: String,
        action: String,
    ) -> Self {
        Self::StepStarted {
            execution_id,
            step_index,
            engine,
            action,
            timestamp: DateTime::now(),
        }
    }

    /// Create a step completed event.
    #[must_use]
    pub fn step_completed(
        execution_id: String,
        step_index: usize,
        engine: String,
        action: String,
        success: bool,
        duration_ms: u64,
    ) -> Self {
        Self::StepCompleted {
            execution_id,
            step_index,
            engine,
            action,
            success,
            duration_ms,
            timestamp: DateTime::now(),
        }
    }

    /// Create a workflow failed event.
    #[must_use]
    pub fn workflow_failed(
        execution_id: String,
        workflow_name: String,
        error: String,
        step_index: Option<usize>,
    ) -> Self {
        Self::WorkflowFailed {
            execution_id,
            workflow_name,
            error,
            step_index,
            timestamp: DateTime::now(),
        }
    }

    /// Get the event type name.
    #[must_use]
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::WorkflowStarted { .. } => "workflow_started",
            Self::WorkflowCompleted { .. } => "workflow_completed",
            Self::StepStarted { .. } => "step_started",
            Self::StepCompleted { .. } => "step_completed",
            Self::StepRetried { .. } => "step_retried",
            Self::WorkflowFailed { .. } => "workflow_failed",
        }
    }

    /// Get the execution ID.
    #[must_use]
    pub fn execution_id(&self) -> &str {
        match self {
            Self::WorkflowStarted { execution_id, .. }
            | Self::WorkflowCompleted { execution_id, .. }
            | Self::StepStarted { execution_id, .. }
            | Self::StepCompleted { execution_id, .. }
            | Self::StepRetried { execution_id, .. }
            | Self::WorkflowFailed { execution_id, .. } => execution_id,
        }
    }

    /// Get the timestamp.
    #[must_use]
    pub fn timestamp(&self) -> DateTime {
        match self {
            Self::WorkflowStarted { timestamp, .. }
            | Self::WorkflowCompleted { timestamp, .. }
            | Self::StepStarted { timestamp, .. }
            | Self::StepCompleted { timestamp, .. }
            | Self::StepRetried { timestamp, .. }
            | Self::WorkflowFailed { timestamp, .. } => *timestamp,
        }
    }
}

/// Execution status for workflows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    /// Workflow is currently running.
    #[default]
    Running,
    /// Workflow completed successfully.
    Completed,
    /// Workflow failed.
    Failed,
    /// Workflow was cancelled.
    Cancelled,
}

/// Persisted workflow state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    /// Unique execution identifier.
    pub execution_id: String,
    /// Name of the workflow.
    pub workflow_name: String,
    /// Current status.
    pub status: WorkflowStatus,
    /// Current step index.
    pub current_step_index: usize,
    /// Total number of steps.
    pub total_steps: usize,
    /// When execution started.
    pub start_time: DateTime,
    /// When execution ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime>,
    /// Request ID.
    pub request_id: String,
    /// Correlation ID.
    pub correlation_id: String,
    /// Input payload.
    pub payload: serde_json::Value,
    /// Step results so far.
    pub step_results: Vec<WorkflowStepResult>,
    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the state was created.
    pub created_at: DateTime,
    /// When the state was last updated.
    pub updated_at: DateTime,
}

impl WorkflowState {
    /// Create a new workflow state.
    #[must_use]
    pub fn new(
        execution_id: String,
        workflow_name: String,
        total_steps: usize,
        request_id: String,
        correlation_id: String,
        payload: serde_json::Value,
    ) -> Self {
        let now = DateTime::now();
        Self {
            execution_id,
            workflow_name,
            status: WorkflowStatus::Running,
            current_step_index: 0,
            total_steps,
            start_time: now,
            end_time: None,
            request_id,
            correlation_id,
            payload,
            step_results: Vec::new(),
            error: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark the workflow as completed.
    pub fn complete(&mut self) {
        self.status = WorkflowStatus::Completed;
        self.end_time = Some(DateTime::now());
        self.updated_at = DateTime::now();
    }

    /// Mark the workflow as failed.
    pub fn fail(&mut self, error: String) {
        self.status = WorkflowStatus::Failed;
        self.error = Some(error);
        self.end_time = Some(DateTime::now());
        self.updated_at = DateTime::now();
    }

    /// Mark the workflow as cancelled.
    pub fn cancel(&mut self) {
        self.status = WorkflowStatus::Cancelled;
        self.end_time = Some(DateTime::now());
        self.updated_at = DateTime::now();
    }

    /// Add a step result.
    pub fn add_step_result(&mut self, result: WorkflowStepResult) {
        self.step_results.push(result);
        self.current_step_index = self.step_results.len();
        self.updated_at = DateTime::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_execution_options_default() {
        let options = WorkflowExecutionOptions::default();
        assert!(!options.request_id.is_empty());
        assert!(!options.correlation_id.is_empty());
        assert!(!options.parallel);
        assert!(!options.continue_on_error);
    }

    #[test]
    fn test_workflow_state_lifecycle() {
        let mut state = WorkflowState::new(
            "exec-123".to_string(),
            "test-workflow".to_string(),
            3,
            "req-456".to_string(),
            "cor-789".to_string(),
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(state.status, WorkflowStatus::Running);
        assert!(state.end_time.is_none());

        state.complete();
        assert_eq!(state.status, WorkflowStatus::Completed);
        assert!(state.end_time.is_some());
    }

    #[test]
    fn test_workflow_state_failure() {
        let mut state = WorkflowState::new(
            "exec-123".to_string(),
            "test-workflow".to_string(),
            3,
            "req-456".to_string(),
            "cor-789".to_string(),
            serde_json::json!({}),
        );

        state.fail("Engine timeout".to_string());
        assert_eq!(state.status, WorkflowStatus::Failed);
        assert_eq!(state.error, Some("Engine timeout".to_string()));
    }

    #[test]
    fn test_workflow_execution_event_serialization() {
        let event = WorkflowExecutionEvent::workflow_started(
            "exec-123".to_string(),
            "daily-flow".to_string(),
        );

        let json_result = serde_json::to_string(&event);
        assert!(json_result.is_ok());

        // INVARIANT: json_result.is_ok() was verified above
        if let Ok(json) = json_result {
            assert!(json.contains("workflow_started"));
            assert!(json.contains("exec-123"));
        }
    }

    #[test]
    fn test_engine_response() {
        let response = EngineResponse {
            status: EngineStatus::Ok,
            output: Some(serde_json::json!({"result": 42})),
            error: None,
            next_cursor: None,
            metadata: HashMap::new(),
            execution_time_ms: Some(150),
        };

        assert_eq!(response.status, EngineStatus::Ok);
        assert!(response.output.is_some());
    }
}
