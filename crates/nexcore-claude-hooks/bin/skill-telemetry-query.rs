//! Skill Telemetry Query - CLI Tool
//!
//! Analyzes skill invocation telemetry from JSONL file.
//!
//! # Usage
//!
//! ```bash
//! skill-telemetry-query              # Full summary
//! skill-telemetry-query --top 10     # Top 10 skills by invocation count
//! skill-telemetry-query --skill foo  # Stats for specific skill
//! skill-telemetry-query --json       # Output as JSON
//! ```
//!
//! # Codex Compliance
//!
//! - **Tier**: T2-C (Cross-Domain Composite)
//! - **Grounding**: Types ground to T1 via metrics aggregation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Telemetry JSONL file location
const TELEMETRY_FILE: &str = "/home/matthew/.claude/brain/telemetry/skill_invocations.jsonl";

/// Skill invocation record (must match collector schema).
#[derive(Debug, Deserialize)]
struct SkillInvocationRecord {
    #[allow(dead_code)]
    timestamp: DateTime<Utc>,
    skill: String,
    #[allow(dead_code)]
    args: String,
    success: bool,
    duration_ms: Option<u64>,
    #[allow(dead_code)]
    session_id: String,
}

/// Aggregated metrics for a single skill.
#[derive(Debug, Default, Serialize)]
struct SkillMetrics {
    invocations: u64,
    successes: u64,
    failures: u64,
    total_duration_ms: u64,
    duration_count: u64,
}

impl SkillMetrics {
    fn success_rate(&self) -> f64 {
        if self.invocations == 0 {
            return 0.0;
        }
        (self.successes as f64 / self.invocations as f64) * 100.0
    }

    fn avg_duration_ms(&self) -> Option<f64> {
        if self.duration_count == 0 {
            return None;
        }
        Some(self.total_duration_ms as f64 / self.duration_count as f64)
    }

    fn record(&mut self, record: &SkillInvocationRecord) {
        self.invocations += 1;
        if record.success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
        if let Some(d) = record.duration_ms {
            self.total_duration_ms += d;
            self.duration_count += 1;
        }
    }
}

/// Full telemetry summary.
#[derive(Debug, Serialize)]
struct TelemetrySummary {
    total_invocations: u64,
    total_skills: usize,
    overall_success_rate: f64,
    skills: Vec<SkillSummary>,
}

/// Per-skill summary for output.
#[derive(Debug, Serialize)]
struct SkillSummary {
    skill: String,
    invocations: u64,
    success_rate: f64,
    avg_duration_ms: Option<f64>,
}

/// CLI configuration.
struct Config {
    top_n: Option<usize>,
    skill_filter: Option<String>,
    json_output: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(&args);

    let records = load_records_or_exit();
    if records.is_empty() {
        println!("No skill invocation telemetry found.");
        return;
    }

    let metrics = aggregate_metrics(&records);
    let summary = build_summary(&metrics, config.top_n);
    output_summary(&summary, &config);
}

/// Load records or exit on error.
fn load_records_or_exit() -> Vec<SkillInvocationRecord> {
    match load_records() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error loading telemetry: {e}");
            std::process::exit(1);
        }
    }
}

/// Output summary based on config.
fn output_summary(summary: &TelemetrySummary, config: &Config) {
    if config.json_output {
        print_json(summary, &config.skill_filter);
    } else {
        print_text(summary, &config.skill_filter);
    }
}

/// Parse command line arguments.
fn parse_args(args: &[String]) -> Config {
    let mut config = Config {
        top_n: None,
        skill_filter: None,
        json_output: false,
    };

    let mut i = 1;
    while i < args.len() {
        config = parse_single_arg(&config, args, &mut i);
        i += 1;
    }
    config
}

/// Parse a single argument.
fn parse_single_arg(config: &Config, args: &[String], i: &mut usize) -> Config {
    let mut new_config = Config {
        top_n: config.top_n,
        skill_filter: config.skill_filter.clone(),
        json_output: config.json_output,
    };

    match args[*i].as_str() {
        "--top" if *i + 1 < args.len() => {
            new_config.top_n = args[*i + 1].parse().ok();
            *i += 1;
        }
        "--skill" if *i + 1 < args.len() => {
            new_config.skill_filter = Some(args[*i + 1].clone());
            *i += 1;
        }
        "--json" => new_config.json_output = true,
        _ => {}
    }
    new_config
}

