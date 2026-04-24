//! Generation loop for quantized Qwen2. Greedy + temperature + top-p sampling.
//!
//! Phase R1 scope: correctness over speed. We run the full forward pass once
//! per token and rely on Candle's KV cache internally.

use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_qwen2::ModelWeights as QuantQwen2;
use candle_transformers::models::qwen2::ModelForCausalLM as NativeQwen2;
use nexcore_error::{NexError, Result};
use tokenizers::Tokenizer;

use crate::tokenizer;

/// Sampling knobs.
#[derive(Clone, Debug)]
pub struct SamplingConfig {
    pub temperature: f64,
    pub top_p: Option<f64>,
    pub seed: u64,
    pub max_new_tokens: usize,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: Some(0.9),
            seed: 42,
            max_new_tokens: 1024,
        }
    }
}

/// Stop token IDs for Qwen2.5 ChatML — "<|im_end|>" and EOS.
fn stop_tokens(tok: &Tokenizer) -> Vec<u32> {
    let mut v = Vec::new();
    for marker in ["<|im_end|>", "<|endoftext|>"] {
        if let Some(id) = tok.token_to_id(marker) {
            v.push(id);
        }
    }
    v
}

/// Run a single generation from a ChatML-formatted prompt. Returns the decoded
/// assistant text (excluding the prompt) and the number of tokens generated.
pub fn generate(
    model: &mut QuantQwen2,
    tok: &Tokenizer,
    device: &Device,
    prompt: &str,
    cfg: &SamplingConfig,
) -> Result<(String, usize)> {
    let prompt_ids = tokenizer::encode(tok, prompt)?;
    let stops = stop_tokens(tok);
    let mut logits_processor = LogitsProcessor::new(cfg.seed, Some(cfg.temperature), cfg.top_p);

    let mut all_ids = prompt_ids.clone();
    let mut output_ids: Vec<u32> = Vec::new();

    // Prefill pass — feed the whole prompt at position 0.
    let input = Tensor::new(all_ids.as_slice(), device)
        .map_err(|e| NexError::new(format!("prefill tensor: {e}")))?
        .unsqueeze(0)
        .map_err(|e| NexError::new(format!("prefill unsqueeze: {e}")))?;
    let mut logits = model
        .forward(&input, 0)
        .map_err(|e| NexError::new(format!("prefill forward: {e}")))?;
    logits = logits
        .squeeze(0)
        .and_then(|t| t.squeeze(0))
        .map_err(|e| NexError::new(format!("squeeze prefill: {e}")))?;

    // Decode loop — one token at a time from prompt length onward.
    for step in 0..cfg.max_new_tokens {
        let next_id = logits_processor
            .sample(&logits)
            .map_err(|e| NexError::new(format!("sample step {step}: {e}")))?;
        if stops.contains(&next_id) {
            break;
        }
        output_ids.push(next_id);
        all_ids.push(next_id);

        let pos = all_ids.len() - 1;
        let single = Tensor::new(&[next_id], device)
            .map_err(|e| NexError::new(format!("step {step} tensor: {e}")))?
            .unsqueeze(0)
            .map_err(|e| NexError::new(format!("step {step} unsqueeze: {e}")))?;
        logits = model
            .forward(&single, pos)
            .map_err(|e| NexError::new(format!("step {step} forward: {e}")))?;
        logits = logits
            .squeeze(0)
            .and_then(|t| t.squeeze(0))
            .map_err(|e| NexError::new(format!("step {step} squeeze: {e}")))?;
    }

    let text = tokenizer::decode(tok, &output_ids)?;
    Ok((text, output_ids.len()))
}

/// Clear the KV cache between REPL turns. Each turn in a REPL is an
/// independent request — without this, the cache from the previous turn's
/// prompt carries forward and produces shape mismatches on the next turn.
pub fn clear_native_cache(model: &mut NativeQwen2) {
    model.clear_kv_cache();
}

/// Native (fp16/bf16) generation. Mirrors `generate` for the non-quantized
/// Qwen2 model. Used by the `infer-native` CLI command.
pub fn generate_native(
    model: &mut NativeQwen2,
    tok: &Tokenizer,
    device: &Device,
    prompt: &str,
    cfg: &SamplingConfig,
) -> Result<(String, usize)> {
    let prompt_ids = tokenizer::encode(tok, prompt)?;
    let stops = stop_tokens(tok);
    let mut logits_processor = LogitsProcessor::new(cfg.seed, Some(cfg.temperature), cfg.top_p);

    let mut all_ids = prompt_ids.clone();
    let mut output_ids: Vec<u32> = Vec::new();

    let input = Tensor::new(all_ids.as_slice(), device)
        .map_err(|e| NexError::new(format!("prefill tensor: {e}")))?
        .unsqueeze(0)
        .map_err(|e| NexError::new(format!("prefill unsqueeze: {e}")))?;
    let mut logits = model
        .forward(&input, 0)
        .map_err(|e| NexError::new(format!("prefill forward: {e}")))?;
    logits = logits
        .squeeze(0)
        .and_then(|t| t.squeeze(0))
        .map_err(|e| NexError::new(format!("squeeze prefill: {e}")))?;

    for step in 0..cfg.max_new_tokens {
        let next_id = logits_processor
            .sample(&logits)
            .map_err(|e| NexError::new(format!("sample step {step}: {e}")))?;
        if stops.contains(&next_id) {
            break;
        }
        output_ids.push(next_id);
        all_ids.push(next_id);

        let pos = all_ids.len() - 1;
        let single = Tensor::new(&[next_id], device)
            .map_err(|e| NexError::new(format!("step {step} tensor: {e}")))?
            .unsqueeze(0)
            .map_err(|e| NexError::new(format!("step {step} unsqueeze: {e}")))?;
        logits = model
            .forward(&single, pos)
            .map_err(|e| NexError::new(format!("step {step} forward: {e}")))?;
        logits = logits
            .squeeze(0)
            .and_then(|t| t.squeeze(0))
            .map_err(|e| NexError::new(format!("step {step} squeeze: {e}")))?;
    }

    let text = tokenizer::decode(tok, &output_ids)?;
    Ok((text, output_ids.len()))
}
