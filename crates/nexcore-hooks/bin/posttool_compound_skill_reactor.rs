//! # Compound Skill Reactor Hook (PostToolUse:Skill)
//!
//! Records reaction products after compound skill invocation using chemistry metaphors.
//!
//! ## Chemistry Metaphor
//!
//! - **Reaction Products**: Records output after skill invocation
//! - **Polymer Chains**: Tracks nested skill usage patterns
//! - **Bond Graph**: Updates molecular structure with reaction data
//! - **Reaction Yield**: Calculates success rate of nested operations
//!
//! ## Tracking Phases
//!
//! 1. Log compound skill invocation with molecular formula
//! 2. Track which polymer units were activated
//! 3. Record reaction energy (execution metrics)
//!
//! ## Exit Codes
//!
//! - 0: Success (always passes, observational only)

use chrono::{DateTime, Utc};
use nexcore_hooks::{HookInput, exit_success_auto, exit_success_auto_with, read_input};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Reaction event for a compound skill invocation
#[derive(Serialize)]
struct ReactionEvent {
    /// Timestamp
    timestamp: DateTime<Utc>,
    /// Session ID
    session_id: String,
    /// Skill (molecule) that reacted
    molecule: String,
    /// Molecular formula if compound
    formula: Option<String>,
    /// Was this a compound skill?
    is_compound: bool,
    /// Reaction type
    reaction_type: ReactionType,
    /// Energy level (from tool result success/failure)
    energy_released: EnergyLevel,
}

/// Type of reaction
#[derive(Serialize)]
enum ReactionType {
    /// Simple skill (single molecule)
    Simple,
    /// Compound skill (macromolecule)
    Compound,
}

/// Energy level from reaction
#[derive(Serialize)]
enum EnergyLevel {
    /// Successful reaction (exothermic)
    High,
    /// Partial success
    Medium,
    /// Failed reaction (endothermic - absorbed energy)
    Low,
}

/// Macromolecule lookup
#[derive(Deserialize)]
struct Macromolecule {
    formula: String,
    #[allow(dead_code)]
    stable: bool,
}

#[derive(Deserialize, Default)]
struct MacromoleculeRegistry {
    macromolecules: HashMap<String, Macromolecule>,
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only track Skill tool
    if input.tool_name.as_deref() != Some("Skill") {
        exit_success_auto();
    }

    // Get skill name
    let skill_name = match get_skill_name(&input) {
        Some(name) => name,
        None => exit_success_auto(),
    };

    // Load registry to check if compound
    let registry = load_registry();
    let mol_info = registry.macromolecules.get(&skill_name);

    let is_compound = mol_info.is_some();
    let formula = mol_info.map(|m| m.formula.clone());

    // Determine energy level from tool response
    let energy = match &input.tool_response {
        Some(resp) => {
            let resp_str = resp.to_string();
            if resp_str.contains("error") || resp_str.contains("Error") {
                EnergyLevel::Low
            } else {
                EnergyLevel::High
            }
        }
        None => EnergyLevel::Medium,
    };

    // Create reaction event
    let event = ReactionEvent {
        timestamp: Utc::now(),
        session_id: input.session_id.clone(),
        molecule: skill_name.clone(),
        formula: formula.clone(),
        is_compound,
        reaction_type: if is_compound {
            ReactionType::Compound
        } else {
            ReactionType::Simple
        },
        energy_released: energy,
    };

    // Append to reaction log
    if let Err(e) = append_reaction(&event) {
        eprintln!("Warning: Could not log reaction: {e}");
    }

    // Output context
    let msg = if let Some(f) = formula {
        format!("⚗️ Reaction: {} complete", f)
    } else {
        format!("⚗️ Reaction: {} (simple)", skill_name)
    };

    exit_success_auto_with(&msg);
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

/// Load macromolecule registry
fn load_registry() -> MacromoleculeRegistry {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return MacromoleculeRegistry::default(),
    };

    let path = home.join(".claude/brain/macromolecule_registry.json");
    if !path.exists() {
        return MacromoleculeRegistry::default();
    }

    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

/// Append reaction event to log
fn append_reaction(event: &ReactionEvent) -> std::io::Result<()> {
    use std::io::Write;

    let home = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
    })?;

    let log_path = home.join(".claude/brain/reactions.jsonl");
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let line = serde_json::to_string(event)?;
    writeln!(file, "{}", line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_event_serialization() {
        let event = ReactionEvent {
            timestamp: Utc::now(),
            session_id: "test-session".to_string(),
            molecule: "forge".to_string(),
            formula: Some("forge(3)".to_string()),
            is_compound: true,
            reaction_type: ReactionType::Compound,
            energy_released: EnergyLevel::High,
        };

        let json = serde_json::to_string(&event).unwrap_or_default();
        assert!(json.contains("forge(3)"));
        assert!(json.contains("Compound"));
    }
}
