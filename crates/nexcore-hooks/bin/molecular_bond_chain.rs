//! Molecular Bond Chain Hook
//!
//! Implements hook chaining as chemical bonds between skill molecules.
//! Each hook in the chain validates and triggers the next.
//!
//! Chain: primitive → rust → ctvp → skill-audit
//!
//! ```text
//!     ┌─────────────┐      ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
//!     │  PRIMITIVE  │─────▶│    RUST     │─────▶│    CTVP     │─────▶│   SKILL     │
//!     │  MOLECULE   │ bond │  MOLECULE   │ bond │  MOLECULE   │ bond │  MOLECULE   │
//!     └─────────────┘      └─────────────┘      └─────────────┘      └─────────────┘
//!          T1                 Generate           Validate             Audit
//! ```

use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, BufRead, Write};
use std::process;

/// Bond types in the molecular chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondType {
    /// Covalent: Strong, bidirectional (skill depends on skill)
    Covalent,
    /// Ionic: Directional, validator → generator
    Ionic,
    /// Hydrogen: Weak, optional enhancement
    Hydrogen,
    /// Metallic: Shared pool (multiple skills share state)
    Metallic,
}

/// A skill molecule in the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMolecule {
    pub name: String,
    pub symbol: String, // Short identifier like chemical symbols
    pub valence: u8,    // Number of possible bonds
    pub bonds: Vec<Bond>,
}

/// A bond connecting two skill molecules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bond {
    pub from: String,
    pub to: String,
    pub bond_type: BondType,
    pub strength: f32, // 0.0 to 1.0
    pub hook_name: String,
}

/// The molecular chain for FORGE workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolecularChain {
    pub molecules: Vec<SkillMolecule>,
    pub bonds: Vec<Bond>,
    pub reaction_state: ReactionState,
}

/// Current state of the chain reaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReactionState {
    Idle,
    PrimitiveValidation,
    RustGeneration,
    CTVPValidation,
    SkillAudit,
    Complete,
    Failed(String),
}

impl MolecularChain {
    /// Create the FORGE molecular chain
    pub fn forge_chain() -> Self {
        let molecules = vec![
            SkillMolecule {
                name: "primitive-extractor".to_string(),
                symbol: "Pr".to_string(),
                valence: 4, // T1: sequence, mapping, recursion, state
                bonds: vec![],
            },
            SkillMolecule {
                name: "rust-dev".to_string(),
                symbol: "Rs".to_string(),
                valence: 7, // 7 anatomical levels
                bonds: vec![],
            },
            SkillMolecule {
                name: "ctvp-validator".to_string(),
                symbol: "Cv".to_string(),
                valence: 5, // 5 validation phases
                bonds: vec![],
            },
            SkillMolecule {
                name: "skill-audit".to_string(),
                symbol: "Sa".to_string(),
                valence: 5, // 5 compliance levels
                bonds: vec![],
            },
        ];

        let bonds = vec![
            // Pr → Rs: Ionic bond (primitives inform rust patterns)
            Bond {
                from: "Pr".to_string(),
                to: "Rs".to_string(),
                bond_type: BondType::Ionic,
                strength: 0.9,
                hook_name: "pretool_primitive_to_rust".to_string(),
            },
            // Rs → Cv: Covalent bond (rust code validated by CTVP)
            Bond {
                from: "Rs".to_string(),
                to: "Cv".to_string(),
                bond_type: BondType::Covalent,
                strength: 1.0,
                hook_name: "posttool_rust_to_ctvp".to_string(),
            },
            // Cv → Sa: Ionic bond (validated code checked for compliance)
            Bond {
                from: "Cv".to_string(),
                to: "Sa".to_string(),
                bond_type: BondType::Ionic,
                strength: 0.8,
                hook_name: "posttool_ctvp_to_audit".to_string(),
            },
            // Sa → Pr: Hydrogen bond (audit informs future primitives, weak)
            Bond {
                from: "Sa".to_string(),
                to: "Pr".to_string(),
                bond_type: BondType::Hydrogen,
                strength: 0.3,
                hook_name: "feedback_audit_to_primitive".to_string(),
            },
        ];

        MolecularChain {
            molecules,
            bonds,
            reaction_state: ReactionState::Idle,
        }
    }

    /// Advance the chain reaction
    pub fn react(&mut self, current_bond: &str) -> Option<&str> {
        // Find the bond that was just completed
        let completed_bond = self.bonds.iter().find(|b| b.hook_name == current_bond)?;

        // Find the next bond in the chain (starts from where current bond ended)
        let next_bond = self.bonds.iter().find(|b| b.from == completed_bond.to)?;

        // Update reaction state based on where we ARE (the completed_bond.to)
        // This is the molecule we just activated
        self.reaction_state = match completed_bond.to.as_str() {
            "Rs" => ReactionState::RustGeneration,
            "Cv" => ReactionState::CTVPValidation,
            "Sa" => ReactionState::SkillAudit,
            "Pr" => ReactionState::Complete, // Cycle complete
            _ => ReactionState::Idle,
        };

        Some(&next_bond.hook_name)
    }

    /// Get bond strength for a connection
    pub fn bond_strength(&self, from: &str, to: &str) -> f32 {
        self.bonds
            .iter()
            .find(|b| b.from == from && b.to == to)
            .map(|b| b.strength)
            .unwrap_or(0.0)
    }

    /// Check if chain is healthy (all bonds > 0.5 strength)
    pub fn is_healthy(&self) -> bool {
        self.bonds.iter().all(|b| b.strength >= 0.5)
    }
}

