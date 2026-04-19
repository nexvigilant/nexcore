//! Adapter merge — fold LoRA adapters into base weights, producing a
//! standalone model directory that `load_native` can read directly.
//!
//! Strategy (all in-Rust, zero Python):
//! 1. Read base `model.safetensors` into a `HashMap<String, Tensor>`.
//! 2. Build a LoRA bundle with matching shapes, load adapter weights.
//! 3. For each `(layer, target)` adapter, look up the matching base weight
//!    (`model.layers.{L}.self_attn.{q,k,v,o}_proj.weight`) and replace it
//!    with `W + (α/r) · B · A`.
//! 4. Write the merged tensors to `output_dir/model.safetensors`.
//! 5. Copy `config.json` + `tokenizer.json` so the dir is immediately usable.

use candle_core::{DType, Device, Tensor};
use nexcore_error::{NexError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::lora::{LoraBundle, LoraConfig, LoraTarget};

#[derive(Clone, Debug)]
pub struct MergeConfig {
    pub base_model_dir: PathBuf,
    pub adapter_path: PathBuf,
    pub output_dir: PathBuf,
    pub lora_r: usize,
    pub lora_alpha: f64,
    pub dtype: DType,
}

impl Default for MergeConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
        Self {
            base_model_dir: PathBuf::from(format!("{home}/.claude/brain/vigil-base-v1")),
            adapter_path: PathBuf::from(format!(
                "{home}/.claude/brain/vigil-lora-v1/adapter.safetensors"
            )),
            output_dir: PathBuf::from(format!("{home}/.claude/brain/vigil-qwen-v1")),
            lora_r: 16,
            lora_alpha: 32.0,
            dtype: DType::F32,
        }
    }
}

/// Compose the tensor name the base model uses for a given (layer, target).
fn base_weight_name(layer: usize, target: LoraTarget) -> String {
    format!("model.layers.{}.self_attn.{}.weight", layer, target.name())
}

/// Load a safetensors file into an in-memory HashMap of (name → Tensor).
/// All tensors are loaded at `dtype` on `device`.
fn load_safetensors_map(
    path: &Path,
    dtype: DType,
    device: &Device,
) -> Result<HashMap<String, Tensor>> {
    let bytes =
        std::fs::read(path).map_err(|e| NexError::new(format!("read {}: {e}", path.display())))?;
    let tensors = candle_core::safetensors::load_buffer(&bytes, device)
        .map_err(|e| NexError::new(format!("safetensors parse {}: {e}", path.display())))?;
    let mut out = HashMap::with_capacity(tensors.len());
    for (name, t) in tensors {
        let cast = t
            .to_dtype(dtype)
            .map_err(|e| NexError::new(format!("cast {name}: {e}")))?;
        out.insert(name, cast);
    }
    Ok(out)
}

pub fn run(cfg: &MergeConfig) -> Result<()> {
    let device = Device::Cpu;

    // Validate inputs
    let base_weights_path = cfg.base_model_dir.join("model.safetensors");
    let config_path = cfg.base_model_dir.join("config.json");
    let tokenizer_path = cfg.base_model_dir.join("tokenizer.json");
    if !base_weights_path.is_file() {
        return Err(NexError::new(format!(
            "base model.safetensors missing at {}",
            base_weights_path.display()
        )));
    }
    if !cfg.adapter_path.is_file() {
        return Err(NexError::new(format!(
            "adapter missing at {}",
            cfg.adapter_path.display()
        )));
    }
    std::fs::create_dir_all(&cfg.output_dir)
        .map_err(|e| NexError::new(format!("create output_dir: {e}")))?;

    // Parse Qwen2 config to get architecture dims
    let qwen2_cfg = crate::model::load_config(&cfg.base_model_dir)?;

    // Build a matching LoRA bundle (zero-init), then load the trained adapter
    let lora_cfg = LoraConfig {
        r: cfg.lora_r,
        alpha: cfg.lora_alpha,
        dropout: 0.0,
    };
    let mut bundle = LoraBundle::attach_canonical(
        qwen2_cfg.num_hidden_layers,
        qwen2_cfg.hidden_size,
        qwen2_cfg.num_attention_heads,
        qwen2_cfg.num_key_value_heads,
        lora_cfg,
        cfg.dtype,
        &device,
    )?;
    tracing::info!(
        "[merge] bundle: {} adapters, loading from {}",
        bundle.adapters.len(),
        cfg.adapter_path.display()
    );
    bundle.load(&cfg.adapter_path)?;

    // Load full base tensor map
    tracing::info!(
        "[merge] loading base weights from {}",
        base_weights_path.display()
    );
    let mut tensors = load_safetensors_map(&base_weights_path, cfg.dtype, &device)?;
    tracing::info!("[merge] base tensors: {}", tensors.len());

    // Fold each adapter into its matching base weight
    let mut folded = 0usize;
    let mut missing = Vec::new();
    for adapter in &bundle.adapters {
        let name = base_weight_name(adapter.layer, adapter.target);
        let base_w = match tensors.get(&name) {
            Some(t) => t.clone(),
            None => {
                missing.push(name.clone());
                continue;
            }
        };
        let merged = adapter.fold_into(&base_w)?;
        tensors.insert(name, merged);
        folded += 1;
    }
    if !missing.is_empty() {
        return Err(NexError::new(format!(
            "could not locate {} base tensors for fold (examples: {:?})",
            missing.len(),
            missing.iter().take(3).collect::<Vec<_>>()
        )));
    }
    tracing::info!("[merge] folded {} adapters into base weights", folded);

    // Save merged tensors as a single safetensors file
    let out_weights = cfg.output_dir.join("model.safetensors");
    let data: Vec<(String, Tensor)> = tensors.into_iter().collect();
    candle_core::safetensors::save(
        &data.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        &out_weights,
    )
    .map_err(|e| NexError::new(format!("save merged safetensors: {e}")))?;
    tracing::info!("[merge] wrote {}", out_weights.display());

    // Copy config.json + tokenizer.json so the output dir is immediately usable
    // by load_native()
    if config_path.is_file() {
        std::fs::copy(&config_path, cfg.output_dir.join("config.json"))
            .map_err(|e| NexError::new(format!("copy config.json: {e}")))?;
    }
    if tokenizer_path.is_file() {
        std::fs::copy(&tokenizer_path, cfg.output_dir.join("tokenizer.json"))
            .map_err(|e| NexError::new(format!("copy tokenizer.json: {e}")))?;
    }

    // Success JSON to stdout
    let out = serde_json::json!({
        "status": "merged",
        "base_model_dir": cfg.base_model_dir.display().to_string(),
        "adapter_path": cfg.adapter_path.display().to_string(),
        "output_dir": cfg.output_dir.display().to_string(),
        "adapters_folded": folded,
        "lora_r": cfg.lora_r,
        "lora_alpha": cfg.lora_alpha,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| NexError::new(format!("json: {e}")))?
    );
    Ok(())
}
