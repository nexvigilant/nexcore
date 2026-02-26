//! Compendious Machine — text density optimization tools.
//!
//! Consolidated from `compendious-machine` satellite MCP server.
//! 5 tools: score_text, compress_text, compare_texts, analyze_patterns, get_domain_target.
//!
//! Implements the Compendious Score: Cs = (I / E) × C × R
//!
//! Tier: T3 (N Quantity + κ Comparison + σ Sequence + μ Mapping)

use std::collections::BTreeSet;
use std::sync::LazyLock;

use crate::params::{
    CompendiousAnalyzeParams, CompendiousCompareParams, CompendiousCompressParams,
    CompendiousDomainTargetParams, CompendiousScoreParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// Verbose patterns (const at module level)
// ============================================================================

/// (verbose_phrase, replacement) — empty replacement means DELETE.
const VERBOSE_PATTERNS: &[(&str, &str)] = &[
    // Throat-clearing deletions
    ("it is important to note that", ""),
    ("it should be mentioned that", ""),
    ("as a matter of fact", ""),
    ("for all intents and purposes", ""),
    ("at the end of the day", ""),
    ("the fact of the matter is", ""),
    ("it is worth noting that", ""),
    ("needless to say", ""),
    // Prepositional bloat
    ("in order to", "to"),
    ("for the purpose of", "to"),
    ("with regard to", "about"),
    ("in reference to", "about"),
    ("in terms of", "regarding"),
    ("on the basis of", "based on"),
    ("in the event that", "if"),
    ("at this point in time", "now"),
    ("at the present time", "now"),
    ("prior to", "before"),
    ("subsequent to", "after"),
    ("in spite of the fact that", "although"),
    ("due to the fact that", "because"),
    ("in light of the fact that", "because"),
    // Redundancies
    ("completely finished", "finished"),
    ("absolutely essential", "essential"),
    ("basic fundamentals", "fundamentals"),
    ("past history", "history"),
    ("future plans", "plans"),
    ("end result", "result"),
    ("final outcome", "outcome"),
    ("close proximity", "near"),
    ("each and every", "each"),
    ("any and all", "all"),
    ("first and foremost", "first"),
    // Nominalizations
    ("make a decision", "decide"),
    ("give consideration to", "consider"),
    ("reach a conclusion", "conclude"),
    ("perform an analysis", "analyze"),
    ("conduct an investigation", "investigate"),
    ("provide assistance to", "help"),
    ("make an improvement", "improve"),
    ("take action", "act"),
    // Common verbose phrases
    ("a large number of", "many"),
    ("a significant number of", "many"),
    ("the vast majority of", "most"),
    ("in the near future", "soon"),
    ("at some point in the future", "eventually"),
];

const STOPWORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had", "do", "does",
    "did", "will", "would", "could", "should", "may", "might", "must", "shall", "can", "this",
    "that", "these", "those", "it", "its", "they", "them", "their",
];

/// Domain targets: ((domain, content_type), target_cs, rationale)
const DOMAIN_TARGETS: &[(&str, &str, f64, &str)] = &[
    (
        "technical",
        "api_reference",
        2.5,
        "Maximum density, developers expect precision",
    ),
    (
        "technical",
        "tutorial",
        1.7,
        "Balance density with learning scaffolding",
    ),
    ("technical", "readme", 2.0, "Quick orientation, high signal"),
    (
        "business",
        "executive_summary",
        2.5,
        "Executives have zero tolerance for fluff",
    ),
    (
        "business",
        "proposal",
        2.0,
        "Persuasion requires some elaboration",
    ),
    ("business", "email", 2.0, "Respect recipient time"),
    (
        "academic",
        "abstract",
        3.0,
        "Extreme density required by word limits",
    ),
    ("academic", "paper_body", 1.4, "Argumentation needs room"),
    (
        "legal",
        "contract",
        1.8,
        "Precision over brevity, but no redundancy",
    ),
    ("legal", "brief", 2.0, "Courts value concision"),
    (
        "medical",
        "clinical_note",
        2.5,
        "Time-critical, standardized",
    ),
    (
        "medical",
        "patient_instructions",
        1.6,
        "Clarity over density for lay readers",
    ),
    ("journalism", "headline", 4.0, "Maximum compression"),
    ("journalism", "lead_paragraph", 2.5, "5W1H in minimum words"),
    (
        "journalism",
        "article_body",
        1.8,
        "Inverted pyramid allows pruning",
    ),
    (
        "pharmacovigilance",
        "icsr_narrative",
        2.3,
        "Regulatory scrutiny, completeness critical",
    ),
    (
        "pharmacovigilance",
        "signal_report",
        2.5,
        "Evidence synthesis, no fluff",
    ),
    (
        "pharmacovigilance",
        "regulatory_alert",
        3.0,
        "Time-sensitive, action-oriented",
    ),
];

