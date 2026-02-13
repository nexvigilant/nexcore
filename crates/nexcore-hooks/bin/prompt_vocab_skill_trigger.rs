//! # Vocabulary-Skill Trigger Hook
//!
//! **Event:** `UserPromptSubmit`
//! **Blocking:** No (suggestions only)
//! **Purpose:** Detect vocabulary/primitive usage and suggest associated skills.
//!
//! ## What It Does
//!
//! 1. Loads vocab_skill_map.json
//! 2. Scans prompt for vocabulary shorthands and primitives
//! 3. Outputs skill suggestions based on matches
//!
//! ## Output
//!
//! Suggests primary and secondary skills when vocab/primitives detected.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct SkillMapping {
    primary: String,
    #[serde(default)]
    secondary: Vec<String>,
    #[serde(default)]
    hooks: Vec<String>,
    #[serde(default)]
    primitives: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SkillChain {
    trigger: String,
    chain: Vec<String>,
    subagent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VocabSkillMap {
    #[serde(default)]
    vocab_to_skills: std::collections::HashMap<String, SkillMapping>,
    #[serde(default)]
    primitive_to_skills: std::collections::HashMap<String, Vec<String>>,
    #[serde(default)]
    skill_chains: std::collections::HashMap<String, SkillChain>,
}

fn load_vocab_map() -> Option<VocabSkillMap> {
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(home).join(".claude/implicit/vocab_skill_map.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn get_prompt() -> Option<String> {
    // Read from stdin (hook input)
    let stdin = std::io::stdin();
    let input: serde_json::Value = serde_json::from_reader(stdin.lock()).ok()?;

    // Extract prompt from various possible locations
    input
        .get("prompt")
        .or_else(|| input.get("content"))
        .or_else(|| input.get("message"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase())
}

fn main() {
    let prompt = match get_prompt() {
        Some(p) => p,
        None => {
            println!("{}", json!({"continue": true}));
            return;
        }
    };

    let map = match load_vocab_map() {
        Some(m) => m,
        None => {
            println!("{}", json!({"continue": true}));
            return;
        }
    };

    let mut detected_vocab: Vec<&str> = Vec::new();
    let mut suggested_skills: HashSet<&str> = HashSet::new();
    let mut triggered_chains: Vec<(&str, &SkillChain)> = Vec::new();

    // Check for vocabulary matches
    for (vocab, mapping) in &map.vocab_to_skills {
        if prompt.contains(&vocab.to_lowercase()) || prompt.contains(&vocab.replace("-", " ")) {
            detected_vocab.push(vocab);
            suggested_skills.insert(&mapping.primary);
            for sec in &mapping.secondary {
                suggested_skills.insert(sec);
            }
        }
    }

    // Check for primitive matches
    for (primitive, skills) in &map.primitive_to_skills {
        if prompt.contains(&primitive.to_lowercase()) {
            for skill in skills {
                suggested_skills.insert(skill);
            }
        }
    }

    // Check for skill chain triggers
    for (name, chain) in &map.skill_chains {
        if prompt.contains(&chain.trigger.to_lowercase()) {
            triggered_chains.push((name, chain));
        }
    }

    // Build output
    if detected_vocab.is_empty() && triggered_chains.is_empty() {
        println!("{}", json!({"continue": true}));
        return;
    }

    let mut output_lines = Vec::new();

    if !detected_vocab.is_empty() {
        output_lines.push(format!(
            "🔗 **VOCAB DETECTED**: {}",
            detected_vocab.join(", ")
        ));
    }

    if !suggested_skills.is_empty() {
        let skills: Vec<&str> = suggested_skills.into_iter().collect();
        output_lines.push(format!("   **Suggested skills**: {}", skills.join(", ")));
    }

    for (name, chain) in &triggered_chains {
        output_lines.push(format!(
            "⚡ **CHAIN TRIGGERED**: {} → {:?}",
            name, chain.chain
        ));
        if let Some(agent) = &chain.subagent {
            output_lines.push(format!("   **Subagent**: {}", agent));
        }
    }

    let output = json!({
        "continue": true,
        "stopReason": output_lines.join("\n")
    });

    println!("{}", output);
}
