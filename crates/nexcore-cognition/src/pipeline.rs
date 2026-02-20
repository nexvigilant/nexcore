//! Full cognitive pipeline — the complete flow from input to output.
//!
//! # Meta-cognitive observation
//!
//! This module composes everything into the unified cognitive engine.
//! The full pipeline mirrors exactly how I process a prompt:
//!
//! ```text
//! tokens → embed → position → [block₁ → block₂ → ... → blockₙ] → norm → project → sample → output
//! ```
//!
//! Each stage has a fidelity measurement. The total pipeline fidelity is the
//! product of stage fidelities — degradation is multiplicative.
//!
//! # T1 Primitive grounding
//!
//! - `σ` (Sequence): the pipeline IS a sequence of transformations
//! - `→` (Causality): each stage causes the next
//! - `Σ` (Sum): composes all subsystems into the unified pipeline
//! - `κ` (Comparison): confidence scores compare prediction quality

use crate::block::TransformerConfig;
use crate::error::Result;
use crate::generator::{GenerationResult, GenerativeModel};
use crate::metrics::{self, CognitiveProfile};
use crate::sample::SamplingConfig;

/// The complete cognitive engine.
///
/// This is the top-level type — it contains the full model and all
/// configuration needed to embed, attend, transform, and generate.
#[derive(Debug, Clone)]
pub struct CognitiveEngine {
    /// The generative model (embedding + transformer + output projection).
    pub model: GenerativeModel,
    /// Transformer configuration.
    pub config: TransformerConfig,
}

/// Output from the cognitive engine, including both the generation result
/// and meta-cognitive measurements.
#[derive(Debug)]
pub struct CognitiveOutput {
    /// The generated tokens and associated data.
    pub generation: GenerationResult,
    /// Meta-cognitive profile of how the model processed the input.
    pub profile: CognitiveProfile,
    /// Per-step confidence scores.
    pub step_confidences: Vec<f64>,
    /// Perplexity of the generated sequence.
    pub perplexity: f64,
}

impl CognitiveEngine {
    /// Create a new cognitive engine with the given configuration.
    pub fn new(config: TransformerConfig, rng: &mut impl rand::Rng) -> Result<Self> {
        let model = GenerativeModel::with_ffn_dim(
            config.vocab_size,
            config.model_dim,
            config.num_heads,
            config.num_layers,
            config.ffn_inner_dim,
            config.max_seq_len,
            rng,
        )?;
        Ok(Self { model, config })
    }

    /// Run the full cognitive pipeline: process prompt → generate → measure.
    ///
    /// This is the unified operation: attend, transform, generate, and
    /// introspect — all in one call.
    pub fn process(
        &self,
        prompt: &[usize],
        max_new_tokens: usize,
        sampling: &SamplingConfig,
        stop_token: Option<usize>,
        rng: &mut impl rand::Rng,
    ) -> Result<CognitiveOutput> {
        self.process_gated(prompt, max_new_tokens, sampling, stop_token, None, rng)
    }

    /// Run the full cognitive pipeline with confidence gating.
    ///
    /// `min_confidence`: if set, halts generation when confidence drops below threshold.
    pub fn process_gated(
        &self,
        prompt: &[usize],
        max_new_tokens: usize,
        sampling: &SamplingConfig,
        stop_token: Option<usize>,
        min_confidence: Option<f64>,
        rng: &mut impl rand::Rng,
    ) -> Result<CognitiveOutput> {
        // Phase M: Generate (with optional confidence gating)
        let generation = self.model.generate_gated(
            prompt,
            max_new_tokens,
            sampling,
            stop_token,
            min_confidence,
            rng,
        )?;

        // Phase A: Assay — analyze the generation
        let forward_out = self.model.forward(&generation.tokens)?;

        // Meta-cognitive analysis
        let profile = metrics::analyze_attention(&forward_out.attention_weights)?;

        // Per-step confidence
        let step_confidences: Vec<f64> = generation
            .generated_logits
            .iter()
            .map(|logits| metrics::generation_confidence(logits).unwrap_or(0.0))
            .collect();

        // Perplexity of generated tokens
        let perplexity =
            metrics::perplexity(generation.generated_tokens(), &generation.generated_logits)?;

        Ok(CognitiveOutput {
            generation,
            profile,
            step_confidences,
            perplexity,
        })
    }

