//! # Grammar Types
//!
//! Type-safe representations of domain grammars built via the
//! Domain Discovery Framework (Phase 4: BUILD_GRAMMAR).
//!
//! ## Structure
//!
//! A grammar consists of:
//! - **Terminals**: Primitive symbols (from Phase 3)
//! - **Non-terminals**: Induced categories (Subject, Event, etc.)
//! - **Productions**: Rules for combining symbols

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Grammatical category for a terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum GrammarCategory {
    /// Subject of a statement (entity, state, vigilance_system).
    Subject,
    /// Object of a statement (manifold, detection_result).
    Object,
    /// Dynamic process (observation, perturbation).
    Action,
    /// Extended dynamics (emergence, attenuation).
    Process,
    /// Discrete occurrence (harm_event).
    Event,
    /// Connection between concepts (constraint, conservation_law).
    Relation,
    /// Quantifiable property (safety_margin, signed_distance).
    Measure,
    /// Qualification (time, level).
    Modifier,
    /// Categorical assignment (harm_type, conservation_law_type).
    Classification,
    /// Organizational pattern (hierarchy, harm_taxonomy).
    Structure,
}

impl GrammarCategory {
    /// Returns all grammar categories.
    pub const ALL: [GrammarCategory; 10] = [
        Self::Subject,
        Self::Object,
        Self::Action,
        Self::Process,
        Self::Event,
        Self::Relation,
        Self::Measure,
        Self::Modifier,
        Self::Classification,
        Self::Structure,
    ];

    /// Returns human-readable name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Subject => "Subject",
            Self::Object => "Object",
            Self::Action => "Action",
            Self::Process => "Process",
            Self::Event => "Event",
            Self::Relation => "Relation",
            Self::Measure => "Measure",
            Self::Modifier => "Modifier",
            Self::Classification => "Classification",
            Self::Structure => "Structure",
        }
    }
}

/// A terminal symbol in the grammar (derived from a primitive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terminal {
    /// Terminal name (matches primitive name).
    pub name: String,
    /// Primitive tier (T1/T2/T3).
    pub tier: super::PrimitiveTier,
    /// Grammatical category.
    pub category: GrammarCategory,
    /// BNF symbol (uppercase).
    pub symbol: String,
}

impl Terminal {
    /// Creates a new terminal.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        tier: super::PrimitiveTier,
        category: GrammarCategory,
    ) -> Self {
        let name = name.into();
        let symbol = name.to_uppercase();
        Self {
            name,
            tier,
            category,
            symbol,
        }
    }
}

/// A non-terminal symbol in the grammar (induced category).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonTerminal {
    /// Non-terminal name (e.g., "Statement", "AxiomStatement").
    pub name: String,
    /// Description of what this non-terminal represents.
    #[serde(default)]
    pub description: Option<String>,
}

impl NonTerminal {
    /// Creates a new non-terminal.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    /// Adds a description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A symbol in a production rule (terminal or non-terminal).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Symbol {
    /// Terminal symbol (primitive name or literal).
    Terminal(String),
    /// Non-terminal symbol (category name).
    NonTerminal(String),
    /// Optional symbol (may or may not appear).
    Optional(Box<Symbol>),
    /// Repeated symbol (one or more).
    Repeat(Box<Symbol>),
}

impl Symbol {
    /// Creates a terminal symbol.
    #[must_use]
    pub fn terminal(name: impl Into<String>) -> Self {
        Self::Terminal(name.into())
    }

    /// Creates a non-terminal symbol.
    #[must_use]
    pub fn non_terminal(name: impl Into<String>) -> Self {
        Self::NonTerminal(name.into())
    }

    /// Wraps symbol as optional.
    #[must_use]
    pub fn optional(self) -> Self {
        Self::Optional(Box::new(self))
    }

    /// Wraps symbol as repeated (one or more).
    #[must_use]
    pub fn repeat(self) -> Self {
        Self::Repeat(Box::new(self))
    }
}

/// Unique identifier for a production rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductionId(pub String);

impl ProductionId {
    /// Creates a new production ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ProductionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A production rule in the grammar.
///
/// Format: `LHS → RHS[0] RHS[1] ... RHS[n]`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Production {
    /// Unique identifier (e.g., "PROD-001").
    pub id: ProductionId,

    /// Human-readable name.
    #[serde(default)]
    pub name: Option<String>,

    /// Left-hand side (non-terminal being expanded).
    pub lhs: String,

    /// Right-hand side (sequence of symbols).
    pub rhs: Vec<Symbol>,

    /// Production weight for probabilistic generation.
    #[serde(default = "default_weight")]
    pub weight: f64,

    /// Description of what this rule produces.
    #[serde(default)]
    pub description: Option<String>,

    /// Example output of this rule.
    #[serde(default)]
    pub example: Option<String>,
}

fn default_weight() -> f64 {
    1.0
}

impl Production {
    /// Creates a new production rule.
    #[must_use]
    pub fn new(id: impl Into<String>, lhs: impl Into<String>, rhs: Vec<Symbol>) -> Self {
        Self {
            id: ProductionId::new(id),
            name: None,
            lhs: lhs.into(),
            rhs,
            weight: 1.0,
            description: None,
            example: None,
        }
    }

