//! # Chomsky Level Classification
//!
//! Classifies each workspace crate by its computational complexity using the
//! Lex Primitiva generator algebra: `Lex = ⟨σ, Σ, ρ, κ, ∃ | ∅ = ¬∃⟩`.
//!
//! The 5 generators produce all 16 T1 symbols. Each generator added raises
//! the Chomsky level by one:
//!
//! | Level | Generators | Automaton | Architecture |
//! |-------|-----------|-----------|-------------|
//! | Type-3 | {σ, Σ} | Finite automaton | Flat pipeline, state machine |
//! | Type-2 | + ρ | Pushdown automaton | Recursive parser, tree walker |
//! | Type-1 | + κ | Linear bounded | Type checker, validator |
//! | Type-0 | + ∃ | Turing machine | Interpreter, Bayesian engine |
//!
//! ## Primitive Foundation
//! - σ, Σ, ρ, κ, ∃: The 5 generators themselves
//! - μ (Mapping): Crate → Chomsky level classification
//! - N (Quantity): Generator count → level determination

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// The 5 generators of the Lex Primitiva algebra.
///
/// Tier: T1 (irreducible)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Generator {
    /// σ (Sequence): Ordering, pipelines, iteration.
    Sigma,
    /// Σ (Sum/Coproduct): Alternation, enums, branching.
    Sum,
    /// ρ (Recursion): Self-reference, tree structures, recursive types.
    Rho,
    /// κ (Comparison): Ordering, matching, validation, constraints.
    Kappa,
    /// ∃ (Existence): Dynamic creation, optional values, runtime decisions.
    Exists,
}

impl Generator {
    /// Returns the Lex Primitiva symbol.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Sigma => "σ",
            Self::Sum => "Σ",
            Self::Rho => "ρ",
            Self::Kappa => "κ",
            Self::Exists => "∃",
        }
    }

    /// Returns the human-readable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sigma => "Sequence",
            Self::Sum => "Sum",
            Self::Rho => "Recursion",
            Self::Kappa => "Comparison",
            Self::Exists => "Existence",
        }
    }
}

/// Chomsky hierarchy level.
///
/// Tier: T2-P (κ + N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ChomskyLevel {
    /// Type-3: Regular language. Generators: {σ, Σ}.
    /// Finite automaton. Flat pipelines, state machines.
    Type3Regular = 3,
    /// Type-2: Context-free. Generators: {σ, Σ, ρ}.
    /// Pushdown automaton. Recursive parsers, tree walkers.
    Type2ContextFree = 2,
    /// Type-1: Context-sensitive. Generators: {σ, Σ, ρ, κ}.
    /// Linear bounded automaton. Type checkers, validators.
    Type1ContextSensitive = 1,
    /// Type-0: Recursively enumerable. Generators: {σ, Σ, ρ, κ, ∃}.
    /// Turing machine. Interpreters, Bayesian engines.
    Type0Unrestricted = 0,
}

impl ChomskyLevel {
    /// Classify from a set of generators used.
    #[must_use]
    pub fn from_generators(generators: &HashSet<Generator>) -> Self {
        if generators.contains(&Generator::Exists) {
            Self::Type0Unrestricted
        } else if generators.contains(&Generator::Kappa) {
            Self::Type1ContextSensitive
        } else if generators.contains(&Generator::Rho) {
            Self::Type2ContextFree
        } else {
            Self::Type3Regular
        }
    }

    /// Returns the human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Type3Regular => "Type-3 (Regular)",
            Self::Type2ContextFree => "Type-2 (Context-Free)",
            Self::Type1ContextSensitive => "Type-1 (Context-Sensitive)",
            Self::Type0Unrestricted => "Type-0 (Unrestricted)",
        }
    }

    /// Returns the corresponding automaton model.
    #[must_use]
    pub const fn automaton(self) -> &'static str {
        match self {
            Self::Type3Regular => "Finite Automaton",
            Self::Type2ContextFree => "Pushdown Automaton",
            Self::Type1ContextSensitive => "Linear Bounded Automaton",
            Self::Type0Unrestricted => "Turing Machine",
        }
    }

    /// Returns the overengineering delta if this level is used but `needed` suffices.
    /// `overengineering = generators_used - generators_needed`.
    #[must_use]
    pub fn overengineering_vs(self, needed: Self) -> i8 {
        let used_gens = match self {
            Self::Type3Regular => 2,
            Self::Type2ContextFree => 3,
            Self::Type1ContextSensitive => 4,
            Self::Type0Unrestricted => 5,
        };
        let needed_gens = match needed {
            Self::Type3Regular => 2,
            Self::Type2ContextFree => 3,
            Self::Type1ContextSensitive => 4,
            Self::Type0Unrestricted => 5,
        };
        used_gens - needed_gens
    }
}

