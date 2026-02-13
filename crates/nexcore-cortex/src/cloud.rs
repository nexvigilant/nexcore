//! # NexCloud GPU Fine-Tuning
//!
//! Dispatch fine-tuning jobs to NexCloud GPU instances.
//! Uses `nexcore-cloud` primitives for VM/compute modeling.
//!
//! ## T1 Grounding
//! - σ (Sequence): Job lifecycle stages
//! - ς (State): Job status transitions
//! - π (Persistence): Training checkpoints
//! - N (Quantity): Epochs, batch size, learning rate

use crate::lora::LoraConfig;
use crate::model::ModelConfig;
use serde::{Deserialize, Serialize};

/// Status of a fine-tuning job.
///
/// Tier: T1 (ς — State)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job created, awaiting resources.
    Pending,
    /// GPU allocated, training started.
    Running,
    /// Training complete, adapter ready.
    Completed,
    /// Job failed with error.
    Failed(String),
    /// Job was cancelled.
    Cancelled,
}

/// Configuration for training data.
///
/// Tier: T2-P (σ + N — Sequence + Quantity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetConfig {
    /// Path or URI to training data (JSONL format).
    pub train_path: String,
    /// Optional path to validation data.
    pub eval_path: Option<String>,
    /// Maximum sequence length for training examples.
    pub max_seq_length: usize,
}

impl Default for DatasetConfig {
    fn default() -> Self {
        Self {
            train_path: String::new(),
            eval_path: None,
            max_seq_length: 512,
        }
    }
}

/// Training hyperparameters.
///
/// Tier: T2-P (N + ∂ — Quantity + Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingParams {
    /// Number of training epochs.
    pub epochs: usize,
    /// Batch size per step.
    pub batch_size: usize,
    /// Learning rate.
    pub learning_rate: f64,
    /// Weight decay for regularization.
    pub weight_decay: f64,
    /// Warmup steps for learning rate schedule.
    pub warmup_steps: usize,
    /// Save checkpoint every N steps.
    pub save_steps: usize,
}

impl Default for TrainingParams {
    fn default() -> Self {
        Self {
            epochs: 3,
            batch_size: 4,
            learning_rate: 2e-4,
            weight_decay: 0.01,
            warmup_steps: 100,
            save_steps: 500,
        }
    }
}

/// A fine-tuning job definition.
///
/// Tier: T2-C (σ + ς + π + N — Sequence + State + Persistence + Quantity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuneJob {
    /// Unique job identifier.
    pub job_id: String,
    /// Base model to fine-tune.
    pub base_model: ModelConfig,
    /// Training dataset configuration.
    pub dataset: DatasetConfig,
    /// LoRA hyperparameters.
    pub lora_config: LoraConfig,
    /// Training hyperparameters.
    pub training_params: TrainingParams,
    /// Current job status.
    pub status: JobStatus,
    /// GPU instance type requested (e.g. "a100-40gb", "t4").
    pub gpu_type: String,
    /// Training progress: current step.
    pub current_step: usize,
    /// Training progress: total steps.
    pub total_steps: usize,
    /// Current training loss (if running).
    pub current_loss: Option<f64>,
}

impl FineTuneJob {
    /// Create a new fine-tuning job.
    pub fn new(job_id: impl Into<String>, base_model: ModelConfig, dataset: DatasetConfig) -> Self {
        Self {
            job_id: job_id.into(),
            base_model,
            dataset,
            lora_config: LoraConfig::default(),
            training_params: TrainingParams::default(),
            status: JobStatus::Pending,
            gpu_type: "t4".to_string(),
            current_step: 0,
            total_steps: 0,
            current_loss: None,
        }
    }

    /// Set the LoRA configuration.
    pub fn with_lora(mut self, config: LoraConfig) -> Self {
        self.lora_config = config;
        self
    }

    /// Set the training parameters.
    pub fn with_training_params(mut self, params: TrainingParams) -> Self {
        self.training_params = params;
        self
    }

    /// Set the GPU type.
    pub fn with_gpu(mut self, gpu_type: impl Into<String>) -> Self {
        self.gpu_type = gpu_type.into();
        self
    }

    /// Whether the job is still active (pending or running).
    pub fn is_active(&self) -> bool {
        matches!(self.status, JobStatus::Pending | JobStatus::Running)
    }

    /// Training progress as a fraction [0.0, 1.0].
    pub fn progress(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.current_step as f64 / self.total_steps as f64
    }

    /// Summary of job status for display.
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({
            "job_id": self.job_id,
            "status": format!("{:?}", self.status),
            "base_model": self.base_model.repo_id,
            "gpu_type": self.gpu_type,
            "progress": format!("{:.1}%", self.progress() * 100.0),
            "current_step": self.current_step,
            "total_steps": self.total_steps,
            "current_loss": self.current_loss,
            "lora_rank": self.lora_config.rank,
            "epochs": self.training_params.epochs,
            "learning_rate": self.training_params.learning_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_variants() {
        assert_eq!(JobStatus::Pending, JobStatus::Pending);
        assert_ne!(JobStatus::Pending, JobStatus::Running);
    }

    #[test]
    fn test_dataset_config_default() {
        let config = DatasetConfig::default();
        assert!(config.train_path.is_empty());
        assert!(config.eval_path.is_none());
        assert_eq!(config.max_seq_length, 512);
    }

    #[test]
    fn test_training_params_default() {
        let params = TrainingParams::default();
        assert_eq!(params.epochs, 3);
        assert_eq!(params.batch_size, 4);
        assert!((params.learning_rate - 2e-4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fine_tune_job_new() {
        let config = ModelConfig::new("test/model", "weights.gguf");
        let dataset = DatasetConfig::default();
        let job = FineTuneJob::new("job-001", config, dataset);

        assert_eq!(job.job_id, "job-001");
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.is_active());
        assert!((job.progress() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fine_tune_job_progress() {
        let config = ModelConfig::new("test/model", "weights.gguf");
        let dataset = DatasetConfig::default();
        let mut job = FineTuneJob::new("job-002", config, dataset);
        job.total_steps = 1000;
        job.current_step = 500;

        assert!((job.progress() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fine_tune_job_summary() {
        let config = ModelConfig::new("org/model", "weights.gguf");
        let dataset = DatasetConfig::default();
        let job = FineTuneJob::new("job-003", config, dataset);
        let summary = job.summary();

        assert_eq!(summary["job_id"], "job-003");
        assert_eq!(summary["base_model"], "org/model");
    }

    #[test]
    fn test_builder_pattern() {
        let config = ModelConfig::new("test/model", "weights.gguf");
        let dataset = DatasetConfig::default();
        let job = FineTuneJob::new("job-004", config, dataset)
            .with_lora(LoraConfig::with_rank(32))
            .with_gpu("a100-40gb")
            .with_training_params(TrainingParams {
                epochs: 5,
                ..TrainingParams::default()
            });

        assert_eq!(job.lora_config.rank, 32);
        assert_eq!(job.gpu_type, "a100-40gb");
        assert_eq!(job.training_params.epochs, 5);
    }
}
