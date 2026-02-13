//! Strategy Chain Reactor - PostToolUse Hook
//!
//! Implements chemical reaction cascades for strategy-related skill molecules.
//! When a strategy skill completes, this hook triggers the next skill in the
//! reaction chain based on bond affinities.
//!
//! Reaction Types:
//! - **Synthesis**: A + B → AB (combining outputs)
//! - **Decomposition**: AB → A + B (breaking into primitives)
//! - **Single Replacement**: A + BC → AC + B (capability substitution)
//! - **Double Replacement**: AB + CD → AD + CB (cross-domain transfer)
//!
//! Event: PostToolUse
//! Matcher: Task (detects subagent completions)

use chrono::Utc;
use nexcore_hooks::{HookInput, exit_success_auto, exit_success_auto_with, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// =============================================================================
// Reaction Types (Chemistry Metaphor)
// =============================================================================

/// Chemical reaction type for skill chaining
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionType {
    /// Synthesis: Combine skills into unified output
    /// strat-dev + primitive-extractor → unified_strategy
    Synthesis,

    /// Decomposition: Break skill output into primitives
    /// strategy → T1_primitives[] + capabilities[]
    Decomposition,

    /// SingleReplacement: Substitute one skill for another
    /// rust-dev replaces python-dev in capability
    SingleReplacement,

    /// DoubleReplacement: Cross-domain transfer
    /// PV_signal + sports_data → betting_signal
    DoubleReplacement,

    /// Catalyst: Enables reaction without being consumed
    /// skill-advisor ~~ any_skill
    Catalyst,
}

impl ReactionType {
    /// Get the reaction symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Synthesis => "+→",
            Self::Decomposition => "→/",
            Self::SingleReplacement => "↔",
            Self::DoubleReplacement => "⇄",
            Self::Catalyst => "~~",
        }
    }
}

// =============================================================================
// Reaction Rules (Predefined Chains)
// =============================================================================

/// A reaction rule defining how skills chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionRule {
    /// Unique identifier for the rule
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Reactant skills (inputs)
    pub reactants: Vec<String>,

    /// Product skills (outputs - skills to suggest/trigger)
    pub products: Vec<String>,

    /// Reaction type
    pub reaction_type: ReactionType,

    /// Activation energy (priority - lower = triggers first)
    pub activation_energy: u8,

    /// Conditions for reaction (optional)
    #[serde(default)]
    pub conditions: Vec<String>,
}

