//! # Experiment 3: Type-1 Context-Sensitive Grammar — Conservation Guards
//!
//! **Chomsky Level:** Type-1 (Context-Sensitive)
//! **Primitives:** σ (Sequence) + Σ (Sum) + ρ (Recursion) + κ (Comparison)
//! **STEM Traits:** Membership, Bound, Preserve (+ all from Exp 1 & 2)
//!
//! ## What Changes From Experiment 2
//!
//! We add ONE primitive: κ (Comparison).
//! This lets us CHECK CONTEXT before allowing a transition.
//! The grammar becomes: αAβ → αγβ (where α, β are surrounding context)
//!
//! ## Grammar (Context-Sensitive)
//!
//! ```text
//! Machine → StateDecl Machine | ε
//! StateDecl → StateName '{' Machine '}'
//!           | StateName  [IF context_valid(parent, StateName)]
//! ```
//!
//! The `[IF ...]` is the context check — this is what κ enables.
//! A CFG allows any state anywhere. A CSG restricts states based on
//! their surrounding context (parent state, accumulated invariants).
//!
//! ## Practical Output
//!
//! `ConservationValidator` — validates state machines where transitions
//! must preserve invariants (conservation laws) and states are only
//! valid within certain parent contexts.

#![allow(dead_code)]

use stem_math::{Bound, Membership};
use stem_phys::Preserve;

use std::collections::{HashMap, HashSet};

// ============================================================================
// Context Types (T2-C: σ + Σ + ρ + κ)
// ============================================================================

/// A context rule: "state X is only valid inside parent Y"
/// Tier: T2-C (κ Comparison — evaluates context before allowing transition)
#[derive(Debug, Clone)]
pub struct ContextRule {
    /// The state this rule applies to
    pub state: String,
    /// Valid parent states (if empty, valid at root)
    pub valid_parents: HashSet<String>,
    /// Required invariants that must hold
    pub required_invariants: Vec<String>,
}

/// An invariant — a conserved quantity across state transitions
/// Tier: T2-C (κ + ς — comparison of state property before/after)
#[derive(Debug, Clone)]
pub struct Invariant {
    pub name: String,
    pub value: f64,
    pub tolerance: f64,
}

/// Context — the accumulated state during validation
/// This is the "α" and "β" in the CSG production αAβ → αγβ
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Stack of parent state names (innermost last)
    pub parent_stack: Vec<String>,
    /// Current invariant values
    pub invariants: HashMap<String, f64>,
    /// Accumulated resource budget
    pub resource_budget: f64,
}

impl ValidationContext {
    #[must_use]
    pub fn new(initial_budget: f64) -> Self {
        Self {
            parent_stack: Vec::new(),
            invariants: HashMap::new(),
            resource_budget: initial_budget,
        }
    }

    /// Current parent (top of stack)
    #[must_use]
    pub fn current_parent(&self) -> Option<&str> {
        self.parent_stack.last().map(|s| s.as_str())
    }

    /// Push a parent context (entering a nested state)
    pub fn push_parent(&mut self, name: &str) {
        self.parent_stack.push(name.to_string());
    }

    /// Pop a parent context (leaving a nested state)
    pub fn pop_parent(&mut self) -> Option<String> {
        self.parent_stack.pop()
    }

    /// Depth of nesting
    #[must_use]
    pub fn depth(&self) -> usize {
        self.parent_stack.len()
    }
}

/// Validation result with context information
#[derive(Debug, Clone)]
pub enum ContextValidation {
    Valid,
    InvalidContext {
        state: String,
        found_parent: Option<String>,
        expected_parents: Vec<String>,
    },
    InvariantViolation {
        invariant: String,
        expected: f64,
        actual: f64,
        tolerance: f64,
    },
    BudgetExceeded {
        state: String,
        cost: f64,
        remaining: f64,
    },
}

// ============================================================================
// STEM Trait Implementations
// ============================================================================

