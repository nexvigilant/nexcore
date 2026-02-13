//! SessionStart hook: Capability Gap Detector
//!
//! Identifies missing capabilities based on usage patterns and skill taxonomy.
//! Generates improvement bonds for detected gaps.
//!
//! Gap Detection Strategies:
//! 1. Usage-weighted: High-use tools without supporting skills
//! 2. Taxonomy coverage: Missing skills in active categories
//! 3. Primitive chains: Incomplete T1→T2→T3 progressions
//! 4. Cross-domain: Missing transfer bridges
//!
//! ToV Alignment:
//! - Feedback Loop (ℱ): Usage data drives gap identification
//! - Anticipatory: Predicts needs before they become blockers
//!
//! Exit codes:
//! - 0: Success (gaps reported in context)

use nexcore_hooks::{exit_success, exit_with_session_context, read_input};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Minimum tool calls to consider for gap analysis
const MIN_USAGE_THRESHOLD: u64 = 5;

/// Known skill categories for coverage analysis
const SKILL_CATEGORIES: &[(&str, &[&str])] = &[
    (
        "primitives",
        &[
            "primitive-extractor",
            "primitive-rust-foundation",
            "transfer-confidence",
        ],
    ),
    ("rust-dev", &["rust-dev", "rust-anatomy-expert", "forge"]),
    ("skills", &["skill-dev", "skill-audit", "skill-advisor"]),
    (
        "hooks",
        &["hook-lifecycle", "hook-amplifier", "bond-architect"],
    ),
    (
        "validation",
        &["ctvp-validator", "SMART-dev", "vdag-orchestrator"],
    ),
    ("strategy", &["strat-dev", "SMART-dev"]),
    ("memory", &["brain-dev", "config-dev"]),
    ("mcp", &["mcp-dev", "nexcore-api-dev"]),
    (
        "personas",
        &["persona-dev", "compendious-mach", "socratic-learner"],
    ),
    (
        "epistemology",
        &[
            "constructive-epistemology",
            "cep-pipeline",
            "domain-translator",
        ],
    ),
];

/// Tools that should have supporting skills
const TOOL_SKILL_MAP: &[(&str, &str)] = &[
    ("Task", "subagent-dev"),
    ("Skill", "skill-dev"),
    ("Edit", "rust-dev"),
    ("Write", "rust-dev"),
    ("Bash", "rust-dev"),
    ("WebSearch", "domain-translator"),
    ("WebFetch", "domain-translator"),
];

#[derive(Debug, Serialize, Deserialize, Default)]
struct UsageTelemetry {
    tools: HashMap<String, ToolUsage>,
    skills: HashMap<String, SkillUsage>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ToolUsage {
    total_calls: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SkillUsage {
    total_invocations: u64,
}

#[derive(Debug)]
struct Gap {
    category: String,
    description: String,
    priority: u8, // 1-5, higher = more urgent
    suggestion: String,
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_success(),
    };

    let telemetry = load_telemetry();
    let existing_skills = scan_existing_skills();

    let mut gaps: Vec<Gap> = Vec::new();

    // Strategy 1: Tool usage without skill support
    for (tool, required_skill) in TOOL_SKILL_MAP {
        if let Some(usage) = telemetry.tools.get(*tool) {
            if usage.total_calls >= MIN_USAGE_THRESHOLD {
                if !existing_skills.contains(*required_skill) {
                    let desc = [
                        tool,
                        " used ",
                        &usage.total_calls.to_string(),
                        "x but ",
                        required_skill,
                        " missing",
                    ]
                    .concat();
                    let sugg = ["Add ", required_skill, " to skills"].concat();
                    gaps.push(Gap {
                        category: "tool-support".into(),
                        description: desc,
                        priority: 3,
                        suggestion: sugg,
                    });
                }
            }
        }
    }

    // Strategy 2: Category coverage gaps
    for (category, skills) in SKILL_CATEGORIES {
        let existing_in_category: Vec<_> = skills
            .iter()
            .filter(|s| existing_skills.contains(**s))
            .collect();

        let coverage = existing_in_category.len() as f32 / skills.len() as f32;

        if coverage > 0.0 && coverage < 0.5 {
            let missing: Vec<_> = skills
                .iter()
                .filter(|s| !existing_skills.contains(**s))
                .take(2)
                .copied()
                .collect();

            let desc = [
                *category,
                " category at ",
                &((coverage * 100.0) as u32).to_string(),
                "% coverage",
            ]
            .concat();
            let sugg = ["Add: ", &missing.join(", ")].concat();

            gaps.push(Gap {
                category: "coverage".into(),
                description: desc,
                priority: 2,
                suggestion: sugg,
            });
        }
    }

    // Strategy 3: High-use skills without tests
    for (skill_name, usage) in &telemetry.skills {
        if usage.total_invocations >= 10 {
            let skill_path = skills_dir().join(skill_name);
            let has_tests =
                skill_path.join("tests").exists() || skill_path.join("scripts/test.sh").exists();

            if !has_tests {
                let desc = [
                    skill_name.as_str(),
                    " invoked ",
                    &usage.total_invocations.to_string(),
                    "x but has no tests",
                ]
                .concat();
                let sugg = ["Add tests to ", skill_name].concat();
                gaps.push(Gap {
                    category: "quality".into(),
                    description: desc,
                    priority: 4,
                    suggestion: sugg,
                });
            }
        }
    }

    if gaps.is_empty() {
        exit_success();
    }

    // Sort by priority (highest first)
    gaps.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Build context
    let mut context = String::from("🔍 **GAP DETECTOR** ───────────────────────────────────────\n");
    context.push_str("   Found ");
    context.push_str(&gaps.len().to_string());
    context.push_str(" capability gaps:\n\n");

    for (i, gap) in gaps.iter().take(5).enumerate() {
        let priority_icon = match gap.priority {
            5 => "🔴",
            4 => "🟠",
            3 => "🟡",
            2 => "🟢",
            _ => "⚪",
        };
        context.push_str("   ");
        context.push_str(&(i + 1).to_string());
        context.push_str(". ");
        context.push_str(priority_icon);
        context.push_str(" [");
        context.push_str(&gap.category);
        context.push_str("] ");
        context.push_str(&gap.description);
        context.push_str("\n      → ");
        context.push_str(&gap.suggestion);
        context.push_str("\n\n");
    }

    if gaps.len() > 5 {
        context.push_str("   ... and ");
        context.push_str(&(gaps.len() - 5).to_string());
        context.push_str(" more gaps\n");
    }

    context.push_str("───────────────────────────────────────────────────────────\n");

    exit_with_session_context(&context);
}

fn skills_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".claude/skills")
}

fn telemetry_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".claude/metrics/usage_telemetry.json")
}

fn load_telemetry() -> UsageTelemetry {
    fs::read_to_string(telemetry_path())
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn scan_existing_skills() -> HashSet<String> {
    let dir = skills_dir();
    let mut skills = HashSet::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    skills.insert(name.to_string());
                }
            }
        }
    }

    skills
}
