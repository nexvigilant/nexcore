// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 4: Guard Evaluator (STOS-GD)
//!
//! **Dominant Primitive**: κ (Comparison)
//!
//! Evaluates guard conditions on transitions, determining whether
//! a transition is allowed based on predicates.
//!
//! ## Responsibilities
//!
//! - Guard registration and lookup
//! - Predicate evaluation
//! - Guard composition (AND, OR, NOT)
//! - Guard caching
//!
//! ## Tier Classification
//!
//! `GuardEvaluator` is T2-C (κ + → + ς) — comparison, causality, state.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::MachineId;

/// Unique identifier for a guard.
pub type GuardId = u32;

/// A guard specification.
#[derive(Debug, Clone)]
pub struct GuardSpec {
    /// Unique guard ID.
    pub id: GuardId,
    /// Guard name.
    pub name: String,
    /// Guard expression (string representation).
    pub expression: String,
    /// Whether guard is enabled.
    pub enabled: bool,
}

impl GuardSpec {
    /// Create a new guard.
    #[must_use]
    pub fn new(id: GuardId, name: impl Into<String>, expression: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            expression: expression.into(),
            enabled: true,
        }
    }

    /// Disable the guard.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Result of guard evaluation.
#[derive(Debug, Clone)]
pub struct GuardResult {
    /// The guard that was evaluated.
    pub guard_id: GuardId,
    /// Whether the guard passed.
    pub passed: bool,
    /// Evaluation timestamp.
    pub timestamp: u64,
    /// Optional reason for failure.
    pub reason: Option<String>,
}

impl GuardResult {
    /// Create a passing result.
    #[must_use]
    pub fn pass(guard_id: GuardId, timestamp: u64) -> Self {
        Self {
            guard_id,
            passed: true,
            timestamp,
            reason: None,
        }
    }

    /// Create a failing result.
    #[must_use]
    pub fn fail(guard_id: GuardId, timestamp: u64, reason: impl Into<String>) -> Self {
        Self {
            guard_id,
            passed: false,
            timestamp,
            reason: Some(reason.into()),
        }
    }
}

/// A context for guard evaluation.
#[derive(Debug, Clone, Default)]
pub struct GuardContext {
    /// Variables available during evaluation.
    pub variables: BTreeMap<String, GuardValue>,
}

/// A value in guard context.
#[derive(Debug, Clone, PartialEq)]
pub enum GuardValue {
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// String value.
    String(String),
    /// Null/missing value.
    Null,
}

impl GuardContext {
    /// Create empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a boolean variable.
    pub fn set_bool(&mut self, name: impl Into<String>, value: bool) {
        self.variables.insert(name.into(), GuardValue::Bool(value));
    }

    /// Add an integer variable.
    pub fn set_int(&mut self, name: impl Into<String>, value: i64) {
        self.variables.insert(name.into(), GuardValue::Int(value));
    }

    /// Add a string variable.
    pub fn set_string(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables
            .insert(name.into(), GuardValue::String(value.into()));
    }

    /// Get a boolean variable.
    #[must_use]
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.variables.get(name) {
            Some(GuardValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Get an integer variable.
    #[must_use]
    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.variables.get(name) {
            Some(GuardValue::Int(i)) => Some(*i),
            _ => None,
        }
    }
}

/// Predicate function type.
pub type PredicateFn = Box<dyn Fn(&GuardContext) -> bool + Send + Sync>;

/// The guard evaluator for a machine.
///
/// ## Tier: T2-C (κ + → + ς)
///
/// Dominant primitive: κ (Comparison)
#[derive(Debug)]
pub struct GuardEvaluator {
    /// Machine ID.
    machine_id: MachineId,
    /// Guards by ID.
    guards: BTreeMap<GuardId, GuardSpec>,
    /// Name to ID mapping.
    name_index: BTreeMap<String, GuardId>,
    /// Next guard ID.
    next_id: GuardId,
    /// Evaluation counter.
    eval_counter: u64,
    /// Evaluation history.
    history: Vec<GuardResult>,
}

impl GuardEvaluator {
    /// Create a new guard evaluator.
    #[must_use]
    pub fn new(machine_id: MachineId) -> Self {
        Self {
            machine_id,
            guards: BTreeMap::new(),
            name_index: BTreeMap::new(),
            next_id: 0,
            eval_counter: 0,
            history: Vec::new(),
        }
    }

    /// Register a guard.
    pub fn register(&mut self, name: impl Into<String>, expression: impl Into<String>) -> GuardId {
        let name = name.into();
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        let spec = GuardSpec::new(id, name.clone(), expression);
        self.guards.insert(id, spec);
        self.name_index.insert(name, id);

        id
    }

    /// Get guard by ID.
    #[must_use]
    pub fn get(&self, id: GuardId) -> Option<&GuardSpec> {
        self.guards.get(&id)
    }

    /// Get guard by name.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&GuardSpec> {
        self.name_index.get(name).and_then(|id| self.guards.get(id))
    }