/// Context membership checker — implements Membership (T1: MAPPING μ, but used for κ)
///
/// Checks: "Is this state a valid member of this parent context?"
/// This IS the context-sensitive check. A CFG doesn't have this.
pub struct ContextMembership {
    /// Rules: state_name → set of valid parent contexts
    rules: HashMap<String, HashSet<String>>,
}

impl Membership for ContextMembership {
    type Element = (String, Option<String>); // (state_name, parent_name)

    /// Is (state, parent) a valid membership?
    fn contains(&self, elem: &Self::Element) -> bool {
        let (state, parent) = elem;
        match self.rules.get(state) {
            Some(valid_parents) => {
                if valid_parents.is_empty() {
                    // No restrictions — valid anywhere
                    true
                } else {
                    match parent {
                        Some(p) => valid_parents.contains(p),
                        None => valid_parents.contains("_root_"),
                    }
                }
            }
            None => true, // No rule = no restriction
        }
    }

    fn cardinality(&self) -> usize {
        self.rules.len()
    }
}

impl Default for ContextMembership {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMembership {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Add a context rule: "state X can only appear inside parents Y, Z"
    pub fn add_rule(&mut self, state: &str, valid_parents: &[&str]) {
        self.rules.insert(
            state.to_string(),
            valid_parents.iter().map(|s| s.to_string()).collect(),
        );
    }
}

/// Resource budget bounds — implements Bound (T1: BOUNDARY ∂)
///
/// Each state transition has a cost. The budget must stay within bounds.
/// This is a conservation law applied to the grammar.
pub struct ResourceBounds {
    costs: HashMap<String, f64>,
}

impl Bound for ResourceBounds {
    type Value = f64;

    fn upper_bound(&self) -> Option<f64> {
        Some(f64::INFINITY) // No upper limit on budget
    }

    fn lower_bound(&self) -> Option<f64> {
        Some(0.0) // Budget can't go negative
    }
}

impl Default for ResourceBounds {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceBounds {
    #[must_use]
    pub fn new() -> Self {
        Self {
            costs: HashMap::new(),
        }
    }

    pub fn set_cost(&mut self, state: &str, cost: f64) {
        self.costs.insert(state.to_string(), cost);
    }

    #[must_use]
    pub fn cost_of(&self, state: &str) -> f64 {
        self.costs.get(state).copied().unwrap_or(0.0)
    }
}

/// Invariant preservation — implements Preserve (T1: STATE ς + PERSISTENCE π)
///
/// Conservation law: certain quantities must be preserved across transitions.
/// Before → transition → After: Preserve::is_preserved must hold.
pub struct InvariantPreserver {
    /// Named invariants with their expected values
    invariants: HashMap<String, f64>,
}

impl Preserve for InvariantPreserver {
    type Conserved = HashMap<String, f64>;

    fn conserved(&self) -> HashMap<String, f64> {
        self.invariants.clone()
    }

    /// Check if all invariants are preserved within tolerance
    fn is_preserved(&self, before: &HashMap<String, f64>, tolerance: f64) -> bool {
        for (name, expected) in &self.invariants {
            match before.get(name) {
                Some(actual) => {
                    if (actual - expected).abs() > tolerance {
                        return false;
                    }
                }
                None => return false, // Missing invariant = violation
            }
        }
        true
    }
}

impl Default for InvariantPreserver {
    fn default() -> Self {
        Self::new()
    }
}

impl InvariantPreserver {
    #[must_use]
    pub fn new() -> Self {
        Self {
            invariants: HashMap::new(),
        }
    }

