//! SessionStart hook that loads molecule definitions from ~/.claude/molecules/
//!
//! Parses all .json files and makes chains available for activation.
//! Outputs molecule summaries as additional context.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Molecule {
    name: String,
    description: String,
    bond_type: String,
    stability: f64,
    activation_triggers: Vec<String>,
    chain: Vec<ChainLink>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChainLink {
    order: u32,
    skill: String,
    role: String,
    purpose: String,
}

fn main() {
    let molecules_dir = dirs::home_dir()
        .map(|h| h.join(".claude/molecules"))
        .unwrap_or_default();

    if !molecules_dir.exists() {
        return;
    }

    let molecules = load_molecules(&molecules_dir);

    if molecules.is_empty() {
        return;
    }

    // Output molecule registry as additional context
    let mut context = String::from("\n🧬 **MOLECULE REGISTRY** ─────────────────────────────\n");

    for mol in &molecules {
        context.push_str(&format!(
            "   **{}** ({}, stability: {:.0}%)\n",
            mol.name,
            mol.bond_type,
            mol.stability * 100.0
        ));
        context.push_str(&format!("   Chain: "));

        let skills: Vec<_> = mol.chain.iter().map(|c| c.skill.as_str()).collect();
        context.push_str(&skills.join(" → "));
        context.push('\n');

        context.push_str(&format!(
            "   Triggers: {}\n\n",
            mol.activation_triggers.join(", ")
        ));
    }

    context.push_str("───────────────────────────────────────────────────────\n");

    let output = serde_json::json!({
        "additionalContext": context
    });

    println!("{}", output);
}

fn load_molecules(dir: &PathBuf) -> Vec<Molecule> {
    let mut molecules = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(mol) = serde_json::from_str::<Molecule>(&content) {
                        molecules.push(mol);
                    }
                }
            }
        }
    }

    molecules
}
