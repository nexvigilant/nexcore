//! PreSkill Context Advisor Hook
//!
//! Fires before skill activation to analyze context and inject cargo recommendations.
//!
//! ## What It Does
//! 1. Reads conversation context from hook input
//! 2. Identifies current difficulties (insufficient data, name mismatches, etc.)
//! 3. Detects antipatterns (hardcoded thresholds, mock theater, etc.)
//! 4. Recommends cargo selections based on context
//! 5. Injects menu-ready advice into skill execution
//!
//! ## Hook Event
//! - **Event**: `PreToolUse`
//! - **Matcher**: `Skill`
//! - **Decision**: Always `allow` (advisory, not blocking)

use serde::{Deserialize, Serialize};
use std::io::{self, Read};

/// Hook input from Claude Code
#[derive(Debug, Deserialize)]
struct HookInput {
    /// Tool name (always "Skill" for this hook)
    #[allow(dead_code)]
    tool_name: String,
    /// Tool input containing skill name and args
    tool_input: SkillInput,
}

#[derive(Debug, Deserialize)]
struct SkillInput {
    /// Name of the skill being invoked
    skill: Option<String>,
    /// Arguments passed to the skill
    args: Option<String>,
}

/// Context analysis output
#[derive(Debug, Serialize)]
struct ContextAnalysis {
    difficulties: Vec<Difficulty>,
    antipatterns: Vec<Antipattern>,
    recommended_cargos: Vec<String>,
    advice: Vec<String>,
    menu_hint: String,
}

#[derive(Debug, Serialize)]
struct Difficulty {
    pattern: String,
    confidence: f64,
    context: String,
    cargo_hint: String,
}

#[derive(Debug, Serialize)]
struct Antipattern {
    name: String,
    severity: String,
    fix: String,
}

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        print_allow_no_context();
        return;
    }

    let hook_input: HookInput = match serde_json::from_str(&input) {
        Ok(h) => h,
        Err(_) => {
            print_allow_no_context();
            return;
        }
    };

    // Extract skill name
    let skill_name = hook_input
        .tool_input
        .skill
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let args = hook_input.tool_input.args.clone().unwrap_or_default();

    // Analyze context based on skill and args
    let analysis = analyze_skill_context(&skill_name, &args);

    // Build menu hint
    let menu_hint = build_menu_hint(&skill_name, &analysis);

    // Build output message
    let advice_block = build_advice_block(&analysis);
    let output_msg = format!(
        "\n🍽️ **SKILL MENU** — {}\n\n{}\n\n{}",
        skill_name, menu_hint, advice_block
    );

    // Only include outputToUser if we have meaningful content
    let output = if analysis.difficulties.is_empty()
        && analysis.antipatterns.is_empty()
        && analysis.advice.is_empty()
    {
        // No context detected - just allow silently
        serde_json::json!({"decision": "allow"})
    } else {
        serde_json::json!({
            "decision": "allow",
            "outputToUser": output_msg
        })
    };

    println!(
        "{}",
        serde_json::to_string(&output).unwrap_or_else(|_| r#"{"decision":"allow"}"#.to_string())
    );
}

