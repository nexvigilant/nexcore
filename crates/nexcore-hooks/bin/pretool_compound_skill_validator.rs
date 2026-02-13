//! Compound Skill Validator Hook
//!
//! Event: PreToolUse:Skill
//!
//! Chemistry Metaphor:
//! - Validates molecular stability before reaction (skill invocation)
//! - Checks bond integrity (parent-child references)
//! - Warns on unstable macromolecules (missing nested skills)
//!
//! Validation:
//! 1. If invoking a compound skill, check macromolecule registry
//! 2. Warn if any bonds are broken (missing nested skills)
//! 3. Log molecular formula for context

use nexcore_hooks::{HookInput, exit_success_auto, exit_success_auto_with, exit_warn, read_input};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// Simplified macromolecule entry for lookup
#[derive(Deserialize)]
struct Macromolecule {
    #[allow(dead_code)]
    nucleus: String,
    formula: String,
    stable: bool,
    broken_bonds: Vec<String>,
    nesting_depth: u8,
}

/// Registry for lookup
#[derive(Deserialize, Default)]
struct MacromoleculeRegistry {
    macromolecules: HashMap<String, Macromolecule>,
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only check Skill tool invocations
    if input.tool_name.as_deref() != Some("Skill") {
        exit_success_auto();
    }

    // Get skill name from tool input
    let skill_name = match get_skill_name(&input) {
        Some(name) => name,
        None => exit_success_auto(),
    };

    // Load macromolecule registry
    let registry = load_registry();
    if registry.macromolecules.is_empty() {
        exit_success_auto();
    }

    // Check if this skill is a compound (macromolecule)
    if let Some(mol) = registry.macromolecules.get(&skill_name) {
        if mol.stable {
            exit_success_auto_with(&format!(
                "⚗️ {} stable, depth={}",
                mol.formula, mol.nesting_depth
            ));
        } else {
            let broken = mol.broken_bonds.join(", ");
            exit_warn(&format!(
                "⚠️ {} UNSTABLE - broken bonds: [{}]",
                mol.formula, broken
            ));
        }
    }

    exit_success_auto();
}

/// Extract skill name from hook input
fn get_skill_name(input: &HookInput) -> Option<String> {
    input
        .tool_input
        .as_ref()?
        .get("skill")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Load macromolecule registry from disk
fn load_registry() -> MacromoleculeRegistry {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return MacromoleculeRegistry::default(),
    };

    let path = home.join(".claude/brain/macromolecule_registry.json");
    if !path.exists() {
        return MacromoleculeRegistry::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => MacromoleculeRegistry::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_deserialization() {
        let json = r#"{
            "macromolecules": {
                "forge": {
                    "nucleus": "forge",
                    "formula": "forge(3)",
                    "stable": true,
                    "broken_bonds": [],
                    "nesting_depth": 1
                }
            }
        }"#;

        // Use unwrap_or_default and check result
        let registry: MacromoleculeRegistry = serde_json::from_str(json).unwrap_or_default();
        assert!(registry.macromolecules.contains_key("forge"));
        assert!(registry.macromolecules["forge"].stable);
    }
}
