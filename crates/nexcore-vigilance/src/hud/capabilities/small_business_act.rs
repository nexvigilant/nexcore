//! # Capability 25: Small Business Act (Sub-Agent Support)
//!
//! Implementation of the Small Business Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Sub-Agent Incubation" and "Autonomous Task Allocation" of the Union.
//!
//! Matches 1:1 to the US Small Business Administration (SBA) mandate
//! to aid, counsel, assist and protect the interests of small business
//! concerns.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │            SMALL BUSINESS ACT (CAP-025)                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  TASK ANALYSIS                                               │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐                      │
//! │  │Complexity│  │ Domain  │  │ Skill   │                      │
//! │  │Scoring  │  │Matching │  │Taxonomy │                      │
//! │  └────┬────┘  └────┬────┘  └────┬────┘                      │
//! │       │            │            │                            │
//! │       ▼            ▼            ▼                            │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          AGENT ALLOCATION ENGINE            │            │
//! │  │  • Skill-to-Agent mapping                   │            │
//! │  │  • Model selection (haiku/sonnet/opus)      │            │
//! │  │  • Resource quota assignment                │            │
//! │  └────────────────────┬────────────────────────┘            │
//! │                       │                                      │
//! │                       ▼                                      │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          INCUBATION LIFECYCLE               │            │
//! │  │  • Grant allocation                         │            │
//! │  │  • Growth tracking                          │            │
//! │  │  • Completion chaining                      │            │
//! │  └─────────────────────────────────────────────┘            │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// T1 PRIMITIVES (Universal)
// ============================================================================

/// T1: AgentModel - The compute tier for a sub-agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentModel {
    /// Fast, cheap - simple tasks.
    Haiku,
    /// Balanced - most tasks.
    Sonnet,
    /// Powerful - complex reasoning.
    Opus,
}

impl AgentModel {
    /// Get base quota for model tier.
    pub fn base_quota(&self) -> u64 {
        match self {
            Self::Haiku => 200,
            Self::Sonnet => 500,
            Self::Opus => 1000,
        }
    }
}

impl Default for AgentModel {
    fn default() -> Self {
        Self::Sonnet
    }
}

/// T1: TaskComplexity - Measured task difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// Single-step, obvious solution.
    Trivial,
    /// Few steps, clear path.
    Simple,
    /// Multiple steps, some ambiguity.
    Moderate,
    /// Many steps, significant reasoning.
    Complex,
    /// Unbounded, requires autonomous loop.
    Autonomous,
}

impl TaskComplexity {
    /// Compute from word count and action indicators.
    pub fn from_prompt_heuristics(word_count: usize, action_count: usize) -> Self {
        match (word_count, action_count) {
            (0..=10, _) => Self::Trivial,
            (11..=30, 0..=1) => Self::Simple,
            (31..=60, 0..=2) => Self::Moderate,
            (61..=100, _) => Self::Complex,
            _ => Self::Autonomous,
        }
    }

    /// Get model recommendation based on complexity.
    pub fn recommended_model(&self) -> AgentModel {
        match self {
            Self::Trivial | Self::Simple => AgentModel::Haiku,
            Self::Moderate | Self::Complex => AgentModel::Sonnet,
            Self::Autonomous => AgentModel::Opus,
        }
    }
}

// ============================================================================
// T2-P PRIMITIVES (Cross-Domain)
// ============================================================================

/// T2-P: SubAgentGrowth - The quantified development of a sub-agent's KSBs.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SubAgentGrowth(pub f64);

impl SubAgentGrowth {
    /// Create new growth metric (clamped 0.0-1.0).
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Calculate growth from task completion rate.
    pub fn from_completion_rate(completed: usize, total: usize) -> Self {
        if total == 0 {
            return Self(0.0);
        }
        Self::new(completed as f64 / total as f64)
    }

    /// Inner value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// T2-P: SkillMatch - Confidence that a skill matches a task.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SkillMatch(pub f64);

impl SkillMatch {
    /// Create new match score (clamped 0.0-1.0).
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Is this a strong match (>0.7)?
    pub fn is_strong(&self) -> bool {
        self.0 > 0.7
    }

    /// Inner value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

// ============================================================================
// T2-C COMPOSITES (Cross-Domain)
// ============================================================================

/// T2-C: LoanGrant - A resource allocation for a new sub-agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanGrant {
    /// The identifier of the sub-agent receiving the grant.
    pub agent_id: String,
    /// The agent type (maps to Task tool subagent_type).
    pub agent_type: String,
    /// The model tier to use.
    pub model: AgentModel,
    /// The compute/memory quota granted.
    pub quota_grant: u64,
    /// The specific task assigned for incubation.
    pub task_id: String,
    /// Skills to activate for this agent.
    pub skills: Vec<String>,
    /// Tools to restrict to (empty = all tools).
    pub tool_restrictions: Vec<String>,
}

/// T2-C: AgentAllocation - Result of the allocation engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAllocation {
    /// Primary agent recommendation.
    pub primary: LoanGrant,
    /// Alternative agents (if primary fails).
    pub alternatives: Vec<LoanGrant>,
    /// Task complexity assessment.
    pub complexity: TaskComplexity,
    /// Confidence in this allocation.
    pub confidence: f64,
    /// Reasoning for the allocation.
    pub reasoning: String,
}

