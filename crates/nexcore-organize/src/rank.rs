//! Step 2: **R**ank — Score and prioritize entries.
//!
//! Primitive: κ Comparison — "how does this entry compare to others?"
//!
//! Scores each entry by recency, size, extension relevance, and depth,
//! then sorts by composite score (highest first).

use nexcore_chrono::DateTime;

use crate::config::{OrganizeConfig, RankingConfig};
use crate::error::OrganizeResult;
use crate::observe::{EntryMeta, Inventory};

// ============================================================================
// Types
// ============================================================================

/// Breakdown of how an entry's score was computed.
///
/// Tier: T2-P (κ Comparison — individual scoring dimensions)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreBreakdown {
    /// Recency score (0.0..=1.0). Higher = more recently modified.
    pub recency: f64,
    /// Size score (0.0..=1.0). Normalized log-scale.
    pub size: f64,
    /// Relevance score (0.0..=1.0). Based on extension.
    pub relevance: f64,
    /// Depth score (0.0..=1.0). Shallower = higher.
    pub depth: f64,
    /// Weighted composite score.
    pub composite: f64,
}

/// An entry with its computed rank score.
///
/// Tier: T2-C (κ Comparison + ∃ Existence — scored existence)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RankedEntry {
    /// Original metadata.
    pub meta: EntryMeta,
    /// Score breakdown.
    pub score: ScoreBreakdown,
}

/// Inventory after ranking.
///
/// Tier: T2-C (κ Comparison + σ Sequence — ordered by score)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RankedInventory {
    /// Root directory.
    pub root: std::path::PathBuf,
    /// Entries sorted by composite score (descending).
    pub entries: Vec<RankedEntry>,
    /// Observation timestamp (carried forward).
    pub observed_at: nexcore_chrono::DateTime,
}

// ============================================================================
// Rank Function
// ============================================================================

/// Rank all entries in the inventory by composite score.
pub fn rank(inventory: Inventory, config: &OrganizeConfig) -> OrganizeResult<RankedInventory> {
    let now = DateTime::now();
    let ranking = &config.ranking;

    // Compute max values for normalization
    let max_size = inventory
        .entries
        .iter()
        .map(|e| e.size_bytes)
        .max()
        .unwrap_or(1)
        .max(1);

    let max_depth = inventory
        .entries
        .iter()
        .map(|e| e.depth)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut ranked: Vec<RankedEntry> = inventory
        .entries
        .into_iter()
        .map(|meta| {
            let score = compute_score(&meta, ranking, now, max_size, max_depth);
            RankedEntry { meta, score }
        })
        .collect();

    // Sort by composite score descending
    ranked.sort_by(|a, b| {
        b.score
            .composite
            .partial_cmp(&a.score.composite)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(RankedInventory {
        root: inventory.root,
        entries: ranked,
        observed_at: inventory.observed_at,
    })
}

// ============================================================================
// Scoring
// ============================================================================

/// Compute the score breakdown for a single entry.
fn compute_score(
    meta: &EntryMeta,
    ranking: &RankingConfig,
    now: nexcore_chrono::DateTime,
    max_size: u64,
    max_depth: usize,
) -> ScoreBreakdown {
    let recency = recency_score(meta, now);
    let size = size_score(meta, max_size);
    let relevance = relevance_score(meta);
    let depth = depth_score(meta, max_depth);

    let composite = ranking.recency_weight * recency
        + ranking.size_weight * size
        + ranking.relevance_weight * relevance
        + ranking.depth_weight * depth;

    ScoreBreakdown {
        recency,
        size,
        relevance,
        depth,
        composite,
    }
}

/// Recency: exponential decay based on age in days.
fn recency_score(meta: &EntryMeta, now: nexcore_chrono::DateTime) -> f64 {
    let age_days = (now - meta.modified).num_days().max(0) as f64;
    // Half-life of 30 days
    (-age_days / 30.0_f64).exp()
}

/// Size: log-normalized against max size.
fn size_score(meta: &EntryMeta, max_size: u64) -> f64 {
    if meta.is_dir || max_size == 0 {
        return 0.5;
    }
    let log_size = (meta.size_bytes as f64 + 1.0).ln();
    let log_max = (max_size as f64 + 1.0).ln();
    if log_max > 0.0 {
        log_size / log_max
    } else {
        0.5
    }
}

/// Relevance: score by file extension.
fn relevance_score(meta: &EntryMeta) -> f64 {
    if meta.is_dir {
        return 0.6;
    }
    match meta.extension.as_str() {
        "rs" | "toml" => 1.0,
        "md" | "txt" | "json" => 0.8,
        "yaml" | "yml" | "csv" => 0.7,
        "py" | "js" | "ts" => 0.6,
        "png" | "jpg" | "jpeg" | "gif" | "svg" => 0.4,
        "zip" | "tar" | "gz" | "bz2" => 0.3,
        "log" | "tmp" | "bak" => 0.1,
        "" => 0.5,
        _ => 0.5,
    }
}

/// Depth: shallower entries score higher.
fn depth_score(meta: &EntryMeta, max_depth: usize) -> f64 {
    if max_depth == 0 {
        return 1.0;
    }
    1.0 - (meta.depth as f64 / max_depth as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observe::EntryMeta;
    use nexcore_chrono::Duration;
    use std::path::PathBuf;

    fn make_meta(name: &str, ext: &str, size: u64, days_old: i64, depth: usize) -> EntryMeta {
        EntryMeta {
            path: PathBuf::from(format!("/tmp/{name}.{ext}")),
            is_dir: false,
            size_bytes: size,
            modified: DateTime::now() - Duration::days(days_old),
            extension: ext.to_string(),
            depth,
            name: format!("{name}.{ext}"),
        }
    }

    #[test]
    fn test_recency_score_recent() {
        let meta = make_meta("fresh", "rs", 100, 0, 1);
        let score = recency_score(&meta, DateTime::now());
        assert!(score > 0.9);
    }

    #[test]
    fn test_recency_score_old() {
        let meta = make_meta("old", "rs", 100, 90, 1);
        let score = recency_score(&meta, DateTime::now());
        assert!(score < 0.1);
    }

    #[test]
    fn test_relevance_score_rust() {
        let meta = make_meta("lib", "rs", 100, 0, 1);
        let score = relevance_score(&meta);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_relevance_score_log() {
        let meta = make_meta("debug", "log", 100, 0, 1);
        let score = relevance_score(&meta);
        assert!((score - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_depth_score_shallow() {
        let meta = make_meta("top", "rs", 100, 0, 1);
        let score = depth_score(&meta, 10);
        assert!(score > 0.8);
    }

    #[test]
    fn test_rank_sorts_descending() {
        let inventory = Inventory {
            root: PathBuf::from("/tmp"),
            entries: vec![
                make_meta("old_log", "log", 10, 100, 5),
                make_meta("fresh_rust", "rs", 1000, 0, 1),
            ],
            excluded_count: 0,
            observed_at: DateTime::now(),
        };

        let config = OrganizeConfig::default_for("/tmp");
        let ranked = rank(inventory, &config);
        assert!(ranked.is_ok());
        if let Ok(ranked) = ranked {
            assert_eq!(ranked.entries.len(), 2);
            // Fresh rust file should rank higher
            assert!(ranked.entries[0].score.composite > ranked.entries[1].score.composite);
        }
    }
}