fn print_allow_no_context() {
    println!(r#"{{"decision":"allow"}}"#);
}

fn analyze_skill_context(skill_name: &str, args: &str) -> ContextAnalysis {
    let mut difficulties = Vec::new();
    let mut antipatterns = Vec::new();
    let mut recommended_cargos = Vec::new();
    let mut advice = Vec::new();

    let args_lower = args.to_lowercase();

    // ─────────────────────────────────────────────────────────────────
    // Difficulty Detection
    // ─────────────────────────────────────────────────────────────────

    // Insufficient data patterns
    if args_lower.contains("insufficient")
        || args_lower.contains("n < 3")
        || args_lower.contains("no data")
        || args_lower.contains("not found")
    {
        difficulties.push(Difficulty {
            pattern: "insufficient_data".into(),
            confidence: 0.9,
            context: "Signal detection requires n >= 3 cases".into(),
            cargo_hint: "data-retrieval".into(),
        });
        recommended_cargos.push("data-retrieval".into());
        advice.push("Expand search criteria or check alternative drug/event names".into());
    }

    // Name mismatch patterns
    if args_lower.contains("mismatch")
        || args_lower.contains("spelling")
        || args_lower.contains("typo")
        || args_lower.contains("similar")
    {
        difficulties.push(Difficulty {
            pattern: "name_mismatch".into(),
            confidence: 0.85,
            context: "Drug or event name may have spelling issues".into(),
            cargo_hint: "foundation".into(),
        });
        recommended_cargos.push("foundation".into());
        advice.push("Use fuzzy_search to find correct spelling".into());
    }

    // Performance issues
    if args_lower.contains("slow") || args_lower.contains("timeout") || args_lower.contains("long")
    {
        difficulties.push(Difficulty {
            pattern: "performance".into(),
            confidence: 0.7,
            context: "Operation may be slow or timing out".into(),
            cargo_hint: "optimization".into(),
        });
        advice.push("Consider batch processing or limiting result set".into());
    }

    // ─────────────────────────────────────────────────────────────────
    // Antipattern Detection
    // ─────────────────────────────────────────────────────────────────

    // Hardcoded thresholds
    if args_lower.contains("2.0") && args_lower.contains("threshold") || args_lower.contains("3.84")
    {
        antipatterns.push(Antipattern {
            name: "hardcoded_threshold".into(),
            severity: "medium".into(),
            fix: "Use SignalCriteria::evans() for configurable thresholds".into(),
        });
    }

    // Mock testing theater
    if args_lower.contains("mock") && args_lower.contains("test") {
        antipatterns.push(Antipattern {
            name: "mock_theater".into(),
            severity: "high".into(),
            fix: "Apply CTVP validation - mocks validate simulations, not reality".into(),
        });
    }

    // Magic numbers
    if args_lower.contains("magic")
        || (args_lower.contains("number") && args_lower.contains("hard"))
    {
        antipatterns.push(Antipattern {
            name: "magic_numbers".into(),
            severity: "low".into(),
            fix: "Extract constants with meaningful names".into(),
        });
    }

    // ─────────────────────────────────────────────────────────────────
    // Skill-Specific Cargo Recommendations
    // ─────────────────────────────────────────────────────────────────

    match skill_name {
        "vigilance-dev" | "guardian-orchestrator" => {
            recommended_cargos.push("signal-analysis".into());
            recommended_cargos.push("guardian-risk".into());
        }
        "rust-dev" | "rust-anatomy-expert" | "forge" => {
            recommended_cargos.push("rust-patterns".into());
            recommended_cargos.push("foundation".into());
        }
        "skill-dev" | "skill-advisor" | "skill-audit" => {
            recommended_cargos.push("skill-registry".into());
        }
        "ctvp-validator" => {
            recommended_cargos.push("validation".into());
            advice.push("Run 5-phase validation: Preclinical → Safety → Efficacy → Confirmation → Surveillance".into());
        }
        "strat-dev" => {
            recommended_cargos.push("strategy".into());
            advice.push("Apply Playing to Win framework: Aspiration → Where to Play → How to Win → Capabilities → Management".into());
        }
        "mcp-dev" => {
            recommended_cargos.push("mcp-tools".into());
        }
        "hook-lifecycle" | "hook-amplifier" => {
            recommended_cargos.push("hooks".into());
        }
        _ => {
            // Default: recommend foundation cargo
            if recommended_cargos.is_empty() {
                recommended_cargos.push("foundation".into());
            }
        }
    }

    // Deduplicate
    recommended_cargos.sort();
    recommended_cargos.dedup();

    let menu_hint = format!(
        "Recommended meal: {}",
        if recommended_cargos.is_empty() {
            "Standard Toolkit".into()
        } else {
            recommended_cargos.join(" + ")
        }
    );

    ContextAnalysis {
        difficulties,
        antipatterns,
        recommended_cargos,
        advice,
        menu_hint,
    }
}

fn build_menu_hint(_skill_name: &str, analysis: &ContextAnalysis) -> String {
    let mut lines = Vec::new();

    // Difficulties section
    if !analysis.difficulties.is_empty() {
        lines.push("**Context Detected:**".into());
        for d in &analysis.difficulties {
            lines.push(format!(
                "  • `{}` (confidence: {:.0}%) → use `{}` cargo",
                d.pattern,
                d.confidence * 100.0,
                d.cargo_hint
            ));
        }
        lines.push(String::new());
    }

    // Antipatterns section
    if !analysis.antipatterns.is_empty() {
        lines.push("**⚠️ Antipatterns Detected:**".into());
        for a in &analysis.antipatterns {
            lines.push(format!("  • `{}` [{}]: {}", a.name, a.severity, a.fix));
        }
        lines.push(String::new());
    }

    // Recommended cargos
    if !analysis.recommended_cargos.is_empty() {
        lines.push(format!(
            "**Recommended Cargos:** {}",
            analysis.recommended_cargos.join(", ")
        ));
    }

    lines.join("\n")
}

fn build_advice_block(analysis: &ContextAnalysis) -> String {
    if analysis.advice.is_empty() {
        return String::new();
    }

    let mut lines = vec!["**💡 Advice:**".to_string()];
    for a in &analysis.advice {
        lines.push(format!("  • {a}"));
    }
    lines.join("\n")
}
