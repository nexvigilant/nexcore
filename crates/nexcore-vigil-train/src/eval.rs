//! Evaluation: sample N SFT rows, generate a completion per row, grade via
//! `rsk mcg test` (external binary), return per-row and aggregate scores.
//!
//! This mirrors `vigil-lora-train eval` (Python) one-for-one so scores are
//! directly comparable across stacks.

use nexcore_error::{NexError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Canonical path to the rsk binary.
pub const RSK_BIN: &str =
    "/home/matthew/Projects/Active/nucleus/workspaces/rsk-core/target/release/rsk";

/// Per-row eval result.
#[derive(Clone, Debug, Serialize)]
pub struct EvalRow {
    pub index: usize,
    pub source: Option<String>,
    pub score: f32,
    pub passed: u32,
    pub failed: u32,
    pub total: u32,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Aggregated eval summary.
#[derive(Clone, Debug, Serialize)]
pub struct EvalSummary {
    pub n: usize,
    pub mean_score: f32,
    pub median_score: f32,
    pub min_score: f32,
    pub max_score: f32,
    pub zero_count: u32,
    pub perfect_count: u32,
    pub histogram: Vec<(String, u32)>,
    pub elapsed_s: f64,
    pub rows: Vec<EvalRow>,
}

/// Raw JSON returned by `rsk mcg test` (subset of fields we care about).
#[derive(Deserialize)]
struct RskResult {
    #[serde(default)]
    total: u32,
    #[serde(default)]
    passed: u32,
    #[serde(default)]
    failed: u32,
}

/// Grade a YAML payload by piping it to `rsk mcg test` via a temp file.
pub fn grade(yaml_text: &str, rsk_path: &Path) -> Result<(u32, u32, u32)> {
    let tmp = std::env::temp_dir().join(format!(
        "vigil-grade-{}-{}.yaml",
        std::process::id(),
        fastrand_like()
    ));
    std::fs::write(&tmp, yaml_text)
        .map_err(|e| NexError::new(format!("write temp {}: {e}", tmp.display())))?;
    let out = Command::new(rsk_path)
        .args(["mcg", "test"])
        .arg(&tmp)
        .output()
        .map_err(|e| NexError::new(format!("spawn rsk: {e}")))?;
    // Best-effort cleanup; ignore failure.
    let _ = std::fs::remove_file(&tmp);
    if out.stdout.is_empty() {
        return Err(NexError::new(format!(
            "rsk empty stdout (rc={:?}): {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    let result: RskResult = serde_json::from_slice(&out.stdout).map_err(|e| {
        NexError::new(format!(
            "rsk non-JSON stdout: {e} — body: {}",
            String::from_utf8_lossy(&out.stdout)
                .chars()
                .take(200)
                .collect::<String>()
        ))
    })?;
    Ok((result.passed, result.failed, result.total))
}

/// Pseudo-random filename suffix — uses nanoseconds to avoid pulling in rand.
fn fastrand_like() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default()
}

/// Compute aggregate stats from per-row scores. Separate so unit tests can hit it.
pub fn aggregate(rows: Vec<EvalRow>, elapsed_s: f64) -> EvalSummary {
    let n = rows.len();
    let mut scores: Vec<f32> = rows.iter().map(|r| r.score).collect();
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean_score = if n == 0 {
        0.0
    } else {
        scores.iter().sum::<f32>() / n as f32
    };
    let median_score = if n == 0 {
        0.0
    } else if n % 2 == 1 {
        scores[n / 2]
    } else {
        (scores[n / 2 - 1] + scores[n / 2]) / 2.0
    };
    let min_score = *scores.first().unwrap_or(&0.0);
    let max_score = *scores.last().unwrap_or(&0.0);
    let zero_count = rows.iter().filter(|r| r.score == 0.0).count() as u32;
    let perfect_count = rows.iter().filter(|r| r.score >= 1.0).count() as u32;

    // Histogram bucketed to 1 decimal.
    let mut bins: [(String, u32); 11] =
        core::array::from_fn(|i| (format!("{:.1}", i as f32 / 10.0), 0));
    for r in &rows {
        let b = ((r.score.clamp(0.0, 1.0) * 10.0).round() as usize).min(10);
        bins[b].1 += 1;
    }

    EvalSummary {
        n,
        mean_score,
        median_score,
        min_score,
        max_score,
        zero_count,
        perfect_count,
        histogram: bins.into_iter().filter(|(_, c)| *c > 0).collect(),
        elapsed_s,
        rows,
    }
}

/// Default RSK binary path — use env `VIGIL_RSK_BIN` or the canonical path.
pub fn rsk_path() -> PathBuf {
    std::env::var("VIGIL_RSK_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(RSK_BIN))
}
