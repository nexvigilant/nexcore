//! Nested Skill Resolver Hook
//!
//! Event: SessionStart
//!
//! Chemistry Metaphor:
//! - Skill = Molecule (atoms bonded together)
//! - Compound Skill = Macromolecule (molecules bonded into larger structure)
//! - Nested Skills = Polymer chain (repeating units linked)
//! - Molecular Formula = `parent(n)` where n = nested count
//!
//! Scans skills with `nested-skills:` frontmatter and validates:
//! 1. All declared nested skill paths exist
//! 2. Nested skills have valid `parent:` reference back (covalent bond)
//! 3. Creates macromolecule registry for session
//! 4. Computes molecular formula for each compound
//!
//! Safety Axiom: A1 (Conservation of Intent) - preserves skill hierarchy.

use nexcore_hooks::{exit_skip_session, exit_with_session_context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Chemistry-inspired compound skill entry
#[derive(Serialize, Deserialize)]
struct Macromolecule {
    /// Parent skill name (central atom)
    nucleus: String,
    /// Parent skill path
    nucleus_path: String,
    /// Molecular formula: `name(nested_count)` e.g., `forge(3)`
    formula: String,
    /// Molecular weight (sum of nested skill weights)
    weight: u32,
    /// Declared polymer chain (from frontmatter)
    declared_chain: Vec<String>,
    /// Resolved polymer units
    polymer_units: Vec<PolymerUnit>,
    /// Broken bonds (missing nested skills)
    broken_bonds: Vec<String>,
    /// Is this a valid macromolecule? (all bonds intact)
    stable: bool,
    /// Depth of nesting (1 = direct children only)
    nesting_depth: u8,
    /// Is this a super-macromolecule? (contains other compounds)
    contains_compounds: bool,
}

/// Single unit in the polymer chain (nested skill)
#[derive(Serialize, Deserialize)]
struct PolymerUnit {
    /// Unit name
    name: String,
    /// Full path
    path: String,
    /// Bond type to parent
    bond_type: BondType,
    /// Bond strength (based on parent reference validity)
    bond_strength: u8,
    /// Does this unit also have nested skills? (polymer of polymers)
    is_compound: bool,
    /// Atomic weight contribution
    weight: u32,
}

/// Bond types following chemistry metaphor
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum BondType {
    /// Strong bond - nested skill has valid parent reference
    Covalent,
    /// Weak bond - nested skill exists but no parent reference
    Ionic,
    /// No bond - nested skill missing
    Broken,
}

impl BondType {
    fn strength(&self) -> u8 {
        match self {
            BondType::Covalent => 100,
            BondType::Ionic => 50,
            BondType::Broken => 0,
        }
    }
}

/// Registry of macromolecules (compound skills)
#[derive(Serialize, Deserialize, Default)]
struct MacromoleculeRegistry {
    version: String,
    created_at: String,
    /// All macromolecules indexed by nucleus name
    macromolecules: HashMap<String, Macromolecule>,
    /// Total polymer units across all compounds
    total_units: usize,
    /// Total molecular weight of ecosystem
    total_weight: u32,
    /// Synthesis errors (broken bonds, invalid references)
    synthesis_errors: Vec<String>,
    /// Molecular formulas for quick reference
    formulas: Vec<String>,
}

fn main() {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => exit_skip_session(),
    };

    let skills_dir = home.join(".claude/skills");
    if !skills_dir.exists() {
        exit_skip_session();
    }

    // Synthesize macromolecules (scan compound skills)
    let registry = synthesize_macromolecules(&skills_dir);

    if registry.macromolecules.is_empty() {
        exit_skip_session();
    }

    // Persist registry
    let registry_path = home.join(".claude/brain/macromolecule_registry.json");
    if let Some(parent) = registry_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: Could not create registry directory: {e}");
        }
    }
    match serde_json::to_string_pretty(&registry) {
        Ok(json) => {
            if let Err(e) = fs::write(&registry_path, json) {
                eprintln!("Warning: Could not write registry: {e}");
            }
        }
        Err(e) => eprintln!("Warning: Could not serialize registry: {e}"),
    }

    // Build context message with chemistry notation
    let compound_count = registry.macromolecules.len();
    let mut ctx = format!(
        "🧬 **MACROMOLECULE SYNTHESIS** ─────────────────────────────\n\
         Synthesized {compound_count} compound skills (total weight: {})\n\n\
         Molecular Formulas:\n",
        registry.total_weight
    );

    for formula in &registry.formulas {
        ctx.push_str(&format!("  {formula}\n"));
    }

    ctx.push_str("\nStructure:\n");
    for (_name, mol) in &registry.macromolecules {
        let stability = if mol.stable {
            "✓ stable"
        } else {
            "⚠ unstable"
        };
        ctx.push_str(&format!(
            "  {} [{stability}] depth={}\n",
            mol.formula, mol.nesting_depth
        ));
        for unit in &mol.polymer_units {
            let bond_sym = match unit.bond_type {
                BondType::Covalent => "══",
                BondType::Ionic => "──",
                BondType::Broken => "╳╳",
            };
            let compound_marker = if unit.is_compound { " 🔗" } else { "" };
            ctx.push_str(&format!("    {bond_sym} {}{compound_marker}\n", unit.name));
        }
    }

    if !registry.synthesis_errors.is_empty() {
        ctx.push_str(&format!(
            "\n⚠️ Synthesis errors ({}):\n",
            registry.synthesis_errors.len()
        ));
        for err in registry.synthesis_errors.iter().take(5) {
            ctx.push_str(&format!("  • {err}\n"));
        }
    }

    ctx.push_str("───────────────────────────────────────────────────────────\n");

    exit_with_session_context(&ctx);
}