/// Get the predefined strategy reaction rules
fn strategy_reaction_rules() -> Vec<ReactionRule> {
    vec![
        // =========================================
        // SYNTHESIS REACTIONS (Combining outputs)
        // =========================================
        ReactionRule {
            id: "SR-001".into(),
            name: "Strategy-to-Primitive Synthesis".into(),
            reactants: vec!["strat-dev".into()],
            products: vec!["primitive-extractor".into()],
            reaction_type: ReactionType::Synthesis,
            activation_energy: 1,
            conditions: vec!["phase >= 4".into(), "capabilities identified".into()],
        },
        ReactionRule {
            id: "SR-002".into(),
            name: "Primitive-to-Rust Synthesis".into(),
            reactants: vec!["primitive-extractor".into()],
            products: vec!["rust-anatomy-expert".into(), "rust-dev".into()],
            reaction_type: ReactionType::Synthesis,
            activation_energy: 2,
            conditions: vec!["T1 primitives extracted".into()],
        },
        ReactionRule {
            id: "SR-003".into(),
            name: "Strategy-Rust Chain".into(),
            reactants: vec!["strat-dev".into(), "primitive-extractor".into()],
            products: vec!["forge".into()],
            reaction_type: ReactionType::Synthesis,
            activation_energy: 3,
            conditions: vec!["full strategy complete".into()],
        },
        // =========================================
        // DECOMPOSITION REACTIONS (Breaking down)
        // =========================================
        ReactionRule {
            id: "DR-001".into(),
            name: "Strategy Decomposition".into(),
            reactants: vec!["strat-dev".into()],
            products: vec!["primitive-extractor".into()],
            reaction_type: ReactionType::Decomposition,
            activation_energy: 1,
            conditions: vec!["capabilities need primitives".into()],
        },
        ReactionRule {
            id: "DR-002".into(),
            name: "Capability Decomposition".into(),
            reactants: vec!["strat-dev".into()],
            products: vec!["constructive-epistemology".into()],
            reaction_type: ReactionType::Decomposition,
            activation_energy: 2,
            conditions: vec!["learning gaps identified".into()],
        },
        // =========================================
        // SINGLE REPLACEMENT REACTIONS
        // =========================================
        ReactionRule {
            id: "RR-001".into(),
            name: "Python-to-Rust Replacement".into(),
            reactants: vec!["rust-dev".into()],
            products: vec!["guardian-orchestrator".into()],
            reaction_type: ReactionType::SingleReplacement,
            activation_energy: 2,
            conditions: vec!["python migration active".into()],
        },
        // =========================================
        // DOUBLE REPLACEMENT (Cross-domain)
        // =========================================
        ReactionRule {
            id: "XR-001".into(),
            name: "PV-to-Sports Signal Transfer".into(),
            reactants: vec!["vigilance-dev".into()],
            products: vec!["nexcore-api-dev".into()],
            reaction_type: ReactionType::DoubleReplacement,
            activation_energy: 3,
            conditions: vec!["cross-domain signal detection".into()],
        },
        // =========================================
        // CATALYST REACTIONS (Enabling)
        // =========================================
        ReactionRule {
            id: "CAT-001".into(),
            name: "Skill Advisor Catalyst".into(),
            reactants: vec!["skill-advisor".into()],
            products: vec![], // Enables any skill
            reaction_type: ReactionType::Catalyst,
            activation_energy: 0,
            conditions: vec![],
        },
        ReactionRule {
            id: "CAT-002".into(),
            name: "CTVP Validation Catalyst".into(),
            reactants: vec!["ctvp-validator".into()],
            products: vec![], // Validates any code
            reaction_type: ReactionType::Catalyst,
            activation_energy: 1,
            conditions: vec!["code produced".into()],
        },
    ]
}

// =============================================================================
// Reaction State
// =============================================================================

/// State tracking active reactions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReactionState {
    /// Currently active reactants (skills in progress)
    pub active_reactants: Vec<String>,

    /// Reactions that have completed
    pub completed_reactions: Vec<String>,

    /// Pending products (skills suggested but not yet invoked)
    pub pending_products: Vec<String>,

    /// Session context
    pub session_id: String,

    /// Last updated
    pub updated_at: String,
}

impl ReactionState {
    /// Load state from disk
    fn load(session_id: &str) -> Self {
        let path = state_path(session_id);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self {
            session_id: session_id.to_string(),
            ..Default::default()
        }
    }

