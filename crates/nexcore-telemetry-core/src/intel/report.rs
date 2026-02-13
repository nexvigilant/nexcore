//! Intelligence report structures and generation.
//!
//! Aggregates telemetry data into actionable intelligence reports
//! for understanding external assistant behavior patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{Snapshot, Source, TokenUsage};

/// File access pattern detected across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessPattern {
    /// Path to the file
    pub path: String,
    /// Number of read operations
    pub read_count: usize,
    /// Number of write operations
    pub write_count: usize,
    /// First access timestamp
    pub first_access: DateTime<Utc>,
    /// Most recent access timestamp
    pub last_access: DateTime<Utc>,
    /// Source IDs that accessed this file
    pub source_ids: Vec<String>,
}

/// Governance-related file access tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAccess {
    /// Category of governance file (e.g., "codex", "constitution", "skill")
    pub category: String,
    /// Specific file or resource accessed
    pub resource: String,
    /// Number of accesses
    pub access_count: usize,
    /// Source IDs that accessed this resource
    pub source_ids: Vec<String>,
    /// Timestamps of accesses
    pub timestamps: Vec<DateTime<Utc>>,
}

/// Summary of activity for a specific source session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    /// When this activity summary was created
    pub timestamp: DateTime<Utc>,
    /// Source session identifier
    pub source_id: String,
    /// Total number of operations in this session
    pub operation_count: usize,
    /// Number of unique files touched
    pub files_touched: usize,
}

/// Comprehensive intelligence report from telemetry analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelReport {
    /// Timestamp when this report was generated
    pub generated_at: DateTime<Utc>,
    /// Number of source sessions analyzed
    pub sources_analyzed: usize,
    /// Aggregated token usage across all sources
    pub total_tokens: TokenUsage,
    /// File access patterns detected
    pub file_access_patterns: Vec<FileAccessPattern>,
    /// Governance-related access tracking
    pub governance_access: Vec<GovernanceAccess>,
    /// Per-source activity summaries
    pub recent_activity: Vec<ActivitySummary>,
}

impl Default for IntelReport {
    fn default() -> Self {
        Self {
            generated_at: Utc::now(),
            sources_analyzed: 0,
            total_tokens: TokenUsage::default(),
            file_access_patterns: Vec::new(),
            governance_access: Vec::new(),
            recent_activity: Vec::new(),
        }
    }
}

/// Known governance file patterns for classification.
const GOVERNANCE_PATTERNS: &[(&str, &str)] = &[
    ("codex", "primitive-codex"),
    ("codex", "CODEX"),
    ("constitution", "constitution"),
    ("constitution", "CONSTITUTION"),
    ("skill", ".claude/skills/"),
    ("skill", "SKILL.md"),
    ("brain", ".claude/brain/"),
    ("implicit", ".claude/implicit/"),
    ("hooks", "nexcore-hooks"),
    ("vigilance", "nexcore-vigilance"),
];

/// Classify a file path into a governance category and matched resource (allocation-free).
fn classify_governance(path: &str) -> Option<(&'static str, &'static str)> {
    GOVERNANCE_PATTERNS.iter().find_map(|(category, pattern)| {
        if path.contains(pattern) {
            Some((*category, *pattern))
        } else {
            None
        }
    })
}