/// T2-C: AgentChain - Defines agent completion chaining.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentChain {
    /// Agent that just completed.
    pub completed: String,
    /// Agent to trigger next.
    pub next: String,
    /// Condition for triggering (always, with_errors, stable).
    pub condition: ChainCondition,
}

/// T2-P: ChainCondition - When to trigger the next agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainCondition {
    /// Always trigger.
    Always,
    /// Only if previous had errors.
    WithErrors,
    /// Only if previous was stable (no errors).
    Stable,
    /// Custom condition (evaluated externally).
    Custom,
}

// ============================================================================
// T3 DOMAIN-SPECIFIC (SmallBusinessAct)
// ============================================================================

/// T3: SmallBusinessAct - Capability 25 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallBusinessAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the sub-agent incubation engine is active.
    pub incubation_active: bool,
    /// Skill-to-agent mappings for delegation.
    skill_agent_map: HashMap<String, AgentSpec>,
    /// Active incubations.
    active_incubations: Vec<LoanGrant>,
    /// Agent completion chains.
    chains: Vec<AgentChain>,
}

/// Specification for an agent type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    /// The subagent_type value for Task tool.
    pub agent_type: String,
    /// Default model to use.
    pub model: AgentModel,
    /// Skills this agent has access to.
    pub skills: Vec<String>,
    /// Tool restrictions (empty = all).
    pub tools: Vec<String>,
    /// Domains this agent handles.
    pub domains: Vec<String>,
}

impl Default for SmallBusinessAct {
    fn default() -> Self {
        Self::new()
    }
}

impl SmallBusinessAct {
    /// Creates a new instance with default agent mappings.
    pub fn new() -> Self {
        let mut act = Self {
            id: "CAP-025".into(),
            incubation_active: true,
            skill_agent_map: HashMap::new(),
            active_incubations: Vec::new(),
            chains: Vec::new(),
        };
        act.register_default_agents();
        act.register_default_chains();
        act
    }

    /// Register the default agent catalog.
    fn register_default_agents(&mut self) {
        // Exploration agent
        self.skill_agent_map.insert(
            "explore".into(),
            AgentSpec {
                agent_type: "Explore".into(),
                model: AgentModel::Sonnet,
                skills: vec![],
                tools: vec!["Read".into(), "Glob".into(), "Grep".into()],
                domains: vec!["codebase".into(), "search".into(), "navigation".into()],
            },
        );

        // Planning agent
        self.skill_agent_map.insert(
            "plan".into(),
            AgentSpec {
                agent_type: "Plan".into(),
                model: AgentModel::Sonnet,
                skills: vec![],
                tools: vec!["Read".into(), "Glob".into(), "Grep".into()],
                domains: vec!["architecture".into(), "design".into(), "planning".into()],
            },
        );

        // Rust expert agent
        self.skill_agent_map.insert(
            "rust".into(),
            AgentSpec {
                agent_type: "rust-anatomy-expert".into(),
                model: AgentModel::Sonnet,
                skills: vec![
                    "rust-anatomy-expert".into(),
                    "primitive-rust-foundation".into(),
                ],
                tools: vec![],
                domains: vec!["rust".into(), "architecture".into(), "patterns".into()],
            },
        );

        // Strategy agent
        self.skill_agent_map.insert(
            "strategy".into(),
            AgentSpec {
                agent_type: "strat-dev".into(),
                model: AgentModel::Sonnet,
                skills: vec!["strat-dev".into()],
                tools: vec![],
                domains: vec!["strategy".into(), "business".into(), "capabilities".into()],
            },
        );

        // CTVP validator agent
        self.skill_agent_map.insert(
            "validation".into(),
            AgentSpec {
                agent_type: "ctvp-validator".into(),
                model: AgentModel::Sonnet,
                skills: vec!["ctvp-validator".into()],
                tools: vec!["Read".into(), "Grep".into(), "Glob".into(), "Bash".into()],
                domains: vec!["testing".into(), "validation".into(), "quality".into()],
            },
        );

        // Primitive extraction agent
        self.skill_agent_map.insert(
            "primitives".into(),
            AgentSpec {
                agent_type: "primitive-extractor".into(),
                model: AgentModel::Sonnet,
                skills: vec!["primitive-extractor".into()],
                tools: vec![],
                domains: vec!["decomposition".into(), "analysis".into(), "transfer".into()],
            },
        );

        // General purpose agent
        self.skill_agent_map.insert(
            "general".into(),
            AgentSpec {
                agent_type: "general-purpose".into(),
                model: AgentModel::Sonnet,
                skills: vec![],
                tools: vec![],
                domains: vec!["any".into()],
            },
        );

        // FORGE autonomous Rust agent
        self.skill_agent_map.insert(
            "forge".into(),
            AgentSpec {
                agent_type: "forge".into(),
                model: AgentModel::Opus,
                skills: vec!["forge".into()],
                tools: vec![],
                domains: vec!["rust".into(), "autonomous".into(), "generation".into()],
            },
        );
    }

