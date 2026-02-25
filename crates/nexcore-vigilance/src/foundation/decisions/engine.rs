//! # Decision Tree Engine
//!
//! Deterministic execution engine for logic trees defined in YAML/JSON.
//!
//! ## Features
//! - Execute decision trees with conditional branching
//! - Support for multiple operators (Eq, Neq, Gt, Lt, Contains, Matches, etc.)
//! - Variable interpolation with `{{variable}}` syntax
//! - Intrinsic function execution
//! - Strict mode for Diamond compliance (no LLM fallbacks)

use nexcore_error::Error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Decision engine errors
#[derive(Error, Debug, Clone)]
pub enum DecisionError {
    /// YAML parsing error
    #[error("Invalid decision tree YAML: {0}")]
    YamlParse(String),
    /// Node not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    /// Max depth exceeded
    #[error("Max execution depth exceeded")]
    MaxDepthExceeded,
    /// Strict mode violation
    #[error("Strict mode violation: {0}")]
    StrictModeViolation(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Comparison operators for conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    /// Equal
    Eq,
    /// Not equal
    Neq,
    /// Greater than
    Gt,
    /// Greater than or equal
    Gte,
    /// Less than
    Lt,
    /// Less than or equal
    Lte,
    /// Contains (string or array)
    Contains,
    /// Does not contain
    NotContains,
    /// Regex match
    Matches,
    /// Is null
    IsNull,
    /// Is not null
    IsNotNull,
}

/// Dynamic value type for decision tree operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum Value {
    /// Null value
    #[default]
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Array value
    Array(Vec<Value>),
    /// Object/map value
    Object(HashMap<String, Value>),
}

impl Value {
    /// Convert to string representation
    #[must_use]
    pub fn as_string(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => format!("{arr:?}"),
            Value::Object(obj) => format!("{obj:?}"),
        }
    }

    /// Try to convert to f64
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Check if value is null
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// A node in a decision tree
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecisionNode {
    /// Conditional branch node
    Condition {
        /// Variable to check
        variable: String,
        /// Comparison operator
        operator: Operator,
        /// Value to compare against
        #[serde(default)]
        value: Option<Value>,
        /// Node to go to if condition is true
        true_next: String,
        /// Node to go to if condition is false
        false_next: String,
    },
    /// Action execution node
    Action {
        /// Action name
        action: String,
        /// Target variable
        #[serde(default)]
        target: Option<String>,
        /// Value to set
        #[serde(default)]
        value: Option<Value>,
        /// Next node
        #[serde(default)]
        next: Option<String>,
    },
    /// Return node (terminates execution)
    Return {
        /// Return value
        value: Value,
    },
    /// LLM fallback node (not allowed in strict mode)
    LlmFallback {
        /// Prompt for the LLM
        prompt: String,
        /// Optional output schema
        schema: Option<Value>,
    },
    /// Intrinsic function call
    Intrinsic {
        /// Function name
        function: String,
        /// Input variable name
        input_variable: String,
        /// Output variable name
        output_variable: String,
        /// Next node
        next: String,
    },
}

/// A decision tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    /// Starting node ID
    pub start: String,
    /// All nodes in the tree
    pub nodes: HashMap<String, DecisionNode>,
}

/// Execution context for decision tree
#[derive(Debug, Clone, Default)]
pub struct DecisionContext {
    /// Variables available during execution
    pub variables: HashMap<String, Value>,
    /// Path of nodes traversed
    pub execution_path: Vec<String>,
}

impl DecisionContext {
    /// Create a new empty context
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable
    pub fn set(&mut self, key: &str, value: Value) {
        self.variables.insert(key.to_string(), value);
    }

    /// Get a variable
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }
}

