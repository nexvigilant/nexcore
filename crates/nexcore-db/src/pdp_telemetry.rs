//! PDP (Point-Driven Prompting) evaluation telemetry writer.
//!
//! Appends structured evaluation records to a JSONL file at
//! `~/.claude/brain/telemetry/pdp_evaluations.jsonl`. Each record captures
//! a single PDP gate evaluation (Foundation, Structural, or Calibration)
//! with individual check results, block status, and provenance metadata.
//!
//! This module is the telemetry counterpart to the `autopsy` module:
//! - `autopsy` stores aggregate session-level records in SQLite
//! - `pdp_telemetry` stores per-evaluation event records in JSONL
//!
//! # File Format
//!
//! One JSON object per line (JSONL). Each line is a serialized [`PdpEvaluation`].
//! The file is append-only — no records are modified or deleted.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{DbError, Result};

/// Which PDP gate produced this evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PdpGate {
    /// Tier 1 — semantic evaluation (G1 Proposition, G2 Specificity, G3 Singularity).
    Foundation,
    /// Tier 2 — deterministic pattern matching (S1–S5).
    Structural,
    /// Tier 3 — post-response advisory (C1–C5).
    Calibration,
}

/// Result of a single PDP check within a gate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckResult {
    /// Check identifier (e.g. "G1", "S3", "C5").
    pub check_id: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Optional detail about the check outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// A single PDP gate evaluation event.
///
/// One record is produced each time a PDP gate fires on a prompt or response.
/// Multiple records may exist for the same `session_id` if multiple gates
/// fire (Foundation + Structural fire in parallel on submission; Calibration
/// fires after response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdpEvaluation {
    /// ISO 8601 timestamp of the evaluation.
    pub timestamp: String,
    /// Brain session ID, if available.
    pub session_id: String,
    /// Which gate produced this evaluation.
    pub gate: PdpGate,
    /// Individual check results within the gate.
    pub checks: Vec<CheckResult>,
    /// Whether this evaluation blocked the prompt/response.
    pub blocked: bool,
    /// SHA-256 hash of the prompt text (for deduplication without storing content).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,
    /// Human-readable reason if blocked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Returns the default JSONL file path: `~/.claude/brain/telemetry/pdp_evaluations.jsonl`.
fn default_jsonl_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|e| DbError::Migration(format!("HOME environment variable not set: {e}")))?;
    Ok(PathBuf::from(home).join(".claude/brain/telemetry/pdp_evaluations.jsonl"))
}

/// Appends a single [`PdpEvaluation`] record to the JSONL file.
///
/// Creates the parent directory and file if they do not exist.
/// Each call serializes the record to one JSON line and appends it.
///
/// # Errors
///
/// Returns [`DbError::Migration`] if the file cannot be opened or written,
/// or [`DbError::Json`] if serialization fails.
pub fn write_evaluation(eval: &PdpEvaluation) -> Result<()> {
    let path = default_jsonl_path()?;
    write_evaluation_to(eval, &path)
}

/// Appends a [`PdpEvaluation`] to a specific file path (testable).
pub fn write_evaluation_to(eval: &PdpEvaluation, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            DbError::Migration(format!(
                "Cannot create telemetry directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    let mut line = serde_json::to_string(eval).map_err(DbError::Json)?;
    line.push('\n');

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| {
            DbError::Migration(format!("Cannot open {} for append: {e}", path.display()))
        })?;

    file.write_all(line.as_bytes())
        .map_err(|e| DbError::Migration(format!("Cannot write to {}: {e}", path.display())))?;

    Ok(())
}

/// Reads all [`PdpEvaluation`] records from the default JSONL file.
///
/// Skips blank lines and lines that fail to parse (logging a tracing warning).
/// Returns an empty vec if the file does not exist.
pub fn read_evaluations() -> Result<Vec<PdpEvaluation>> {
    let path = default_jsonl_path()?;
    read_evaluations_from(&path)
}

/// Reads all [`PdpEvaluation`] records from a specific file path (testable).
pub fn read_evaluations_from(path: &Path) -> Result<Vec<PdpEvaluation>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| DbError::Migration(format!("Cannot read {}: {e}", path.display())))?;

    let mut records = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<PdpEvaluation>(trimmed) {
            Ok(eval) => records.push(eval),
            Err(e) => {
                tracing::warn!("Skipping malformed PDP evaluation record: {e}");
            }
        }
    }

    Ok(records)
}

/// Counts the number of evaluations in the default JSONL file.
///
/// More efficient than `read_evaluations().len()` for large files —
/// only counts newlines without full deserialization.
pub fn count_evaluations() -> Result<usize> {
    let path = default_jsonl_path()?;
    count_evaluations_from(&path)
}

/// Counts evaluations in a specific file path (testable).
pub fn count_evaluations_from(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }

    let content = fs::read_to_string(path)
        .map_err(|e| DbError::Migration(format!("Cannot read {}: {e}", path.display())))?;

    let count = content.lines().filter(|l| !l.trim().is_empty()).count();

    Ok(count)
}

/// Reads evaluations filtered by gate type from the default JSONL file.
pub fn read_evaluations_by_gate(gate: &PdpGate) -> Result<Vec<PdpEvaluation>> {
    let all = read_evaluations()?;
    Ok(all.into_iter().filter(|e| &e.gate == gate).collect())
}