    /// Register default agent completion chains.
    fn register_default_chains(&mut self) {
        // FORGE loop
        self.chains.push(AgentChain {
            completed: "forge-primitive-miner".into(),
            next: "forge-codifier".into(),
            condition: ChainCondition::Always,
        });
        self.chains.push(AgentChain {
            completed: "forge-codifier".into(),
            next: "forge-validator".into(),
            condition: ChainCondition::Always,
        });
        self.chains.push(AgentChain {
            completed: "forge-validator".into(),
            next: "forge-primitive-miner".into(),
            condition: ChainCondition::WithErrors,
        });
        self.chains.push(AgentChain {
            completed: "forge-validator".into(),
            next: "ctvp-validator".into(),
            condition: ChainCondition::Stable,
        });

        // Rust development flow
        self.chains.push(AgentChain {
            completed: "rust-anatomy-expert".into(),
            next: "ctvp-validator".into(),
            condition: ChainCondition::Always,
        });
    }

    /// Allocate an agent for a given task.
    pub fn allocate_agent(&self, task_description: &str) -> Measured<AgentAllocation> {
        let lower = task_description.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let word_count = words.len();

        // Count action indicators
        let action_words = ["and", "then", "also", "next", "after", "first", "finally"];
        let action_count = action_words.iter().filter(|w| lower.contains(*w)).count();

        // Determine complexity
        let complexity = TaskComplexity::from_prompt_heuristics(word_count, action_count);

        // Find matching agent by domain keywords
        let (agent_key, match_score) = self.match_agent_to_task(&lower);

        let spec = self
            .skill_agent_map
            .get(&agent_key)
            .cloned()
            .unwrap_or_else(|| {
                self.skill_agent_map
                    .get("general")
                    .cloned()
                    .unwrap_or_else(|| AgentSpec {
                        agent_type: "general-purpose".into(),
                        model: AgentModel::Sonnet,
                        skills: vec![],
                        tools: vec![],
                        domains: vec!["any".into()],
                    })
            });

        // Override model based on complexity if needed
        let model = if complexity >= TaskComplexity::Complex {
            complexity.recommended_model()
        } else {
            spec.model
        };

        let primary = LoanGrant {
            agent_id: format!(
                "SBA-{}-{}",
                agent_key.to_uppercase(),
                nexcore_chrono::DateTime::now().timestamp()
            ),
            agent_type: spec.agent_type.clone(),
            model,
            quota_grant: model.base_quota(),
            task_id: task_description.chars().take(50).collect(),
            skills: spec.skills.clone(),
            tool_restrictions: spec.tools.clone(),
        };

        // Build alternatives
        let mut alternatives = Vec::new();
        if agent_key != "general" {
            if let Some(gen_spec) = self.skill_agent_map.get("general") {
                alternatives.push(LoanGrant {
                    agent_id: format!(
                        "SBA-GENERAL-{}",
                        nexcore_chrono::DateTime::now().timestamp()
                    ),
                    agent_type: gen_spec.agent_type.clone(),
                    model: gen_spec.model,
                    quota_grant: gen_spec.model.base_quota(),
                    task_id: task_description.chars().take(50).collect(),
                    skills: gen_spec.skills.clone(),
                    tool_restrictions: gen_spec.tools.clone(),
                });
            }
        }

        let allocation = AgentAllocation {
            primary,
            alternatives,
            complexity,
            confidence: match_score.value(),
            reasoning: format!(
                "Matched '{}' agent (complexity: {:?}, match: {:.0}%)",
                agent_key,
                complexity,
                match_score.value() * 100.0
            ),
        };

        Measured::uncertain(allocation, Confidence::new(match_score.value()))
    }