/// Result of executing a decision tree
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Returned a value
    Value(Value),
    /// Requires LLM call (only in non-strict mode)
    LlmRequest {
        /// Prompt for the LLM
        prompt: String,
        /// Context at time of request
        context: HashMap<String, Value>,
    },
    /// Error occurred
    Error(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// DECISION ENGINE
// ═══════════════════════════════════════════════════════════════════════════

/// The decision tree execution engine
pub struct DecisionEngine {
    tree: DecisionTree,
}

impl DecisionEngine {
    /// Create a new engine with the given tree
    #[must_use]
    pub fn new(tree: DecisionTree) -> Self {
        Self { tree }
    }

    /// Execute the decision tree with the given context
    pub fn execute(&self, ctx: &mut DecisionContext) -> ExecutionResult {
        let mut current_id = self.tree.start.clone();
        let max_steps = 1000;
        let mut steps = 0;

        loop {
            if steps >= max_steps {
                return ExecutionResult::Error("Max depth exceeded".to_string());
            }
            steps += 1;
            ctx.execution_path.push(current_id.clone());

            let node = match self.tree.nodes.get(&current_id) {
                Some(n) => n,
                None => return ExecutionResult::Error(format!("Node not found: {current_id}")),
            };

            match node {
                DecisionNode::Condition {
                    variable,
                    operator,
                    value,
                    true_next,
                    false_next,
                } => {
                    let var_val = ctx.get(variable).unwrap_or(&Value::Null);
                    let result = self.evaluate_condition(var_val, operator, value.as_ref());
                    current_id = if result {
                        true_next.clone()
                    } else {
                        false_next.clone()
                    };
                }
                DecisionNode::Action {
                    action,
                    target,
                    value,
                    next,
                } => {
                    let processed_val = value
                        .as_ref()
                        .map(|v| self.interpolate_value(v, ctx))
                        .unwrap_or(Value::Null);

                    match action.as_str() {
                        "set_variable" => {
                            if let Some(t) = target {
                                ctx.set(t, processed_val);
                            }
                        }
                        "log" => {
                            // Log action - could be captured elsewhere
                        }
                        _ => {}
                    }

                    match next {
                        Some(n) => current_id = n.clone(),
                        None => {
                            return ExecutionResult::Error("Action has no next node".to_string());
                        }
                    }
                }
                DecisionNode::Return { value } => {
                    return ExecutionResult::Value(self.interpolate_value(value, ctx));
                }
                DecisionNode::LlmFallback { prompt, .. } => {
                    return ExecutionResult::LlmRequest {
                        prompt: prompt.clone(),
                        context: ctx.variables.clone(),
                    };
                }
                DecisionNode::Intrinsic {
                    function,
                    input_variable,
                    output_variable,
                    next,
                } => {
                    let input = ctx.get(input_variable).cloned().unwrap_or(Value::Null);
                    let result = self.execute_intrinsic(function, input);
                    ctx.set(output_variable, result);
                    current_id = next.clone();
                }
            }
        }
    }

    /// Interpolate variables in a value (handles `{{variable}}` syntax)
    pub fn interpolate_value(&self, val: &Value, ctx: &DecisionContext) -> Value {
        match val {
            Value::String(s) if s.contains("{{") => {
                let mut result = s.clone();
                while let Some(start) = result.find("{{") {
                    if let Some(end) = result[start..].find("}}") {
                        let full_tag = &result[start..start + end + 2];
                        let key_path = result[start + 2..start + end].trim();
                        let replacement = self.resolve_path(key_path, ctx).as_string();
                        result = result.replace(full_tag, &replacement);
                    } else {
                        break;
                    }
                }
                Value::String(result)
            }
            Value::Object(map) => {
                let mut new_map = HashMap::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.interpolate_value(v, ctx));
                }
                Value::Object(new_map)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| self.interpolate_value(v, ctx)).collect())
            }
            _ => val.clone(),
        }
    }

    fn resolve_path(&self, path: &str, ctx: &DecisionContext) -> Value {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current: Option<Value> = None;

        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                current = self.resolve_part(part, ctx, None);
            } else if let Some(val) = current {
                current = self.resolve_part(part, ctx, Some(val));
            } else {
                return Value::Null;
            }
        }
        current.unwrap_or(Value::Null)
    }

    fn resolve_part(
        &self,
        part: &str,
        ctx: &DecisionContext,
        base: Option<Value>,
    ) -> Option<Value> {
        if part.contains('[') && part.contains(']') {
            let base_key = part.split('[').next().unwrap_or_default();
            let idx_str = part
                .split('[')
                .nth(1)
                .unwrap_or_default()
                .trim_end_matches(']');

            let array_val = if let Some(b) = base {
                if let Value::Object(map) = b {
                    map.get(base_key).cloned()
                } else {
                    None
                }
            } else {
                ctx.variables.get(base_key).cloned()
            };

            if let (Some(Value::Array(arr)), Ok(idx)) = (array_val, idx_str.parse::<usize>()) {
                arr.get(idx).cloned()
            } else {
                None
            }
        } else if let Some(b) = base {
            match b {
                Value::Object(map) => map.get(part).cloned(),
                _ => None,
            }
        } else {
            ctx.variables.get(part).cloned()
        }
    }

    fn execute_intrinsic(&self, function: &str, input: Value) -> Value {
        match function {
            "to_uppercase" => {
                if let Value::String(s) = input {
                    Value::String(s.to_uppercase())
                } else {
                    Value::String(input.as_string().to_uppercase())
                }
            }
            "to_lowercase" => {
                if let Value::String(s) = input {
                    Value::String(s.to_lowercase())
                } else {
                    Value::String(input.as_string().to_lowercase())
                }
            }
            "length" => match input {
                Value::String(s) => Value::Int(s.len() as i64),
                Value::Array(arr) => Value::Int(arr.len() as i64),
                Value::Object(map) => Value::Int(map.len() as i64),
                _ => Value::Int(0),
            },
            "is_empty" => match input {
                Value::String(s) => Value::Bool(s.is_empty()),
                Value::Array(arr) => Value::Bool(arr.is_empty()),
                Value::Object(map) => Value::Bool(map.is_empty()),
                Value::Null => Value::Bool(true),
                _ => Value::Bool(false),
            },
            "trim" => {
                if let Value::String(s) = input {
                    Value::String(s.trim().to_string())
                } else {
                    input
                }
            }
            _ => Value::String(format!("Unknown intrinsic: {function}")),
        }
    }

    fn evaluate_condition(&self, actual: &Value, op: &Operator, target: Option<&Value>) -> bool {
        match op {
            Operator::Eq => actual == target.unwrap_or(&Value::Null),
            Operator::Neq => actual != target.unwrap_or(&Value::Null),
            Operator::IsNull => matches!(actual, Value::Null),
            Operator::IsNotNull => !matches!(actual, Value::Null),
            Operator::Gt => {
                actual.as_f64().unwrap_or(0.0) > target.and_then(|v| v.as_f64()).unwrap_or(0.0)
            }
            Operator::Gte => {
                actual.as_f64().unwrap_or(0.0) >= target.and_then(|v| v.as_f64()).unwrap_or(0.0)
            }
            Operator::Lt => {
                actual.as_f64().unwrap_or(0.0) < target.and_then(|v| v.as_f64()).unwrap_or(0.0)
            }
            Operator::Lte => {
                actual.as_f64().unwrap_or(0.0) <= target.and_then(|v| v.as_f64()).unwrap_or(0.0)
            }
            Operator::Contains => {
                if let (Value::String(s), Some(Value::String(t))) = (actual, target) {
                    s.contains(t)
                } else if let (Value::Array(a), Some(t)) = (actual, target) {
                    a.contains(t)
                } else {
                    false
                }
            }
            Operator::NotContains => {
                if let (Value::String(s), Some(Value::String(t))) = (actual, target) {
                    !s.contains(t)
                } else if let (Value::Array(a), Some(t)) = (actual, target) {
                    !a.contains(t)
                } else {
                    true
                }
            }
            Operator::Matches => {
                if let (Value::String(s), Some(Value::String(t))) = (actual, target) {
                    Regex::new(t).is_ok_and(|re| re.is_match(s))
                } else {
                    false
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TREE LOADING
// ═══════════════════════════════════════════════════════════════════════════

/// Load a decision tree from YAML
///
/// # Errors
///
/// Returns `DecisionError::YamlParse` if parsing fails.
pub fn load_tree(yaml: &str) -> Result<DecisionTree, DecisionError> {
    serde_yml::from_str(yaml).map_err(|e| DecisionError::YamlParse(e.to_string()))
}

/// Load a decision tree in strict mode (no LLM fallbacks allowed)
///
/// # Errors
///
/// Returns `DecisionError` if parsing fails or if any node uses `LlmFallback`.
pub fn load_tree_strict(yaml: &str) -> Result<DecisionTree, DecisionError> {
    let tree: DecisionTree =
        serde_yml::from_str(yaml).map_err(|e| DecisionError::YamlParse(e.to_string()))?;

    // Diamond compliance: No LlmFallback allowed
    for (id, node) in &tree.nodes {
        if matches!(node, DecisionNode::LlmFallback { .. }) {
            return Err(DecisionError::StrictModeViolation(format!(
                "Node '{id}' uses forbidden LlmFallback. Deterministic logic required."
            )));
        }
    }

    Ok(tree)
}

// ═══════════════════════════════════════════════════════════════════════════
// LEGACY COMPATIBILITY
// ═══════════════════════════════════════════════════════════════════════════

/// Legacy decision node (simple structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyDecisionNode {
    /// Unique node identifier
    pub id: String,
    /// Condition to evaluate (if branch node)
    pub condition: Option<String>,
    /// Action to execute (if leaf node)
    pub action: Option<String>,
    /// Child node IDs
    pub children: Vec<String>,
}

/// Result of decision tree execution (legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    /// Path taken through the tree
    pub path: Vec<String>,
    /// Final action (if reached a leaf)
    pub action: Option<String>,
    /// Variables collected during traversal
    pub variables: HashMap<String, String>,
}

/// Legacy decision engine (simple condition evaluator pattern)
#[derive(Debug, Clone, Default)]
pub struct LegacyDecisionEngine {
    nodes: HashMap<String, LegacyDecisionNode>,
    root_id: Option<String>,
}

impl LegacyDecisionEngine {
    /// Create a new decision engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_id: None,
        }
    }

    /// Add a node to the tree
    pub fn add_node(&mut self, node: LegacyDecisionNode) {
        if self.root_id.is_none() {
            self.root_id = Some(node.id.clone());
        }
        self.nodes.insert(node.id.clone(), node);
    }

    /// Set the root node
    pub fn set_root(&mut self, id: &str) {
        self.root_id = Some(id.to_string());
    }

    /// Execute the decision tree with given context
    pub fn execute<F>(
        &self,
        context: &HashMap<String, String>,
        condition_evaluator: F,
    ) -> DecisionResult
    where
        F: Fn(&str, &HashMap<String, String>) -> bool,
    {
        let mut path = Vec::new();
        let mut current_id = self.root_id.clone();

        while let Some(id) = current_id {
            path.push(id.clone());

            if let Some(node) = self.nodes.get(&id) {
                if let Some(action) = &node.action {
                    return DecisionResult {
                        path,
                        action: Some(action.clone()),
                        variables: context.clone(),
                    };
                }

                if let Some(condition) = &node.condition {
                    let result = condition_evaluator(condition, context);
                    current_id = if result && !node.children.is_empty() {
                        Some(node.children[0].clone())
                    } else if node.children.len() > 1 {
                        Some(node.children[1].clone())
                    } else {
                        None
                    };
                } else {
                    current_id = node.children.first().cloned();
                }
            } else {
                break;
            }
        }

        DecisionResult {
            path,
            action: None,
            variables: context.clone(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_return() {
        let yaml = r#"
start: end
nodes:
  end:
    type: return
    value: "success"
"#;
        let tree = load_tree(yaml).unwrap();
        let engine = DecisionEngine::new(tree);
        let mut ctx = DecisionContext::new();

        match engine.execute(&mut ctx) {
            ExecutionResult::Value(v) => assert_eq!(v, Value::String("success".to_string())),
            _ => panic!("Expected Value result"),
        }
    }

    #[test]
    fn test_condition_true_branch() {
        let yaml = r#"
start: check
nodes:
  check:
    type: condition
    variable: x
    operator: gt
    value: 0
    true_next: positive
    false_next: negative
  positive:
    type: return
    value: "positive"
  negative:
    type: return
    value: "negative"
"#;
        let tree = load_tree(yaml).unwrap();
        let engine = DecisionEngine::new(tree);
        let mut ctx = DecisionContext::new();
        ctx.set("x", Value::Int(5));

        match engine.execute(&mut ctx) {
            ExecutionResult::Value(v) => assert_eq!(v, Value::String("positive".to_string())),
            _ => panic!("Expected Value result"),
        }
    }

    #[test]
    fn test_condition_false_branch() {
        let yaml = r#"
start: check
nodes:
  check:
    type: condition
    variable: x
    operator: gt
    value: 0
    true_next: positive
    false_next: negative
  positive:
    type: return
    value: "positive"
  negative:
    type: return
    value: "negative"
"#;
        let tree = load_tree(yaml).unwrap();
        let engine = DecisionEngine::new(tree);
        let mut ctx = DecisionContext::new();
        ctx.set("x", Value::Int(-5));

        match engine.execute(&mut ctx) {
            ExecutionResult::Value(v) => assert_eq!(v, Value::String("negative".to_string())),
            _ => panic!("Expected Value result"),
        }
    }

    #[test]
    fn test_interpolation() {
        let tree = DecisionTree {
            start: "end".to_string(),
            nodes: HashMap::from([(
                "end".to_string(),
                DecisionNode::Return {
                    value: Value::String("Hello {{name}}!".to_string()),
                },
            )]),
        };

        let engine = DecisionEngine::new(tree);
        let mut ctx = DecisionContext::new();
        ctx.set("name", Value::String("World".to_string()));

        match engine.execute(&mut ctx) {
            ExecutionResult::Value(v) => assert_eq!(v, Value::String("Hello World!".to_string())),
            _ => panic!("Expected Value result"),
        }
    }

    #[test]
    fn test_strict_mode_rejects_llm() {
        let yaml = r#"
start: node1
nodes:
  node1:
    type: llm_fallback
    prompt: "Lazy logic"
"#;
        let result = load_tree_strict(yaml);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("uses forbidden LlmFallback")
        );
    }

    #[test]
    fn test_intrinsic_uppercase() {
        let yaml = r#"
start: transform
nodes:
  transform:
    type: intrinsic
    function: to_uppercase
    input_variable: text
    output_variable: result
    next: end
  end:
    type: return
    value: "{{result}}"
"#;
        let tree = load_tree(yaml).unwrap();
        let engine = DecisionEngine::new(tree);
        let mut ctx = DecisionContext::new();
        ctx.set("text", Value::String("hello".to_string()));

        match engine.execute(&mut ctx) {
            ExecutionResult::Value(v) => assert_eq!(v, Value::String("HELLO".to_string())),
            _ => panic!("Expected Value result"),
        }
    }

    #[test]
    fn test_contains_operator() {
        let yaml = r#"
start: check
nodes:
  check:
    type: condition
    variable: text
    operator: contains
    value: "world"
    true_next: found
    false_next: not_found
  found:
    type: return
    value: "found"
  not_found:
    type: return
    value: "not_found"
"#;
        let tree = load_tree(yaml).unwrap();
        let engine = DecisionEngine::new(tree);
        let mut ctx = DecisionContext::new();
        ctx.set("text", Value::String("hello world".to_string()));

        match engine.execute(&mut ctx) {
            ExecutionResult::Value(v) => assert_eq!(v, Value::String("found".to_string())),
            _ => panic!("Expected Value result"),
        }
    }

    #[test]
    fn test_is_null_operator() {
        let yaml = r#"
start: check
nodes:
  check:
    type: condition
    variable: missing
    operator: is_null
    true_next: null_branch
    false_next: not_null_branch
  null_branch:
    type: return
    value: "was_null"
  not_null_branch:
    type: return
    value: "was_not_null"
"#;
        let tree = load_tree(yaml).unwrap();
        let engine = DecisionEngine::new(tree);
        let mut ctx = DecisionContext::new();
        // Don't set 'missing' - it should be null

        match engine.execute(&mut ctx) {
            ExecutionResult::Value(v) => assert_eq!(v, Value::String("was_null".to_string())),
            _ => panic!("Expected Value result"),
        }
    }

    #[test]
    fn test_legacy_decision_engine() {
        let mut engine = LegacyDecisionEngine::new();

        engine.add_node(LegacyDecisionNode {
            id: "root".to_string(),
            condition: Some("is_valid".to_string()),
            action: None,
            children: vec!["yes".to_string(), "no".to_string()],
        });
        engine.add_node(LegacyDecisionNode {
            id: "yes".to_string(),
            condition: None,
            action: Some("proceed".to_string()),
            children: vec![],
        });
        engine.add_node(LegacyDecisionNode {
            id: "no".to_string(),
            condition: None,
            action: Some("reject".to_string()),
            children: vec![],
        });

        let mut context = HashMap::new();
        context.insert("is_valid".to_string(), "true".to_string());

        let result = engine.execute(&context, |cond, ctx| {
            ctx.get(cond).map_or(false, |v| v == "true")
        });

        assert_eq!(result.action, Some("proceed".to_string()));
        assert!(result.path.contains(&"yes".to_string()));
    }
}
