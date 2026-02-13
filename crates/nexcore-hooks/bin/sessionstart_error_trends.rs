//! SessionStart Error Trends Hook
//!
//! Reads `error_index.jsonl` and computes frequency, trending, top-N patterns.
//!
//! # Event
//! SessionStart
//!
//! # Input
//! `~/.claude/debug/error_index.jsonl`
//!
//! # Output
//! `~/.claude/debug/error_trends.json`
//!
//! # Exit Codes
//! - 0: Always (best-effort, never blocks)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct ErrorEntry {
    timestamp: String,
    session: String,
    category: String,
    #[allow(dead_code)]
    message: String,
    #[allow(dead_code)]
    count: u32,
}

#[derive(Serialize)]
struct TrendReport {
    generated: String,
    sessions_analyzed: usize,
    total_errors: usize,
    errors_per_session: f64,
    top_categories: Vec<CategoryTrend>,
    new_errors: Vec<String>,
    recommendations: Vec<String>,
}

#[derive(Serialize)]
struct CategoryTrend {
    category: String,
    count: usize,
    trend: String,
}

fn main() {
    let debug_dir = debug_dir_path();
    let index_path = debug_dir.join("error_index.jsonl");

    if !index_path.exists() {
        std::process::exit(0);
    }

    let entries = load_index(&index_path);
    if entries.is_empty() {
        std::process::exit(0);
    }

    let report = build_report(&entries);
    write_report(&debug_dir.join("error_trends.json"), &report);
    emit_warnings(&report);

    std::process::exit(0);
}

fn debug_dir_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("debug")
}

fn load_index(path: &Path) -> Vec<ErrorEntry> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect()
}

fn build_report(entries: &[ErrorEntry]) -> TrendReport {
    let sessions: HashSet<&str> = entries.iter().map(|e| e.session.as_str()).collect();
    let session_count = sessions.len().max(1);

    let category_counts = count_categories(entries);
    let category_trends = compute_trends(entries, &category_counts);
    let new_errors = detect_new_categories(entries);
    let recommendations = generate_recommendations(&category_trends, &new_errors);

    let generated = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    TrendReport {
        generated,
        sessions_analyzed: session_count,
        total_errors: entries.len(),
        errors_per_session: entries.len() as f64 / session_count as f64,
        top_categories: category_trends,
        new_errors,
        recommendations,
    }
}

fn count_categories(entries: &[ErrorEntry]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        *counts.entry(entry.category.clone()).or_default() += 1;
    }
    counts
}

fn compute_trends(
    entries: &[ErrorEntry],
    totals: &HashMap<String, usize>,
) -> Vec<CategoryTrend> {
    // Split entries into halves by position to detect trend direction
    let midpoint = entries.len() / 2;
    let first_half = &entries[..midpoint.max(1)];
    let second_half = &entries[midpoint..];

    let first_counts = count_categories(first_half);
    let second_counts = count_categories(second_half);

    let mut trends: Vec<CategoryTrend> = totals
        .iter()
        .map(|(cat, total)| {
            let first = *first_counts.get(cat).unwrap_or(&0) as f64;
            let second = *second_counts.get(cat).unwrap_or(&0) as f64;
            let trend = trend_label(first, second);

            CategoryTrend {
                category: cat.clone(),
                count: *total,
                trend,
            }
        })
        .collect();

    // Sort by count descending
    trends.sort_by(|a, b| b.count.cmp(&a.count));
    trends.truncate(10);
    trends
}

fn trend_label(first: f64, second: f64) -> String {
    if first == 0.0 && second > 0.0 {
        return "new".to_string();
    }
    if first == 0.0 {
        return "stable".to_string();
    }

    let ratio = second / first;
    if ratio > 1.5 {
        "increasing".to_string()
    } else if ratio < 0.5 {
        "decreasing".to_string()
    } else {
        "stable".to_string()
    }
}

fn detect_new_categories(entries: &[ErrorEntry]) -> Vec<String> {
    // Categories that appear only in the last 20% of entries
    let threshold = entries.len() * 4 / 5;
    let early: HashSet<&str> = entries[..threshold.max(1)]
        .iter()
        .map(|e| e.category.as_str())
        .collect();
    let late: HashSet<&str> = entries[threshold..]
        .iter()
        .map(|e| e.category.as_str())
        .collect();

    late.difference(&early).map(|s| (*s).to_string()).collect()
}

fn generate_recommendations(trends: &[CategoryTrend], new_errors: &[String]) -> Vec<String> {
    let mut recs = Vec::new();

    for trend in trends {
        if trend.trend != "increasing" {
            continue;
        }
        let rec = match trend.category.as_str() {
            "mcp_disconnect" => "mcp_disconnect trending up: check MCP server stability",
            "agent_failure" => "agent_failure trending up: review subagent configurations",
            "timeout" => "timeout trending up: consider increasing hook timeout thresholds",
            "permission_denied" => "permission_denied trending up: audit file permissions",
            "executable_not_found" => "executable_not_found trending up: verify binary paths",
            _ => continue,
        };
        recs.push(rec.to_string());
    }

    for cat in new_errors {
        recs.push(format!("New error category '{cat}' detected: flag for review"));
    }

    recs
}

fn write_report(path: &Path, report: &TrendReport) {
    match serde_json::to_string_pretty(report) {
        Ok(json) => {
            if let Err(e) = fs::write(path, json) {
                eprintln!("[error-trends] Cannot write report: {e}");
            }
        }
        Err(e) => eprintln!("[error-trends] Serialize error: {e}"),
    }
}

fn emit_warnings(report: &TrendReport) {
    let increasing: Vec<&CategoryTrend> = report
        .top_categories
        .iter()
        .filter(|t| t.trend == "increasing")
        .collect();

    if increasing.is_empty() && report.new_errors.is_empty() {
        return;
    }

    eprintln!(
        "[error-trends] {} errors across {} sessions ({:.2}/session)",
        report.total_errors, report.sessions_analyzed, report.errors_per_session
    );

    for trend in increasing {
        eprintln!(
            "[error-trends] ⚠ {} is INCREASING ({} occurrences)",
            trend.category, trend.count
        );
    }

    for cat in &report.new_errors {
        eprintln!("[error-trends] 🆕 New category: {cat}");
    }
}
