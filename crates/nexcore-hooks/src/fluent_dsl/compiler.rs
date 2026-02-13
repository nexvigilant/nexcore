//! TOML DSL compiler - converts TOML config to CompiledRule structures.

use regex::Regex;

use crate::rules::RuleAction;

use super::types::{InputCondition, OperatorCondition, TomlAction, TomlCondition, TomlConfig, TomlRule};

/// Compiled condition ready for evaluation.
#[derive(Debug, Clone)]
pub enum CompiledCondition {
    /// All conditions must match.
    All(Vec<CompiledCondition>),
    /// Any condition must match.
    Any(Vec<CompiledCondition>),
    /// Negate a condition.
    Not(Box<CompiledCondition>),
    /// Input contains text.
    InputContains(String),
    /// Input matches regex.
    InputMatches(Regex),
    /// Path contains text.
    PathContains(String),
    /// Path matches regex.
    PathMatches(Regex),
    /// Path ends with suffix.
    PathEndsWith(String),
    /// Command contains text.
    CommandContains(String),
    /// Command matches regex.
    CommandMatches(Regex),
}

/// A rule compiled from TOML configuration.
#[derive(Debug, Clone)]
pub struct TomlCompiledRule {
    /// Rule ID/name.
    pub id: String,
    /// Hook event (PreToolUse, PostToolUse, etc.).
    pub event: String,
    /// Tool pattern (glob-style).
    pub tool_pattern: Option<String>,
    /// Compiled conditions.
    pub conditions: Option<CompiledCondition>,
    /// Action to take.
    pub action: RuleAction,
    /// Message to display.
    pub message: String,
    /// Priority.
    pub priority: i32,
    /// Whether enabled.
    pub enabled: bool,
}

