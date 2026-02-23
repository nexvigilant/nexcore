//! Cortex MCP tools — local LLM inference with HuggingFace Candle.
//!
//! # T1 Grounding
//! - μ (Mapping): prompt → response, text → embeddings
//! - σ (Sequence): download → load → generate pipeline
//! - ς (State): model loaded/unloaded state
//! - N (Quantity): token counts, model size, parameters

use nexcore_cortex::cloud::{DatasetConfig, FineTuneJob, JobStatus};
use nexcore_cortex::generate::GenerateParams;
use nexcore_cortex::lora::LoraConfig;
use nexcore_cortex::model::{ModelConfig, ModelFormat, list_cached_models};
use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    CortexDownloadParams, CortexEmbedParams, CortexFineTuneStatusParams, CortexGenerateParams,
    CortexListModelsParams, CortexModelInfoParams,
};

// ============================================================================
// Download & Cache Management
// ============================================================================

/// Download a model from HuggingFace Hub to the local cache.
pub fn cortex_download_model(params: CortexDownloadParams) -> Result<CallToolResult, McpError> {
    let config = ModelConfig::new(&params.repo_id, &params.filename);
    let cache_path = config.cached_path();

    // Check if already cached
    if cache_path.exists() {
        let size = std::fs::metadata(&cache_path).map(|m| m.len()).unwrap_or(0);
        let response = serde_json::json!({
            "status": "already_cached",
            "repo_id": params.repo_id,
            "filename": params.filename,
            "path": cache_path.display().to_string(),
            "size_bytes": size,
            "size_mb": format!("{:.1}", size as f64 / 1_048_576.0),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]));
    }

    // Attempt download
    match nexcore_cortex::download::download_model(&config) {
        Ok(path) => {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let response = serde_json::json!({
                "status": "downloaded",
                "repo_id": params.repo_id,
                "filename": params.filename,
                "path": path.display().to_string(),
                "size_bytes": size,
                "size_mb": format!("{:.1}", size as f64 / 1_048_576.0),
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
        Err(e) => {
            let response = serde_json::json!({
                "status": "error",
                "repo_id": params.repo_id,
                "filename": params.filename,
                "error": e.to_string(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                response.to_string(),
            )]))
        }
    }
}

/// List all locally cached models.
pub fn cortex_list_models(params: CortexListModelsParams) -> Result<CallToolResult, McpError> {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".cache")
        .join("nexcore-cortex")
        .join("models");

    let mut models = list_cached_models(&cache_dir);

    // Apply filter if provided
    if let Some(ref filter) = params.filter {
        let filter_lower = filter.to_lowercase();
        models.retain(|m| m.config.repo_id.to_lowercase().contains(&filter_lower));
    }

    let model_list: Vec<serde_json::Value> = models
        .iter()
        .map(|m| {
            serde_json::json!({
                "repo_id": m.config.repo_id,
                "filename": m.config.filename,
                "format": format!("{:?}", m.config.format),
                "size_bytes": m.size_bytes,
                "size_mb": format!("{:.1}", m.size_bytes as f64 / 1_048_576.0),
                "path": m.config.cached_path().display().to_string(),
            })
        })
        .collect();

    let response = serde_json::json!({
        "cache_dir": cache_dir.display().to_string(),
        "total_models": model_list.len(),
        "models": model_list,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Model Info
// ============================================================================

/// Get detailed info about a specific cached model.
pub fn cortex_model_info(params: CortexModelInfoParams) -> Result<CallToolResult, McpError> {
    let config = ModelConfig::new(&params.repo_id, &params.filename);
    let cache_path = config.cached_path();

    if !cache_path.exists() {
        let response = serde_json::json!({
            "status": "not_cached",
            "repo_id": params.repo_id,
            "filename": params.filename,
            "hint": "Use cortex_download_model to download first.",
        });
        return Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]));
    }

    let size = std::fs::metadata(&cache_path).map(|m| m.len()).unwrap_or(0);
    let format = ModelFormat::from_filename(&params.filename);

    let response = serde_json::json!({
        "status": "cached",
        "repo_id": params.repo_id,
        "filename": params.filename,
        "format": format.map(|f| format!("{f:?}")),
        "path": cache_path.display().to_string(),
        "size_bytes": size,
        "size_mb": format!("{:.1}", size as f64 / 1_048_576.0),
        "device": "cpu",
        "grounding": {
            "tier": "T2-P",
            "dominant": "ς (State)",
            "primitives": ["ς", "π", "∃"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Generation (placeholder — requires loaded model)
// ============================================================================

/// Generate text using a local model.
///
/// Note: Full autoregressive generation requires downloading and loading a model.
/// This tool returns engine readiness info and generation parameters.
pub fn cortex_generate(params: CortexGenerateParams) -> Result<CallToolResult, McpError> {
    let config = ModelConfig::new(&params.repo_id, "");
    let gen_params = GenerateParams {
        max_tokens: params.max_tokens,
        temperature: params.temperature,
        ..GenerateParams::default()
    };

    let cached = config
        .cached_path()
        .parent()
        .map(|p| p.exists())
        .unwrap_or(false);

    let response = serde_json::json!({
        "status": if cached { "model_available" } else { "model_not_cached" },
        "repo_id": params.repo_id,
        "prompt_preview": if params.prompt.len() > 100 {
            format!("{}...", &params.prompt[..100])
        } else {
            params.prompt.clone()
        },
        "params": {
            "max_tokens": gen_params.max_tokens,
            "temperature": gen_params.temperature,
            "top_p": gen_params.top_p,
            "repeat_penalty": gen_params.repeat_penalty,
        },
        "note": "Full generation requires: cortex_download_model → load model → generate. Engine scaffold ready.",
        "grounding": {
            "tier": "T3",
            "dominant": "μ (Mapping)",
            "primitives": ["σ", "μ", "ς", "N", "→", "∂"],
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Generate text embeddings for a given input.
pub fn cortex_embed(params: CortexEmbedParams) -> Result<CallToolResult, McpError> {
    let config = ModelConfig::new(&params.repo_id, "");
    let cached = config
        .cached_path()
        .parent()
        .map(|p| p.exists())
        .unwrap_or(false);

    let response = serde_json::json!({
        "status": if cached { "model_available" } else { "model_not_cached" },
        "repo_id": params.repo_id,
        "text_length": params.text.len(),
        "note": "Embedding extraction requires loaded model. Engine scaffold ready.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Fine-Tuning Status
// ============================================================================

/// Check the status of a cloud fine-tuning job.
pub fn cortex_fine_tune_status(
    params: CortexFineTuneStatusParams,
) -> Result<CallToolResult, McpError> {
    // Placeholder: in production this would query NexCloud job registry
    let config = ModelConfig::new("placeholder/model", "weights.gguf");
    let dataset = DatasetConfig::default();
    let job = FineTuneJob::new(&params.job_id, config, dataset).with_lora(LoraConfig::default());

    let response = serde_json::json!({
        "job_id": params.job_id,
        "status": format!("{:?}", job.status),
        "progress": format!("{:.1}%", job.progress() * 100.0),
        "lora_config": {
            "rank": job.lora_config.rank,
            "alpha": job.lora_config.alpha,
            "scaling": job.lora_config.scaling(),
            "target_modules": job.lora_config.target_modules,
        },
        "training_params": {
            "epochs": job.training_params.epochs,
            "batch_size": job.training_params.batch_size,
            "learning_rate": job.training_params.learning_rate,
        },
        "note": "Fine-tuning dispatch requires NexCloud GPU allocation. Pipeline scaffold ready.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
