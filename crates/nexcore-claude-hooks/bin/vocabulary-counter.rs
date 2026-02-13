//! Vocabulary Counter - UserPromptSubmit Hook
//!
//! Scans every user prompt for vocabulary shorthand usage, persists counters,
//! and computes live compression metrics (HHI, Cs-empirical, token savings, dead weight).
//!
//! Closes the theoretical vocabulary compression analysis with empirical data.
//!
//! # Tier: T2-C (Cross-Domain Composite)
//! Grounding: T1(String, u64, f64, Vec, HashMap) via shorthand scan + metric computation.
//!
//! Hook Protocol:
//! - Input: JSON on stdin with prompt, session_id
//! - Output: Empty JSON `{}` on stdout
//! - Exit: 0 = pass (observation-only, never blocks)
//!
//! Persists to: ~/.claude/implicit/vocabulary_counters.json
//!
//! # Cytokine Integration
//! - **Hook Completed**: Emits TGF-beta (regulation) via cytokine bridge

use chrono::Utc;
use nexcore_hook_lib::cytokine::emit_hook_completed;
use nexcore_hook_lib::pass;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

const HOOK_NAME: &str = "vocabulary-counter";
const COUNTERS_FILE: &str = "/home/matthew/.claude/implicit/vocabulary_counters.json";
const STATE_VERSION: &str = "1.0.0";

/// Embedded shorthand list: (name, compression_ratio).
/// Compile-time constant — zero file I/O on hot path.
///
/// # Tier: T1 (static sequence of tuples)
const SHORTHANDS: &[(&str, usize)] = &[
    ("skill-bonds", 6),
    ("chain-composer", 7),
    ("epistemic-mesh", 7),
    ("build-doctrine", 5),
    ("guardian-context", 6),
    ("academy-grade", 5),
    ("diamond-v2", 6),
    ("ctvp-validated", 6),
    ("preflight-protocol", 5),
    ("brain-session", 5),
    ("hook-enforced", 5),
    ("kbs-growth", 4),
    ("tov-axioms", 5),
    ("primitive-mode", 7),
    ("t2c-factory", 5),
    ("growth-loop", 6),
    ("ccp-loop", 5),
    ("hud-mapped", 5),
    ("signal-verified", 5),
    ("skill-determinism", 6),
];

/// Average tokens per constraint line.
const TOKENS_PER_CONSTRAINT: usize = 11;
/// Tokens for the shorthand name itself.
const TOKENS_FOR_NAME: usize = 3;

/// UserPromptSubmit input structure.
///
/// # Tier: T2-C
/// Grounds to: T1(String) via Option.
#[derive(Debug, Deserialize)]
struct PromptInput {
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    session_id: Option<String>,
}

/// Persisted counter state.
///
/// # Tier: T2-C
/// Grounds to: T1(String, u64, HashMap, Vec, f64).
#[derive(Debug, Serialize, Deserialize)]
struct CounterState {
    version: String,
    updated_at: String,
    total_prompts: u64,
    total_hits: u64,
    counts: HashMap<String, u64>,
    metrics: CompressionMetrics,
}

/// Live compression metrics computed from empirical usage.
///
/// # Tier: T2-C
/// Grounds to: T1(f64, u64, Vec<String>, Vec<(String, u64)>).
#[derive(Debug, Serialize, Deserialize)]
struct CompressionMetrics {
    /// Herfindahl-Hirschman Index: concentration of usage.
    /// Range [1/N, 1.0]. Uniform = 1/N = 0.05.
    hhi: f64,
    /// Empirical Compendious Score: (I/E) * C * R
    cs_empirical: f64,
    /// Estimated tokens saved by shorthand usage.
    estimated_tokens_saved: u64,
    /// Shorthands with zero hits (pruning candidates).
    dead_weight: Vec<String>,
    /// Top 3 most-used shorthands.
    top_3: Vec<(String, u64)>,
}

fn main() {
    let prompt = read_prompt();
    let lowered = prompt.to_lowercase();

    let hits = scan_shorthands(&lowered);

    let mut state = load_state();
    state.total_prompts += 1;

    for name in &hits {
        state.total_hits += 1;
        *state.counts.entry(name.clone()).or_insert(0) += 1;
    }

    state.metrics = compute_metrics(&state.counts);
    state.updated_at = Utc::now().to_rfc3339();

    if let Err(e) = save_state(&state) {
        eprintln!("[{HOOK_NAME}] Warning: persist failed: {e}");
    }

    let hit_count = hits.len();
    emit_hook_completed(HOOK_NAME, 0, &format!("hits_{hit_count}"));

    pass();
}

/// Read stdin, parse prompt, and return it. Calls pass() on skip conditions.
///
/// # Tier: T1 (Sequence: read -> parse -> validate)
fn read_prompt() -> String {
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        pass();
    }
    if buffer.trim().is_empty() {
        pass();
    }

    let input: PromptInput = match serde_json::from_str(&buffer) {
        Ok(i) => i,
        Err(_) => pass(),
    };

    match input.prompt {
        Some(p) if !p.is_empty() => p,
        _ => pass(),
    }
}

