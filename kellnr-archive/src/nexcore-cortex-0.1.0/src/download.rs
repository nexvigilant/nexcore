//! # HuggingFace Hub Download
//!
//! Downloads model weights and tokenizer files from HuggingFace Hub.
//!
//! ## T1 Grounding
//! - σ (Sequence): Download pipeline stages
//! - π (Persistence): Cached files on disk
//! - → (Causality): Download triggers availability

use crate::CortexError;
use crate::model::ModelConfig;
use std::path::PathBuf;
use tracing::info;

/// Download a model file from HuggingFace Hub to local cache.
///
/// Returns the local path of the downloaded file.
///
/// Tier: T2-C (σ + π + → + ∃ — Sequence + Persistence + Causality + Existence)
pub fn download_model(config: &ModelConfig) -> Result<PathBuf, CortexError> {
    let cache_path = config.cached_path();

    // Check if already cached
    if cache_path.exists() {
        info!(
            repo_id = %config.repo_id,
            filename = %config.filename,
            path = %cache_path.display(),
            "Model already cached"
        );
        return Ok(cache_path);
    }

    // Ensure parent directory exists
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CortexError::DownloadError(format!("Failed to create cache dir: {e}")))?;
    }

    info!(
        repo_id = %config.repo_id,
        filename = %config.filename,
        "Downloading model from HuggingFace Hub"
    );

    // Use hf-hub to download
    let api = hf_hub::api::sync::Api::new()
        .map_err(|e| CortexError::DownloadError(format!("Failed to create HF API client: {e}")))?;

    let repo = api.model(config.repo_id.clone());

    let downloaded_path = repo
        .get(&config.filename)
        .map_err(|e| CortexError::DownloadError(format!("Download failed: {e}")))?;

    // Copy or symlink to our cache directory
    if downloaded_path != cache_path {
        std::fs::copy(&downloaded_path, &cache_path).map_err(|e| {
            CortexError::DownloadError(format!(
                "Failed to copy {} to {}: {e}",
                downloaded_path.display(),
                cache_path.display()
            ))
        })?;
    }

    info!(
        path = %cache_path.display(),
        "Model downloaded successfully"
    );

    Ok(cache_path)
}

/// Download the tokenizer.json for a model from HuggingFace Hub.
///
/// Returns the local path of the tokenizer file.
pub fn download_tokenizer(repo_id: &str) -> Result<PathBuf, CortexError> {
    let api = hf_hub::api::sync::Api::new()
        .map_err(|e| CortexError::DownloadError(format!("Failed to create HF API client: {e}")))?;

    let repo = api.model(repo_id.to_string());

    let tokenizer_path = repo
        .get("tokenizer.json")
        .map_err(|e| CortexError::DownloadError(format!("Tokenizer download failed: {e}")))?;

    info!(
        repo_id = %repo_id,
        path = %tokenizer_path.display(),
        "Tokenizer downloaded successfully"
    );

    Ok(tokenizer_path)
}

/// Check if a model exists on HuggingFace Hub (without downloading).
pub fn model_exists(repo_id: &str, filename: &str) -> Result<bool, CortexError> {
    let api = hf_hub::api::sync::Api::new()
        .map_err(|e| CortexError::DownloadError(format!("Failed to create HF API client: {e}")))?;

    let repo = api.model(repo_id.to_string());

    // Try to get info — if the file exists, this succeeds
    match repo.get(filename) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_download_model_cached() {
        // Create a temp file to simulate a cached model
        let dir = std::env::temp_dir().join("nexcore-cortex-dl-test");
        let _ = std::fs::create_dir_all(dir.join("org--model"));
        let cached = dir.join("org--model").join("weights.gguf");
        let _ = std::fs::write(&cached, b"fake model data");

        let config = ModelConfig {
            repo_id: "org/model".to_string(),
            filename: "weights.gguf".to_string(),
            format: crate::model::ModelFormat::Gguf,
            cache_dir: dir.clone(),
            device: crate::model::DeviceChoice::Cpu,
        };

        let result = download_model(&config);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(cached.clone()));

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
