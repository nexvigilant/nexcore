//! Fluent DSL rules engine - ported from claude-hooks.
//!
//! Provides pattern matching and rule evaluation for hook inputs.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    /// Block the operation
    Block,
    /// Warn but allow
    Warn,
    /// Allow explicitly
    Allow,
    /// Ask the user
    Ask,
}

/// A single validation rule.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    /// Rule identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Regex pattern to match
    pub pattern: String,
    /// Action when matched
    pub action: RuleAction,
    /// Message to show
    #[serde(default)]
    pub message: Option<String>,
    /// Priority (higher = checked first)
    #[serde(default)]
    pub priority: i32,
    /// Whether the rule is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl Rule {
    /// Check if the pattern matches the given text.
    pub fn matches(&self, text: &str) -> bool {
        if !self.enabled {
            return false;
        }
        match Regex::new(&self.pattern) {
            Ok(re) => re.is_match(text),
            Err(_) => text.contains(&self.pattern),
        }
    }

    /// Get the message to display, or a default.
    pub fn get_message(&self) -> String {
        self.message
            .clone()
            .unwrap_or_else(|| self.description.clone())
    }
}

/// A set of rules for a specific context.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RuleSet {
    /// Rules for bash commands
    #[serde(default)]
    pub bash: Vec<Rule>,
    /// Rules for file paths
    #[serde(default)]
    pub files: Vec<Rule>,
    /// Rules for web URLs
    #[serde(default)]
    pub web: Vec<Rule>,
    /// Rules for user prompts
    #[serde(default)]
    pub prompts: Vec<Rule>,
    /// Generic rules
    #[serde(default)]
    pub generic: Vec<Rule>,
}

impl RuleSet {
    /// Load rules from a JSON file.
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let content = fs::read_to_string(path.as_ref())?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Create an empty rule set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a bash rule.
    pub fn add_bash_rule(mut self, rule: Rule) -> Self {
        self.bash.push(rule);
        self
    }

    /// Add a file rule.
    pub fn add_file_rule(mut self, rule: Rule) -> Self {
        self.files.push(rule);
        self
    }

    /// Sort all rules by priority (descending).
    pub fn sort_by_priority(&mut self) {
        self.bash.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.files.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.web.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.prompts.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.generic.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
}

/// Result of checking rules against input.
#[derive(Debug, Clone)]
pub struct RuleMatch {
    /// The matched rule
    pub rule: Rule,
    /// The input that matched
    pub matched_text: String,
}

/// Categories of rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Bash,
    Files,
    Web,
    Prompts,
    Generic,
}

/// Rules engine for validating inputs.
#[derive(Debug, Clone, Default)]
pub struct RulesEngine {
    rules: RuleSet,
}

impl RulesEngine {
    /// Create a new rules engine with the given rules.
    pub fn new(mut rules: RuleSet) -> Self {
        rules.sort_by_priority();
        Self { rules }
    }

    /// Check bash command against rules.
    pub fn check_bash(&self, command: &str) -> Option<RuleMatch> {
        self.check_against(&self.rules.bash, command)
    }

    /// Check file path against rules.
    pub fn check_file(&self, path: &str) -> Option<RuleMatch> {
        self.check_against(&self.rules.files, path)
    }

    /// Check URL against rules.
    pub fn check_web(&self, url: &str) -> Option<RuleMatch> {
        self.check_against(&self.rules.web, url)
    }

    /// Check user prompt against rules.
    pub fn check_prompt(&self, prompt: &str) -> Option<RuleMatch> {
        self.check_against(&self.rules.prompts, prompt)
    }

    /// Check text against generic rules.
    pub fn check_generic(&self, text: &str) -> Option<RuleMatch> {
        self.check_against(&self.rules.generic, text)
    }

    fn check_against(&self, rules: &[Rule], text: &str) -> Option<RuleMatch> {
        for rule in rules {
            if rule.matches(text) {
                return Some(RuleMatch {
                    rule: rule.clone(),
                    matched_text: text.to_string(),
                });
            }
        }
        None
    }