/// Synthesize macromolecules from skills directory
fn synthesize_macromolecules(skills_dir: &Path) -> MacromoleculeRegistry {
    let mut registry = MacromoleculeRegistry {
        version: "1.0".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };

    // First pass: identify all compound skills
    let Ok(entries) = fs::read_dir(skills_dir) else {
        return registry;
    };

    // Collect all skills that have nested-skills
    let mut compounds: Vec<(String, PathBuf, Vec<String>)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let Ok(content) = fs::read_to_string(&skill_md) else {
            continue;
        };

        let Some(frontmatter) = parse_frontmatter(&content) else {
            continue;
        };

        // Check for nested-skills field
        if let Some(nested_skills) = frontmatter.get("nested-skills") {
            let skill_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let declared: Vec<String> = match nested_skills {
                serde_yaml::Value::Sequence(seq) => seq
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
                _ => continue,
            };

            if !declared.is_empty() {
                compounds.push((skill_name, path, declared));
            }
        }
    }

    // Second pass: resolve polymer units and compute properties
    for (skill_name, skill_path, declared) in compounds {
        let mut polymer_units = Vec::new();
        let mut broken_bonds = Vec::new();
        let mut total_weight = 10u32; // Base weight for nucleus
        let mut max_depth = 1u8;
        let mut contains_compounds = false;

        for nested_path in &declared {
            let full_path = skill_path.join(nested_path);
            let nested_skill_md = full_path.join("SKILL.md");

            if nested_skill_md.exists() {
                let (bond_type, is_compound, unit_weight) =
                    analyze_polymer_unit(&nested_skill_md, &skill_name);

                if is_compound {
                    contains_compounds = true;
                    max_depth = max_depth.max(2);
                }

                let nested_name = full_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                total_weight += unit_weight;

                polymer_units.push(PolymerUnit {
                    name: nested_name,
                    path: full_path.to_string_lossy().to_string(),
                    bond_type,
                    bond_strength: bond_type.strength(),
                    is_compound,
                    weight: unit_weight,
                });
            } else {
                broken_bonds.push(nested_path.clone());
                registry.synthesis_errors.push(format!(
                    "{}: broken bond to '{}' (not found)",
                    skill_name, nested_path
                ));
            }
        }

        let stable = broken_bonds.is_empty()
            && polymer_units
                .iter()
                .all(|u| matches!(u.bond_type, BondType::Covalent));

        // Generate molecular formula: name(n) or name(n)* if contains compounds
        let formula = if contains_compounds {
            format!("{}({})⁺", skill_name, polymer_units.len())
        } else {
            format!("{}({})", skill_name, polymer_units.len())
        };

        registry.formulas.push(formula.clone());
        registry.total_units += polymer_units.len();
        registry.total_weight += total_weight;

        registry.macromolecules.insert(
            skill_name.clone(),
            Macromolecule {
                nucleus: skill_name.clone(),
                nucleus_path: skill_path.to_string_lossy().to_string(),
                formula,
                weight: total_weight,
                declared_chain: declared,
                polymer_units,
                broken_bonds,
                stable,
                nesting_depth: max_depth,
                contains_compounds,
            },
        );
    }

    registry
}

/// Analyze a polymer unit (nested skill) and return bond type and properties
fn analyze_polymer_unit(skill_md: &Path, expected_parent: &str) -> (BondType, bool, u32) {
    let Ok(content) = fs::read_to_string(skill_md) else {
        return (BondType::Broken, false, 0);
    };

    let Some(frontmatter) = parse_frontmatter(&content) else {
        return (BondType::Ionic, false, 5); // Exists but no valid frontmatter
    };

    // Check parent reference
    let has_valid_parent = frontmatter
        .get("parent")
        .and_then(|v| v.as_str())
        .map(|p| p == expected_parent)
        .unwrap_or(false);

    // Check if this is also a compound
    let is_compound = frontmatter.get("nested-skills").is_some();

    // Calculate weight based on complexity
    let weight = if is_compound { 15 } else { 5 };

    let bond_type = if has_valid_parent {
        BondType::Covalent
    } else {
        BondType::Ionic
    };

    (bond_type, is_compound, weight)
}

/// Parse YAML frontmatter from SKILL.md content
fn parse_frontmatter(content: &str) -> Option<serde_yaml::Value> {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        return None;
    }

    let rest = &trimmed[3..];
    let end = rest.find("---")?;
    let yaml_str = &rest[..end].trim();

    serde_yaml::from_str(yaml_str).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_type_strength() {
        assert_eq!(BondType::Covalent.strength(), 100);
        assert_eq!(BondType::Ionic.strength(), 50);
        assert_eq!(BondType::Broken.strength(), 0);
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-skill
nested-skills:
  - skills/child1
  - skills/child2
---

# Test Skill
"#;

        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm["name"].as_str(), Some("test-skill"));

        let nested = fm["nested-skills"].as_sequence().unwrap();
        assert_eq!(nested.len(), 2);
    }

    #[test]
    fn test_parse_frontmatter_with_parent() {
        let content = r#"---
name: child-skill
parent: forge
---

# Child Skill
"#;

        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm["parent"].as_str(), Some("forge"));
    }
}
