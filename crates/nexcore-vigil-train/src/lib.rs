//! Full-Rust sovereign LoRA training pipeline for Vigil.
//!
//! Replaces the Python (transformers + peft + trl) stack end-to-end with
//! Candle-based Rust. Zero Python, zero llama.cpp (once R4 lands).
//!
//! Five phases:
//! - **R1** (shipped): [`inference`] + [`eval`] from GGUF via [`model::VigilModel::Quantized`].
//! - **R2**: [`sft`] — LoRA fine-tune on [`dataset::SftRow`].
//! - **R3**: [`dpo`] — preference-pair tune on [`dataset::DpoRow`].
//! - **R4**: [`merge`] + [`serve`] — adapter fold + Ollama-compat HTTP.
//! - **R5**: nexcore-mcp tool wiring (external, in nexcore-mcp/src/tools/).
//!
//! Discipline: no unwrap, no expect, no panic, no unsafe, no Python.
//! Errors route through [`nexcore_error::NexError`].

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod dataset;
pub mod dpo;
pub mod eval;
pub mod inference;
pub mod lora;
pub mod merge;
pub mod model;
pub mod qwen2_lora;
pub mod serve;
pub mod sft;
pub mod tokenizer;

pub use nexcore_error::{NexError, Result};