    /// Get all matching rules (not just the first).
    pub fn all_matches(&self, category: RuleCategory, text: &str) -> Vec<RuleMatch> {
        let rules = match category {
            RuleCategory::Bash => &self.rules.bash,
            RuleCategory::Files => &self.rules.files,
            RuleCategory::Web => &self.rules.web,
            RuleCategory::Prompts => &self.rules.prompts,
            RuleCategory::Generic => &self.rules.generic,
        };

        rules
            .iter()
            .filter(|r| r.matches(text))
            .map(|r| RuleMatch {
                rule: r.clone(),
                matched_text: text.to_string(),
            })
            .collect()
    }
}

/// Builder for creating rules.
#[derive(Debug, Clone, Default)]
pub struct RuleBuilder {
    id: Option<String>,
    description: Option<String>,
    pattern: Option<String>,
    action: Option<RuleAction>,
    message: Option<String>,
    priority: i32,
    enabled: bool,
}

impl RuleBuilder {
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    pub fn action(mut self, action: RuleAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn build(self) -> Result<Rule, &'static str> {
        Ok(Rule {
            id: self.id.ok_or("id is required")?,
            description: self.description.ok_or("description is required")?,
            pattern: self.pattern.ok_or("pattern is required")?,
            action: self.action.ok_or("action is required")?,
            message: self.message,
            priority: self.priority,
            enabled: self.enabled,
        })
    }
}

// =============================================================================
// Fluent DSL API
// =============================================================================

/// Fluent DSL for building rules declaratively.
#[derive(Debug, Clone, Default)]
pub struct Dsl {
    rules: Vec<DslRule>,
    current: Option<DslRule>,
}

#[derive(Debug, Clone, Default)]
struct DslRule {
    tool_pattern: String,
    conditions: Vec<DslCondition>,
    action: Option<RuleAction>,
    message: Option<String>,
    priority: i32,
}

#[derive(Debug, Clone)]
enum DslCondition {
    InputContains(String),
    InputMatches(String),
    PathContains(String),
    PathMatches(String),
    CommandContains(String),
    CommandMatches(String),
}

impl Dsl {
    /// Create a new DSL builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a rule that matches a tool pattern.
    pub fn when_tool(mut self, pattern: &str) -> Self {
        if let Some(rule) = self.current.take() {
            if rule.action.is_some() {
                self.rules.push(rule);
            }
        }
        self.current = Some(DslRule {
            tool_pattern: pattern.to_string(),
            ..Default::default()
        });
        self
    }

    /// Add condition: input field contains text.
    pub fn input_contains(mut self, text: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.conditions
                .push(DslCondition::InputContains(text.to_string()));
        }
        self
    }

    /// Add condition: input field matches regex.
    pub fn input_matches(mut self, pattern: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.conditions
                .push(DslCondition::InputMatches(pattern.to_string()));
        }
        self
    }

    /// Add condition: path field contains text.
    pub fn path_contains(mut self, text: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.conditions
                .push(DslCondition::PathContains(text.to_string()));
        }
        self
    }

    /// Add condition: path field matches regex.
    pub fn path_matches(mut self, pattern: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.conditions
                .push(DslCondition::PathMatches(pattern.to_string()));
        }
        self
    }

    /// Add condition: command field contains text.
    pub fn command_contains(mut self, text: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.conditions
                .push(DslCondition::CommandContains(text.to_string()));
        }
        self
    }

    /// Add condition: command field matches regex.
    pub fn command_matches(mut self, pattern: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.conditions
                .push(DslCondition::CommandMatches(pattern.to_string()));
        }
        self
    }

    /// Set action: deny (block) with message.
    pub fn deny(mut self, message: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.action = Some(RuleAction::Block);
            rule.message = Some(message.to_string());
        }
        self
    }

    /// Set action: warn with message.
    pub fn warn(mut self, message: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.action = Some(RuleAction::Warn);
            rule.message = Some(message.to_string());
        }
        self
    }

    /// Set action: allow explicitly.
    pub fn allow(mut self) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.action = Some(RuleAction::Allow);
        }
        self
    }

    /// Set action: ask user for confirmation.
    pub fn ask(mut self, prompt: &str) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.action = Some(RuleAction::Ask);
            rule.message = Some(prompt.to_string());
        }
        self
    }

    /// Set rule priority (higher = checked first).
    pub fn priority(mut self, priority: i32) -> Self {
        if let Some(ref mut rule) = self.current {
            rule.priority = priority;
        }
        self
    }

    /// Build the DSL into a vector of compiled rules.
    pub fn build(mut self) -> Vec<CompiledRule> {
        if let Some(rule) = self.current.take() {
            if rule.action.is_some() {
                self.rules.push(rule);
            }
        }

        self.rules
            .into_iter()
            .enumerate()
            .filter_map(|(i, r)| CompiledRule::from_dsl(r, i))
            .collect()
    }
}

