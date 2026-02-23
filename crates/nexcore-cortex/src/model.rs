//! # Model Configuration and Registry
//!
//! Defines model formats, configurations, and the local model registry.
//!
//! ## T1 Grounding
//! - ς (State): Model configuration represents system state
//! - π (Persistence): Cached models persist across sessions
//! - ∃ (Existence): Model existence validation

use nexcore_fs::dirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported model file formats.
///
/// Tier: T2-P (ς + ∂ — State + Boundary)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    /// Quantized GGUF format — CPU-friendly, smaller footprint.
    Gguf,
    /// SafeTensors format — full precision, GPU-preferred.
    SafeTensors,
}

impl ModelFormat {
    /// Detect format from filename extension.
    pub fn from_filename(filename: &str) -> Option<Self> {
        if filename.ends_with(".gguf") {
            Some(Self::Gguf)
        } else if filename.ends_with(".safetensors") {
            Some(Self::SafeTensors)
        } else {
            None
        }
    }

    /// File extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Gguf => "gguf",
            Self::SafeTensors => "safetensors",
        }
    }
}

/// Device selection for inference.
///
/// Tier: T1 (ς — State)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceChoice {
    /// CPU inference (default, always available).
    Cpu,
    /// CUDA GPU inference with device ordinal.
    Cuda(usize),
}

impl Default for DeviceChoice {
    fn default() -> Self {
        Self::Cpu
    }
}

/// Configuration for loading a model.
///
/// Tier: T2-P (ς + π + ∃ — State + Persistence + Existence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// HuggingFace repository ID (e.g. "QuantFactory/SmolLM2-135M-Instruct-GGUF").
    pub repo_id: String,
    /// Filename within the repo (e.g. "SmolLM2-135M-Instruct-Q4_K_M.gguf").
    pub filename: String,
    /// Model file format.
    pub format: ModelFormat,
    /// Local cache directory for downloaded models.
    pub cache_dir: PathBuf,
    /// Device to run inference on.
    pub device: DeviceChoice,
}

impl ModelConfig {
    /// Create a new model config with default cache directory.
    pub fn new(repo_id: impl Into<String>, filename: impl Into<String>) -> Self {
        let filename = filename.into();
        let format = ModelFormat::from_filename(&filename).unwrap_or(ModelFormat::Gguf);
        Self {
            repo_id: repo_id.into(),
            filename,
            format,
            cache_dir: default_cache_dir(),
            device: DeviceChoice::default(),
        }
    }

    /// Set the device for inference.
    pub fn with_device(mut self, device: DeviceChoice) -> Self {
        self.device = device;
        self
    }

    /// Set a custom cache directory.
    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Self {
        self.cache_dir = cache_dir;
        self
    }

    /// Full path where the model file should be cached.
    pub fn cached_path(&self) -> PathBuf {
        self.cache_dir
            .join(self.repo_id.replace('/', "--"))
            .join(&self.filename)
    }

    /// Check whether the model file exists in the local cache.
    pub fn is_cached(&self) -> bool {
        self.cached_path().exists()
    }
}

/// Default model cache directory: `~/.cache/nexcore-cortex/models/`.
fn default_cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("nexcore-cortex")
        .join("models")
}

/// Registry entry for a downloaded model.
///
/// Tier: T2-P (π + ∃ + N — Persistence + Existence + Quantity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Model configuration used to download.
    pub config: ModelConfig,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Whether the model is currently loaded in memory.
    pub loaded: bool,
}

/// List all cached models from the cache directory.
pub fn list_cached_models(cache_dir: &std::path::Path) -> Vec<ModelEntry> {
    let mut entries = Vec::new();

    let Ok(dirs) = std::fs::read_dir(cache_dir) else {
        return entries;
    };

    for dir_entry in dirs.flatten() {
        if !dir_entry.path().is_dir() {
            continue;
        }
        let repo_dir = dir_entry.path();
        let repo_id = dir_entry.file_name().to_string_lossy().replace("--", "/");

        let Ok(files) = std::fs::read_dir(&repo_dir) else {
            continue;
        };

        for file_entry in files.flatten() {
            let path = file_entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = path.file_name().map(|n| n.to_string_lossy().to_string());
            let Some(filename) = filename else { continue };

            if ModelFormat::from_filename(&filename).is_none() {
                continue;
            }

            let size_bytes = file_entry.metadata().map(|m| m.len()).unwrap_or(0);
            let config = ModelConfig::new(repo_id.clone(), filename);
            entries.push(ModelEntry {
                config: ModelConfig {
                    cache_dir: cache_dir.to_path_buf(),
                    ..config
                },
                size_bytes,
                loaded: false,
            });
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_format_detection() {
        assert_eq!(
            ModelFormat::from_filename("model.gguf"),
            Some(ModelFormat::Gguf)
        );
        assert_eq!(
            ModelFormat::from_filename("model.safetensors"),
            Some(ModelFormat::SafeTensors)
        );
        assert_eq!(ModelFormat::from_filename("model.bin"), None);
    }

    #[test]
    fn test_model_config_cached_path() {
        let config = ModelConfig::new("org/model", "weights.gguf")
            .with_cache_dir(PathBuf::from("/tmp/test-cache"));
        assert_eq!(
            config.cached_path(),
            PathBuf::from("/tmp/test-cache/org--model/weights.gguf")
        );
    }

    #[test]
    fn test_device_choice_default() {
        assert_eq!(DeviceChoice::default(), DeviceChoice::Cpu);
    }

    #[test]
    fn test_model_format_extension() {
        assert_eq!(ModelFormat::Gguf.extension(), "gguf");
        assert_eq!(ModelFormat::SafeTensors.extension(), "safetensors");
    }

    #[test]
    fn test_list_cached_models_empty_dir() {
        let dir = std::env::temp_dir().join("nexcore-cortex-test-nonexistent");
        let models = list_cached_models(&dir);
        assert!(models.is_empty());
    }
}
