//! Prompt Kinetics MCP tools — pharmacokinetic model for prompts.
//!
//! ADME model: Absorption, Distribution, Metabolism, Elimination.
//! Treats prompts as "drugs" entering the model's "system".
//!
//! ## T1 Primitive Grounding
//! - Absorption: μ(Mapping) — input→context
//! - Distribution: λ(Location) — across attention heads
//! - Metabolism: →(Causality) — reasoning/transformation
//! - Elimination: ∝(Irreversibility) — context eviction

use crate::params::prompt_kinetics::{
    PromptBioavailabilityParams, PromptKineticsAnalyzeParams, PromptKineticsModelParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Analyze a prompt through the ADME model.
pub fn analyze(params: PromptKineticsAnalyzeParams) -> Result<CallToolResult, McpError> {
    let prompt = &params.prompt;
    let target = params.target_model.as_deref().unwrap_or("claude-opus-4-6");

    let tokens_est = estimate_tokens(prompt);
    let absorption = analyze_absorption(prompt);
    let distribution = analyze_distribution(prompt);
    let metabolism = analyze_metabolism(prompt);
    let elimination = analyze_elimination(prompt, tokens_est);

    let overall_bioavailability = absorption.score * distribution.score * metabolism.score;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "prompt_length": prompt.len(),
            "estimated_tokens": tokens_est,
            "target_model": target,
            "bioavailability": (overall_bioavailability * 100.0).round() / 100.0,
            "adme": {
                "absorption": {
                    "score": absorption.score,
                    "factors": absorption.factors,
                },
                "distribution": {
                    "score": distribution.score,
                    "factors": distribution.factors,
                },
                "metabolism": {
                    "score": metabolism.score,
                    "factors": metabolism.factors,
                },
                "elimination": {
                    "score": elimination.score,
                    "factors": elimination.factors,
                },
            },
            "recommendations": generate_recommendations(
                &absorption, &distribution, &metabolism, &elimination
            ),
        })
        .to_string(),
    )]))
}

/// Compute bioavailability of a prompt.
pub fn bioavailability(params: PromptBioavailabilityParams) -> Result<CallToolResult, McpError> {
    let prompt = &params.prompt;
    let tokens = estimate_tokens(prompt);
    let expected_output = params.expected_output_tokens.unwrap_or(1000);

    let information_density = count_unique_concepts(prompt) as f64 / tokens.max(1) as f64;
    let clarity = score_clarity(prompt);
    let specificity = score_specificity(prompt);

    let bioavailability =
        (information_density.min(1.0) * 0.3 + clarity * 0.4 + specificity * 0.3).min(1.0);
    let therapeutic_index = if expected_output > 0 {
        tokens as f64 / expected_output as f64
    } else {
        0.0
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "prompt_tokens": tokens,
            "expected_output_tokens": expected_output,
            "bioavailability": (bioavailability * 100.0).round() / 100.0,
            "information_density": (information_density * 1000.0).round() / 1000.0,
            "clarity": (clarity * 100.0).round() / 100.0,
            "specificity": (specificity * 100.0).round() / 100.0,
            "therapeutic_index": (therapeutic_index * 100.0).round() / 100.0,
            "efficiency_note": if therapeutic_index < 0.5 {
                "Good: small input → large output"
            } else if therapeutic_index < 2.0 {
                "Moderate: balanced I/O ratio"
            } else {
                "Consider: high input for expected output"
            },
        })
        .to_string(),
    )]))
}

/// Get PK model parameter definitions.
pub fn model(_params: PromptKineticsModelParams) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "model": "ADME Pharmacokinetics for Prompts",
            "phases": {
                "A_absorption": {
                    "analog": "Drug entering bloodstream",
                    "prompt_meaning": "How well the prompt is parsed and understood",
                    "factors": ["clarity", "structure", "unambiguity", "formatting"],
                    "primitive": "μ(Mapping)",
                },
                "D_distribution": {
                    "analog": "Drug reaching target tissues",
                    "prompt_meaning": "How information distributes across attention",
                    "factors": ["focus", "coherence", "context relevance", "recency"],
                    "primitive": "λ(Location)",
                },
                "M_metabolism": {
                    "analog": "Drug being processed by liver",
                    "prompt_meaning": "How the model transforms prompt into reasoning",
                    "factors": ["actionability", "constraint clarity", "examples present"],
                    "primitive": "→(Causality)",
                },
                "E_elimination": {
                    "analog": "Drug clearance from body",
                    "prompt_meaning": "How quickly context is lost/evicted",
                    "factors": ["token count vs window", "repetition", "compression potential"],
                    "primitive": "∝(Irreversibility)",
                },
            },
            "bioavailability": "Product of A × D × M scores (0.0-1.0)",
            "therapeutic_index": "Input tokens / Expected output tokens (lower = more efficient)",
        })
        .to_string(),
    )]))
}

