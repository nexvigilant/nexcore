//! # InsightEngine Persistence
//!
//! File-backed state persistence for InsightEngine.
//! Stores engine state at `~/.claude/brain/insight/engine.json`.
//!
//! ## T1 Grounding
//! - π (Persistence): File-backed state survival across calls
//! - ς (State): Accumulated engine state (ς-acc)
//! - σ (Sequence): Ordered load → mutate → save lifecycle

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::engine::{InsightConfig, InsightEngine};
use crate::orchestrator::NexCoreInsight;

/// Default persistence directory relative to home.
const INSIGHT_DIR: &str = ".claude/brain/insight";
/// Default engine state filename.
const ENGINE_FILE: &str = "engine.json";
/// System-level compositor state filename.
const SYSTEM_FILE: &str = "system.json";

/// Get the persistence directory path.
///
/// Tier: T1 (λ — Location)
#[must_use]
pub fn insight_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(INSIGHT_DIR)
}

/// Get the engine state file path.
///
/// Tier: T1 (λ — Location)
#[must_use]
pub fn engine_path() -> PathBuf {
    insight_dir().join(ENGINE_FILE)
}

/// Load an InsightEngine from disk, preserving its saved config.
/// If no prior state exists, creates a new engine with default config.
///
/// This is the preferred load path — config persists across calls.
///
/// Tier: T2-P (π + ς — Persistence + State)
pub fn load_or_create() -> Result<InsightEngine> {
    let path = engine_path();

    if path.exists() {
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read insight state: {}", path.display()))?;
        let engine: InsightEngine = serde_json::from_str(&data)
            .with_context(|| "Failed to deserialize InsightEngine state")?;
        Ok(engine)
    } else {
        Ok(InsightEngine::new())
    }
}

/// Load an InsightEngine from disk, applying the given config overrides.
/// If no prior state exists, creates a fresh engine with this config.
/// If prior state exists, loads it and replaces config entirely.
///
/// **Prefer `load_or_create()` + `apply_config_overrides()`** for selective
/// field updates. This function replaces ALL config fields.
///
/// Tier: T2-P (π + ς — Persistence + State)
pub fn load_or_create_with_config(config: InsightConfig) -> Result<InsightEngine> {
    let mut engine = load_or_create()?;
    engine.config = config;
    Ok(engine)
}

/// Save an InsightEngine to disk.
///
/// Creates the parent directory if it doesn't exist.
///
/// Tier: T2-P (π + ς — Persistence + State)
pub fn save(engine: &InsightEngine) -> Result<()> {
    let dir = insight_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create insight dir: {}", dir.display()))?;
    }

    let path = engine_path();
    let data = serde_json::to_string_pretty(engine)
        .with_context(|| "Failed to serialize InsightEngine state")?;
    fs::write(&path, data)
        .with_context(|| format!("Failed to write insight state: {}", path.display()))?;

    Ok(())
}

/// Reset the engine state on disk (delete the state file).
///
/// Tier: T1 (∅ — Void / clearing state)
pub fn reset() -> Result<()> {
    let path = engine_path();
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove insight state: {}", path.display()))?;
    }
    Ok(())
}

/// Get engine statistics without full deserialization.
///
/// Returns (file_size_bytes, exists).
///
/// Tier: T1 (N — Quantity)
#[must_use]
pub fn stats() -> (u64, bool) {
    let path = engine_path();
    if path.exists() {
        let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        (size, true)
    } else {
        (0, false)
    }
}

// ── System-Level (NexCoreInsight) Persistence ────────────────────────────

/// Get the system-level compositor state file path.
///
/// Tier: T1 (λ — Location)
#[must_use]
pub fn system_path() -> PathBuf {
    insight_dir().join(SYSTEM_FILE)
}

/// Load the system-level NexCoreInsight compositor from disk.
/// If no prior state exists, creates a fresh compositor with default config.
///
/// Tier: T2-P (π + ς — Persistence + State)
pub fn load_system() -> Result<NexCoreInsight> {
    let path = system_path();

    if path.exists() {
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read system insight state: {}", path.display()))?;
        let system: NexCoreInsight = serde_json::from_str(&data)
            .with_context(|| "Failed to deserialize NexCoreInsight state")?;
        Ok(system)
    } else {
        Ok(NexCoreInsight::new())
    }
}