    pub fn set_invariant(&mut self, name: &str, value: f64) {
        self.invariants.insert(name.to_string(), value);
    }
}

// ============================================================================
// The Context-Sensitive Validator
// ============================================================================

/// Context-sensitive grammar validator for state machines.
///
/// Extends Experiment 2's parser with THREE new capabilities
/// (all powered by κ Comparison):
///
/// 1. **Context Membership**: States are only valid in certain parent contexts
/// 2. **Resource Bounds**: Transitions consume budget, must stay ≥ 0
/// 3. **Invariant Preservation**: Conserved quantities must not change
///
/// None of these are possible in a CFG. They all require checking
/// the CONTEXT (surrounding states, accumulated values) before allowing
/// a production.
pub struct ConservationValidator {
    membership: ContextMembership,
    bounds: ResourceBounds,
    preserver: InvariantPreserver,
}

impl ConservationValidator {
    #[must_use]
    pub fn new(
        membership: ContextMembership,
        bounds: ResourceBounds,
        preserver: InvariantPreserver,
    ) -> Self {
        Self {
            membership,
            bounds,
            preserver,
        }
    }

    /// Validate a state within its context
    /// This is the αAβ → αγβ check: does A (state) fit within α,β (context)?
    pub fn validate_in_context(
        &self,
        state: &str,
        ctx: &mut ValidationContext,
    ) -> ContextValidation {
        // Check 1: Context membership (κ — does this state belong here?)
        let parent = ctx.current_parent().map(|s| s.to_string());
        let elem = (state.to_string(), parent.clone());
        if !self.membership.contains(&elem) {
            let expected = self
                .membership
                .rules
                .get(state)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default();
            return ContextValidation::InvalidContext {
                state: state.to_string(),
                found_parent: parent,
                expected_parents: expected,
            };
        }

        // Check 2: Resource budget (∂ — within bounds?)
        let cost = self.bounds.cost_of(state);
        if ctx.resource_budget - cost < 0.0 {
            return ContextValidation::BudgetExceeded {
                state: state.to_string(),
                cost,
                remaining: ctx.resource_budget,
            };
        }
        ctx.resource_budget -= cost;

        // Check 3: Invariant preservation (π + ς — conserved?)
        if !self.preserver.is_preserved(&ctx.invariants, 0.001) {
            // Find which invariant is violated
            for (name, expected) in &self.preserver.invariants {
                if let Some(actual) = ctx.invariants.get(name) {
                    if (actual - expected).abs() > 0.001 {
                        return ContextValidation::InvariantViolation {
                            invariant: name.clone(),
                            expected: *expected,
                            actual: *actual,
                            tolerance: 0.001,
                        };
                    }
                }
            }
        }

        ContextValidation::Valid
    }
}

// ============================================================================
// Tests — Experimental Validation
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a case management validator:
    /// - "Escalated" only valid inside "Triage"
    /// - "Step1", "Step2" only valid inside "Processing"
    /// - Each state costs resource budget
    /// - Case count invariant must be preserved
    fn case_validator() -> ConservationValidator {
        let mut membership = ContextMembership::new();
        membership.add_rule("Escalated", &["Triage"]);
        membership.add_rule("Step1", &["Processing"]);
        membership.add_rule("Step2", &["Processing"]);
        // "New", "Triage", "Processing", "Done" valid anywhere

        let mut bounds = ResourceBounds::new();
        bounds.set_cost("New", 1.0);
        bounds.set_cost("Triage", 2.0);
        bounds.set_cost("Processing", 3.0);
        bounds.set_cost("Escalated", 5.0);
        bounds.set_cost("Done", 0.5);

        let mut preserver = InvariantPreserver::new();
        preserver.set_invariant("case_count", 1.0);

        ConservationValidator::new(membership, bounds, preserver)
    }

    #[test]
    fn test_valid_context_accepted() {
        let validator = case_validator();
        let mut ctx = ValidationContext::new(100.0);
        ctx.invariants.insert("case_count".to_string(), 1.0);

        // "Triage" is valid at root
        let result = validator.validate_in_context("Triage", &mut ctx);
        assert!(matches!(result, ContextValidation::Valid));
    }

