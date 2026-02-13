//! Report generation for ORGANIZE pipeline results.
//!
//! Produces human-readable markdown and machine-readable JSON reports.

use crate::pipeline::OrganizeResult2;

// ============================================================================
// Markdown Report
// ============================================================================

/// Generate a markdown report from pipeline results.
pub fn markdown_report(result: &OrganizeResult2) -> String {
    let mut out = String::new();

    out.push_str("# ORGANIZE Report\n\n");
    out.push_str(&format!("**Root:** `{}`\n\n", result.root.display()));
    out.push_str(&format!(
        "**Mode:** {}\n\n",
        if result.plan.dry_run {
            "Dry Run"
        } else {
            "Live"
        }
    ));

    // Integration summary
    out.push_str("## Integration\n\n");
    out.push_str(&format!("| Metric | Value |\n"));
    out.push_str(&format!("|--------|-------|\n"));
    out.push_str(&format!("| Total operations | {} |\n", result.plan.total));
    out.push_str(&format!("| Succeeded | {} |\n", result.plan.succeeded));
    out.push_str(&format!("| Failed | {} |\n", result.plan.failed));
    out.push_str(&format!(
        "| Bytes affected | {} |\n",
        format_bytes(result.plan.bytes_affected)
    ));

    // Operations by action
    out.push_str("\n### Operations\n\n");
    out.push_str("| Source | Action | Target |\n");
    out.push_str("|--------|--------|--------|\n");
    for op in &result.plan.operations {
        out.push_str(&format!(
            "| `{}` | {} | `{}` |\n",
            op.source.display(),
            op.action,
            op.target.display(),
        ));
    }

    // Cleanup summary
    out.push_str("\n## Cleanup\n\n");
    if result.cleanup.empty_dirs.is_empty() && result.cleanup.duplicates.is_empty() {
        out.push_str("No cleanup needed.\n");
    } else {
        if !result.cleanup.empty_dirs.is_empty() {
            out.push_str(&format!(
                "**Empty directories:** {}\n\n",
                result.cleanup.empty_dirs.len()
            ));
            for dir in &result.cleanup.empty_dirs {
                out.push_str(&format!("- `{}`\n", dir.display()));
            }
            out.push('\n');
        }

        if !result.cleanup.duplicates.is_empty() {
            out.push_str(&format!(
                "**Duplicate groups:** {} (wasted: {})\n\n",
                result.cleanup.duplicates.len(),
                format_bytes(result.cleanup.total_wasted_bytes),
            ));
            for group in &result.cleanup.duplicates {
                out.push_str(&format!(
                    "- Hash `{}..` ({} files, {} each)\n",
                    &group.hash[..8_usize.min(group.hash.len())],
                    group.paths.len(),
                    format_bytes(group.size_bytes),
                ));
                for path in &group.paths {
                    out.push_str(&format!("  - `{}`\n", path.display()));
                }
            }
        }
    }

    // State snapshot
    out.push_str("\n## State\n\n");
    out.push_str(&format!("**Files tracked:** {}\n", result.state.count));
    out.push_str(&format!("**Snapshot:** {}\n", result.state.timestamp));

    // Drift report
    if let Some(ref drift) = result.drift {
        out.push_str("\n## Drift Detection\n\n");
        if drift.has_drift {
            out.push_str(&format!(
                "**Changes detected:** {}\n\n",
                drift.change_count()
            ));
            if !drift.added.is_empty() {
                out.push_str(&format!("**Added ({}):**\n", drift.added.len()));
                for f in &drift.added {
                    out.push_str(&format!("- `{f}`\n"));
                }
                out.push('\n');
            }
            if !drift.removed.is_empty() {
                out.push_str(&format!("**Removed ({}):**\n", drift.removed.len()));
                for f in &drift.removed {
                    out.push_str(&format!("- `{f}`\n"));
                }
                out.push('\n');
            }
            if !drift.modified.is_empty() {
                out.push_str(&format!("**Modified ({}):**\n", drift.modified.len()));
                for f in &drift.modified {
                    out.push_str(&format!("- `{f}`\n"));
                }
            }
        } else {
            out.push_str("No drift detected since last snapshot.\n");
        }
    }

    out
}

// ============================================================================
// JSON Report
// ============================================================================

/// Generate a JSON report from pipeline results.
pub fn json_report(result: &OrganizeResult2) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(result)
}

// ============================================================================
// Formatting Helpers
// ============================================================================

/// Format bytes into human-readable form.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_b() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_format_bytes_kb() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }

    #[test]
    fn test_format_bytes_mb() {
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
    }

    #[test]
    fn test_format_bytes_gb() {
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_markdown_report_contains_header() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let config = crate::config::OrganizeConfig::default_for(dir.path());
            let pipeline = crate::pipeline::OrganizePipeline::new(config);
            let result = pipeline.run();
            if let Ok(result) = result {
                let report = markdown_report(&result);
                assert!(report.contains("# ORGANIZE Report"));
                assert!(report.contains("Dry Run"));
                assert!(report.contains("## Integration"));
                assert!(report.contains("## State"));
            }
        }
    }

    #[test]
    fn test_json_report_parses() {
        let tmp = tempfile::tempdir().ok();
        if let Some(ref dir) = tmp {
            let config = crate::config::OrganizeConfig::default_for(dir.path());
            let pipeline = crate::pipeline::OrganizePipeline::new(config);
            let result = pipeline.run();
            if let Ok(result) = result {
                let json = json_report(&result);
                assert!(json.is_ok());
                if let Ok(json) = json {
                    // Verify it's valid JSON
                    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
                    assert!(parsed.is_ok());
                }
            }
        }
    }
}
