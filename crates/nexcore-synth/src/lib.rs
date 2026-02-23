//! # NexVigilant Core — synth — Autonomous Primitive Synthesis
//!
//! Implements the Level 5 Evolution loop:
//! 1. **Analyze** statistical drift (antitransformer)
//! 2. **Synthesize** structural schema (transcriptase)
//! 3. **Compose** new T1/T2 primitives (lex-primitiva)
//!
//! ## Evolution Loop (ρ-synth)
//!
//! ```text
//! Statistical Drift (ν) → Structural Inference (μ) → Primitive Synthesis (Σ)
//! ```
//!
//! Tier: T3 | Grounding: ρ (Recursion) + Σ (Sum) + κ (Comparison)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use antitransformer::pipeline::{self, AnalysisConfig};
use nexcore_lex_primitiva::prelude::*;
use nexcore_transcriptase as transcriptase;
use serde::{Deserialize, Serialize};
use core::fmt;

pub mod grounding;

/// Errors during self-synthesis operations.
#[derive(Debug)]
pub enum SynthError {
    Analysis(String),
    Transcription(transcriptase::TranscriptaseError),
    Synthesis(SynthesisError),
    LowNovelty(f64),
}

impl fmt::Display for SynthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Analysis(msg) => write!(f, "Statistical analysis failed: {}", msg),
            Self::Transcription(err) => write!(f, "Schema synthesis failed: {}", err),
            Self::Synthesis(err) => write!(f, "Reverse synthesis failed: {}", err),
            Self::LowNovelty(score) => write!(f, "Insufficient novelty detected (score: {})", score),
        }
    }
}

impl std::error::Error for SynthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transcription(err) => Some(err),
            Self::Synthesis(err) => Some(err),
            _ => None,
        }
    }
}

impl From<transcriptase::TranscriptaseError> for SynthError {
    fn from(err: transcriptase::TranscriptaseError) -> Self {
        Self::Transcription(err)
    }
}

impl From<SynthesisError> for SynthError {
    fn from(err: SynthesisError) -> Self {
        Self::Synthesis(err)
    }
}

/// A newly synthesized primitive candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthCandidate {
    pub id: String,
    pub name: String,
    pub tier: Tier,
    pub composition: Vec<String>,
    pub dominant_primitive: String,
    pub confidence: f64,
    pub derivation_path: String,
}

/// The Self-Synth Engine.
pub struct SynthEngine {
    rev_synth: RevSynthesizer,
    analysis_config: AnalysisConfig,
}

impl Default for SynthEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SynthEngine {
    /// Create a new synth engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rev_synth: RevSynthesizer::new(),
            analysis_config: AnalysisConfig::default(),
        }
    }

    /// Run the full evolution loop on a raw input sample.
    pub fn evolve(
        &self,
        sample_text: &str,
        sample_data: &serde_json::Value,
    ) -> Result<SynthCandidate, SynthError> {
        // 1. Analyze statistical fingerprint (ν)
        let analysis = pipeline::analyze(sample_text, &self.analysis_config);

        // 2. Infer structural schema (μ)
        let schema = transcriptase::infer(sample_data);

        // 3. Map features to T1 primitives (κ)
        let primitives = self.map_to_primitives(&analysis, &schema);

        // 4. Reverse synthesize candidate (Σ)
        let synth_result = self
            .rev_synth
            .synthesize(primitives, SynthesisOpts::default())?;

        let id = nexcore_id::NexId::v4().to_string();
        let name = format!("Synth-{}", &id[..8]);

        Ok(SynthCandidate {
            id,
            name,
            tier: synth_result.tier,
            composition: synth_result
                .composition
                .unique()
                .iter()
                .map(|p| p.name().to_string())
                .collect(),
            dominant_primitive: synth_result
                .dominant
                .map(|p| p.name().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            confidence: synth_result.coherence,
            derivation_path: format!(
                "{} → {} → {}",
                analysis.verdict, schema.observations, synth_result.tier
            ),
        })
    }

    fn map_to_primitives(
        &self,
        analysis: &pipeline::AnalysisResult,
        schema: &transcriptase::Schema,
    ) -> Vec<LexPrimitiva> {
        let mut prims = Vec::new();

        // Statistical mapping
        if analysis.features.entropy_std < 0.1 {
            prims.push(LexPrimitiva::State); // Uniform entropy = stable state
        }
        if analysis.features.zipf_deviation > 0.5 {
            prims.push(LexPrimitiva::Boundary); // High deviation = boundary crossing
        }

        // Structural mapping
        match schema.kind {
            transcriptase::SchemaKind::Record(_) => prims.push(LexPrimitiva::Mapping),
            transcriptase::SchemaKind::Array { .. } => prims.push(LexPrimitiva::Sequence),
            transcriptase::SchemaKind::Int { .. } | transcriptase::SchemaKind::Float { .. } => {
                prims.push(LexPrimitiva::Quantity)
            }
            _ => prims.push(LexPrimitiva::Void),
        }

        // Mandatory grounding
        prims.push(LexPrimitiva::Existence);

        prims
    }
}
