//! Tokenizer loading for Qwen2.5 family models.
//!
//! We prefer loading tokenizer.json from a local path. For convenience we also
//! search a small set of standard locations (HF cache, Ollama model dir if it
//! bundles one). We do NOT download at runtime — per sovereignty posture, all
//! model artifacts are prepositioned on disk.

use nexcore_error::{NexError, Result};
use std::path::{Path, PathBuf};
use tokenizers::Tokenizer;

/// Locations to probe when `--tokenizer` is not given explicitly.
fn default_candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        let h = PathBuf::from(home);
        // Common HF cache layout
        v.push(
            h.join(".cache/huggingface/hub/models--Qwen--Qwen2.5-1.5B-Instruct/snapshots")
                .join("current/tokenizer.json"),
        );
        v.push(
            h.join(".cache/huggingface/hub/models--Qwen--Qwen2.5-Coder-1.5B-Instruct/snapshots")
                .join("current/tokenizer.json"),
        );
        // Brain-managed cache (post-download)
        v.push(h.join(".claude/brain/vigil-tokenizer.json"));
    }
    v
}

/// Load a tokenizer from an explicit path, falling back to a search of
/// sensible locations if `path` is `None`.
pub fn load(path: Option<&Path>) -> Result<Tokenizer> {
    let probe = match path {
        Some(p) => vec![p.to_path_buf()],
        None => default_candidates(),
    };
    for candidate in &probe {
        if candidate.is_file() {
            let tok = Tokenizer::from_file(candidate).map_err(|e| {
                NexError::new(format!("tokenizer load {}: {e}", candidate.display()))
            })?;
            tracing::info!("tokenizer loaded: {}", candidate.display());
            return Ok(tok);
        }
    }
    Err(NexError::new(format!(
        "no tokenizer.json found. Checked: {}. Download with: huggingface-cli download Qwen/Qwen2.5-1.5B-Instruct tokenizer.json --local-dir ~/.claude/brain/",
        probe
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

/// Encode a prompt and return token ids as a `Vec<u32>`.
pub fn encode(tokenizer: &Tokenizer, text: &str) -> Result<Vec<u32>> {
    let enc = tokenizer
        .encode(text, true)
        .map_err(|e| NexError::new(format!("encode: {e}")))?;
    Ok(enc.get_ids().to_vec())
}

/// Decode token ids back to text, skipping special tokens.
pub fn decode(tokenizer: &Tokenizer, ids: &[u32]) -> Result<String> {
    tokenizer
        .decode(ids, true)
        .map_err(|e| NexError::new(format!("decode: {e}")))
}

/// Build a Qwen2.5 ChatML prompt string.
/// Format:
///   <|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n
pub fn format_chatml(system: &str, user: &str) -> String {
    let mut s = String::new();
    if !system.is_empty() {
        s.push_str("<|im_start|>system\n");
        s.push_str(system);
        s.push_str("<|im_end|>\n");
    }
    s.push_str("<|im_start|>user\n");
    s.push_str(user);
    s.push_str("<|im_end|>\n<|im_start|>assistant\n");
    s
}