/// Generate an intelligence report from telemetry sources and snapshots.
///
/// Aggregates data from multiple source sessions and their snapshots
/// to produce a comprehensive intelligence report.
///
/// # Arguments
///
/// * `sources` - Slice of source sessions to analyze
/// * `snapshots` - Slice of snapshots to include in analysis
///
/// # Returns
///
/// A complete `IntelReport` with aggregated metrics and patterns.
#[must_use]
pub fn generate_report(sources: &[Source], snapshots: &[Snapshot]) -> IntelReport {
    let mut report = IntelReport {
        generated_at: Utc::now(),
        sources_analyzed: sources.len(),
        ..Default::default()
    };

    // Track file access patterns: path -> (reads, writes, first, last, sources)
    let mut file_patterns: HashMap<
        String,
        (usize, usize, DateTime<Utc>, DateTime<Utc>, HashSet<String>),
    > = HashMap::new();

    // Track governance access: (category, resource) -> (count, sources, timestamps)
    let mut governance_map: HashMap<
        (&'static str, String),
        (usize, HashSet<String>, Vec<DateTime<Utc>>),
    > = HashMap::new();

    let mut total_tokens = TokenUsage::default();

    // Process each source session
    for source in sources {
        process_source_session(
            source,
            &mut total_tokens,
            &mut file_patterns,
            &mut governance_map,
            &mut report.recent_activity,
        );
    }

    // Include snapshot files in analysis
    for snapshot in snapshots {
        process_snapshot(snapshot, &mut governance_map);
    }

    report.total_tokens = total_tokens;

    // Convert file patterns to sorted vec
    report.file_access_patterns = finalize_file_patterns(file_patterns);

    // Convert governance map to sorted vec
    report.governance_access = finalize_governance_access(governance_map);

    // Sort recent activity by timestamp descending (most recent first)
    report
        .recent_activity
        .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    report
}

/// Helper to process a single source session.
fn process_source_session(
    source: &Source,
    total_tokens: &mut TokenUsage,
    file_patterns: &mut HashMap<
        String,
        (usize, usize, DateTime<Utc>, DateTime<Utc>, HashSet<String>),
    >,
    governance_map: &mut HashMap<
        (&'static str, String),
        (usize, HashSet<String>, Vec<DateTime<Utc>>),
    >,
    recent_activity: &mut Vec<ActivitySummary>,
) {
    let source_tokens = source.total_tokens();
    total_tokens.input += source_tokens.input;
    total_tokens.output += source_tokens.output;
    total_tokens.cached += source_tokens.cached;
    total_tokens.thoughts += source_tokens.thoughts;
    total_tokens.tool += source_tokens.tool;
    total_tokens.total += source_tokens.total;

    let mut files_touched_set: HashSet<String> = HashSet::new();
    let mut operation_count = 0;

    // Process operations from all entries
    for entry in &source.messages {
        for op in &entry.operations {
            operation_count += 1;
            process_intel_operation(
                op,
                source,
                file_patterns,
                governance_map,
                &mut files_touched_set,
            );
        }
    }

    // Create activity summary for this source
    recent_activity.push(ActivitySummary {
        timestamp: source.last_updated,
        source_id: source.id.clone(),
        operation_count,
        files_touched: files_touched_set.len(),
    });
}

/// Helper to process a single intelligence operation.
fn process_intel_operation(
    op: &crate::types::Operation,
    source: &Source,
    file_patterns: &mut HashMap<
        String,
        (usize, usize, DateTime<Utc>, DateTime<Utc>, HashSet<String>),
    >,
    governance_map: &mut HashMap<
        (&'static str, String),
        (usize, HashSet<String>, Vec<DateTime<Utc>>),
    >,
    files_touched_set: &mut HashSet<String>,
) {
    if let Some(path) = op.file_path() {
        files_touched_set.insert(path.clone());

        let is_write = matches!(op.activity_type(), crate::ActivityType::FileWrite);

        let entry = file_patterns
            .entry(path.clone())
            .or_insert_with(|| (0, 0, op.timestamp, op.timestamp, HashSet::new()));

        if is_write {
            entry.1 += 1;
        } else {
            entry.0 += 1;
        }

        if op.timestamp < entry.2 {
            entry.2 = op.timestamp;
        }
        if op.timestamp > entry.3 {
            entry.3 = op.timestamp;
        }

        entry.4.insert(source.id.clone());

        // Check for governance access
        if let Some((category, resource)) = classify_governance(&path) {
            let gov_entry = governance_map
                .entry((category, resource.to_string()))
                .or_insert_with(|| (0, HashSet::new(), Vec::new()));
            gov_entry.0 += 1;
            gov_entry.1.insert(source.id.clone());
            gov_entry.2.push(op.timestamp);
        }
    }
}

/// Helper to process a single snapshot.
fn process_snapshot(
    snapshot: &Snapshot,
    governance_map: &mut HashMap<
        (&'static str, String),
        (usize, HashSet<String>, Vec<DateTime<Utc>>),
    >,
) {
    let path = snapshot.path.to_string_lossy().to_string();
    if let Some((category, resource)) = classify_governance(&path) {
        let gov_entry = governance_map
            .entry((category, resource.to_string()))
            .or_insert_with(|| (0, HashSet::new(), Vec::new()));
        gov_entry.0 += 1;
        gov_entry.1.insert(snapshot.session_id.clone());
        if let Some(meta) = &snapshot.metadata {
            gov_entry.2.push(meta.updated_at);
        }
    }
}

/// Helper to finalize file patterns list.
fn finalize_file_patterns(
    file_patterns: HashMap<String, (usize, usize, DateTime<Utc>, DateTime<Utc>, HashSet<String>)>,
) -> Vec<FileAccessPattern> {
    let mut patterns: Vec<FileAccessPattern> = file_patterns
        .into_iter()
        .map(
            |(path, (reads, writes, first, last, sources))| FileAccessPattern {
                path,
                read_count: reads,
                write_count: writes,
                first_access: first,
                last_access: last,
                source_ids: sources.into_iter().collect(),
            },
        )
        .collect();

    // Sort by total access count descending
    patterns.sort_by(|a, b| {
        let total_a = a.read_count + a.write_count;
        let total_b = b.read_count + b.write_count;
        total_b.cmp(&total_a)
    });
    patterns
}

/// Helper to finalize governance access list.
fn finalize_governance_access(
    governance_map: HashMap<(&'static str, String), (usize, HashSet<String>, Vec<DateTime<Utc>>)>,
) -> Vec<GovernanceAccess> {
    let mut access: Vec<GovernanceAccess> = governance_map
        .into_iter()
        .map(
            |((category, resource), (count, sources, timestamps))| GovernanceAccess {
                category: category.to_string(),
                resource,
                access_count: count,
                source_ids: sources.into_iter().collect(),
                timestamps,
            },
        )
        .collect();

    // Sort governance by access count descending
    access.sort_by(|a, b| b.access_count.cmp(&a.access_count));
    access
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_governance_codex() {
        let result = classify_governance("/home/user/.claude/knowledge/primitive-codex/SKILL.md");
        assert!(result.is_some());
        let (category, _) = result.expect("Should match codex pattern");
        assert_eq!(category, "codex");
    }

    #[test]
    fn test_classify_governance_skill() {
        let result = classify_governance("/home/user/.claude/skills/my-skill/SKILL.md");
        assert!(result.is_some());
        let (category, _) = result.expect("Should match skill pattern");
        assert_eq!(category, "skill");
    }

    #[test]
    fn test_classify_governance_none() {
        let result = classify_governance("/home/user/some/random/file.rs");
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_report() {
        let report = generate_report(&[], &[]);
        assert_eq!(report.sources_analyzed, 0);
        assert_eq!(report.total_tokens.total, 0);
        assert!(report.file_access_patterns.is_empty());
        assert!(report.governance_access.is_empty());
        assert!(report.recent_activity.is_empty());
    }

    #[test]
    fn test_intel_report_default() {
        let report = IntelReport::default();
        assert_eq!(report.sources_analyzed, 0);
        assert!(report.file_access_patterns.is_empty());
    }
}