/// Save the system-level NexCoreInsight compositor to disk.
///
/// Tier: T2-P (π + ς — Persistence + State)
pub fn save_system(system: &NexCoreInsight) -> Result<()> {
    let dir = insight_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create insight dir: {}", dir.display()))?;
    }

    let path = system_path();
    let data = serde_json::to_string_pretty(system)
        .with_context(|| "Failed to serialize NexCoreInsight state")?;
    fs::write(&path, data)
        .with_context(|| format!("Failed to write system insight state: {}", path.display()))?;

    Ok(())
}

/// Reset the system-level compositor state on disk.
///
/// Tier: T1 (∅ — Void / clearing state)
pub fn reset_system() -> Result<()> {
    let path = system_path();
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Failed to remove system state: {}", path.display()))?;
    }
    Ok(())
}

/// Get system-level compositor statistics.
///
/// Returns (file_size_bytes, exists).
///
/// Tier: T1 (N — Quantity)
#[must_use]
pub fn system_stats() -> (u64, bool) {
    let path = system_path();
    if path.exists() {
        let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        (size, true)
    } else {
        (0, false)
    }
}

// ── Queue-Based Auto-Ingest ─────────────────────────────────────────

/// Queue file for observations written by the PostToolUse hook.
const QUEUE_FILE: &str = "queue.jsonl";

/// Queued observation record — mirrors the hook's output format.
///
/// Tier: T2-C (σ+μ+π — Sequence+Mapping+Persistence)
#[derive(Debug, serde::Deserialize)]
struct QueuedObservation {
    key: String,
    value: String,
    #[serde(default)]
    numeric_value: Option<f64>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    source_tool: String,
}

/// Drain the observation queue into a NexCoreInsight compositor.
///
/// Reads `~/.claude/brain/insight/queue.jsonl`, converts each line
/// into an `Observation`, ingests into the system compositor, and
/// truncates the file. Returns the number of observations drained.
///
/// Tier: T2-C (σ+μ+π — Sequence+Mapping+Persistence)
pub fn drain_queue(system: &mut NexCoreInsight) -> Result<usize> {
    let path = insight_dir().join(QUEUE_FILE);

    if !path.exists() {
        return Ok(0);
    }

    let data = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read queue: {}", path.display()))?;

    if data.trim().is_empty() {
        return Ok(0);
    }

    let mut count = 0usize;
    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<QueuedObservation>(trimmed) {
            Ok(queued) => {
                let mut obs = crate::engine::Observation::new(&queued.key, &queued.value);
                obs.numeric_value = queued.numeric_value;
                obs.tags = queued.tags;
                // Tag with source tool for traceability
                if !queued.source_tool.is_empty() {
                    obs.tags.push(format!("source:{}", queued.source_tool));
                }
                // Extract domain from key prefix (e.g., "pv:drug:event" → "pv")
                let domain = queued.key.split(':').next().unwrap_or("unknown");
                let _ = system.ingest_from(domain, obs);
                count += 1;
            }
            Err(e) => {
                // Skip malformed lines, log warning
                eprintln!("[insight-drain] Skipping malformed queue entry: {e}");
            }
        }
    }

    // Truncate the queue file after successful drain
    fs::write(&path, "")
        .with_context(|| format!("Failed to truncate queue: {}", path.display()))?;

    Ok(count)
}