/// Classification of a single crate's Chomsky level.
///
/// Tier: T2-C (μ + κ + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateChomskyProfile {
    /// Crate name.
    pub name: String,
    /// Generators detected in this crate.
    pub generators: Vec<String>,
    /// Number of generators used.
    pub generator_count: usize,
    /// Classified Chomsky level.
    pub level: ChomskyLevel,
    /// Architectural style implied by the level.
    pub architecture: String,
}

/// Heuristic: classify a crate's Chomsky level from its name and dependencies.
///
/// This is a heuristic classifier based on naming conventions and dependency patterns.
/// A full analysis would require AST inspection of the actual source code.
///
/// Rules:
/// - Crates with "parser", "ast", "grammar", "prima" → at least Type-2 (ρ)
/// - Crates with "validator", "checker", "guardian", "compliance" → at least Type-1 (κ)
/// - Crates with "engine", "interpreter", "bayesian", "brain", "vigil" → Type-0 (∃)
/// - Crates with "pipeline", "stream", "signal" → Type-3 (σ + Σ)
/// - Default: Type-1 (most domain crates do validation)
#[must_use]
pub fn classify_crate_chomsky(name: &str, dep_names: &[String]) -> CrateChomskyProfile {
    let mut generators: HashSet<Generator> = HashSet::new();

    // All crates use σ (Sequence) and Σ (Sum) at minimum
    generators.insert(Generator::Sigma);
    generators.insert(Generator::Sum);

    let name_lower = name.to_lowercase();

    // Detect ρ (Recursion): recursive parsers, tree structures, AST
    let rho_indicators = [
        "parser", "ast", "grammar", "prima", "pvdsl", "recursiv", "tree", "dtree",
    ];
    if rho_indicators.iter().any(|ind| name_lower.contains(ind)) {
        generators.insert(Generator::Rho);
    }

    // Detect κ (Comparison): validators, checkers, comparators
    let kappa_indicators = [
        "validator",
        "checker",
        "guardian",
        "compliance",
        "audit",
        "measure",
        "compare",
        "threshold",
        "anatomy",
        "vigilance",
        "signal",
        "detection",
        "classify",
        "triage",
    ];
    if kappa_indicators.iter().any(|ind| name_lower.contains(ind)) {
        generators.insert(Generator::Kappa);
        // κ implies ρ (can't compare recursively without recursion)
        generators.insert(Generator::Rho);
    }

    // Detect ∃ (Existence): dynamic creation, interpreters, engines
    let exists_indicators = [
        "engine",
        "interpret",
        "bayesian",
        "brain",
        "vigil",
        "friday",
        "cortex",
        "energy",
        "insight",
        "mcp",
        "api",
        "cli",
        "sentinel",
    ];
    if exists_indicators.iter().any(|ind| name_lower.contains(ind)) {
        generators.insert(Generator::Exists);
        generators.insert(Generator::Kappa);
        generators.insert(Generator::Rho);
    }

    // Dependency-based detection: if a crate depends on recursive/dynamic crates
    for dep in dep_names {
        let dep_lower = dep.to_lowercase();
        if dep_lower.contains("grammar") || dep_lower.contains("prima") {
            generators.insert(Generator::Rho);
        }
    }

    let level = ChomskyLevel::from_generators(&generators);

    let mut gen_symbols: Vec<String> = generators.iter().map(|g| g.symbol().to_string()).collect();
    gen_symbols.sort();

    CrateChomskyProfile {
        name: name.to_string(),
        generators: gen_symbols.clone(),
        generator_count: gen_symbols.len(),
        level,
        architecture: level.automaton().to_string(),
    }
}

/// Workspace-level Chomsky classification report.
///
/// Tier: T3 (μ + κ + N + Σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChomskyReport {
    /// Per-crate profiles.
    pub profiles: Vec<CrateChomskyProfile>,
    /// Count of crates at each Chomsky level.
    pub level_distribution: Vec<(String, usize)>,
    /// Average generator count across workspace.
    pub avg_generators: f64,
    /// Overengineering candidates: crates using more generators than needed.
    pub overengineering_candidates: Vec<String>,
}

