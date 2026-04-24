//! Model loading. Phase R1 supports Candle's quantized Qwen2 reading a GGUF
//! blob (reusable from Ollama's cache). Phase R2+ will add fp16/bf16 native
//! loading from safetensors for LoRA training.
//!
//! The `VigilModel` enum unifies inference for both paths so downstream code
//! (inference.rs, eval.rs) can be backend-agnostic once R2 lands.

use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use candle_transformers::models::quantized_qwen2::ModelWeights as QuantQwen2;
use candle_transformers::models::qwen2::{Config as Qwen2Config, ModelForCausalLM as NativeQwen2};
use nexcore_error::{NexError, Result};
use std::path::{Path, PathBuf};

/// Canonical GGUF path for the Qwen2.5:3b blob managed by system-installed Ollama.
pub const OLLAMA_QWEN25_3B_BLOB: &str = "/usr/share/ollama/.ollama/models/blobs/sha256-5ee4f07cdb9beadbbb293e85803c569b01bd37ed059d2715faa7bb405f31caa6";

/// Backend-agnostic wrapper around loaded model weights.
pub enum VigilModel {
    /// Quantized GGUF path (Phase R1 — inference only, no training).
    Quantized(QuantQwen2),
    /// Non-quantized fp16/bf16 loaded from safetensors (Phase R2 — training).
    Native(NativeQwen2),
}

/// Precision for the native path. Qwen2.5 ships bf16-friendly; Intel 13th gen has avx_vnni.
#[derive(Clone, Copy, Debug)]
pub enum NativeDType {
    F32,
    F16,
    BF16,
}

impl NativeDType {
    pub fn to_candle(self) -> DType {
        match self {
            NativeDType::F32 => DType::F32,
            NativeDType::F16 => DType::F16,
            NativeDType::BF16 => DType::BF16,
        }
    }
}

/// Resolve a GGUF path. If `path` is `None`, try the canonical Ollama blob.
pub fn resolve_gguf(path: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = path {
        if !p.is_file() {
            return Err(NexError::new(format!("gguf not found: {}", p.display())));
        }
        return Ok(p.to_path_buf());
    }
    let fallback = PathBuf::from(OLLAMA_QWEN25_3B_BLOB);
    if fallback.is_file() {
        return Ok(fallback);
    }
    Err(NexError::new(format!(
        "no GGUF path given and fallback not present: {}. Install Ollama + pull qwen2.5:3b, or pass --gguf.",
        OLLAMA_QWEN25_3B_BLOB
    )))
}

/// Load a quantized Qwen2 from a GGUF file.
pub fn load_quantized(path: &Path, device: &Device) -> Result<VigilModel> {
    tracing::info!("loading quantized Qwen2 from {}", path.display());
    let mut file = std::fs::File::open(path)
        .map_err(|e| NexError::new(format!("open gguf {}: {e}", path.display())))?;
    let gguf = candle_core::quantized::gguf_file::Content::read(&mut file)
        .map_err(|e| NexError::new(format!("read gguf header: {e}")))?;
    let weights = QuantQwen2::from_gguf(gguf, &mut file, device)
        .map_err(|e| NexError::new(format!("materialize weights: {e}")))?;
    Ok(VigilModel::Quantized(weights))
}

/// Choose the best device. Phase R1 is CPU-only; GPU path is opt-in via env.
pub fn pick_device() -> Result<Device> {
    // Deliberately no-GPU for R1 to simplify validation. R2 will add
    // `VIGIL_TRAIN_DEVICE=cuda|metal` env-driven selection.
    Ok(Device::Cpu)
}

// ─────────────────────────────────────────────────────────────────────────────
// Native (non-quantized) loader — Phase R2 entry
// ─────────────────────────────────────────────────────────────────────────────

/// Load a Qwen2 config.json from a HuggingFace-shaped model directory.
pub fn load_config(dir: &Path) -> Result<Qwen2Config> {
    let cfg_path = dir.join("config.json");
    let bytes = std::fs::read(&cfg_path)
        .map_err(|e| NexError::new(format!("read config {}: {e}", cfg_path.display())))?;
    let cfg: Qwen2Config = serde_json::from_slice(&bytes)
        .map_err(|e| NexError::new(format!("parse config {}: {e}", cfg_path.display())))?;
    Ok(cfg)
}

/// Find a single safetensors file in a model directory.
/// Sharded models (index.json + multiple shards) are rejected — they require
/// mmap (unsafe) which we don't use in R2. Use smaller single-file models
/// (e.g. Qwen2.5-0.5B, Qwen2.5-1.5B both ship as one file).
fn find_single_safetensors(dir: &Path) -> Result<PathBuf> {
    let single = dir.join("model.safetensors");
    if single.is_file() {
        return Ok(single);
    }
    let sharded_index = dir.join("model.safetensors.index.json");
    if sharded_index.is_file() {
        return Err(NexError::new(format!(
            "sharded safetensors not yet supported (found {}). Use a single-file model (e.g. Qwen2.5-0.5B-Instruct).",
            sharded_index.display()
        )));
    }
    Err(NexError::new(format!(
        "no model.safetensors at {}",
        dir.display()
    )))
}

/// Load a non-quantized Qwen2 model from a HF-shaped model directory.
///
/// Expected directory layout:
///   {dir}/
///     config.json
///     model.safetensors
///
/// Does NOT use mmap — reads the safetensors fully into RAM (keeps
/// `forbid(unsafe_code)` intact). Fine for Qwen2.5-0.5B (~1 GB).
pub fn load_native(dir: &Path, dtype: NativeDType, device: &Device) -> Result<VigilModel> {
    let cfg = load_config(dir)?;
    let weights_path = find_single_safetensors(dir)?;
    tracing::info!(
        "loading native Qwen2 from {} ({} weights, dtype={:?})",
        dir.display(),
        weights_path.display(),
        dtype
    );
    let bytes = std::fs::read(&weights_path)
        .map_err(|e| NexError::new(format!("read weights {}: {e}", weights_path.display())))?;
    let vb = VarBuilder::from_buffered_safetensors(bytes, dtype.to_candle(), device)
        .map_err(|e| NexError::new(format!("VarBuilder: {e}")))?;
    let model =
        NativeQwen2::new(&cfg, vb).map_err(|e| NexError::new(format!("Qwen2 construct: {e}")))?;
    Ok(VigilModel::Native(model))
}