/// Scan lowered prompt for shorthand matches.
/// Returns list of matched shorthand names.
///
/// # Tier: T1 (Mapping: shorthand -> bool)
fn scan_shorthands(lowered_prompt: &str) -> Vec<String> {
    SHORTHANDS
        .iter()
        .filter(|(name, _)| lowered_prompt.contains(name))
        .map(|(name, _)| (*name).to_string())
        .collect()
}

/// Load persisted state from disk. Returns default state if missing or corrupt.
///
/// # Tier: T2-C (file I/O + deserialization)
fn load_state() -> CounterState {
    let path = PathBuf::from(COUNTERS_FILE);
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| default_state()),
        Err(_) => default_state(),
    }
}

/// Save state to disk.
///
/// # Tier: T2-C (serialization + file I/O)
fn save_state(state: &CounterState) -> Result<(), std::io::Error> {
    let path = PathBuf::from(COUNTERS_FILE);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&path, json)?;
    Ok(())
}

/// Create default empty state.
fn default_state() -> CounterState {
    CounterState {
        version: STATE_VERSION.to_string(),
        updated_at: Utc::now().to_rfc3339(),
        total_prompts: 0,
        total_hits: 0,
        counts: HashMap::new(),
        metrics: CompressionMetrics {
            hhi: 0.0,
            cs_empirical: 0.0,
            estimated_tokens_saved: 0,
            dead_weight: SHORTHANDS
                .iter()
                .map(|(name, _)| (*name).to_string())
                .collect(),
            top_3: Vec::new(),
        },
    }
}

/// Compute compression metrics from empirical shorthand counts.
///
/// - **HHI** = sum(share^2) where share = count/total_hits.
///   Range [1/N, 1.0]. Uniform = 1/20 = 0.05. 0.0 if no hits.
/// - **Cs** = (I/E) * C * R
///   I/E = used_count / total_shorthands (coverage ratio)
///   C = 1 - HHI (evenness)
///   R = sum(used_CR) / sum(all_CR) (compression ratio utilization)
/// - **Token savings** = sum(hits * (CR * TOKENS_PER_CONSTRAINT - TOKENS_FOR_NAME))
/// - **Dead weight** = shorthands with 0 hits
///
/// # Tier: T1 (Mapping + Reduction)
fn compute_metrics(counts: &HashMap<String, u64>) -> CompressionMetrics {
    let total_shorthands = SHORTHANDS.len();
    let total_hits: u64 = counts.values().sum();

    // HHI
    let hhi = compute_hhi(counts, total_hits);

    // Coverage and compression ratio
    let used_shorthands: Vec<(&str, usize)> = SHORTHANDS
        .iter()
        .filter(|(name, _)| counts.get(*name).copied().unwrap_or(0) > 0)
        .copied()
        .collect();

    let used_count = used_shorthands.len();
    let ie_ratio = if total_shorthands > 0 {
        used_count as f64 / total_shorthands as f64
    } else {
        0.0
    };

    let all_cr_sum: usize = SHORTHANDS.iter().map(|(_, cr)| *cr).sum();
    let used_cr_sum: usize = used_shorthands.iter().map(|(_, cr)| *cr).sum();
    let r_ratio = if all_cr_sum > 0 {
        used_cr_sum as f64 / all_cr_sum as f64
    } else {
        0.0
    };

    let c_evenness = 1.0 - hhi;
    let cs_empirical = ie_ratio * c_evenness * r_ratio;

    // Token savings
    let estimated_tokens_saved = compute_token_savings(counts);

    // Dead weight
    let dead_weight: Vec<String> = SHORTHANDS
        .iter()
        .filter(|(name, _)| counts.get(*name).copied().unwrap_or(0) == 0)
        .map(|(name, _)| (*name).to_string())
        .collect();

    // Top 3
    let top_3 = compute_top_3(counts);

    CompressionMetrics {
        hhi,
        cs_empirical,
        estimated_tokens_saved,
        dead_weight,
        top_3,
    }
}

/// Compute HHI from usage counts.
/// HHI = sum(share_i^2) where share_i = count_i / total_hits.
///
/// # Tier: T1 (Reduction: sum of squares)
fn compute_hhi(counts: &HashMap<String, u64>, total_hits: u64) -> f64 {
    if total_hits == 0 {
        return 0.0;
    }
    let total = total_hits as f64;
    counts
        .values()
        .map(|&c| {
            let share = c as f64 / total;
            share * share
        })
        .sum()
}

