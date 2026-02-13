//! Tier Coverage Analyzer (Accelerator 2)
//!
//! Analyzes T1/T2/T3 distribution in extraction code and warns
//! on unbalanced ratios that suggest incomplete decomposition.
//!
//! # Purpose
//!
//! Healthy extractions have balanced tier distributions:
//! - T1: 15-25% (universal bedrock)
//! - T2: 25-35% (cross-domain bridges)
//! - T3: 40-60% (domain-specific)
//!
//! High T3 ratios suggest primitives haven't been fully decomposed.
//!
//! # Exit Codes
//!
//! - `0`: Balanced or not an extraction file
//! - `1`: Warning - unbalanced tier distribution

use nexcore_hooks::{exit_success_auto, exit_success_auto_with, exit_warn, read_input};
use regex::Regex;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = input.get_file_path().unwrap_or("");
    let content = input.get_written_content().unwrap_or("");

    // Only analyze extraction/primitive files
    let is_extraction_file = file_path.contains("primitive")
        || file_path.contains("extraction")
        || file_path.contains("cep")
        || file_path.contains("tier");

    if !is_extraction_file {
        exit_success_auto();
    }

    // Count tier references
    let t1_re = Regex::new(r"T1|tier_1|Universal").ok();
    let t2_re = Regex::new(r"T2|tier_2|CrossDomain").ok();
    let t3_re = Regex::new(r"T3|tier_3|DomainSpecific").ok();

    let t1_count = t1_re.map(|r| r.find_iter(content).count()).unwrap_or(0);
    let t2_count = t2_re.map(|r| r.find_iter(content).count()).unwrap_or(0);
    let t3_count = t3_re.map(|r| r.find_iter(content).count()).unwrap_or(0);

    let total = t1_count + t2_count + t3_count;

    if total < 3 {
        exit_success_auto();
    }

    // Calculate ratios
    let t1_ratio = t1_count as f64 / total as f64;
    let t3_ratio = t3_count as f64 / total as f64;

    // Check for imbalance
    if t3_ratio > 0.75 {
        let msg = format!(
            "📊 High T3 concentration ({:.0}%) - consider deeper decomposition to T1/T2 primitives",
            t3_ratio * 100.0
        );
        exit_warn(&msg);
    }

    if t1_ratio < 0.10 && total > 5 {
        let msg = format!(
            "📊 Low T1 coverage ({:.0}%) - ensure universal primitives are identified",
            t1_ratio * 100.0
        );
        exit_warn(&msg);
    }

    let detail = format!("T1:{} T2:{} T3:{}", t1_count, t2_count, t3_count);
    exit_success_auto_with(&detail);
}