struct PhaseResult {
    score: f64,
    factors: Vec<serde_json::Value>,
}

fn analyze_absorption(prompt: &str) -> PhaseResult {
    let mut score = 0.7f64; // baseline
    let mut factors = Vec::new();

    // Clear structure (headers, bullets, numbered items)
    let has_structure = prompt.contains("##") || prompt.contains("- ") || prompt.contains("1.");
    if has_structure {
        score += 0.1;
        factors.push(json!({"factor": "structured_formatting", "effect": "+0.1"}));
    }

    // Code blocks
    let has_code = prompt.contains("```") || prompt.contains("    ");
    if has_code {
        score += 0.05;
        factors.push(json!({"factor": "code_blocks", "effect": "+0.05"}));
    }

    // Ambiguity markers
    let has_ambiguity =
        prompt.contains("maybe") || prompt.contains("possibly") || prompt.contains("might");
    if has_ambiguity {
        score -= 0.1;
        factors.push(json!({"factor": "ambiguous_language", "effect": "-0.1"}));
    }

    // Very long prompts have diminishing absorption
    let tokens = estimate_tokens(prompt);
    if tokens > 2000 {
        score -= 0.05;
        factors.push(json!({"factor": "length_penalty", "tokens": tokens, "effect": "-0.05"}));
    }
    if tokens > 5000 {
        score -= 0.1;
        factors.push(json!({"factor": "excessive_length", "tokens": tokens, "effect": "-0.1"}));
    }

    PhaseResult {
        score: score.clamp(0.0, 1.0),
        factors,
    }
}

fn analyze_distribution(prompt: &str) -> PhaseResult {
    let mut score = 0.7f64;
    let mut factors = Vec::new();

    // Single topic focus
    let sentences: Vec<&str> = prompt.split('.').filter(|s| s.len() > 10).collect();
    let topics = count_unique_concepts(prompt);
    let focus = if sentences.len() > 0 {
        topics as f64 / sentences.len().max(1) as f64
    } else {
        1.0
    };

    if focus < 2.0 {
        score += 0.15;
        factors.push(json!({"factor": "focused", "topics_per_sentence": focus, "effect": "+0.15"}));
    } else if focus > 5.0 {
        score -= 0.1;
        factors
            .push(json!({"factor": "scattered", "topics_per_sentence": focus, "effect": "-0.1"}));
    }

    // Important info at beginning or end (primacy/recency)
    let first_100 = &prompt[..prompt.len().min(400)];
    let has_key_instruction_early = first_100.contains("must")
        || first_100.contains("MUST")
        || first_100.contains("important")
        || first_100.contains("critical");
    if has_key_instruction_early {
        score += 0.1;
        factors.push(json!({"factor": "primacy_effect", "effect": "+0.1"}));
    }

    PhaseResult {
        score: score.clamp(0.0, 1.0),
        factors,
    }
}

fn analyze_metabolism(prompt: &str) -> PhaseResult {
    let mut score = 0.6f64;
    let mut factors = Vec::new();

    // Actionability — verbs/imperatives
    let action_words = [
        "create",
        "build",
        "fix",
        "add",
        "remove",
        "update",
        "implement",
        "write",
        "run",
        "test",
    ];
    let action_count = action_words
        .iter()
        .filter(|&&w| prompt.to_lowercase().contains(w))
        .count();
    if action_count > 0 {
        score += (action_count as f64 * 0.05).min(0.2);
        factors.push(json!({"factor": "actionable_verbs", "count": action_count, "effect": format!("+{:.2}", (action_count as f64 * 0.05).min(0.2))}));
    }

    // Constraints (narrows solution space)
    let constraint_words = [
        "must", "should", "only", "never", "always", "exactly", "at most", "at least",
    ];
    let constraint_count = constraint_words
        .iter()
        .filter(|&&w| prompt.to_lowercase().contains(w))
        .count();
    if constraint_count > 0 {
        score += (constraint_count as f64 * 0.04).min(0.15);
        factors.push(json!({"factor": "constraints", "count": constraint_count}));
    }

    // Examples (boosts metabolism significantly)
    let has_examples = prompt.contains("example")
        || prompt.contains("e.g.")
        || prompt.contains("for instance")
        || prompt.contains("```");
    if has_examples {
        score += 0.15;
        factors.push(json!({"factor": "examples_present", "effect": "+0.15"}));
    }

    PhaseResult {
        score: score.clamp(0.0, 1.0),
        factors,
    }
}