    /// Match task description to best agent.
    fn match_agent_to_task(&self, task: &str) -> (String, SkillMatch) {
        let mut best_key = "general".to_string();
        let mut best_score = SkillMatch::new(0.3); // Baseline for general

        for (key, spec) in &self.skill_agent_map {
            let domain_matches = spec
                .domains
                .iter()
                .filter(|d| task.contains(d.as_str()))
                .count();

            if domain_matches > 0 {
                let score = SkillMatch::new(0.5 + (domain_matches as f64 * 0.15));
                if score > best_score {
                    best_score = score;
                    best_key = key.clone();
                }
            }
        }

        // Boost for explicit keywords
        if task.contains("rust") && task.contains("architecture") {
            return ("rust".into(), SkillMatch::new(0.95));
        }
        if task.contains("forge") || task.contains("autonomous") {
            return ("forge".into(), SkillMatch::new(0.90));
        }
        if task.contains("strategy") || task.contains("capability") {
            return ("strategy".into(), SkillMatch::new(0.90));
        }
        if task.contains("test") || task.contains("validate") || task.contains("ctvp") {
            return ("validation".into(), SkillMatch::new(0.90));
        }
        if task.contains("primitive") || task.contains("decompose") {
            return ("primitives".into(), SkillMatch::new(0.90));
        }
        if task.contains("explore") || task.contains("find") || task.contains("search") {
            return ("explore".into(), SkillMatch::new(0.85));
        }
        if task.contains("plan") || task.contains("design") || task.contains("architect") {
            return ("plan".into(), SkillMatch::new(0.85));
        }

        (best_key, best_score)
    }

    /// Incubate a new sub-agent for a specialized task (legacy API).
    pub fn incubate_agent(&self, task_id: &str) -> Measured<LoanGrant> {
        let allocation = self.allocate_agent(task_id);
        Measured::uncertain(allocation.value.primary, allocation.confidence)
    }

    /// Get next agent in chain after completion.
    pub fn get_chain_next(&self, completed: &str, had_errors: bool) -> Option<&AgentChain> {
        self.chains.iter().find(|chain| {
            chain.completed == completed
                && match chain.condition {
                    ChainCondition::Always => true,
                    ChainCondition::WithErrors => had_errors,
                    ChainCondition::Stable => !had_errors,
                    ChainCondition::Custom => false,
                }
        })
    }

    /// Register a custom agent spec.
    pub fn register_agent(&mut self, key: &str, spec: AgentSpec) {
        self.skill_agent_map.insert(key.to_string(), spec);
    }

    /// Register a completion chain.
    pub fn register_chain(&mut self, chain: AgentChain) {
        self.chains.push(chain);
    }

    /// Get all registered agent types.
    pub fn list_agents(&self) -> Vec<&str> {
        self.skill_agent_map
            .values()
            .map(|s| s.agent_type.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_heuristics() {
        assert_eq!(
            TaskComplexity::from_prompt_heuristics(5, 0),
            TaskComplexity::Trivial
        );
        assert_eq!(
            TaskComplexity::from_prompt_heuristics(20, 1),
            TaskComplexity::Simple
        );
        assert_eq!(
            TaskComplexity::from_prompt_heuristics(50, 2),
            TaskComplexity::Moderate
        );
        assert_eq!(
            TaskComplexity::from_prompt_heuristics(80, 3),
            TaskComplexity::Complex
        );
        assert_eq!(
            TaskComplexity::from_prompt_heuristics(150, 5),
            TaskComplexity::Autonomous
        );
    }

    #[test]
    fn test_agent_allocation_rust() {
        let sba = SmallBusinessAct::new();
        let result = sba.allocate_agent("Help me with rust architecture for the new crate");

        assert_eq!(result.value.primary.agent_type, "rust-anatomy-expert");
        assert!(result.value.confidence > 0.9);
    }

    #[test]
    fn test_agent_allocation_explore() {
        let sba = SmallBusinessAct::new();
        let result = sba.allocate_agent("Search for and find all error handling code");

        assert_eq!(result.value.primary.agent_type, "Explore");
    }

    #[test]
    fn test_agent_chain() {
        let sba = SmallBusinessAct::new();

        // Stable validator should chain to CTVP
        let next = sba.get_chain_next("forge-validator", false);
        assert!(next.is_some());
        assert_eq!(next.map(|c| c.next.as_str()), Some("ctvp-validator"));

        // Errored validator should loop back
        let next = sba.get_chain_next("forge-validator", true);
        assert!(next.is_some());
        assert_eq!(next.map(|c| c.next.as_str()), Some("forge-primitive-miner"));
    }

    #[test]
    fn test_growth_tracking() {
        let growth = SubAgentGrowth::from_completion_rate(7, 10);
        assert!((growth.value() - 0.7).abs() < 0.01);

        let perfect = SubAgentGrowth::from_completion_rate(10, 10);
        assert!((perfect.value() - 1.0).abs() < 0.01);

        let zero = SubAgentGrowth::from_completion_rate(0, 0);
        assert!((zero.value() - 0.0).abs() < 0.01);
    }
}