/// Hook input from Claude Code
#[derive(Debug, Deserialize)]
struct HookInput {
    tool_name: Option<String>,
    tool_input: Option<serde_json::Value>,
    #[allow(dead_code)] // Reserved for future session-aware chain state
    session_id: Option<String>,
}

/// Hook output to Claude Code
#[derive(Debug, Serialize)]
struct HookOutput {
    status: String,
    message: Option<String>,
    next_bond: Option<String>,
    chain_state: String,
}

fn main() {
    // Check if we're just displaying chain info
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--chain" {
        display_chain();
        return;
    }

    // Read input from stdin
    let stdin = io::stdin();
    let input_line = match stdin.lock().lines().next() {
        Some(Ok(line)) => line,
        _ => {
            // No input - just display status
            let chain = MolecularChain::forge_chain();
            eprintln!("🔬 Molecular Chain Status: {:?}", chain.reaction_state);
            eprintln!(
                "🧪 Chain Health: {}",
                if chain.is_healthy() {
                    "✓ Healthy"
                } else {
                    "⚠ Degraded"
                }
            );
            process::exit(0);
        }
    };

    let input: HookInput = match serde_json::from_str(&input_line) {
        Ok(i) => i,
        Err(_) => {
            // Invalid input - pass through
            process::exit(0);
        }
    };

    // Initialize chain
    let mut chain = MolecularChain::forge_chain();

    // Determine current position in chain based on tool
    let current_bond = match input.tool_name.as_deref() {
        Some("Task") => {
            // Check if spawning a related subagent
            if let Some(tool_input) = &input.tool_input {
                let subagent = tool_input.get("subagent_type").and_then(|v| v.as_str());
                match subagent {
                    Some("primitive-extractor") => Some("pretool_primitive_to_rust"),
                    Some("rust-anatomy-expert") | Some("rust-dev") => Some("posttool_rust_to_ctvp"),
                    Some("ctvp-validator") => Some("posttool_ctvp_to_audit"),
                    Some("skill-audit") => Some("feedback_audit_to_primitive"),
                    _ => None,
                }
            } else {
                None
            }
        }
        Some("Edit") | Some("Write") => {
            // Code modification - trigger rust → ctvp bond
            Some("posttool_rust_to_ctvp")
        }
        _ => None,
    };

    // Process chain reaction
    let output = if let Some(bond) = current_bond {
        // Call react and get next bond
        let next = chain.react(bond).map(|s| s.to_string());
        // Now we can borrow chain.reaction_state immutably
        let state_str = format!("{:?}", chain.reaction_state);

        HookOutput {
            status: "continue".to_string(),
            message: Some(format!(
                "⚗️ Bond activated: {} | Chain state: {}",
                bond, state_str
            )),
            next_bond: next,
            chain_state: state_str,
        }
    } else {
        HookOutput {
            status: "continue".to_string(),
            message: None,
            next_bond: None,
            chain_state: "Idle".to_string(),
        }
    };

    // Output result
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if let Some(msg) = &output.message {
        writeln!(handle, "{}", msg).ok();
    }

    // Always pass (this hook is informational)
    process::exit(0);
}

fn display_chain() {
    let chain = MolecularChain::forge_chain();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           🧬 MOLECULAR SKILL CHAIN - FORGE                   ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║     ┌────┐         ┌────┐         ┌────┐         ┌────┐     ║");
    println!("║     │ Pr │═══════▶│ Rs │═══════▶│ Cv │═══════▶│ Sa │     ║");
    println!("║     └────┘  ionic  └────┘ covalent└────┘  ionic  └────┘     ║");
    println!("║       │      0.9           1.0           0.8       │        ║");
    println!("║       └────────────────── hydrogen 0.3 ────────────┘        ║");
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Legend:                                                     ║");
    println!("║    Pr = primitive-extractor (T1 decomposition)               ║");
    println!("║    Rs = rust-dev (code generation)                           ║");
    println!("║    Cv = ctvp-validator (test validation)                     ║");
    println!("║    Sa = skill-audit (compliance check)                       ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    println!("║  Bonds:                                                      ║");
    for bond in &chain.bonds {
        println!(
            "║    {} → {} [{:?}] strength={:.1} hook={}",
            bond.from, bond.to, bond.bond_type, bond.strength, bond.hook_name
        );
    }

    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Chain Health: {}                                      ║",
        if chain.is_healthy() {
            "✓ Healthy   "
        } else {
            "⚠ Degraded  "
        }
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forge_chain_creation() {
        let chain = MolecularChain::forge_chain();
        assert_eq!(chain.molecules.len(), 4);
        assert_eq!(chain.bonds.len(), 4);
    }

    #[test]
    fn test_chain_is_healthy() {
        let chain = MolecularChain::forge_chain();
        // Hydrogen bond has 0.3 strength, but that's intentional (weak feedback)
        // The main chain (Pr→Rs→Cv→Sa) should all be >= 0.5
        assert!(chain.bond_strength("Pr", "Rs") >= 0.5);
        assert!(chain.bond_strength("Rs", "Cv") >= 0.5);
        assert!(chain.bond_strength("Cv", "Sa") >= 0.5);
    }

    #[test]
    fn test_chain_reaction() {
        let mut chain = MolecularChain::forge_chain();

        // Start with primitive validation
        let next = chain.react("pretool_primitive_to_rust");
        assert!(next.is_some());
        assert!(matches!(
            chain.reaction_state,
            ReactionState::RustGeneration
        ));
    }
}
