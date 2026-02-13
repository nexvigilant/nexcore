//! # Condition Chain Execution
//!
//! Execute chains of conditions for skill sequencing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A condition in a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition expression
    pub expression: String,
    /// Action if condition is true
    pub then_action: String,
    /// Action if condition is false (optional)
    pub else_action: Option<String>,
}

/// A chain of conditions to evaluate
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConditionChain {
    /// Conditions in order of evaluation
    pub conditions: Vec<Condition>,
    /// Default action if no conditions match
    pub default_action: Option<String>,
}

impl ConditionChain {
    /// Create a new empty chain
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a condition to the chain
    pub fn add(&mut self, condition: Condition) {
        self.conditions.push(condition);
    }

    /// Set the default action
    pub fn set_default(&mut self, action: &str) {
        self.default_action = Some(action.to_string());
    }
}

/// Chain execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainResult {
    /// Action determined by the chain
    pub action: String,
    /// Index of the matching condition (-1 if default)
    pub matched_index: i32,
    /// Condition that matched (if any)
    pub matched_condition: Option<String>,
}

/// Executor for condition chains
#[derive(Debug, Clone, Default)]
pub struct ChainExecutor {
    chains: HashMap<String, ConditionChain>,
}

impl ChainExecutor {
    /// Create a new executor
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a chain
    pub fn register(&mut self, name: &str, chain: ConditionChain) {
        self.chains.insert(name.to_string(), chain);
    }

    /// Execute a chain by name
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the chain to execute
    /// * `context` - Variables for condition evaluation
    /// * `evaluator` - Function to evaluate condition expressions
    pub fn execute<F>(
        &self,
        name: &str,
        context: &HashMap<String, String>,
        evaluator: F,
    ) -> Option<ChainResult>
    where
        F: Fn(&str, &HashMap<String, String>) -> bool,
    {
        let chain = self.chains.get(name)?;

        for (idx, condition) in chain.conditions.iter().enumerate() {
            if evaluator(&condition.expression, context) {
                return Some(ChainResult {
                    action: condition.then_action.clone(),
                    matched_index: idx as i32,
                    matched_condition: Some(condition.expression.clone()),
                });
            }
        }

        // No condition matched - use default
        chain.default_action.as_ref().map(|action| ChainResult {
            action: action.clone(),
            matched_index: -1,
            matched_condition: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_execution() {
        let mut chain = ConditionChain::new();
        chain.add(Condition {
            expression: "level == high".to_string(),
            then_action: "alert".to_string(),
            else_action: None,
        });
        chain.add(Condition {
            expression: "level == medium".to_string(),
            then_action: "warn".to_string(),
            else_action: None,
        });
        chain.set_default("ignore");

        let mut executor = ChainExecutor::new();
        executor.register("risk-chain", chain);

        let mut context = HashMap::new();
        context.insert("level".to_string(), "high".to_string());

        let result = executor.execute("risk-chain", &context, |expr, ctx| {
            let parts: Vec<&str> = expr.split(" == ").collect();
            if parts.len() == 2 {
                ctx.get(parts[0]).map_or(false, |v| v == parts[1])
            } else {
                false
            }
        });

        assert_eq!(result.unwrap().action, "alert");
    }

    #[test]
    fn test_chain_default() {
        let mut chain = ConditionChain::new();
        chain.add(Condition {
            expression: "never_true".to_string(),
            then_action: "never".to_string(),
            else_action: None,
        });
        chain.set_default("default_action");

        let mut executor = ChainExecutor::new();
        executor.register("test", chain);

        let result = executor.execute("test", &HashMap::new(), |_, _| false);
        assert_eq!(result.unwrap().action, "default_action");
    }
}
