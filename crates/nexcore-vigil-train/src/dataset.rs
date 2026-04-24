//! JSONL dataset readers for SFT and DPO training.
//!
//! Two shapes are read:
//! - **SFT row**: `{messages: [{role, content}, ...], weight?, source?, tags?}`
//!   The OpenAI chat format. Used for `train` (SFT) and `eval` prompts.
//! - **DPO row**: `{prompt, system?, chosen, rejected, margin?, mutation_type?}`
//!   Produced by `vigil-lora-train dpo-build`. Used for `dpo-train`.

use nexcore_error::{NexError, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A single message in an OpenAI-style chat row.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// An SFT training row in OpenAI chat format.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SftRow {
    pub messages: Vec<Message>,
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A DPO preference-pair row from `vigil-lora-train dpo-build`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DpoRow {
    pub prompt: String,
    #[serde(default)]
    pub system: String,
    pub chosen: String,
    pub rejected: String,
    #[serde(default)]
    pub chosen_score: f32,
    #[serde(default)]
    pub rejected_score: f32,
    #[serde(default)]
    pub margin: f32,
    #[serde(default)]
    pub mutation_type: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

fn default_weight() -> f32 {
    1.0
}

impl SftRow {
    /// Extract the system prompt, user prompt, and assistant response.
    /// Returns an error if any of the three roles is missing.
    pub fn split(&self) -> Result<(&str, &str, &str)> {
        let mut system = "";
        let mut user = "";
        let mut assistant = "";
        for m in &self.messages {
            match m.role.as_str() {
                "system" => system = &m.content,
                "user" => user = &m.content,
                "assistant" => assistant = &m.content,
                _ => {}
            }
        }
        if user.is_empty() {
            bail!("row missing user message");
        }
        if assistant.is_empty() {
            bail!("row missing assistant message");
        }
        Ok((system, user, assistant))
    }
}

/// Read every line of a JSONL file and parse into `T`.
/// Blank lines and lines starting with `#` are skipped.
pub fn read_jsonl<T, P>(path: P) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    let p = path.as_ref();
    let f = File::open(p).map_err(|e| NexError::new(format!("open {}: {e}", p.display())))?;
    let reader = BufReader::new(f);
    let mut rows = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| NexError::new(format!("read line {i}: {e}")))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let row: T = serde_json::from_str(trimmed)
            .map_err(|e| NexError::new(format!("parse line {i}: {e}")))?;
        rows.push(row);
    }
    Ok(rows)
}

/// Convenience: read SFT rows from the canonical path.
pub fn read_sft<P: AsRef<Path>>(path: P) -> Result<Vec<SftRow>> {
    read_jsonl(path)
}

/// Convenience: read DPO rows from the canonical path.
pub fn read_dpo<P: AsRef<Path>>(path: P) -> Result<Vec<DpoRow>> {
    read_jsonl(path)
}

// ─────────────────────────────────────────────────────────────────────────────
// Primitive stratification — leverages T1 primitive tags on micrograms.
//
// The training set is brutally imbalanced across the 15 operational Lex
// Primitiva. Measured 2026-04-17: κ=208, σ=35, →=18, ∂=15, ς=7, ...,
// ∝=∅=1. Standard uniform batch sampling would drown the rare primitives.
//
// Two strategies are exposed:
//   - `Stratification::Uniform`: every row equally likely. Baseline.
//   - `Stratification::InverseFrequency`: row weight ∝ 1 / freq(primitive).
//     Rare primitives get 30× the sampling weight of κ. The model sees every
//     primitive at roughly equal rate.
// ─────────────────────────────────────────────────────────────────────────────

/// Canonical set of operational T1 primitives we stratify over.
/// These match the single-character tags emitted by vigil-dataset-builder.
pub const PRIMITIVE_TAGS: &[&str] = &[
    "∃", "ς", "∂", "→", "σ", "Σ", "ν", "N", "λ", "∝", "∅", "ρ", "κ", "π", "μ", "×",
];

/// Sampling strategy across primitive classes.
#[derive(Clone, Copy, Debug)]
pub enum Stratification {
    Uniform,
    InverseFrequency,
}

/// Extract the dominant primitive tag from a row's `tags` field, if any.
pub fn dominant_primitive(row: &SftRow) -> Option<&str> {
    row.tags
        .iter()
        .find(|t| PRIMITIVE_TAGS.contains(&t.as_str()))
        .map(String::as_str)
}