/// Compilation errors.
#[derive(Debug)]
pub enum CompileError {
    /// Invalid regex pattern.
    InvalidRegex { rule: String, pattern: String, error: String },
    /// Invalid event name.
    InvalidEvent { rule: String, event: String },
    /// Invalid action.
    InvalidAction { rule: String, message: String },
    /// Missing required field.
    MissingField { rule: String, field: String },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRegex { rule, pattern, error } => {
                write!(f, "Rule '{rule}': invalid regex '{pattern}': {error}")
            }
            Self::InvalidEvent { rule, event } => {
                write!(f, "Rule '{rule}': invalid event '{event}'")
            }
            Self::InvalidAction { rule, message } => {
                write!(f, "Rule '{rule}': invalid action: {message}")
            }
            Self::MissingField { rule, field } => {
                write!(f, "Rule '{rule}': missing required field '{field}'")
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Compiler for TOML DSL configurations.
#[derive(Debug, Default)]
pub struct Compiler {
    errors: Vec<CompileError>,
}

impl Compiler {
    /// Create a new compiler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compile a TOML configuration into rules.
    pub fn compile(&mut self, config: &TomlConfig) -> Result<Vec<TomlCompiledRule>, Vec<CompileError>> {
        self.errors.clear();
        let mut rules = Vec::new();

        for toml_rule in &config.rules {
            match self.compile_rule(toml_rule) {
                Ok(rule) => rules.push(rule),
                Err(e) => self.errors.push(e),
            }
        }

        if self.errors.is_empty() {
            // Sort by priority (descending)
            rules.sort_by(|a, b| b.priority.cmp(&a.priority));
            Ok(rules)
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn compile_rule(&self, rule: &TomlRule) -> Result<TomlCompiledRule, CompileError> {
        // Validate event
        if !Self::is_valid_event(&rule.event) {
            return Err(CompileError::InvalidEvent {
                rule: rule.name.clone(),
                event: rule.event.clone(),
            });
        }

        // Compile conditions
        let conditions = if let Some(ref cond) = rule.condition {
            Some(self.compile_condition(&rule.name, cond)?)
        } else {
            None
        };

        // Compile action
        let (action, message) = self.compile_action(&rule.name, &rule.action)?;

        Ok(TomlCompiledRule {
            id: rule.name.clone(),
            event: rule.event.clone(),
            tool_pattern: rule.matcher.clone(),
            conditions,
            action,
            message,
            priority: rule.priority,
            enabled: rule.enabled,
        })
    }

    fn compile_condition(&self, rule_name: &str, cond: &TomlCondition) -> Result<CompiledCondition, CompileError> {
        match cond {
            TomlCondition::All { all } => {
                let compiled: Result<Vec<_>, _> = all
                    .iter()
                    .map(|c| self.compile_condition(rule_name, c))
                    .collect();
                Ok(CompiledCondition::All(compiled?))
            }
            TomlCondition::Any { any } => {
                let compiled: Result<Vec<_>, _> = any
                    .iter()
                    .map(|c| self.compile_condition(rule_name, c))
                    .collect();
                Ok(CompiledCondition::Any(compiled?))
            }
            TomlCondition::Not { not } => {
                let inner = self.compile_condition(rule_name, not)?;
                Ok(CompiledCondition::Not(Box::new(inner)))
            }
            TomlCondition::Input { input } => self.compile_input_condition(rule_name, input),
            TomlCondition::FlatField(map) => self.compile_flat_field_condition(rule_name, map),
        }
    }

    fn compile_input_condition(&self, rule_name: &str, input: &InputCondition) -> Result<CompiledCondition, CompileError> {
        match input {
            InputCondition::Command { command } => {
                self.compile_operator_condition(rule_name, command, ConditionTarget::Command)
            }
            InputCondition::FilePath { file_path } => {
                self.compile_operator_condition(rule_name, file_path, ConditionTarget::Path)
            }
            InputCondition::Generic(op) => {
                self.compile_operator_condition(rule_name, op, ConditionTarget::Input)
            }
        }
    }

    fn compile_operator_condition(
        &self,
        rule_name: &str,
        op: &OperatorCondition,
        target: ConditionTarget,
    ) -> Result<CompiledCondition, CompileError> {
        // Check each operator type
        if let Some(ref value) = op.contains {
            return Ok(match target {
                ConditionTarget::Command => CompiledCondition::CommandContains(value.clone()),
                ConditionTarget::Path => CompiledCondition::PathContains(value.clone()),
                ConditionTarget::Input => CompiledCondition::InputContains(value.clone()),
            });
        }

        if let Some(ref pattern) = op.matches {
            let regex = Regex::new(pattern).map_err(|e| CompileError::InvalidRegex {
                rule: rule_name.to_string(),
                pattern: pattern.clone(),
                error: e.to_string(),
            })?;
            return Ok(match target {
                ConditionTarget::Command => CompiledCondition::CommandMatches(regex),
                ConditionTarget::Path => CompiledCondition::PathMatches(regex),
                ConditionTarget::Input => CompiledCondition::InputMatches(regex),
            });
        }

        if let Some(ref suffix) = op.ends_with {
            return Ok(CompiledCondition::PathEndsWith(suffix.clone()));
        }

        if let Some(ref prefix) = op.starts_with {
            // For now, convert starts_with to a contains check
            return Ok(match target {
                ConditionTarget::Command => CompiledCondition::CommandContains(prefix.clone()),
                ConditionTarget::Path => CompiledCondition::PathContains(prefix.clone()),
                ConditionTarget::Input => CompiledCondition::InputContains(prefix.clone()),
            });
        }

        if let Some(ref exact) = op.equals {
            // Convert equals to a regex that matches exactly
            let pattern = format!("^{}$", regex::escape(exact));
            let regex = Regex::new(&pattern).map_err(|e| CompileError::InvalidRegex {
                rule: rule_name.to_string(),
                pattern,
                error: e.to_string(),
            })?;
            return Ok(match target {
                ConditionTarget::Command => CompiledCondition::CommandMatches(regex),
                ConditionTarget::Path => CompiledCondition::PathMatches(regex),
                ConditionTarget::Input => CompiledCondition::InputMatches(regex),
            });
        }

        Err(CompileError::InvalidAction {
            rule: rule_name.to_string(),
            message: "No valid operator found in condition".to_string(),
        })
    }

    fn compile_flat_field_condition(&self, rule_name: &str, map: &std::collections::HashMap<String, String>) -> Result<CompiledCondition, CompileError> {
        // Look for known patterns in the flat map
        for (key, value) in map {
            if key.ends_with(".contains") {
                if key.starts_with("input.command") {
                    return Ok(CompiledCondition::CommandContains(value.clone()));
                }
                if key.starts_with("input.file_path") {
                    return Ok(CompiledCondition::PathContains(value.clone()));
                }
                return Ok(CompiledCondition::InputContains(value.clone()));
            }
            if key.ends_with(".matches") {
                let regex = Regex::new(value).map_err(|e| CompileError::InvalidRegex {
                    rule: rule_name.to_string(),
                    pattern: value.clone(),
                    error: e.to_string(),
                })?;

                if key.starts_with("input.command") {
                    return Ok(CompiledCondition::CommandMatches(regex));
                }
                if key.starts_with("input.file_path") {
                    return Ok(CompiledCondition::PathMatches(regex));
                }
                return Ok(CompiledCondition::InputMatches(regex));
            }
            if key.ends_with(".ends_with") {
                return Ok(CompiledCondition::PathEndsWith(value.clone()));
            }
        }

        Err(CompileError::InvalidAction {
            rule: rule_name.to_string(),
            message: "Unknown condition format".to_string(),
        })
    }

    fn compile_action(&self, rule_name: &str, action: &TomlAction) -> Result<(RuleAction, String), CompileError> {
        match action {
            TomlAction::Deny { deny } => Ok((RuleAction::Block, deny.clone())),
            TomlAction::Allow { allow: _ } => Ok((RuleAction::Allow, String::new())),
            TomlAction::Ask { ask } => Ok((RuleAction::Ask, ask.clone())),
            TomlAction::Warn { warn } => Ok((RuleAction::Warn, warn.clone())),
            TomlAction::Run { run: _ } => {
                // Run actions are handled specially at runtime
                Ok((RuleAction::Allow, String::new()))
            }
            TomlAction::Log { log: _ } => {
                // Log actions are handled specially at runtime
                Ok((RuleAction::Allow, String::new()))
            }
            TomlAction::Multiple(_) => {
                Err(CompileError::InvalidAction {
                    rule: rule_name.to_string(),
                    message: "Multiple actions not yet supported".to_string(),
                })
            }
        }
    }

    fn is_valid_event(event: &str) -> bool {
        matches!(
            event,
            "PreToolUse"
                | "PostToolUse"
                | "PostToolUseFailure"
                | "PermissionRequest"
                | "UserPromptSubmit"
                | "Stop"
                | "SubagentStop"
                | "SubagentStart"
                | "SessionStart"
                | "SessionEnd"
                | "Setup"
                | "PreCompact"
                | "Notification"
        )
    }
}

/// Target for condition evaluation.
#[derive(Debug, Clone, Copy)]
enum ConditionTarget {
    Command,
    Path,
    Input,
}

impl TomlCompiledRule {
    /// Check if this rule matches the given tool and input.
    pub fn matches(&self, tool_name: &str, input: &serde_json::Value) -> bool {
        if !self.enabled {
            return false;
        }

        // Check tool pattern
        if let Some(ref pattern) = self.tool_pattern {
            if !self.matches_tool(pattern, tool_name) {
                return false;
            }
        }

        // Check conditions
        if let Some(ref cond) = self.conditions {
            self.check_condition(cond, input)
        } else {
            true
        }
    }

    fn matches_tool(&self, pattern: &str, tool_name: &str) -> bool {
        // Handle OR patterns like "Write|Edit"
        if pattern.contains('|') {
            return pattern.split('|').any(|p| self.matches_single_tool(p.trim(), tool_name));
        }

        self.matches_single_tool(pattern, tool_name)
    }

    fn matches_single_tool(&self, pattern: &str, tool_name: &str) -> bool {
        // Handle glob patterns
        if pattern.contains('*') {
            let regex_pattern = pattern.replace('.', r"\.").replace('*', ".*");
            return Regex::new(&format!("^{regex_pattern}$"))
                .map(|re| re.is_match(tool_name))
                .unwrap_or(false);
        }

        pattern == tool_name
    }

    fn check_condition(&self, cond: &CompiledCondition, input: &serde_json::Value) -> bool {
        check_condition_recursive(cond, input)
    }
}

/// Recursive condition evaluation (standalone to satisfy clippy).
fn check_condition_recursive(cond: &CompiledCondition, input: &serde_json::Value) -> bool {
    match cond {
        CompiledCondition::All(conditions) => {
            conditions.iter().all(|c| check_condition_recursive(c, input))
        }
        CompiledCondition::Any(conditions) => {
            conditions.iter().any(|c| check_condition_recursive(c, input))
        }
        CompiledCondition::Not(inner) => !check_condition_recursive(inner, input),
        CompiledCondition::InputContains(text) => {
            let input_str = input.to_string();
            input_str.contains(text)
        }
        CompiledCondition::InputMatches(regex) => {
            let input_str = input.to_string();
            regex.is_match(&input_str)
        }
        CompiledCondition::PathContains(text) => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s.contains(text)),
        CompiledCondition::PathMatches(regex) => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .is_some_and(|s| regex.is_match(s)),
        CompiledCondition::PathEndsWith(suffix) => input
            .get("file_path")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s.ends_with(suffix)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_basic_rule() {
        let config = TomlConfig::parse(r#"
[[rules]]
name = "test"
event = "PreToolUse"
matcher = "Bash"
[rules.action]
deny = "Blocked"
"#).unwrap();

        let mut compiler = Compiler::new();
        let rules = compiler.compile(&config).unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "test");
        assert_eq!(rules[0].action, RuleAction::Block);
    }

    #[test]
    fn test_compile_with_condition() {
        let config = TomlConfig::parse(r#"
[[rules]]
name = "test"
event = "PreToolUse"
matcher = "Bash"
[rules.when]
input.command.contains = "rm -rf"
[rules.action]
deny = "Dangerous"
"#).unwrap();

        let mut compiler = Compiler::new();
        let rules = compiler.compile(&config).unwrap();

        assert_eq!(rules.len(), 1);
        assert!(rules[0].conditions.is_some());

        // Test matching
        let input = serde_json::json!({ "command": "rm -rf /" });
        assert!(rules[0].matches("Bash", &input));

        let input = serde_json::json!({ "command": "ls" });
        assert!(!rules[0].matches("Bash", &input));
    }

    #[test]
    fn test_compile_invalid_event() {
        let config = TomlConfig::parse(r#"
[[rules]]
name = "test"
event = "InvalidEvent"
[rules.action]
deny = "X"
"#).unwrap();

        let mut compiler = Compiler::new();
        let result = compiler.compile(&config);

        assert!(result.is_err());
    }

    #[test]
    fn test_glob_pattern_matching() {
        let config = TomlConfig::parse(r#"
[[rules]]
name = "mcp-delete"
event = "PreToolUse"
matcher = "mcp__*__delete_*"
[rules.action]
ask = "Confirm?"
"#).unwrap();

        let mut compiler = Compiler::new();
        let rules = compiler.compile(&config).unwrap();

        let input = serde_json::json!({});
        assert!(rules[0].matches("mcp__nexcore__delete_file", &input));
        assert!(rules[0].matches("mcp__other__delete_user", &input));
        assert!(!rules[0].matches("mcp__nexcore__read_file", &input));
    }

    #[test]
    fn test_or_pattern_matching() {
        let config = TomlConfig::parse(r#"
[[rules]]
name = "write-edit"
event = "PreToolUse"
matcher = "Write|Edit"
[rules.action]
warn = "Modifying file"
"#).unwrap();

        let mut compiler = Compiler::new();
        let rules = compiler.compile(&config).unwrap();

        let input = serde_json::json!({});
        assert!(rules[0].matches("Write", &input));
        assert!(rules[0].matches("Edit", &input));
        assert!(!rules[0].matches("Read", &input));
    }
}
