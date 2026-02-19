//! Hook composition and chaining framework - ported from claude-hooks.
//!
//! Chain multiple validators together with different aggregation strategies.

use super::rules::{RuleAction, RuleMatch, RuleSet, RulesEngine};

/// Strategy for aggregating results from multiple validators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AggregationStrategy {
    /// Stop at first blocking rule (fail fast)
    #[default]
    FirstBlock,
    /// Run all validators, collect all matches
    CollectAll,
    /// All validators must pass (strictest)
    AllMustPass,
    /// Any allow rule permits the action (most permissive)
    AnyAllow,
}

/// A named validator in the chain.
#[derive(Debug, Clone)]
pub struct ChainedValidator {
    /// Validator name (e.g., "security", "project", "team")
    pub name: String,
    /// The rules engine for this validator
    pub engine: RulesEngine,
    /// Priority (higher = runs first)
    pub priority: i32,
}

impl ChainedValidator {
    /// Create a new chained validator.
    pub fn new(name: impl Into<String>, rules: RuleSet) -> Self {
        Self {
            name: name.into(),
            engine: RulesEngine::new(rules),
            priority: 0,
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Result from a chained validation.
#[derive(Debug, Clone)]
pub struct ChainResult {
    /// Which validator matched
    pub validator_name: String,
    /// The rule match
    pub rule_match: RuleMatch,
}

/// Aggregated result from running the chain.
#[derive(Debug, Clone, Default)]
pub struct ChainOutput {
    /// All matches found
    pub matches: Vec<ChainResult>,
    /// Final action to take
    pub action: Option<RuleAction>,
    /// Combined message
    pub message: Option<String>,
}

impl ChainOutput {
    /// Check if any blocking rule matched.
    pub fn is_blocked(&self) -> bool {
        self.action == Some(RuleAction::Block)
    }

    /// Check if explicitly allowed.
    pub fn is_allowed(&self) -> bool {
        self.action == Some(RuleAction::Allow)
    }

    /// Check if user should be asked.
    pub fn should_ask(&self) -> bool {
        self.action == Some(RuleAction::Ask)
    }

    /// Get all blocking matches.
    pub fn blocking_matches(&self) -> Vec<&ChainResult> {
        self.matches
            .iter()
            .filter(|m| m.rule_match.rule.action == RuleAction::Block)
            .collect()
    }

    /// Get all warning matches.
    pub fn warning_matches(&self) -> Vec<&ChainResult> {
        self.matches
            .iter()
            .filter(|m| m.rule_match.rule.action == RuleAction::Warn)
            .collect()
    }
}

/// A chain of validators.
#[derive(Debug, Clone, Default)]
pub struct HookChain {
    validators: Vec<ChainedValidator>,
    strategy: AggregationStrategy,
}

impl HookChain {
    /// Create a new empty chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the aggregation strategy.
    pub fn with_strategy(mut self, strategy: AggregationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Add a validator with rules.
    pub fn add_rules(mut self, name: impl Into<String>, rules: RuleSet) -> Self {
        self.validators.push(ChainedValidator::new(name, rules));
        self.sort_by_priority();
        self
    }

    /// Add a pre-built validator.
    pub fn add_validator(mut self, validator: ChainedValidator) -> Self {
        self.validators.push(validator);
        self.sort_by_priority();
        self
    }

    fn sort_by_priority(&mut self) {
        self.validators.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Check a bash command through the chain.
    pub fn check_bash(&self, command: &str) -> ChainOutput {
        self.run_chain(|engine| engine.check_bash(command))
    }

    /// Check a file path through the chain.
    pub fn check_file(&self, path: &str) -> ChainOutput {
        self.run_chain(|engine| engine.check_file(path))
    }

    /// Check a URL through the chain.
    pub fn check_web(&self, url: &str) -> ChainOutput {
        self.run_chain(|engine| engine.check_web(url))
    }

    /// Check a prompt through the chain.
    pub fn check_prompt(&self, prompt: &str) -> ChainOutput {
        self.run_chain(|engine| engine.check_prompt(prompt))
    }

    fn run_chain<F>(&self, check_fn: F) -> ChainOutput
    where
        F: Fn(&RulesEngine) -> Option<RuleMatch>,
    {
        let mut output = ChainOutput::default();

        for validator in &self.validators {
            if let Some(rule_match) = check_fn(&validator.engine) {
                let chain_result = ChainResult {
                    validator_name: validator.name.clone(),
                    rule_match: rule_match.clone(),
                };

                match self.strategy {
                    AggregationStrategy::FirstBlock | AggregationStrategy::AllMustPass => {
                        if rule_match.rule.action == RuleAction::Block {
                            output.action = Some(RuleAction::Block);
                            output.message = Some(rule_match.rule.get_message());
                            output.matches.push(chain_result);
                            return output;
                        }
                        output.matches.push(chain_result);
                    }
                    AggregationStrategy::CollectAll => {
                        output.matches.push(chain_result);
                    }
                    AggregationStrategy::AnyAllow => {
                        if rule_match.rule.action == RuleAction::Allow {
                            output.action = Some(RuleAction::Allow);
                            output.message = Some(rule_match.rule.get_message());
                            output.matches.push(chain_result);
                            return output;
                        }
                        output.matches.push(chain_result);
                    }
                }
            }
        }

        // Determine final action from collected matches
        if output.action.is_none() && !output.matches.is_empty() {
            let actions: Vec<_> = output
                .matches
                .iter()
                .map(|m| m.rule_match.rule.action)
                .collect();

            if actions.contains(&RuleAction::Block) {
                output.action = Some(RuleAction::Block);
                output.message = output
                    .blocking_matches()
                    .first()
                    .map(|m| m.rule_match.rule.get_message());
            } else if actions.contains(&RuleAction::Ask) {
                output.action = Some(RuleAction::Ask);
            } else if actions.contains(&RuleAction::Warn) {
                output.action = Some(RuleAction::Warn);
            } else if actions.contains(&RuleAction::Allow) {
                output.action = Some(RuleAction::Allow);
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fluent_dsl::rules::{Rule, RuleBuilder};

    fn make_block_rule(id: &str, pattern: &str) -> Rule {
        RuleBuilder::new()
            .id(id)
            .description(format!("Block {}", id))
            .pattern(pattern)
            .action(RuleAction::Block)
            .build()
            .unwrap()
    }

    #[test]
    fn test_first_block_strategy() {
        let security_rules =
            RuleSet::default().add_bash_rule(make_block_rule("rm-rf", r"rm\s+-rf"));

        let chain = HookChain::new()
            .with_strategy(AggregationStrategy::FirstBlock)
            .add_rules("security", security_rules);

        let result = chain.check_bash("rm -rf /tmp");
        assert!(result.is_blocked());
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].validator_name, "security");
    }

    #[test]
    fn test_empty_chain_allows() {
        let chain = HookChain::new();
        let result = chain.check_bash("ls -la");
        assert!(!result.is_blocked());
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_collect_all_strategy() {
        let rules1 = RuleSet::default().add_bash_rule(make_block_rule("rule1", r"dangerous"));
        let rules2 = RuleSet::default().add_bash_rule(make_block_rule("rule2", r"dangerous"));

        let chain = HookChain::new()
            .with_strategy(AggregationStrategy::CollectAll)
            .add_rules("validator1", rules1)
            .add_rules("validator2", rules2);

        let result = chain.check_bash("dangerous command");
        assert_eq!(result.matches.len(), 2);
    }
}