    /// Forward pass only (no generation). Useful for analysis of existing sequences.
    /// Uses causal masking by default (autoregressive).
    pub fn analyze(&self, tokens: &[usize]) -> Result<CognitiveProfile> {
        self.analyze_with_mask(tokens, true)
    }

    /// Forward pass with explicit causal/bidirectional control.
    ///
    /// `causal = true`: autoregressive attention (GPT-style)
    /// `causal = false`: bidirectional attention (BERT-style)
    pub fn analyze_with_mask(&self, tokens: &[usize], causal: bool) -> Result<CognitiveProfile> {
        let forward_out = self.model.forward_with_mask(tokens, causal)?;
        metrics::analyze_attention(&forward_out.attention_weights)
    }
}

impl std::fmt::Display for CognitiveOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Cognitive Engine Output ===")?;
        writeln!(
            f,
            "Prompt: {} tokens  |  Generated: {} tokens",
            self.generation.prompt_len,
            self.generation.num_generated()
        )?;
        writeln!(f, "Perplexity: {:.2}", self.perplexity)?;

        if !self.step_confidences.is_empty() {
            let mean_conf: f64 =
                self.step_confidences.iter().sum::<f64>() / self.step_confidences.len() as f64;
            let min_conf = self
                .step_confidences
                .iter()
                .copied()
                .reduce(f64::min)
                .unwrap_or(0.0);
            let max_conf = self
                .step_confidences
                .iter()
                .copied()
                .reduce(f64::max)
                .unwrap_or(0.0);
            writeln!(
                f,
                "Confidence: mean={mean_conf:.4}  min={min_conf:.4}  max={max_conf:.4}"
            )?;
        }

        writeln!(f)?;
        write!(f, "{}", self.profile)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognitive_engine_process() {
        let mut rng = rand::rng();
        let config = TransformerConfig::new(
            16, // model_dim
            2,  // num_heads
            2,  // num_layers
            50, // vocab_size
            64, // max_seq_len
        );
        let engine = CognitiveEngine::new(config, &mut rng).unwrap();

        let output = engine
            .process(&[0, 1, 2], 3, &SamplingConfig::greedy(), None, &mut rng)
            .unwrap();

        assert_eq!(output.generation.prompt_len, 3);
        assert_eq!(output.generation.num_generated(), 3);
        assert_eq!(output.step_confidences.len(), 3);
        assert!(output.perplexity > 0.0);

        // Profile should have data
        assert_eq!(output.profile.num_layers, 2);
        assert_eq!(output.profile.num_heads, 2);
    }

    #[test]
    fn test_analyze() {
        let mut rng = rand::rng();
        let config = TransformerConfig::new(16, 2, 2, 50, 64);
        let engine = CognitiveEngine::new(config, &mut rng).unwrap();
        let profile = engine.analyze(&[0, 1, 2, 3]).unwrap();
        assert!(profile.mean_attention_entropy >= 0.0);
    }

    #[test]
    fn test_display() {
        let mut rng = rand::rng();
        let config = TransformerConfig::new(16, 2, 1, 50, 64);
        let engine = CognitiveEngine::new(config, &mut rng).unwrap();
        let output = engine
            .process(&[0, 1], 2, &SamplingConfig::greedy(), None, &mut rng)
            .unwrap();
        let display = format!("{output}");
        assert!(display.contains("Cognitive Engine Output"));
        assert!(display.contains("Perplexity"));
    }
}