fn analyze_elimination(prompt: &str, tokens: u64) -> PhaseResult {
    let mut score = 0.8f64;
    let mut factors = Vec::new();

    // Context window pressure
    let window = 200000u64; // Claude default
    let usage = tokens as f64 / window as f64;
    if usage > 0.5 {
        score -= 0.2;
        factors.push(json!({"factor": "context_pressure", "usage_ratio": usage}));
    }
    if usage > 0.8 {
        score -= 0.3;
        factors.push(json!({"factor": "context_critical", "usage_ratio": usage}));
    }

    // Repetition (wastes tokens)
    let words: Vec<&str> = prompt.split_whitespace().collect();
    let unique_words: std::collections::HashSet<&str> = words.iter().copied().collect();
    let repetition = 1.0 - (unique_words.len() as f64 / words.len().max(1) as f64);
    if repetition > 0.3 {
        score -= 0.1;
        factors.push(json!({"factor": "high_repetition", "ratio": repetition}));
    }

    PhaseResult {
        score: score.clamp(0.0, 1.0),
        factors,
    }
}

fn generate_recommendations(
    a: &PhaseResult,
    d: &PhaseResult,
    m: &PhaseResult,
    e: &PhaseResult,
) -> Vec<String> {
    let mut recs = Vec::new();

    if a.score < 0.7 {
        recs.push("Improve structure: use headers, bullets, or numbered steps".into());
    }
    if d.score < 0.7 {
        recs.push("Focus on fewer topics per prompt for better attention distribution".into());
    }
    if m.score < 0.7 {
        recs.push("Add concrete examples and explicit constraints for better reasoning".into());
    }
    if e.score < 0.7 {
        recs.push("Reduce token count: compress repetitive sections, remove filler".into());
    }

    if recs.is_empty() {
        recs.push("Prompt is well-optimized across all ADME dimensions".into());
    }

    recs
}

fn estimate_tokens(text: &str) -> u64 {
    // Rough estimate: ~4 chars per token for English
    (text.len() as f64 / 4.0).ceil() as u64
}

fn count_unique_concepts(text: &str) -> usize {
    let words: std::collections::HashSet<&str> = text
        .split_whitespace()
        .filter(|w| w.len() > 4)
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect();
    words.len()
}

fn score_clarity(prompt: &str) -> f64 {
    let mut score = 0.6f64;
    if prompt.contains("##") || prompt.contains("**") {
        score += 0.15;
    }
    if prompt.contains("1.") || prompt.contains("- ") {
        score += 0.1;
    }
    let avg_sentence_len = {
        let sentences: Vec<&str> = prompt.split('.').filter(|s| s.len() > 5).collect();
        if sentences.is_empty() {
            50
        } else {
            sentences
                .iter()
                .map(|s| s.split_whitespace().count())
                .sum::<usize>()
                / sentences.len()
        }
    };
    if avg_sentence_len < 20 {
        score += 0.1;
    } // Short sentences are clearer
    if avg_sentence_len > 40 {
        score -= 0.1;
    }
    score.clamp(0.0, 1.0)
}

fn score_specificity(prompt: &str) -> f64 {
    let mut score = 0.5f64;
    let specific_markers = [
        "exactly",
        "must",
        "only",
        "specific",
        "precisely",
        "the file",
        "the function",
    ];
    let count = specific_markers
        .iter()
        .filter(|&&m| prompt.to_lowercase().contains(m))
        .count();
    score += (count as f64 * 0.1).min(0.4);
    // Numbers indicate specificity
    let has_numbers = prompt.chars().any(|c| c.is_ascii_digit());
    if has_numbers {
        score += 0.1;
    }
    score.clamp(0.0, 1.0)
}