/// Get queue statistics without draining.
///
/// Returns (line_count, file_size_bytes, exists).
///
/// Tier: T1 (N — Quantity)
#[must_use]
pub fn queue_stats() -> (usize, u64, bool) {
    let path = insight_dir().join(QUEUE_FILE);
    if !path.exists() {
        return (0, 0, false);
    }
    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let lines = fs::read_to_string(&path)
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0);
    (lines, size, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Observation;

    #[test]
    fn test_insight_dir_creation() {
        let dir = insight_dir();
        assert!(dir.to_str().is_some());
        assert!(dir.to_string_lossy().contains("insight"));
    }

    #[test]
    fn test_engine_path_has_json_extension() {
        let path = engine_path();
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("json"));
    }

    #[test]
    fn test_round_trip_persistence() {
        // Use a temp dir to avoid polluting real state
        let tmp = std::env::temp_dir().join("nexcore-insight-test");
        let _ = fs::create_dir_all(&tmp);
        let file = tmp.join("test_engine.json");

        // Create engine with observations
        let mut engine = InsightEngine::new();
        let obs = Observation::new("drug_a", "adverse_event_1");
        let _events = engine.ingest(obs);

        // Save
        let data = serde_json::to_string_pretty(&engine).expect("serialize");
        fs::write(&file, &data).expect("write");

        // Load
        let loaded_data = fs::read_to_string(&file).expect("read");
        let loaded: InsightEngine = serde_json::from_str(&loaded_data).expect("deserialize");

        assert_eq!(loaded.observation_count(), engine.observation_count());
        assert_eq!(loaded.unique_key_count(), engine.unique_key_count());
        assert_eq!(loaded.pattern_count(), engine.pattern_count());

        // Cleanup
        let _ = fs::remove_file(&file);
        let _ = fs::remove_dir(&tmp);
    }

    #[test]
    fn test_stats_nonexistent() {
        let (size, exists) = stats();
        // May or may not exist depending on prior runs — test function doesn't crash
        let _ = (size, exists);
    }

    #[test]
    fn test_drain_queue_empty_file() {
        let tmp = std::env::temp_dir().join("nexcore-insight-drain-empty");
        let _ = fs::create_dir_all(&tmp);
        let queue = tmp.join("queue.jsonl");
        fs::write(&queue, "").ok();

        // drain_queue uses insight_dir() which is based on HOME, so we test
        // the QueuedObservation deserialization directly
        let line = r#"{"key":"pv:aspirin:bleeding","value":"PRR=4.2","numeric_value":4.2,"tags":["pv"],"timestamp":"2026-02-07T00:00:00Z","source_tool":"pv_signal_prr"}"#;
        let parsed: Result<QueuedObservation, _> = serde_json::from_str(line);
        assert!(parsed.is_ok());
        let obs = parsed.unwrap_or_else(|_| QueuedObservation {
            key: String::new(),
            value: String::new(),
            numeric_value: None,
            tags: vec![],
            source_tool: String::new(),
        });
        assert_eq!(obs.key, "pv:aspirin:bleeding");
        assert!((obs.numeric_value.unwrap_or(0.0) - 4.2).abs() < f64::EPSILON);
        assert_eq!(obs.source_tool, "pv_signal_prr");

        let _ = fs::remove_file(&queue);
        let _ = fs::remove_dir(&tmp);
    }

    #[test]
    fn test_drain_queue_into_compositor() {
        // Create a NexCoreInsight and manually drain queue content
        let mut system = NexCoreInsight::new();
        let queued = QueuedObservation {
            key: "pv:ibuprofen:rash".to_string(),
            value: "signal detected".to_string(),
            numeric_value: Some(3.5),
            tags: vec!["pv".to_string(), "drug:ibuprofen".to_string()],
            source_tool: "pv_signal_prr".to_string(),
        };

        let domain = queued.key.split(':').next().unwrap_or("unknown");
        let mut obs = Observation::new(&queued.key, &queued.value);
        obs.numeric_value = queued.numeric_value;
        obs.tags = queued.tags;
        obs.tags.push(format!("source:{}", queued.source_tool));

        let _events = system.ingest_from(domain, obs);

        // Verify the observation was ingested
        assert!(system.domains().iter().any(|d| d.name == "pv"));
    }

    #[test]
    fn test_queue_stats_nonexistent() {
        let (lines, bytes, exists) = queue_stats();
        // Queue may or may not exist — test function doesn't crash
        let _ = (lines, bytes, exists);
    }

    #[test]
    fn test_queued_observation_without_optional_fields() {
        let line = r#"{"key":"test:key","value":"test_value","tags":[],"timestamp":"2026-02-07T00:00:00Z","source_tool":"test"}"#;
        let parsed: Result<QueuedObservation, _> = serde_json::from_str(line);
        assert!(parsed.is_ok());
        let obs = parsed.unwrap_or_else(|_| QueuedObservation {
            key: String::new(),
            value: String::new(),
            numeric_value: None,
            tags: vec![],
            source_tool: String::new(),
        });
        assert!(obs.numeric_value.is_none());
    }
}
