//! # Skill Routing Engine
//!
//! Multi-strategy routing for skill selection.

use serde::{Deserialize, Serialize};

use super::registry::SkillRegistry;
use crate::foundation::levenshtein::fuzzy_search;

/// Routing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Exact name match
    Exact,
    /// Fuzzy name match
    Fuzzy,
    /// Tag-based routing
    TagBased,
    /// Intent matching
    IntentBased,
    /// Decision tree-based routing (data-driven)
    DtreeBased,
}

/// Routing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// Selected skill name
    pub skill_name: String,
    /// Match score (0-1)
    pub score: f64,
    /// Strategy used
    pub strategy: RoutingStrategy,
}

/// Skill routing engine
pub struct RoutingEngine {
    registry: SkillRegistry,
    default_strategy: RoutingStrategy,
}

impl RoutingEngine {
    /// Create a new routing engine
    #[must_use]
    pub fn new(registry: SkillRegistry) -> Self {
        Self {
            registry,
            default_strategy: RoutingStrategy::Fuzzy,
        }
    }

    /// Set default routing strategy
    pub fn set_default_strategy(&mut self, strategy: RoutingStrategy) {
        self.default_strategy = strategy;
    }

    /// Route a query to a skill
    #[must_use]
    pub fn route(&self, query: &str) -> Option<RoutingResult> {
        self.route_with_strategy(query, self.default_strategy)
    }

    /// Route with a specific strategy
    #[must_use]
    pub fn route_with_strategy(
        &self,
        query: &str,
        strategy: RoutingStrategy,
    ) -> Option<RoutingResult> {
        match strategy {
            RoutingStrategy::Exact => self.route_exact(query),
            RoutingStrategy::Fuzzy => self.route_fuzzy(query),
            RoutingStrategy::TagBased => self.route_by_tag(query),
            RoutingStrategy::IntentBased => self.route_by_intent(query),
            RoutingStrategy::DtreeBased => None, // Handled by DtreeRouter
        }
    }

    /// Exact name match
    fn route_exact(&self, name: &str) -> Option<RoutingResult> {
        self.registry.get(name).map(|skill| RoutingResult {
            skill_name: skill.name.clone(),
            score: 1.0,
            strategy: RoutingStrategy::Exact,
        })
    }

    /// Fuzzy name match
    fn route_fuzzy(&self, query: &str) -> Option<RoutingResult> {
        let names: Vec<String> = self
            .registry
            .list()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        if names.is_empty() {
            return None;
        }

        let matches = fuzzy_search(query, &names, 1);
        matches.first().map(|m| RoutingResult {
            skill_name: m.candidate.clone(),
            score: m.similarity,
            strategy: RoutingStrategy::Fuzzy,
        })
    }

    /// Tag-based routing
    fn route_by_tag(&self, tag: &str) -> Option<RoutingResult> {
        let skills = self.registry.search_by_tag(tag);
        skills.first().map(|skill| RoutingResult {
            skill_name: skill.name.clone(),
            score: 1.0,
            strategy: RoutingStrategy::TagBased,
        })
    }

    /// Intent-based routing
    fn route_by_intent(&self, query: &str) -> Option<RoutingResult> {
        let skills = self.registry.list();
        let intents: Vec<String> = skills
            .iter()
            .filter_map(|s| s.intent.as_ref().map(|i| format!("{}: {}", s.name, i)))
            .collect();

        if intents.is_empty() {
            return None;
        }

        let matches = fuzzy_search(query, &intents, 1);
        matches.first().and_then(|m| {
            let name = m.candidate.split(':').next()?;
            Some(RoutingResult {
                skill_name: name.to_string(),
                score: m.similarity,
                strategy: RoutingStrategy::IntentBased,
            })
        })
    }
}