/// Reads evaluations filtered by gate type from a specific file (testable).
pub fn read_evaluations_by_gate_from(gate: &PdpGate, path: &Path) -> Result<Vec<PdpEvaluation>> {
    let all = read_evaluations_from(path)?;
    Ok(all.into_iter().filter(|e| &e.gate == gate).collect())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_jsonl_path() -> PathBuf {
        let dir = std::env::temp_dir().join("nexcore-db-pdp-test");
        fs::create_dir_all(&dir).ok();
        dir.join(format!(
            "pdp_eval_test_{}.jsonl",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn sample_evaluation(gate: PdpGate, blocked: bool) -> PdpEvaluation {
        PdpEvaluation {
            timestamp: "2026-02-28T12:00:00Z".into(),
            session_id: "test-session-001".into(),
            gate,
            checks: vec![
                CheckResult {
                    check_id: "G1".into(),
                    passed: true,
                    detail: None,
                },
                CheckResult {
                    check_id: "G2".into(),
                    passed: !blocked,
                    detail: if blocked {
                        Some("Too vague".into())
                    } else {
                        None
                    },
                },
            ],
            blocked,
            prompt_hash: Some("abc123def456".into()),
            reason: if blocked {
                Some("G2 specificity failed".into())
            } else {
                None
            },
        }
    }

    #[test]
    fn test_write_and_read_back() {
        let path = temp_jsonl_path();
        let eval = sample_evaluation(PdpGate::Foundation, false);

        write_evaluation_to(&eval, &path).unwrap();
        let records = read_evaluations_from(&path).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].session_id, "test-session-001");
        assert_eq!(records[0].gate, PdpGate::Foundation);
        assert!(!records[0].blocked);
        assert_eq!(records[0].checks.len(), 2);
        assert!(records[0].checks[0].passed);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_append_multiple() {
        let path = temp_jsonl_path();

        let eval1 = sample_evaluation(PdpGate::Foundation, false);
        let eval2 = sample_evaluation(PdpGate::Structural, true);
        let eval3 = sample_evaluation(PdpGate::Calibration, false);

        write_evaluation_to(&eval1, &path).unwrap();
        write_evaluation_to(&eval2, &path).unwrap();
        write_evaluation_to(&eval3, &path).unwrap();

        let records = read_evaluations_from(&path).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].gate, PdpGate::Foundation);
        assert_eq!(records[1].gate, PdpGate::Structural);
        assert!(records[1].blocked);
        assert_eq!(records[2].gate, PdpGate::Calibration);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_count_evaluations() {
        let path = temp_jsonl_path();

        assert_eq!(count_evaluations_from(&path).unwrap(), 0);

        write_evaluation_to(&sample_evaluation(PdpGate::Foundation, false), &path).unwrap();
        write_evaluation_to(&sample_evaluation(PdpGate::Structural, true), &path).unwrap();

        assert_eq!(count_evaluations_from(&path).unwrap(), 2);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_filter_by_gate() {
        let path = temp_jsonl_path();

        write_evaluation_to(&sample_evaluation(PdpGate::Foundation, false), &path).unwrap();
        write_evaluation_to(&sample_evaluation(PdpGate::Structural, true), &path).unwrap();
        write_evaluation_to(&sample_evaluation(PdpGate::Foundation, true), &path).unwrap();

        let foundation = read_evaluations_by_gate_from(&PdpGate::Foundation, &path).unwrap();
        assert_eq!(foundation.len(), 2);

        let structural = read_evaluations_by_gate_from(&PdpGate::Structural, &path).unwrap();
        assert_eq!(structural.len(), 1);
        assert!(structural[0].blocked);

        let calibration = read_evaluations_by_gate_from(&PdpGate::Calibration, &path).unwrap();
        assert_eq!(calibration.len(), 0);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_read_nonexistent_file() {
        let path = PathBuf::from("/tmp/nexcore-db-pdp-nonexistent-12345.jsonl");
        let records = read_evaluations_from(&path).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_skip_malformed_lines() {
        let path = temp_jsonl_path();

        // Write a valid record
        write_evaluation_to(&sample_evaluation(PdpGate::Foundation, false), &path).unwrap();

        // Append a malformed line directly
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"not valid json\n").unwrap();

        // Write another valid record
        write_evaluation_to(&sample_evaluation(PdpGate::Structural, true), &path).unwrap();

        // Should get 2 records, skipping the malformed one
        let records = read_evaluations_from(&path).unwrap();
        assert_eq!(records.len(), 2);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_serialization_roundtrip() {
        let eval = PdpEvaluation {
            timestamp: "2026-02-28T14:30:00Z".into(),
            session_id: "roundtrip-session".into(),
            gate: PdpGate::Calibration,
            checks: vec![
                CheckResult {
                    check_id: "C1".into(),
                    passed: false,
                    detail: Some("Missing evaluation criteria".into()),
                },
                CheckResult {
                    check_id: "C2".into(),
                    passed: true,
                    detail: None,
                },
                CheckResult {
                    check_id: "C3".into(),
                    passed: true,
                    detail: None,
                },
                CheckResult {
                    check_id: "C4".into(),
                    passed: false,
                    detail: Some("No decisive ending".into()),
                },
                CheckResult {
                    check_id: "C5".into(),
                    passed: true,
                    detail: None,
                },
            ],
            blocked: true,
            prompt_hash: None,
            reason: Some("C1 + C4 advisory".into()),
        };

        let json = serde_json::to_string(&eval).unwrap();
        let deserialized: PdpEvaluation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.session_id, "roundtrip-session");
        assert_eq!(deserialized.gate, PdpGate::Calibration);
        assert_eq!(deserialized.checks.len(), 5);
        assert!(!deserialized.checks[0].passed);
        assert!(deserialized.checks[1].passed);
        assert!(deserialized.blocked);
        assert!(deserialized.prompt_hash.is_none());
        assert_eq!(deserialized.reason.as_deref(), Some("C1 + C4 advisory"));
    }
}