    /// Adds a name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the weight.
    #[must_use]
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Returns the number of symbols in the RHS.
    #[must_use]
    pub fn rhs_length(&self) -> usize {
        self.rhs.len()
    }

    /// Checks if this is a terminal rule (RHS is a single terminal).
    #[must_use]
    pub fn is_terminal_rule(&self) -> bool {
        self.rhs.len() == 1 && matches!(self.rhs.first(), Some(Symbol::Terminal(_)))
    }
}

/// Grammar validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarValidation {
    /// Whether the grammar is valid.
    pub is_valid: bool,
    /// Soundness score (generated statements are valid).
    pub soundness: f64,
    /// Completeness score (covers domain concepts).
    pub completeness: f64,
    /// Parsability score (statements can be parsed back).
    pub parsability: f64,
    /// Ambiguity score (lower is better).
    pub ambiguity: f64,
    /// List of issues found.
    #[serde(default)]
    pub issues: Vec<String>,
}

impl GrammarValidation {
    /// Checks if validation passed with acceptable scores.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.is_valid
            && self.soundness >= 0.90
            && self.completeness >= 0.90
            && self.parsability >= 0.90
            && self.ambiguity <= 0.15
    }
}

/// A complete domain grammar (Phase 4 output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    /// Domain name.
    pub domain: String,

    /// Domain version.
    pub version: String,

    /// Grammar mode (e.g., "Induction").
    #[serde(default)]
    pub mode: Option<String>,

    /// Start symbol for derivations.
    pub start_symbol: String,

    /// Terminal symbols (primitive-derived).
    pub terminals: HashMap<String, Terminal>,

    /// Non-terminal symbols (induced categories).
    pub non_terminals: Vec<String>,

    /// Production rules.
    pub productions: Vec<Production>,

    /// Validation results.
    #[serde(default)]
    pub validation: Option<GrammarValidation>,

    /// BNF grammar export.
    #[serde(default)]
    pub bnf_grammar: Option<String>,
}

impl Grammar {
    /// Loads grammar from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed.
    pub fn from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let result: Self = serde_yml::from_str(&content)?;
        Ok(result)
    }

    /// Returns terminal count.
    #[must_use]
    pub fn terminal_count(&self) -> usize {
        self.terminals.len()
    }

    /// Returns non-terminal count.
    #[must_use]
    pub fn non_terminal_count(&self) -> usize {
        self.non_terminals.len()
    }

    /// Returns production count.
    #[must_use]
    pub fn production_count(&self) -> usize {
        self.productions.len()
    }

    /// Finds a terminal by name.
    #[must_use]
    pub fn find_terminal(&self, name: &str) -> Option<&Terminal> {
        self.terminals.get(name)
    }

    /// Finds productions by LHS.
    pub fn find_productions_by_lhs(&self, lhs: &str) -> impl Iterator<Item = &Production> {
        self.productions.iter().filter(move |p| p.lhs == lhs)
    }

    /// Returns all axiom statement productions.
    pub fn axiom_productions(&self) -> impl Iterator<Item = &Production> {
        self.find_productions_by_lhs("AxiomStatement")
    }

    /// Returns all harm statement productions.
    pub fn harm_productions(&self) -> impl Iterator<Item = &Production> {
        self.find_productions_by_lhs("HarmStatement")
    }

    /// Returns all safety statement productions.
    pub fn safety_productions(&self) -> impl Iterator<Item = &Production> {
        self.find_productions_by_lhs("SafetyStatement")
    }

    /// Average RHS length across all productions.
    #[must_use]
    pub fn avg_rule_length(&self) -> f64 {
        if self.productions.is_empty() {
            return 0.0;
        }
        let total: usize = self.productions.iter().map(|p| p.rhs_length()).sum();
        total as f64 / self.productions.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_category() {
        assert_eq!(GrammarCategory::Subject.name(), "Subject");
        assert_eq!(GrammarCategory::ALL.len(), 10);
    }

    #[test]
    fn test_terminal_creation() {
        let t = Terminal::new(
            "entity",
            super::super::PrimitiveTier::T1Universal,
            GrammarCategory::Subject,
        );
        assert_eq!(t.name, "entity");
        assert_eq!(t.symbol, "ENTITY");
    }

    #[test]
    fn test_production_creation() {
        let p = Production::new(
            "PROD-001",
            "Statement",
            vec![
                Symbol::non_terminal("Subject"),
                Symbol::non_terminal("Predicate"),
                Symbol::non_terminal("Object"),
            ],
        )
        .with_name("Core Statement")
        .with_weight(1.0);

        assert_eq!(p.rhs_length(), 3);
        assert!(!p.is_terminal_rule());
    }

    #[test]
    fn test_validation_passed() {
        let valid = GrammarValidation {
            is_valid: true,
            soundness: 0.96,
            completeness: 0.94,
            parsability: 0.98,
            ambiguity: 0.08,
            issues: vec![],
        };
        assert!(valid.passed());

        let invalid = GrammarValidation {
            is_valid: true,
            soundness: 0.85, // Too low
            completeness: 0.94,
            parsability: 0.98,
            ambiguity: 0.08,
            issues: vec![],
        };
        assert!(!invalid.passed());
    }
}