impl ChomskyReport {
    /// Build a Chomsky classification report from a dependency graph.
    pub fn from_graph(graph: &crate::graph::DependencyGraph) -> Self {
        let mut profiles: Vec<CrateChomskyProfile> = graph
            .nodes
            .values()
            .map(|node| classify_crate_chomsky(&node.name, &node.dependencies))
            .collect();

        profiles.sort_by(|a, b| a.level.cmp(&b.level));

        let avg_generators = if profiles.is_empty() {
            0.0
        } else {
            profiles
                .iter()
                .map(|p| p.generator_count as f64)
                .sum::<f64>()
                / profiles.len() as f64
        };

        // Level distribution
        let mut counts = std::collections::HashMap::new();
        for p in &profiles {
            *counts.entry(p.level.label().to_string()).or_insert(0usize) += 1;
        }
        let mut level_distribution: Vec<(String, usize)> = counts.into_iter().collect();
        level_distribution.sort_by(|a, b| a.0.cmp(&b.0));

        // Simple overengineering detection:
        // Foundation/primitive crates shouldn't need Type-0
        let overengineering_candidates: Vec<String> = profiles
            .iter()
            .filter(|p| {
                p.level == ChomskyLevel::Type0Unrestricted
                    && (p.name.contains("primitiv")
                        || p.name.contains("constant")
                        || p.name.starts_with("stem-"))
            })
            .map(|p| p.name.clone())
            .collect();

        Self {
            profiles,
            level_distribution,
            avg_generators,
            overengineering_candidates,
        }
    }

    /// Get all crates at a specific Chomsky level.
    #[must_use]
    pub fn at_level(&self, level: ChomskyLevel) -> Vec<&CrateChomskyProfile> {
        self.profiles.iter().filter(|p| p.level == level).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_classification_type3() {
        let mut gens = HashSet::new();
        gens.insert(Generator::Sigma);
        gens.insert(Generator::Sum);
        assert_eq!(
            ChomskyLevel::from_generators(&gens),
            ChomskyLevel::Type3Regular
        );
    }

    #[test]
    fn test_generator_classification_type2() {
        let mut gens = HashSet::new();
        gens.insert(Generator::Sigma);
        gens.insert(Generator::Sum);
        gens.insert(Generator::Rho);
        assert_eq!(
            ChomskyLevel::from_generators(&gens),
            ChomskyLevel::Type2ContextFree
        );
    }

    #[test]
    fn test_generator_classification_type1() {
        let mut gens = HashSet::new();
        gens.insert(Generator::Sigma);
        gens.insert(Generator::Sum);
        gens.insert(Generator::Rho);
        gens.insert(Generator::Kappa);
        assert_eq!(
            ChomskyLevel::from_generators(&gens),
            ChomskyLevel::Type1ContextSensitive
        );
    }

    #[test]
    fn test_generator_classification_type0() {
        let mut gens = HashSet::new();
        gens.insert(Generator::Sigma);
        gens.insert(Generator::Sum);
        gens.insert(Generator::Rho);
        gens.insert(Generator::Kappa);
        gens.insert(Generator::Exists);
        assert_eq!(
            ChomskyLevel::from_generators(&gens),
            ChomskyLevel::Type0Unrestricted
        );
    }

    #[test]
    fn test_crate_heuristic_pipeline() {
        let profile = classify_crate_chomsky("nexcore-pipeline", &[]);
        // "pipeline" doesn't match any higher-level indicators
        assert_eq!(profile.level, ChomskyLevel::Type3Regular);
    }

    #[test]
    fn test_crate_heuristic_parser() {
        let profile = classify_crate_chomsky("nexcore-parser", &[]);
        assert!(profile.generator_count >= 3); // σ + Σ + ρ
        assert!(profile.level <= ChomskyLevel::Type2ContextFree);
    }

    #[test]
    fn test_crate_heuristic_guardian() {
        let profile = classify_crate_chomsky("nexcore-guardian-engine", &[]);
        // "guardian" → κ, "engine" → ∃
        assert_eq!(profile.level, ChomskyLevel::Type0Unrestricted);
    }

    #[test]
    fn test_crate_heuristic_brain() {
        let profile = classify_crate_chomsky("nexcore-brain", &[]);
        assert_eq!(profile.level, ChomskyLevel::Type0Unrestricted);
    }

    #[test]
    fn test_overengineering_delta() {
        let used = ChomskyLevel::Type0Unrestricted;
        let needed = ChomskyLevel::Type3Regular;
        assert_eq!(used.overengineering_vs(needed), 3); // 5 - 2 = 3 excess generators
    }

    #[test]
    fn test_overengineering_zero() {
        let level = ChomskyLevel::Type1ContextSensitive;
        assert_eq!(level.overengineering_vs(level), 0);
    }
}