    #[test]
    fn test_invalid_context_rejected() {
        let validator = case_validator();
        let mut ctx = ValidationContext::new(100.0);
        ctx.invariants.insert("case_count".to_string(), 1.0);

        // "Escalated" is NOT valid at root — only inside "Triage"
        let result = validator.validate_in_context("Escalated", &mut ctx);
        assert!(matches!(result, ContextValidation::InvalidContext { .. }));
    }

    #[test]
    fn test_valid_nested_context() {
        let validator = case_validator();
        let mut ctx = ValidationContext::new(100.0);
        ctx.invariants.insert("case_count".to_string(), 1.0);

        // Enter Triage context
        ctx.push_parent("Triage");

        // Now "Escalated" IS valid — we're inside Triage
        let result = validator.validate_in_context("Escalated", &mut ctx);
        assert!(matches!(result, ContextValidation::Valid));
    }

    #[test]
    fn test_budget_exhaustion() {
        let validator = case_validator();
        let mut ctx = ValidationContext::new(5.0); // Tight budget
        ctx.invariants.insert("case_count".to_string(), 1.0);

        // New costs 1.0, Triage costs 2.0 = 3.0 spent, 2.0 remaining
        let r1 = validator.validate_in_context("New", &mut ctx);
        assert!(matches!(r1, ContextValidation::Valid));
        let r2 = validator.validate_in_context("Triage", &mut ctx);
        assert!(matches!(r2, ContextValidation::Valid));

        // Escalated costs 5.0 but only 2.0 remaining — REJECTED
        ctx.push_parent("Triage");
        let r3 = validator.validate_in_context("Escalated", &mut ctx);
        assert!(matches!(r3, ContextValidation::BudgetExceeded { .. }));
    }

    #[test]
    fn test_invariant_violation() {
        let validator = case_validator();
        let mut ctx = ValidationContext::new(100.0);
        // Set wrong case count — invariant violation
        ctx.invariants.insert("case_count".to_string(), 2.0);

        let result = validator.validate_in_context("New", &mut ctx);
        assert!(matches!(
            result,
            ContextValidation::InvariantViolation { .. }
        ));
    }

    #[test]
    fn test_conservation_across_transitions() {
        let validator = case_validator();
        let mut ctx = ValidationContext::new(100.0);
        ctx.invariants.insert("case_count".to_string(), 1.0);

        // Walk through a valid path, checking conservation at each step
        let path = ["New", "Triage", "Processing", "Done"];
        for state in &path {
            let result = validator.validate_in_context(state, &mut ctx);
            assert!(
                matches!(result, ContextValidation::Valid),
                "Failed at state: {state}"
            );
        }

        // Budget should be: 100 - 1 - 2 - 3 - 0.5 = 93.5
        assert!((ctx.resource_budget - 93.5).abs() < 0.001);
    }

    #[test]
    fn test_context_depth_tracking() {
        let mut ctx = ValidationContext::new(100.0);
        assert_eq!(ctx.depth(), 0);

        ctx.push_parent("System");
        assert_eq!(ctx.depth(), 1);
        assert_eq!(ctx.current_parent(), Some("System"));

        ctx.push_parent("Active");
        assert_eq!(ctx.depth(), 2);
        assert_eq!(ctx.current_parent(), Some("Active"));

        ctx.pop_parent();
        assert_eq!(ctx.depth(), 1);
        assert_eq!(ctx.current_parent(), Some("System"));
    }

    // ========================================================================
    // Key Experimental Observation
    // ========================================================================

    #[test]
    fn test_limitation_cannot_compute() {
        // This is the TYPE-1 LIMITATION.
        // We can check "is this state valid in this context?" (κ)
        // but we CANNOT compute new states or transform values.
        //
        // For example, we can't say "transition to state X AND update
        // the belief probability from 0.3 to 0.7 based on evidence."
        //
        // That requires ∃ (Existence) — the ability to CREATE new values
        // and compute arbitrary functions. That's Experiment 4.
        //
        // A context-sensitive grammar can check constraints.
        // An unrestricted grammar can compute anything.
    }
}
