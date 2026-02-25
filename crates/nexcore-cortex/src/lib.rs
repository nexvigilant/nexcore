//! # NexVigilant Core — cortex
//!
//! Local LLM inference engine using HuggingFace Candle with NexCloud fine-tuning.
//!
//! ## Architecture
//!
//! The cortex is the local brain for Guardian's COMPARE phase:
//!
//! ```text
//! SENSE (PAMPs/DAMPs) → COMPARE (cortex: local model) → ACT (executor)
//!                            ↓ (fallback if uncertain)
//!                        Claude/Gemini API
//! ```
//!
//! ## Type Inventory (10 grounded types)
//!
//! | Type | Tier | Dominant | Description |
//! |------|------|----------|-------------|
//! | `ModelConfig` | T2-P | ς State | Model loading configuration |
//! | `ModelFormat` | T2-P | ς State | GGUF or SafeTensors format |
//! | `GenerateParams` | T2-P | N Quantity | Sampling parameters |
//! | `LoraConfig` | T2-P | N Quantity | LoRA fine-tuning hyperparams |
//! | `DatasetConfig` | T2-P | σ Sequence | Training data config |
//! | `TrainingParams` | T2-P | N Quantity | Training hyperparameters |
//! | `LoraAdapter` | T2-C | N Quantity | LoRA adapter with weights |
//! | `FineTuneJob` | T2-C | σ Sequence | Cloud fine-tuning job |
//! | `InferenceEngine` | T3 | μ Mapping | Core inference engine |
//! | `CortexTokenizer` | T2-P | μ Mapping | Tokenizer wrapper |
//!
//! ## Example
//!
//! ```rust,no_run
//! use nexcore_cortex::model::ModelConfig;
//! use nexcore_cortex::generate::GenerateParams;
//!
//! let config = ModelConfig::new(
//!     "QuantFactory/SmolLM2-135M-Instruct-GGUF",
//!     "SmolLM2-135M-Instruct-Q4_K_M.gguf",
//! );
//! assert!(!config.is_cached());
//!
//! let params = GenerateParams::default();
//! assert_eq!(params.max_tokens, 512);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod cloud;
pub mod download;
pub mod engine;
pub mod generate;
pub mod grounding;
pub mod lora;
pub mod model;
pub mod tokenizer;

// Re-exports for convenience
pub use cloud::{DatasetConfig, FineTuneJob, JobStatus, TrainingParams};
pub use engine::InferenceEngine;
pub use generate::GenerateParams;
pub use lora::{LoraAdapter, LoraConfig, LoraStatus};
pub use model::{DeviceChoice, ModelConfig, ModelEntry, ModelFormat};
pub use tokenizer::CortexTokenizer;

/// Errors that can occur during cortex operations.
///
/// Tier: T2-P (∂ + ∃ — Boundary + Existence)
#[derive(Debug, nexcore_error::Error)]
pub enum CortexError {
    /// Model file not found in cache.
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Failed to load model weights.
    #[error("Model load error: {0}")]
    ModelLoadError(String),

    /// Model not loaded — call load() first.
    #[error("Not loaded: {0}")]
    NotLoaded(String),

    /// Download from HuggingFace Hub failed.
    #[error("Download error: {0}")]
    DownloadError(String),

    /// Tokenizer error.
    #[error("Tokenizer error: {0}")]
    TokenizerError(String),

    /// Device (CPU/CUDA) initialization error.
    #[error("Device error: {0}")]
    DeviceError(String),

    /// Candle tensor operation error.
    #[error("Tensor error: {0}")]
    TensorError(#[from] candle_core::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