    /// Evaluate a guard with simple expression parsing.
    ///
    /// Supports:
    /// - `true`, `false` literals
    /// - Variable names (looked up in context as bools)
    /// - `!expr` negation
    /// - `expr && expr` conjunction
    /// - `expr || expr` disjunction
    pub fn evaluate(&mut self, guard_id: GuardId, context: &GuardContext) -> GuardResult {
        self.eval_counter = self.eval_counter.saturating_add(1);

        let spec = match self.guards.get(&guard_id) {
            Some(s) => s,
            None => {
                return GuardResult::fail(guard_id, self.eval_counter, "Guard not found");
            }
        };

        if !spec.enabled {
            // Disabled guards always pass
            return GuardResult::pass(guard_id, self.eval_counter);
        }

        let passed = self.evaluate_expression(&spec.expression, context);
        let result = if passed {
            GuardResult::pass(guard_id, self.eval_counter)
        } else {
            GuardResult::fail(
                guard_id,
                self.eval_counter,
                alloc::format!("Guard '{}' failed", spec.name),
            )
        };

        self.history.push(result.clone());
        result
    }

    /// Simple expression evaluator.
    fn evaluate_expression(&self, expr: &str, context: &GuardContext) -> bool {
        let expr = expr.trim();

        // Literals
        if expr == "true" {
            return true;
        }
        if expr == "false" {
            return false;
        }

        // Negation
        if let Some(inner) = expr.strip_prefix('!') {
            return !self.evaluate_expression(inner, context);
        }

        // Conjunction (simple, no precedence)
        if let Some((left, right)) = expr.split_once("&&") {
            return self.evaluate_expression(left, context)
                && self.evaluate_expression(right, context);
        }

        // Disjunction
        if let Some((left, right)) = expr.split_once("||") {
            return self.evaluate_expression(left, context)
                || self.evaluate_expression(right, context);
        }

        // Variable lookup
        context.get_bool(expr).unwrap_or(false)
    }

    /// Evaluate guard by name.
    pub fn evaluate_by_name(&mut self, name: &str, context: &GuardContext) -> Option<GuardResult> {
        let id = self.name_index.get(name).copied()?;
        Some(self.evaluate(id, context))
    }

    /// Get evaluation history.
    #[must_use]
    pub fn history(&self) -> &[GuardResult] {
        &self.history
    }

    /// Total guards registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.guards.len()
    }

    /// Whether no guards are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }
}

impl Clone for GuardEvaluator {
    fn clone(&self) -> Self {
        Self {
            machine_id: self.machine_id,
            guards: self.guards.clone(),
            name_index: self.name_index.clone(),
            next_id: self.next_id,
            eval_counter: self.eval_counter,
            history: self.history.clone(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_spec() {
        let spec = GuardSpec::new(0, "is_valid", "valid && !expired");
        assert_eq!(spec.name, "is_valid");
        assert!(spec.enabled);
    }

    #[test]
    fn test_guard_context() {
        let mut ctx = GuardContext::new();
        ctx.set_bool("valid", true);
        ctx.set_int("count", 42);
        ctx.set_string("name", "test");

        assert_eq!(ctx.get_bool("valid"), Some(true));
        assert_eq!(ctx.get_int("count"), Some(42));
    }

    #[test]
    fn test_evaluate_literals() {
        let mut evaluator = GuardEvaluator::new(1);
        let ctx = GuardContext::new();

        let g1 = evaluator.register("always_true", "true");
        let g2 = evaluator.register("always_false", "false");

        assert!(evaluator.evaluate(g1, &ctx).passed);
        assert!(!evaluator.evaluate(g2, &ctx).passed);
    }

    #[test]
    fn test_evaluate_variables() {
        let mut evaluator = GuardEvaluator::new(1);
        let mut ctx = GuardContext::new();
        ctx.set_bool("enabled", true);
        ctx.set_bool("blocked", false);

        let g1 = evaluator.register("check_enabled", "enabled");
        let g2 = evaluator.register("check_blocked", "blocked");

        assert!(evaluator.evaluate(g1, &ctx).passed);
        assert!(!evaluator.evaluate(g2, &ctx).passed);
    }

    #[test]
    fn test_evaluate_conjunction() {
        let mut evaluator = GuardEvaluator::new(1);
        let mut ctx = GuardContext::new();
        ctx.set_bool("a", true);
        ctx.set_bool("b", true);

        let g = evaluator.register("both", "a && b");
        assert!(evaluator.evaluate(g, &ctx).passed);

        ctx.set_bool("b", false);
        assert!(!evaluator.evaluate(g, &ctx).passed);
    }

    #[test]
    fn test_evaluate_negation() {
        let mut evaluator = GuardEvaluator::new(1);
        let mut ctx = GuardContext::new();
        ctx.set_bool("blocked", false);

        let g = evaluator.register("not_blocked", "!blocked");
        assert!(evaluator.evaluate(g, &ctx).passed);

        ctx.set_bool("blocked", true);
        assert!(!evaluator.evaluate(g, &ctx).passed);
    }
}