static STOPWORD_SET: LazyLock<BTreeSet<&'static str>> =
    LazyLock::new(|| STOPWORDS.iter().copied().collect());

// ============================================================================
// Tool implementations
// ============================================================================

/// Calculate the Compendious Score (Cs = I/E × C × R).
pub fn score_text(params: CompendiousScoreParams) -> Result<CallToolResult, McpError> {
    let i = information_content(&params.text);
    let e = expression_cost(&params.text);
    let c = completeness(&params.text, &params.required_elements.unwrap_or_default());
    let r = readability(&params.text);

    let density = if e > 0 { i / e as f64 } else { 0.0 };
    let score = density * c * r;

    let result = json!({
        "score": round2(score),
        "information_bits": round1(i),
        "expression_cost": e,
        "completeness": round2(c),
        "readability": round2(r),
        "limiting_factor": identify_limiting_factor(density, c, r),
        "interpretation": interpret_score(score),
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Apply BLUFF method to compress text.
pub fn compress_text(params: CompendiousCompressParams) -> Result<CallToolResult, McpError> {
    let original_score = score_components(&params.text, &[]);
    let preserve_set: BTreeSet<String> = params.preserve.unwrap_or_default().into_iter().collect();
    // target_cs: caller-supplied density goal. All applicable patterns are applied regardless
    // (static patterns are the ceiling), but target_achieved signals whether the output meets
    // the goal. Defaults to 2.0 (standard dense output target).
    let target = params.target_cs.unwrap_or(2.0);

    let mut compressed = params.text.to_lowercase();
    let mut patterns_applied = Vec::new();

    for &(verbose, replacement) in VERBOSE_PATTERNS {
        if compressed.contains(verbose) {
            let should_skip = preserve_set
                .iter()
                .any(|p| verbose.contains(&p.to_lowercase()));
            if !should_skip {
                let savings = verbose
                    .split_whitespace()
                    .count()
                    .saturating_sub(replacement.split_whitespace().count());
                patterns_applied.push(json!({
                    "pattern": verbose,
                    "replacement": if replacement.is_empty() { "[DELETE]" } else { replacement },
                    "savings": savings,
                }));
                compressed = compressed.replace(verbose, replacement);
            }
        }
    }

    compressed = compressed.split_whitespace().collect::<Vec<_>>().join(" ");
    compressed = restore_sentence_casing(&compressed);

    let compressed_score = score_components(&compressed, &[]);

    let improvement = if original_score.0 > 0.0 {
        ((compressed_score.0 - original_score.0) / original_score.0) * 100.0
    } else {
        0.0
    };

    let target_achieved = compressed_score.0 >= target;

    let result = json!({
        "original_tokens": original_score.1,
        "compressed_tokens": compressed_score.1,
        "original_score": round2(original_score.0),
        "compressed_score": round2(compressed_score.0),
        "improvement_percent": round1(improvement),
        "target_achieved": target_achieved,
        "compressed_text": compressed,
        "patterns_applied": patterns_applied,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compare original vs optimized text.
pub fn compare_texts(params: CompendiousCompareParams) -> Result<CallToolResult, McpError> {
    let orig = score_components(&params.original, &[]);
    let opt = score_components(&params.optimized, &[]);

    let improvement = if orig.0 > 0.0 {
        ((opt.0 - orig.0) / orig.0) * 100.0
    } else {
        0.0
    };

    let result = json!({
        "original": { "score": round2(orig.0), "tokens": orig.1 },
        "optimized": { "score": round2(opt.0), "tokens": opt.1 },
        "improvement_percent": round1(improvement),
        "tokens_saved": orig.1 as i64 - opt.1 as i64,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Identify verbose patterns in text.
pub fn analyze_patterns(params: CompendiousAnalyzeParams) -> Result<CallToolResult, McpError> {
    let text_lower = params.text.to_lowercase();
    let mut matches = Vec::new();

    for &(verbose, replacement) in VERBOSE_PATTERNS {
        if text_lower.contains(verbose) {
            let savings = verbose
                .split_whitespace()
                .count()
                .saturating_sub(replacement.split_whitespace().count());
            matches.push(json!({
                "pattern": verbose,
                "replacement": if replacement.is_empty() { "[DELETE]" } else { replacement },
                "savings": savings,
            }));
        }
    }

    matches.sort_by(|a, b| {
        let sa = a["savings"].as_u64().unwrap_or(0);
        let sb = b["savings"].as_u64().unwrap_or(0);
        sb.cmp(&sa)
    });

    let total_savings: u64 = matches.iter().filter_map(|m| m["savings"].as_u64()).sum();

    let result = json!({
        "patterns_found": matches.len(),
        "total_tokens_recoverable": total_savings,
        "patterns": matches,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get recommended Cs target for a domain/content type.
pub fn get_domain_target(
    params: CompendiousDomainTargetParams,
) -> Result<CallToolResult, McpError> {
    let domain = params.domain.to_lowercase();
    let content_type = params.content_type.to_lowercase();

    let target = DOMAIN_TARGETS
        .iter()
        .find(|&&(d, ct, _, _)| d == domain && ct == content_type);

    let result = match target {
        Some(&(d, ct, cs, rationale)) => json!({
            "domain": d,
            "content_type": ct,
            "target_cs": cs,
            "rationale": rationale,
        }),
        None => json!({
            "domain": domain,
            "content_type": content_type,
            "target_cs": 1.8,
            "rationale": "Generic target; specific domain/content_type combination not found",
        }),
    };
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Scoring helpers
// ============================================================================

/// Returns (score, token_count).
fn score_components(text: &str, required: &[String]) -> (f64, usize) {
    let i = information_content(text);
    let e = expression_cost(text);
    let c = completeness(text, required);
    let r = readability(text);
    let density = if e > 0 { i / e as f64 } else { 0.0 };
    (density * c * r, e)
}

fn information_content(text: &str) -> f64 {
    let lowercased = text.to_lowercase();
    let words: BTreeSet<&str> = lowercased
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .filter(|w| !STOPWORD_SET.contains(*w))
        .collect();
    words.len() as f64 * 4.0
}

fn expression_cost(text: &str) -> usize {
    text.split_whitespace().count()
}

fn completeness(text: &str, required_elements: &[String]) -> f64 {
    if required_elements.is_empty() {
        return 1.0;
    }
    let text_lower = text.to_lowercase();
    let present = required_elements
        .iter()
        .filter(|req| text_lower.contains(&req.to_lowercase()))
        .count();
    present as f64 / required_elements.len() as f64
}

fn readability(text: &str) -> f64 {
    let words: Vec<&str> = text.split_whitespace().collect();
    let sentences = text
        .matches(|c| c == '.' || c == '!' || c == '?')
        .count()
        .max(1);
    let avg_words_per_sentence = words.len() as f64 / sentences as f64;

    // Sentence-length penalty: penalise run-on sentences only.
    // Any sentence at or under 20 words: full score (1.0) — concision is never penalised.
    // Above 20 words: score decays toward 0.5 as sentences get longer.
    // Replaces Flesch formula which penalises technical vocabulary and thereby
    // inverts the compendious score for dense, precise text.
    if avg_words_per_sentence <= 20.0 {
        1.0
    } else {
        let excess = avg_words_per_sentence - 20.0;
        (1.0 / (1.0 + excess * 0.04)).clamp(0.5, 1.0)
    }
}

fn identify_limiting_factor(density: f64, c: f64, r: f64) -> String {
    let factors = [
        (density, "Information Density"),
        (c, "Completeness"),
        (r, "Readability"),
    ];
    factors
        .iter()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(val, name)| format!("{name} ({val:.2})"))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn interpret_score(score: f64) -> &'static str {
    match score {
        s if s < 0.5 => "Verbose - Aggressive compression needed",
        s if s < 1.0 => "Adequate - Minor optimization possible",
        s if s < 2.0 => "Efficient - Good compendious quality",
        s if s < 5.0 => "Excellent - Publishable density",
        _ => "Exceptional - Reference-grade compression",
    }
}

fn restore_sentence_casing(text: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in text.chars() {
        if capitalize_next && c.is_alphabetic() {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
        if c == '.' || c == '!' || c == '?' {
            capitalize_next = true;
        }
    }
    result
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbose_scores_lower() {
        let verbose_score = score_components(
            "In order to facilitate the implementation of the aforementioned solution, it is important to note that we should consider the various factors.",
            &[],
        );
        let compendious_score = score_components(
            "Three factors affect implementation: cost, time, complexity.",
            &[],
        );
        assert!(
            compendious_score.0 > verbose_score.0,
            "compendious={} should be > verbose={}",
            compendious_score.0,
            verbose_score.0
        );
    }

    #[test]
    fn pattern_detection_works() {
        let text = "In order to make a decision, we need to give consideration to all factors.";
        let text_lower = text.to_lowercase();
        let count = VERBOSE_PATTERNS
            .iter()
            .filter(|&&(v, _)| text_lower.contains(v))
            .count();
        assert!(count >= 2);
    }
}