/// Load all records from JSONL file.
fn load_records() -> Result<Vec<SkillInvocationRecord>, std::io::Error> {
    let path = Path::new(TELEMETRY_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let records = parse_lines(reader);
    Ok(records)
}

/// Parse lines from reader into records.
fn parse_lines(reader: BufReader<File>) -> Vec<SkillInvocationRecord> {
    reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect()
}

/// Aggregate metrics by skill name.
fn aggregate_metrics(records: &[SkillInvocationRecord]) -> HashMap<String, SkillMetrics> {
    let mut metrics: HashMap<String, SkillMetrics> = HashMap::new();
    for record in records {
        metrics
            .entry(record.skill.clone())
            .or_default()
            .record(record);
    }
    metrics
}

/// Build summary from aggregated metrics.
fn build_summary(
    metrics: &HashMap<String, SkillMetrics>,
    top_n: Option<usize>,
) -> TelemetrySummary {
    let (total_invocations, total_successes) = compute_totals(metrics);
    let overall_success_rate = compute_success_rate(total_invocations, total_successes);
    let skills = build_skill_summaries(metrics, top_n);

    TelemetrySummary {
        total_invocations,
        total_skills: metrics.len(),
        overall_success_rate,
        skills,
    }
}

/// Compute total invocations and successes.
fn compute_totals(metrics: &HashMap<String, SkillMetrics>) -> (u64, u64) {
    let total_invocations: u64 = metrics.values().map(|m| m.invocations).sum();
    let total_successes: u64 = metrics.values().map(|m| m.successes).sum();
    (total_invocations, total_successes)
}

/// Compute success rate percentage.
fn compute_success_rate(total: u64, successes: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (successes as f64 / total as f64) * 100.0
}

/// Build sorted skill summaries.
fn build_skill_summaries(
    metrics: &HashMap<String, SkillMetrics>,
    top_n: Option<usize>,
) -> Vec<SkillSummary> {
    let mut skills: Vec<SkillSummary> = metrics
        .iter()
        .map(|(name, m)| SkillSummary {
            skill: name.clone(),
            invocations: m.invocations,
            success_rate: m.success_rate(),
            avg_duration_ms: m.avg_duration_ms(),
        })
        .collect();

    skills.sort_by(|a, b| b.invocations.cmp(&a.invocations));

    if let Some(n) = top_n {
        skills.truncate(n);
    }
    skills
}

/// Print summary as JSON.
fn print_json(summary: &TelemetrySummary, skill_filter: &Option<String>) {
    let output = match skill_filter {
        Some(filter) => filter_and_serialize(summary, filter),
        None => serde_json::to_string_pretty(summary).unwrap_or_default(),
    };
    println!("{output}");
}

/// Filter skills and serialize to JSON.
fn filter_and_serialize(summary: &TelemetrySummary, filter: &str) -> String {
    let filtered: Vec<&SkillSummary> = summary
        .skills
        .iter()
        .filter(|s| s.skill.contains(filter))
        .collect();
    serde_json::to_string_pretty(&filtered).unwrap_or_default()
}

/// Print summary as human-readable text.
fn print_text(summary: &TelemetrySummary, skill_filter: &Option<String>) {
    print_header(summary);
    print_skill_table(summary, skill_filter);
}

/// Print summary header.
fn print_header(summary: &TelemetrySummary) {
    println!();
    println!("SKILL INVOCATION TELEMETRY");
    println!("==========================");
    println!();
    println!("Total Invocations: {}", summary.total_invocations);
    println!("Unique Skills:     {}", summary.total_skills);
    println!("Overall Success:   {:.1}%", summary.overall_success_rate);
    println!();
}

/// Print skill table.
fn print_skill_table(summary: &TelemetrySummary, skill_filter: &Option<String>) {
    let skills_to_show = filter_skills(&summary.skills, skill_filter);

    if skills_to_show.is_empty() {
        println!("No matching skills found.");
        return;
    }

    println!(
        "{:<40} {:>8} {:>10} {:>12}",
        "SKILL", "COUNT", "SUCCESS", "AVG MS"
    );
    println!("{}", "-".repeat(72));

    for skill in skills_to_show {
        print_skill_row(skill);
    }
    println!();
}

/// Filter skills by optional filter string.
fn filter_skills<'a>(skills: &'a [SkillSummary], filter: &Option<String>) -> Vec<&'a SkillSummary> {
    match filter {
        Some(f) => skills.iter().filter(|s| s.skill.contains(f)).collect(),
        None => skills.iter().collect(),
    }
}

/// Print a single skill row.
fn print_skill_row(skill: &SkillSummary) {
    let duration_str = skill
        .avg_duration_ms
        .map(|d| format!("{:.0}", d))
        .unwrap_or_else(|| "-".to_string());

    println!(
        "{:<40} {:>8} {:>9.1}% {:>12}",
        truncate_skill_name(&skill.skill, 40),
        skill.invocations,
        skill.success_rate,
        duration_str
    );
}

/// Truncate skill name for display.
fn truncate_skill_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        return name.to_string();
    }
    format!("{}...", &name[..max_len.saturating_sub(3)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_metrics_success_rate() {
        let mut m = SkillMetrics::default();
        m.invocations = 10;
        m.successes = 8;
        m.failures = 2;
        assert!((m.success_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_skill_metrics_avg_duration() {
        let mut m = SkillMetrics::default();
        m.total_duration_ms = 300;
        m.duration_count = 3;
        assert_eq!(m.avg_duration_ms(), Some(100.0));
    }

    #[test]
    fn test_skill_metrics_no_duration() {
        let m = SkillMetrics::default();
        assert_eq!(m.avg_duration_ms(), None);
    }

    #[test]
    fn test_truncate_skill_name_short() {
        assert_eq!(truncate_skill_name("short", 40), "short");
    }

    #[test]
    fn test_truncate_skill_name_long() {
        let long = "a".repeat(50);
        let result = truncate_skill_name(&long, 40);
        assert!(result.len() <= 40);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_compute_success_rate_zero() {
        assert_eq!(compute_success_rate(0, 0), 0.0);
    }

    #[test]
    fn test_compute_success_rate_full() {
        assert!((compute_success_rate(100, 75) - 75.0).abs() < 0.01);
    }
}