/// A compiled rule ready for evaluation.
#[derive(Debug, Clone)]
pub struct CompiledRule {
    /// Rule ID
    pub id: String,
    /// Tool pattern (glob-style)
    pub tool_pattern: String,
    /// Compiled conditions
    conditions: Vec<CompiledCondition>,
    /// Action to take
    pub action: RuleAction,
    /// Message to display
    pub message: String,
    /// Priority
    pub priority: i32,
}

#[derive(Debug, Clone)]
enum CompiledCondition {
    InputContains(String),
    InputMatches(Regex),
    PathContains(String),
    PathMatches(Regex),
    CommandContains(String),
    CommandMatches(Regex),
}

impl CompiledRule {
    fn from_dsl(rule: DslRule, index: usize) -> Option<Self> {
        let action = rule.action?;
        let message = rule
            .message
            .unwrap_or_else(|| format!("Rule {} triggered", index));

        let conditions: Vec<CompiledCondition> = rule
            .conditions
            .into_iter()
            .filter_map(|c| match c {
                DslCondition::InputContains(s) => Some(CompiledCondition::InputContains(s)),
                DslCondition::InputMatches(p) => {
                    Regex::new(&p).ok().map(CompiledCondition::InputMatches)
                }
                DslCondition::PathContains(s) => Some(CompiledCondition::PathContains(s)),
                DslCondition::PathMatches(p) => {
                    Regex::new(&p).ok().map(CompiledCondition::PathMatches)
                }
                DslCondition::CommandContains(s) => Some(CompiledCondition::CommandContains(s)),
                DslCondition::CommandMatches(p) => {
                    Regex::new(&p).ok().map(CompiledCondition::CommandMatches)
                }
            })
            .collect();

        Some(Self {
            id: format!("dsl-rule-{}", index),
            tool_pattern: rule.tool_pattern,
            conditions,
            action,
            message,
            priority: rule.priority,
        })
    }

    /// Check if this rule matches the given tool and input.
    pub fn matches(&self, tool_name: &str, input: &serde_json::Value) -> bool {
        if !self.matches_tool(tool_name) {
            return false;
        }
        self.conditions
            .iter()
            .all(|c| self.check_condition(c, input))
    }

    fn matches_tool(&self, tool_name: &str) -> bool {
        let pattern = &self.tool_pattern;
        if pattern.contains('|') {
            return pattern
                .split('|')
                .any(|p| self.matches_single_tool(p.trim(), tool_name));
        }
        self.matches_single_tool(pattern, tool_name)
    }

    fn matches_single_tool(&self, pattern: &str, tool_name: &str) -> bool {
        if pattern.contains('*') {
            let regex_pattern = pattern.replace('.', r"\.").replace('*', ".*");
            return Regex::new(&format!("^{}$", regex_pattern))
                .map(|re| re.is_match(tool_name))
                .unwrap_or(false);
        }
        pattern == tool_name
    }

