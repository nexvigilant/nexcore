//! # LoRA Adapter
//!
//! Low-Rank Adaptation (LoRA) for efficient model fine-tuning.
//! LoRA freezes base model weights and trains low-rank decomposition matrices.
//!
//! ## T1 Grounding
//! - N (Quantity): Rank, alpha scaling, dropout rate
//! - μ (Mapping): Weight matrix transformation A*B
//! - ∂ (Boundary): Target module selection, dropout

use serde::{Deserialize, Serialize};

/// LoRA configuration for fine-tuning.
///
/// Tier: T2-P (N + μ + ∂ — Quantity + Mapping + Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// LoRA rank (decomposition dimension, typically 8-64).
    pub rank: usize,
    /// LoRA alpha scaling factor.
    pub alpha: f64,
    /// Target module names to apply LoRA to (e.g. ["q_proj", "v_proj"]).
    pub target_modules: Vec<String>,
    /// Dropout rate for LoRA layers (0.0 = no dropout).
    pub dropout: f64,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            rank: 16,
            alpha: 32.0,
            target_modules: vec!["q_proj".to_string(), "v_proj".to_string()],
            dropout: 0.05,
        }
    }
}

impl LoraConfig {
    /// Create a LoRA config with a specific rank.
    pub fn with_rank(rank: usize) -> Self {
        Self {
            rank,
            alpha: rank as f64 * 2.0,
            ..Self::default()
        }
    }

    /// Effective scaling factor: alpha / rank.
    pub fn scaling(&self) -> f64 {
        if self.rank == 0 {
            return 0.0;
        }
        self.alpha / self.rank as f64
    }

    /// Total trainable parameters for a given weight matrix dimension.
    ///
    /// LoRA decomposes W (d_in x d_out) into A (d_in x rank) + B (rank x d_out).
    /// Trainable params = (d_in + d_out) * rank per module.
    pub fn trainable_params(&self, d_in: usize, d_out: usize) -> usize {
        let per_module = (d_in + d_out) * self.rank;
        per_module * self.target_modules.len()
    }

    /// Compression ratio: trainable params / original params.
    pub fn compression_ratio(&self, d_in: usize, d_out: usize) -> f64 {
        let original = d_in * d_out * self.target_modules.len();
        if original == 0 {
            return 0.0;
        }
        self.trainable_params(d_in, d_out) as f64 / original as f64
    }
}

/// Status of a LoRA adapter.
///
/// Tier: T2-P (ς + ∃ — State + Existence)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoraStatus {
    /// Adapter has not been trained yet.
    Untrained,
    /// Training in progress.
    Training,
    /// Training complete, adapter ready.
    Ready,
    /// Adapter loaded and merged with base model.
    Applied,
}

/// A LoRA adapter with its configuration and weights location.
///
/// Tier: T2-C (N + μ + ∂ + ς + π — Quantity + Mapping + Boundary + State + Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraAdapter {
    /// Name identifier for this adapter.
    pub name: String,
    /// LoRA hyperparameters.
    pub config: LoraConfig,
    /// Path to adapter weights (safetensors or bin).
    pub weights_path: Option<String>,
    /// Current status.
    pub status: LoraStatus,
}

impl LoraAdapter {
    /// Create a new untrained adapter.
    pub fn new(name: impl Into<String>, config: LoraConfig) -> Self {
        Self {
            name: name.into(),
            config,
            weights_path: None,
            status: LoraStatus::Untrained,
        }
    }

    /// Whether this adapter is ready for inference.
    pub fn is_ready(&self) -> bool {
        matches!(self.status, LoraStatus::Ready | LoraStatus::Applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lora_config_default() {
        let config = LoraConfig::default();
        assert_eq!(config.rank, 16);
        assert!((config.alpha - 32.0).abs() < f64::EPSILON);
        assert_eq!(config.target_modules.len(), 2);
    }

    #[test]
    fn test_lora_scaling() {
        let config = LoraConfig {
            rank: 8,
            alpha: 16.0,
            ..LoraConfig::default()
        };
        assert!((config.scaling() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lora_scaling_zero_rank() {
        let config = LoraConfig {
            rank: 0,
            ..LoraConfig::default()
        };
        assert!((config.scaling() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_trainable_params() {
        let config = LoraConfig {
            rank: 8,
            target_modules: vec!["q_proj".to_string()],
            ..LoraConfig::default()
        };
        // d_in=512, d_out=512 => (512+512)*8 * 1 module = 8192
        assert_eq!(config.trainable_params(512, 512), 8192);
    }

    #[test]
    fn test_compression_ratio() {
        let config = LoraConfig {
            rank: 8,
            target_modules: vec!["q_proj".to_string()],
            ..LoraConfig::default()
        };
        // trainable: 8192, original: 512*512*1 = 262144
        let ratio = config.compression_ratio(512, 512);
        assert!(ratio < 0.04); // ~3.1%
        assert!(ratio > 0.03);
    }

    #[test]
    fn test_lora_adapter_new() {
        let adapter = LoraAdapter::new("test-adapter", LoraConfig::default());
        assert_eq!(adapter.name, "test-adapter");
        assert_eq!(adapter.status, LoraStatus::Untrained);
        assert!(!adapter.is_ready());
    }

    #[test]
    fn test_lora_adapter_ready() {
        let mut adapter = LoraAdapter::new("test", LoraConfig::default());
        adapter.status = LoraStatus::Ready;
        assert!(adapter.is_ready());
    }
}
