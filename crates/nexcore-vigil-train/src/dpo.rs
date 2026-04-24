//! DPO training loop — Phase R3 (stubbed, interface sketched).
//!
//! When implemented, this module will:
//! - Load base + SFT adapter as the reference policy.
//! - Attach a fresh LoRA on top as the training policy.
//! - For each [`crate::dataset::DpoRow`], compute log-probabilities of `chosen`
//!   and `rejected` under both policy and reference (reference via
//!   `adapter.disable()` + second forward).
//! - Compute DPO loss:
//!     `-log σ(β · ((π_chosen - π_rejected) - (ref_chosen - ref_rejected)))`
//! - Backprop through policy LoRA only.
//!
//! Reference: Rafailov et al. 2023, "Direct Preference Optimization".

use nexcore_error::{NexError, Result};
use std::path::PathBuf;

/// DPO configuration knobs.
#[derive(Clone, Debug)]
pub struct DpoConfig {
    pub base_model: String,
    pub sft_adapter: PathBuf,
    pub dpo_jsonl: PathBuf,
    pub output_dir: PathBuf,
    pub max_steps: u32,
    pub lr: f64,
    pub beta: f32,
    pub lora_r: u32,
    pub lora_alpha: u32,
    pub batch_size: u32,
    pub grad_accum: u32,
    pub max_prompt_len: u32,
    pub max_len: u32,
    pub seed: u64,
}

impl Default for DpoConfig {
    fn default() -> Self {
        Self {
            base_model: "Qwen/Qwen2.5-Coder-1.5B-Instruct".into(),
            sft_adapter: PathBuf::from("/home/matthew/.claude/brain/vigil-lora-v1"),
            dpo_jsonl: PathBuf::from("/home/matthew/.claude/brain/vigil-dpo-dataset.jsonl"),
            output_dir: PathBuf::from("/home/matthew/.claude/brain/vigil-lora-dpo-v1"),
            max_steps: 200,
            lr: 5e-5,
            beta: 0.1,
            lora_r: 8,
            lora_alpha: 16,
            batch_size: 1,
            grad_accum: 8,
            max_prompt_len: 1024,
            max_len: 2048,
            seed: 42,
        }
    }
}

pub fn run(_cfg: &DpoConfig) -> Result<()> {
    Err(NexError::new(
        "dpo::run not implemented yet — Phase R3. \
         Depends on Phase R2 (SFT trainer). \
         Additional complexity: reference-model log-prob computation via \
         adapter enable/disable, numerically-stable sigmoid, gradient \
         isolation to policy LoRA only. \
         Est. 1200 LOC, 5 days focused work.",
    ))
}