    /// Save state to disk
    fn save(&self) -> std::io::Result<()> {
        let path = state_path(&self.session_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Record a skill activation
    fn activate_skill(&mut self, skill: &str) {
        if !self.active_reactants.contains(&skill.to_string()) {
            self.active_reactants.push(skill.to_string());
        }
        // Remove from pending if it was suggested
        self.pending_products.retain(|p| p != skill);
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Check for matching reactions and return suggested products
    fn check_reactions(&mut self, rules: &[ReactionRule]) -> Vec<(ReactionRule, Vec<String>)> {
        let mut triggered = Vec::new();

        for rule in rules {
            // Check if all reactants are active
            let all_present = rule
                .reactants
                .iter()
                .all(|r| self.active_reactants.contains(r));

            if all_present && !self.completed_reactions.contains(&rule.id) {
                // Reaction can proceed
                let products: Vec<String> = rule
                    .products
                    .iter()
                    .filter(|p| !self.active_reactants.contains(*p))
                    .cloned()
                    .collect();

                if !products.is_empty() || rule.reaction_type == ReactionType::Catalyst {
                    triggered.push((rule.clone(), products.clone()));

                    // Mark reaction as completed
                    self.completed_reactions.push(rule.id.clone());

                    // Add products to pending
                    for product in products {
                        if !self.pending_products.contains(&product) {
                            self.pending_products.push(product);
                        }
                    }
                }
            }
        }

        // Sort by activation energy (priority)
        triggered.sort_by_key(|(r, _)| r.activation_energy);
        triggered
    }
}

/// Get path to reaction state file
fn state_path(session_id: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("brain")
        .join("skill_bonds")
        .join("reactions")
        .join(format!("{}.json", session_id))
}

// =============================================================================
// Skill Detection (reused from bonding tracker)
// =============================================================================

/// Detect skill from hook input
fn detect_skill(input: &HookInput) -> Option<String> {
    // Check tool_input for skill field
    if let Some(tool_input) = &input.tool_input {
        if let Some(skill) = tool_input.get("skill").and_then(|v| v.as_str()) {
            return Some(skill.to_string());
        }
        // Check subagent_type
        if let Some(agent_type) = tool_input.get("subagent_type").and_then(|v| v.as_str()) {
            return Some(agent_type.to_lowercase().replace('_', "-"));
        }
    }

    // Check agent_type
    if let Some(agent_type) = &input.agent_type {
        let normalized = agent_type
            .trim_start_matches('/')
            .to_lowercase()
            .replace('_', "-");
        if !normalized.is_empty() && normalized != "general-purpose" {
            return Some(normalized);
        }
    }

    // Check custom instructions for skill patterns
    if let Some(instructions) = &input.custom_instructions {
        if let Ok(re) = regex::Regex::new(r"(?i)skill[:\s]+([a-z0-9-]+)") {
            if let Some(caps) = re.captures(instructions) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }
    }

    None
}

// =============================================================================
// Main Hook Logic
// =============================================================================

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process Task tool completions
    if input.tool_name.as_deref() != Some("Task") {
        exit_success_auto();
    }

    // Detect the skill that just completed
    let skill = match detect_skill(&input) {
        Some(s) => s,
        None => exit_success_auto(),
    };

    // Load reaction state
    let mut state = ReactionState::load(&input.session_id);

    // Activate the completed skill
    state.activate_skill(&skill);

    // Get reaction rules
    let rules = strategy_reaction_rules();

    // Check for triggered reactions
    let triggered = state.check_reactions(&rules);

    // Save state
    if let Err(e) = state.save() {
        eprintln!("Warning: Failed to save reaction state: {}", e);
    }

    // Format output message
    if triggered.is_empty() {
        exit_success_auto_with(&format!("⚛️ Skill activated: {}", skill));
    } else {
        let mut msg = format!("⚛️ Skill {} triggered reactions:\n", skill);

        for (rule, products) in &triggered {
            msg.push_str(&format!(
                "   {} {} {}\n",
                rule.reactants.join(" + "),
                rule.reaction_type.symbol(),
                if products.is_empty() {
                    "(catalyst)".to_string()
                } else {
                    products.join(" + ")
                }
            ));
        }

        if !state.pending_products.is_empty() {
            msg.push_str(&format!(
                "\n   💡 Suggested next: {}",
                state.pending_products.join(", ")
            ));
        }

        exit_success_auto_with(&msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_rules_exist() {
        let rules = strategy_reaction_rules();
        assert!(!rules.is_empty());
    }

    #[test]
    fn test_reaction_state_activate() {
        let mut state = ReactionState::default();
        state.activate_skill("strat-dev");
        assert!(state.active_reactants.contains(&"strat-dev".to_string()));
    }

    #[test]
    fn test_reaction_check() {
        let mut state = ReactionState::default();
        state.activate_skill("strat-dev");

        let rules = strategy_reaction_rules();
        let triggered = state.check_reactions(&rules);

        // Should trigger SR-001 (Strategy-to-Primitive Synthesis)
        assert!(!triggered.is_empty());
        assert!(triggered.iter().any(|(r, _)| r.id == "SR-001"));
    }
}
