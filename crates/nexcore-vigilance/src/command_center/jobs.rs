//! Job domain types: STARK automation job queue.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::enums::{JobStatus, JobType};

/// STARK automation job model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarkJob {
    /// Unique identifier.
    pub id: NexId,

    /// Associated LLC ID (optional for user-level jobs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llc_id: Option<NexId>,

    /// User who triggered the job.
    pub user_id: NexId,

    /// Job type.
    pub job_type: JobType,

    /// Job status.
    #[serde(default)]
    pub status: JobStatus,

    /// Function name (for Apps Script jobs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,

    /// Job parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,

    /// When execution started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    /// When execution completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Execution duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,

    /// Job result (on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error message (on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Error stack trace (on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_trace: Option<String>,

    /// Number of retry attempts.
    #[serde(default)]
    pub retry_count: i32,

    /// Maximum retry attempts.
    #[serde(default = "default_max_retries")]
    pub max_retries: i32,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_max_retries() -> i32 {
    3
}

impl StarkJob {
    /// Create a new job.
    #[must_use]
    pub fn new(user_id: NexId, job_type: JobType) -> Self {
        Self {
            id: NexId::v4(),
            llc_id: None,
            user_id,
            job_type,
            status: JobStatus::Pending,
            function_name: None,
            parameters: None,
            started_at: None,
            completed_at: None,
            duration_ms: None,
            result: None,
            error_message: None,
            error_trace: None,
            retry_count: 0,
            max_retries: 3,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// Create a new job for an LLC.
    #[must_use]
    pub fn new_for_llc(llc_id: NexId, user_id: NexId, job_type: JobType) -> Self {
        let mut job = Self::new(user_id, job_type);
        job.llc_id = Some(llc_id);
        job
    }

    /// Check if job is pending.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status == JobStatus::Pending
    }

    /// Check if job is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.status == JobStatus::Running
    }

    /// Check if job is complete (success or failure).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    /// Check if job can be retried.
    #[must_use]
    pub fn can_retry(&self) -> bool {
        self.status == JobStatus::Failed && self.retry_count < self.max_retries
    }

    /// Start the job.
    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.updated_at = Some(Utc::now());
    }

    /// Complete the job successfully.
    pub fn complete(&mut self, result: Option<Value>) {
        let now = Utc::now();
        self.status = JobStatus::Completed;
        self.completed_at = Some(now);
        self.result = result;

        // Calculate duration
        if let Some(started) = self.started_at {
            self.duration_ms = Some((now - started).num_milliseconds());
        }

        self.updated_at = Some(now);
    }

    /// Mark job as failed.
    pub fn fail(&mut self, error: impl Into<String>, trace: Option<String>) {
        let now = Utc::now();
        self.status = JobStatus::Failed;
        self.completed_at = Some(now);
        self.error_message = Some(error.into());
        self.error_trace = trace;

        // Calculate duration
        if let Some(started) = self.started_at {
            self.duration_ms = Some((now - started).num_milliseconds());
        }

        self.updated_at = Some(now);
    }

    /// Cancel the job.
    pub fn cancel(&mut self) {
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.updated_at = Some(Utc::now());
    }

    /// Retry the job.
    pub fn retry(&mut self) {
        if self.can_retry() {
            self.retry_count += 1;
            self.status = JobStatus::Pending;
            self.started_at = None;
            self.completed_at = None;
            self.duration_ms = None;
            self.result = None;
            self.error_message = None;
            self.error_trace = None;
            self.updated_at = Some(Utc::now());
        }
    }

    /// Get job summary for logging.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Job {} ({:?}): {:?} [retries: {}/{}]",
            self.id, self.job_type, self.status, self.retry_count, self.max_retries
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_new() {
        let user_id = NexId::v4();
        let job = StarkJob::new(user_id, JobType::DriveSearch);

        assert_eq!(job.user_id, user_id);
        assert_eq!(job.job_type, JobType::DriveSearch);
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.llc_id.is_none());
    }

    #[test]
    fn test_job_for_llc() {
        let llc_id = NexId::v4();
        let user_id = NexId::v4();
        let job = StarkJob::new_for_llc(llc_id, user_id, JobType::CalendarSync);

        assert_eq!(job.llc_id, Some(llc_id));
    }

    #[test]
    fn test_job_lifecycle() {
        let user_id = NexId::v4();
        let mut job = StarkJob::new(user_id, JobType::AbasAnalysis);

        assert!(job.is_pending());
        assert!(!job.is_running());
        assert!(!job.is_complete());

        job.start();
        assert!(!job.is_pending());
        assert!(job.is_running());
        assert!(job.started_at.is_some());

        job.complete(Some(serde_json::json!({ "success": true })));
        assert!(job.is_complete());
        assert!(job.completed_at.is_some());
        assert!(job.duration_ms.is_some());
    }

    #[test]
    fn test_job_failure_and_retry() {
        let user_id = NexId::v4();
        let mut job = StarkJob::new(user_id, JobType::AppsScriptTrigger);

        job.start();
        job.fail("Connection timeout", None);

        assert_eq!(job.status, JobStatus::Failed);
        assert!(job.can_retry());
        assert_eq!(job.retry_count, 0);

        job.retry();
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.retry_count, 1);

        // Retry until max
        job.start();
        job.fail("Error 1", None);
        job.retry();

        job.start();
        job.fail("Error 2", None);
        job.retry();

        job.start();
        job.fail("Error 3", None);

        assert!(!job.can_retry()); // Max retries reached
        assert_eq!(job.retry_count, 3);
    }

    #[test]
    fn test_job_cancel() {
        let user_id = NexId::v4();
        let mut job = StarkJob::new(user_id, JobType::DriveSearch);

        job.start();
        job.cancel();

        assert_eq!(job.status, JobStatus::Cancelled);
        assert!(job.is_complete());
    }

    #[test]
    fn test_job_summary() {
        let user_id = NexId::v4();
        let job = StarkJob::new(user_id, JobType::DriveSearch);

        let summary = job.summary();
        assert!(summary.contains("DriveSearch"));
        assert!(summary.contains("Pending"));
    }
}
