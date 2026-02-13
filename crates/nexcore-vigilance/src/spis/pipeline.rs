//! Pipeline, component, and execution models.
//!
//! Data structures for defining and tracking data pipelines.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A data pipeline definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Unique pipeline identifier
    pub id: NexId,
    /// Pipeline name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Pipeline purpose
    pub purpose: String,
    /// Type classification (e.g., "ETL", "Streaming")
    pub pipeline_type: String,
    /// Architecture pattern (e.g., "Lambda", "Kappa")
    pub architecture_pattern: String,
    /// Technology stack configuration
    pub tech_stack: Option<serde_json::Value>,
    /// Workflow step definitions
    pub workflow_steps: Option<serde_json::Value>,
    /// Estimated monthly cost
    pub estimated_monthly_cost: Option<Decimal>,
    /// Current status
    pub status: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Pipeline {
    /// Create a new pipeline with default timestamps.
    #[must_use]
    pub fn new(name: String, purpose: String, pipeline_type: String) -> Self {
        let now = Utc::now();
        Self {
            id: NexId::v4(),
            name,
            description: None,
            purpose,
            pipeline_type,
            architecture_pattern: "default".to_string(),
            tech_stack: None,
            workflow_steps: None,
            estimated_monthly_cost: None,
            status: "draft".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A component within a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineComponent {
    /// Component identifier
    pub id: NexId,
    /// Parent pipeline ID
    pub pipeline_id: NexId,
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: String,
    /// Associated service name
    pub service_name: String,
    /// Component configuration
    pub configuration: Option<serde_json::Value>,
    /// Execution sequence order
    pub sequence_order: i32,
    /// Dependencies on other components
    pub depends_on: Option<serde_json::Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// A pipeline execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecution {
    /// Execution identifier
    pub id: NexId,
    /// Pipeline that was executed
    pub pipeline_id: NexId,
    /// Current execution status
    pub execution_status: String,
    /// How the execution was triggered
    pub trigger_type: String,
    /// Who/what triggered the execution
    pub triggered_by: Option<String>,
    /// Number of records processed
    pub records_processed: Option<i32>,
    /// Duration in seconds
    pub execution_duration_seconds: Option<f64>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Execution metrics
    pub metrics: Option<serde_json::Value>,
    /// Start timestamp
    pub started_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = Pipeline::new(
            "test-pipeline".to_string(),
            "Testing".to_string(),
            "ETL".to_string(),
        );
        assert_eq!(pipeline.name, "test-pipeline");
        assert_eq!(pipeline.status, "draft");
    }
}