/// Compute estimated token savings from shorthand usage.
/// savings_per_hit = CR * TOKENS_PER_CONSTRAINT - TOKENS_FOR_NAME
///
/// # Tier: T1 (Mapping + Reduction)
fn compute_token_savings(counts: &HashMap<String, u64>) -> u64 {
    let mut total: u64 = 0;
    for &(name, cr) in SHORTHANDS {
        let hits = counts.get(name).copied().unwrap_or(0);
        if hits > 0 {
            let savings_per_hit = cr
                .saturating_mul(TOKENS_PER_CONSTRAINT)
                .saturating_sub(TOKENS_FOR_NAME);
            total = total.saturating_add(hits.saturating_mul(savings_per_hit as u64));
        }
    }
    total
}

/// Get top 3 shorthands by usage count.
///
/// # Tier: T1 (Sort + Truncate)
fn compute_top_3(counts: &HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut sorted: Vec<(String, u64)> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(3);
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_single_match() {
        let hits = scan_shorthands("apply build-doctrine to this project");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], "build-doctrine");
    }

    #[test]
    fn test_scan_multiple_matches() {
        let hits = scan_shorthands("use skill-bonds with chain-composer");
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&"skill-bonds".to_string()));
        assert!(hits.contains(&"chain-composer".to_string()));
    }

    #[test]
    fn test_scan_no_match() {
        let hits = scan_shorthands("refactor the login flow");
        assert!(hits.is_empty());
    }

    #[test]
    fn test_scan_case_insensitive() {
        // Input is lowered before scanning, so "Build-Doctrine" becomes "build-doctrine"
        let hits = scan_shorthands("apply build-doctrine now");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_no_false_positive_partial() {
        // "build" alone must NOT match "build-doctrine"
        let hits = scan_shorthands("build the project");
        assert!(hits.is_empty());
    }

    #[test]
    fn test_hhi_uniform() {
        // All 20 shorthands with equal count -> HHI = 1/20 = 0.05
        let mut counts = HashMap::new();
        for &(name, _) in SHORTHANDS {
            counts.insert(name.to_string(), 10);
        }
        let total: u64 = counts.values().sum();
        let hhi = compute_hhi(&counts, total);
        let expected = 1.0 / SHORTHANDS.len() as f64;
        assert!(
            (hhi - expected).abs() < 1e-10,
            "Expected HHI ~{expected}, got {hhi}"
        );
    }

    #[test]
    fn test_hhi_concentrated() {
        // Single dominant shorthand -> HHI near 1.0
        let mut counts = HashMap::new();
        counts.insert("skill-bonds".to_string(), 1000);
        counts.insert("chain-composer".to_string(), 1);
        let total: u64 = counts.values().sum();
        let hhi = compute_hhi(&counts, total);
        assert!(hhi > 0.99, "Expected HHI near 1.0, got {hhi}");
    }

    #[test]
    fn test_hhi_empty() {
        let counts = HashMap::new();
        let hhi = compute_hhi(&counts, 0);
        assert!(
            (hhi - 0.0).abs() < f64::EPSILON,
            "Expected HHI 0.0, got {hhi}"
        );
    }

    #[test]
    fn test_cs_full_utilization() {
        // All shorthands used equally -> high Cs
        let mut counts = HashMap::new();
        for &(name, _) in SHORTHANDS {
            counts.insert(name.to_string(), 10);
        }
        let metrics = compute_metrics(&counts);
        // IE = 20/20 = 1.0, C = 1 - 0.05 = 0.95, R = 1.0 -> Cs = 0.95
        assert!(
            metrics.cs_empirical > 0.9,
            "Expected Cs > 0.9, got {}",
            metrics.cs_empirical
        );
    }

    #[test]
    fn test_cs_zero() {
        let counts = HashMap::new();
        let metrics = compute_metrics(&counts);
        assert!(
            (metrics.cs_empirical - 0.0).abs() < f64::EPSILON,
            "Expected Cs 0.0, got {}",
            metrics.cs_empirical
        );
    }

    #[test]
    fn test_token_savings() {
        // build-doctrine (CR=5): savings_per_hit = 5*11 - 3 = 52
        // 10 hits -> 520 tokens saved
        let mut counts = HashMap::new();
        counts.insert("build-doctrine".to_string(), 10);
        let savings = compute_token_savings(&counts);
        assert_eq!(savings, 520, "Expected 520 tokens saved, got {savings}");
    }

    #[test]
    fn test_dead_weight() {
        // Only one shorthand used -> 19 dead weight
        let mut counts = HashMap::new();
        counts.insert("skill-bonds".to_string(), 5);
        let metrics = compute_metrics(&counts);
        assert_eq!(
            metrics.dead_weight.len(),
            SHORTHANDS.len() - 1,
            "Expected {} dead weight, got {}",
            SHORTHANDS.len() - 1,
            metrics.dead_weight.len()
        );
        assert!(
            !metrics.dead_weight.contains(&"skill-bonds".to_string()),
            "skill-bonds should not be in dead_weight"
        );
    }
}