/// Build per-row sampling weights using the chosen stratification.
/// Output length matches `rows.len()`. Weights sum to `rows.len()` so the
/// "effective batch count" is preserved.
pub fn compute_weights(rows: &[SftRow], strategy: Stratification) -> Vec<f32> {
    let n = rows.len();
    match strategy {
        Stratification::Uniform => vec![1.0; n],
        Stratification::InverseFrequency => {
            use std::collections::HashMap;
            let mut freq: HashMap<&str, usize> = HashMap::new();
            let mut unlabeled = 0usize;
            for r in rows {
                match dominant_primitive(r) {
                    Some(p) => *freq.entry(p).or_insert(0) += 1,
                    None => unlabeled += 1,
                }
            }
            // Raw weights: 1 / freq for labeled, 1 / unlabeled for the rest.
            let mut raw: Vec<f32> = rows
                .iter()
                .map(|r| match dominant_primitive(r) {
                    Some(p) => 1.0 / (*freq.get(p).unwrap_or(&1) as f32),
                    None => {
                        if unlabeled > 0 {
                            1.0 / unlabeled as f32
                        } else {
                            0.0
                        }
                    }
                })
                .collect();
            // Normalize so sum == n (preserve batch-count semantics).
            let sum: f32 = raw.iter().sum();
            if sum > 0.0 {
                let scale = n as f32 / sum;
                for w in &mut raw {
                    *w *= scale;
                }
            }
            raw
        }
    }
}

/// Report per-primitive counts + effective-batch weight sum. Useful in logs
/// before training starts so the stratification is auditable.
#[derive(Debug)]
pub struct StratificationReport {
    pub counts: Vec<(String, usize)>,
    pub unlabeled: usize,
    pub min_weight: f32,
    pub max_weight: f32,
    pub max_over_min: f32,
}

pub fn report(rows: &[SftRow], strategy: Stratification) -> StratificationReport {
    use std::collections::HashMap;
    let mut freq: HashMap<String, usize> = HashMap::new();
    let mut unlabeled = 0usize;
    for r in rows {
        match dominant_primitive(r) {
            Some(p) => *freq.entry(p.to_string()).or_insert(0) += 1,
            None => unlabeled += 1,
        }
    }
    let mut counts: Vec<(String, usize)> = freq.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    let weights = compute_weights(rows, strategy);
    let min_weight = weights.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_weight = weights.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let max_over_min = if min_weight > 0.0 {
        max_weight / min_weight
    } else {
        f32::INFINITY
    };
    StratificationReport {
        counts,
        unlabeled,
        min_weight,
        max_weight,
        max_over_min,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn row(tag: &str) -> SftRow {
        SftRow {
            messages: vec![
                Message {
                    role: "user".into(),
                    content: "u".into(),
                },
                Message {
                    role: "assistant".into(),
                    content: "a".into(),
                },
            ],
            weight: 1.0,
            source: None,
            tags: vec!["microgram".into(), tag.into()],
        }
    }

    #[test]
    fn uniform_strategy_is_uniform() {
        let rows = vec![row("κ"), row("κ"), row("∅")];
        let w = compute_weights(&rows, Stratification::Uniform);
        assert_eq!(w, vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn inverse_frequency_upweights_rare() {
        // 2 × κ, 1 × ∅. Expect ∅ weight ≈ 2× each κ weight.
        let rows = vec![row("κ"), row("κ"), row("∅")];
        let w = compute_weights(&rows, Stratification::InverseFrequency);
        assert!(w[2] > w[0] * 1.8, "rare should outweigh common: {:?}", w);
        // sum preserved
        let sum: f32 = w.iter().sum();
        assert!((sum - 3.0).abs() < 1e-4, "sum should be 3, got {sum}");
    }

    #[test]
    fn real_distribution_matches_imbalance() {
        // Mirror measured (2026-04-17): κ=208, σ=35, ∅=1.
        let mut rows = Vec::new();
        for _ in 0..208 {
            rows.push(row("κ"));
        }
        for _ in 0..35 {
            rows.push(row("σ"));
        }
        rows.push(row("∅"));
        let w = compute_weights(&rows, Stratification::InverseFrequency);
        let kappa_w = w[0];
        let void_w = *w.last().unwrap();
        let ratio = void_w / kappa_w;
        // ∅ should get ~208× the weight of κ (inverse of frequency ratio).
        assert!(
            ratio > 200.0 && ratio < 216.0,
            "expected ~208x, got {ratio}x"
        );
    }
}