    fn check_condition(&self, condition: &CompiledCondition, input: &serde_json::Value) -> bool {
        match condition {
            CompiledCondition::InputContains(text) => input.to_string().contains(text),
            CompiledCondition::InputMatches(regex) => regex.is_match(&input.to_string()),
            CompiledCondition::PathContains(text) => input
                .get("file_path")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.contains(text)),
            CompiledCondition::PathMatches(regex) => input
                .get("file_path")
                .and_then(|v| v.as_str())
                .is_some_and(|s| regex.is_match(s)),
            CompiledCondition::CommandContains(text) => input
                .get("command")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.contains(text)),
            CompiledCondition::CommandMatches(regex) => input
                .get("command")
                .and_then(|v| v.as_str())
                .is_some_and(|s| regex.is_match(s)),
        }
    }
}

/// Evaluator for compiled DSL rules.
#[derive(Debug, Clone, Default)]
pub struct DslEvaluator {
    rules: Vec<CompiledRule>,
}

impl DslEvaluator {
    /// Create a new evaluator with the given rules.
    pub fn new(rules: Vec<CompiledRule>) -> Self {
        let mut rules = rules;
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Self { rules }
    }

    /// Evaluate rules against tool input.
    pub fn evaluate(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
    ) -> Option<(RuleAction, String)> {
        for rule in &self.rules {
            if rule.matches(tool_name, input) {
                return Some((rule.action, rule.message.clone()));
            }
        }
        None
    }

    /// Get all matching rules (not just first).
    pub fn all_matches(&self, tool_name: &str, input: &serde_json::Value) -> Vec<&CompiledRule> {
        self.rules
            .iter()
            .filter(|r| r.matches(tool_name, input))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsl_basic() {
        let rules = Dsl::new()
            .when_tool("Bash")
            .command_contains("rm -rf /")
            .deny("Dangerous rm command")
            .when_tool("Write")
            .path_matches(r"\.env$")
            .deny("Protected file")
            .build();

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].tool_pattern, "Bash");
        assert_eq!(rules[1].tool_pattern, "Write");
    }

    #[test]
    fn test_dsl_evaluator() {
        let rules = Dsl::new()
            .when_tool("Bash")
            .command_contains("rm -rf")
            .deny("Blocked: dangerous rm")
            .build();

        let evaluator = DslEvaluator::new(rules);

        let input = serde_json::json!({ "command": "rm -rf /" });
        let result = evaluator.evaluate("Bash", &input);
        assert!(result.is_some());
        let (action, _) = result.unwrap();
        assert_eq!(action, RuleAction::Block);

        let input = serde_json::json!({ "command": "ls -la" });
        let result = evaluator.evaluate("Bash", &input);
        assert!(result.is_none());
    }

    #[test]
    fn test_glob_patterns() {
        let rules = Dsl::new()
            .when_tool("mcp__*__delete_*")
            .ask("Confirm deletion?")
            .build();

        let evaluator = DslEvaluator::new(rules);

        let input = serde_json::json!({});
        assert!(
            evaluator
                .evaluate("mcp__nexcore__delete_file", &input)
                .is_some()
        );
        assert!(
            evaluator
                .evaluate("mcp__nexcore__read_file", &input)
                .is_none()
        );
    }

    #[test]
    fn test_or_patterns() {
        let rules = Dsl::new()
            .when_tool("Write|Edit")
            .path_contains(".env")
            .deny("Protected")
            .build();

        let evaluator = DslEvaluator::new(rules);

        let input = serde_json::json!({ "file_path": "/app/.env" });
        assert!(evaluator.evaluate("Write", &input).is_some());
        assert!(evaluator.evaluate("Edit", &input).is_some());
        assert!(evaluator.evaluate("Read", &input).is_none());
    }
}
