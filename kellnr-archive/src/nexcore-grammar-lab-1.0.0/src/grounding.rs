//! # GroundsTo implementations for nexcore-grammar-lab types
//!
//! Connects formal grammar experiment types to the Lex Primitiva type system.
//!
//! ## Recursion (rho) Focus
//!
//! Grammar-lab proves that grammar IS math by climbing the Chomsky hierarchy.
//! Each experiment adds one T1 primitive, gaining one Chomsky level:
//! - Exp1 (Type-3): sigma + Sigma -- finite automaton
//! - Exp2 (Type-2): + rho -- pushdown automaton (nesting)
//! - Exp3 (Type-1): + kappa -- linear bounded automaton (context guards)
//! - Exp4 (Type-0): + exists -- Turing machine (value creation)

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::exp1_regular::{
    Recognition, StateSymbol, TokenCategory, TokenClassifier, TransitionRecognizer,
    TransitionRelation, TransitionRule, ValidPath,
};
use crate::exp2_context_free::{GrammarInference, ParseResult, StateComposer, StateNode};
use crate::exp3_context_sensitive::{
    ConservationValidator, ContextMembership, ContextRule, ContextValidation, Invariant,
    InvariantPreserver, ResourceBounds, ValidationContext,
};
use crate::exp4_unrestricted::{
    ActionExecutor, ActionOutcome, BayesianUpdater, Belief, Conclusion, Effect, Evidence, Premise,
    StateMachineInterpreter, TransitionAction, TransitionProver,
};

// ===========================================================================
// Experiment 1: Type-3 Regular (sigma + Sigma)
// ===========================================================================

/// StateSymbol: T2-P (Sigma + sigma), dominant Sigma
///
/// Atomic state identifier -- a named element in an alternation.
/// Sum-dominant: it IS one variant from a set of possible states.
/// Sequence is secondary (states are traversed in order).
impl GroundsTo for StateSymbol {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Sigma -- one of many possible states
            LexPrimitiva::Sequence, // sigma -- part of ordered traversal
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// TransitionRule: T2-P (sigma + Sigma), dominant sigma
///
/// A rule declaring a legal transition from one state to another.
/// Sequence-dominant: the rule defines an ordered pair (from -> to).
impl GroundsTo for TransitionRule {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered from -> to
            LexPrimitiva::Sum,      // Sigma -- state selection
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// Recognition: T2-P (kappa + Sigma), dominant kappa
///
/// Result of recognizing a sequence: Accepted, Rejected, or Partial.
/// Comparison-dominant: it compares the input against the grammar.
impl GroundsTo for Recognition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- acceptance comparison
            LexPrimitiva::Sum,        // Sigma -- result variant
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// TokenCategory: T1 (Sigma), pure sum
///
/// Classification of tokens: Identifier, Operator, Delimiter, Keyword.
impl GroundsTo for TokenCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// TokenClassifier: T2-P (Sigma + mu), dominant Sigma
///
/// Classifies input tokens by STEM Classify trait.
impl GroundsTo for TokenClassifier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,     // Sigma -- categorical classification
            LexPrimitiva::Mapping, // mu -- input -> category mapping
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// TransitionRelation: T2-P (sigma + varsigma), dominant sigma
///
/// Set of valid transitions forming the transition relation.
impl GroundsTo for TransitionRelation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered transition chain
            LexPrimitiva::State,    // varsigma -- relation state
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// ValidPath: T2-P (sigma + kappa), dominant sigma
///
/// A validated sequence of states forming a legal path.
impl GroundsTo for ValidPath {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- ordered path
            LexPrimitiva::Comparison, // kappa -- path validation
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// TransitionRecognizer: T2-P (sigma + Sigma), dominant sigma
///
/// Finite automaton that recognizes valid state transition sequences.
/// This is the Type-3 recognizer -- no stack, no recursion.
impl GroundsTo for TransitionRecognizer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- sequence recognition
            LexPrimitiva::Sum,      // Sigma -- state alternation
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ===========================================================================
// Experiment 2: Type-2 Context-Free (sigma + Sigma + rho)
// ===========================================================================

/// StateNode: T2-P (rho + sigma + Sigma), dominant rho
///
/// Recursive AST node with children. THIS is where rho lives.
/// A Type-3 grammar cannot have children. Adding rho enables nesting.
impl GroundsTo for StateNode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // rho -- recursive children
            LexPrimitiva::Sequence,  // sigma -- child ordering
            LexPrimitiva::Sum,       // Sigma -- node type alternation
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

/// ParseResult: T2-P (rho + kappa), dominant rho
///
/// Result of parsing nested state machine definitions.
/// Recursion-dominant: the parse tree is recursive.
impl GroundsTo for ParseResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // rho -- recursive parse tree
            LexPrimitiva::Comparison, // kappa -- parse success/failure
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

/// StateComposer: T2-P (rho + mu), dominant rho
///
/// Composes nested state machines using recursive descent.
impl GroundsTo for StateComposer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // rho -- recursive composition
            LexPrimitiva::Mapping,   // mu -- input -> tree mapping
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

/// GrammarInference: T2-P (rho + mu + kappa), dominant rho
///
/// Infers grammar rules from observed state machine traces.
impl GroundsTo for GrammarInference {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // rho -- recursive rule inference
            LexPrimitiva::Mapping,    // mu -- traces -> grammar rules
            LexPrimitiva::Comparison, // kappa -- rule matching
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.80)
    }
}

// ===========================================================================
// Experiment 3: Type-1 Context-Sensitive (sigma + Sigma + rho + kappa)
// ===========================================================================

/// ContextRule: T2-C (kappa + rho + Sigma + partial), dominant kappa
///
/// Context-dependent validation rule. The kappa primitive enables
/// checking context BEFORE allowing a transition.
impl GroundsTo for ContextRule {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- context comparison
            LexPrimitiva::Recursion,  // rho -- parent context chain
            LexPrimitiva::Sum,        // Sigma -- valid parent alternation
            LexPrimitiva::Boundary,   // partial -- context boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// Invariant: T2-P (kappa + partial), dominant kappa
///
/// A conservation law that must hold across transitions.
impl GroundsTo for Invariant {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- invariant verification
            LexPrimitiva::Boundary,   // partial -- conservation boundary
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// ValidationContext: T2-C (kappa + rho + varsigma + Sigma), dominant kappa
///
/// Accumulated context for context-sensitive validation.
impl GroundsTo for ValidationContext {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- context comparison
            LexPrimitiva::Recursion,  // rho -- parent stack
            LexPrimitiva::State,      // varsigma -- accumulated state
            LexPrimitiva::Sum,        // Sigma -- invariant set
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// ContextValidation: T2-P (kappa + Sigma), dominant kappa
///
/// Result of context-sensitive validation: Valid, InvalidContext, InvariantViolation.
impl GroundsTo for ContextValidation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- validation result
            LexPrimitiva::Sum,        // Sigma -- result variant
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// ContextMembership: T2-P (kappa + Sigma), dominant kappa
///
/// Tests membership in a context (STEM Membership trait).
impl GroundsTo for ContextMembership {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- membership test
            LexPrimitiva::Sum,        // Sigma -- valid contexts set
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// ResourceBounds: T2-P (partial + N), dominant partial
///
/// Bounds on resources (memory, recursion depth) for LBA simulation.
impl GroundsTo for ResourceBounds {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- resource boundaries
            LexPrimitiva::Quantity, // N -- numeric limits
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// InvariantPreserver: T2-P (kappa + partial + varsigma), dominant kappa
///
/// Validates that transitions preserve conservation invariants.
impl GroundsTo for InvariantPreserver {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- invariant check
            LexPrimitiva::Boundary,   // partial -- conservation boundary
            LexPrimitiva::State,      // varsigma -- invariant state
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// ConservationValidator: T2-C (kappa + rho + sigma + partial), dominant kappa
///
/// Full Type-1 context-sensitive validator with conservation laws.
impl GroundsTo for ConservationValidator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- context-dependent validation
            LexPrimitiva::Recursion,  // rho -- recursive context chain
            LexPrimitiva::Sequence,   // sigma -- transition sequence
            LexPrimitiva::Boundary,   // partial -- conservation boundaries
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ===========================================================================
// Experiment 4: Type-0 Unrestricted (sigma + Sigma + rho + kappa + exists)
// ===========================================================================

/// Belief: T2-C (exists + N + kappa + rho), dominant exists
///
/// Bayesian belief about system state. The exists primitive enables
/// creation of new values during execution.
impl GroundsTo for Belief {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // exists -- belief existence/absence
            LexPrimitiva::Quantity,   // N -- probability, confidence
            LexPrimitiva::Comparison, // kappa -- prior vs posterior
            LexPrimitiva::Recursion,  // rho -- recursive belief updating
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
    }
}

/// Evidence: T2-P (exists + N), dominant exists
///
/// Observed evidence that updates beliefs.
impl GroundsTo for Evidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // exists -- evidence existence
            LexPrimitiva::Quantity,  // N -- likelihood ratio
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

/// TransitionAction: T2-P (causality + exists), dominant causality
///
/// An action that can be executed during a state transition.
impl GroundsTo for TransitionAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // causality -- action causes effect
            LexPrimitiva::Existence, // exists -- action creates outcome
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// Effect: T2-P (causality + Sigma), dominant causality
///
/// Observable effect of an action: CreateState, DestroyState, ModifyVariable.
impl GroundsTo for Effect {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // causality -- causal effect
            LexPrimitiva::Sum,       // Sigma -- effect type variant
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// ActionOutcome: T2-P (exists + causality + N), dominant exists
///
/// Result of executing an action, including created values.
impl GroundsTo for ActionOutcome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // exists -- outcome creates new values
            LexPrimitiva::Causality, // causality -- caused by action
            LexPrimitiva::Quantity,  // N -- confidence in outcome
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

/// Premise: T2-P (kappa + Sigma), dominant kappa
///
/// Logical premise for proof: StateIs, InvariantHolds, ValueEquals.
impl GroundsTo for Premise {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- premise evaluation
            LexPrimitiva::Sum,        // Sigma -- premise type variant
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Conclusion: T2-P (exists + kappa), dominant exists
///
/// Logical conclusion: Proven, Disproven, Unknown.
impl GroundsTo for Conclusion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // exists -- conclusion existence
            LexPrimitiva::Comparison, // kappa -- truth evaluation
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

/// BayesianUpdater: T2-P (exists + rho + N), dominant exists
///
/// Updates beliefs via Bayesian inference (Normalize trait).
impl GroundsTo for BayesianUpdater {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // exists -- creates posterior beliefs
            LexPrimitiva::Recursion, // rho -- iterative updating
            LexPrimitiva::Quantity,  // N -- probability computation
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
    }
}

/// ActionExecutor: T2-P (causality + exists + sigma), dominant causality
///
/// Executes actions and records outcomes (Experiment trait).
impl GroundsTo for ActionExecutor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // causality -- action -> outcome
            LexPrimitiva::Existence, // exists -- creates outcomes
            LexPrimitiva::Sequence,  // sigma -- action sequence
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// TransitionProver: T2-P (kappa + exists + rho), dominant kappa
///
/// Proves logical properties about transitions (Prove trait).
impl GroundsTo for TransitionProver {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- proof verification
            LexPrimitiva::Existence,  // exists -- proof existence
            LexPrimitiva::Recursion,  // rho -- recursive proof steps
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// StateMachineInterpreter: T3 (sigma + Sigma + rho + kappa + exists + causality), dominant rho
///
/// Full Type-0 interpreter composing all five generator primitives.
/// This is a Turing machine -- unrestricted computation.
/// Recursion-dominant: the interpreter loop is recursive interpretation.
impl GroundsTo for StateMachineInterpreter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // rho -- recursive interpretation
            LexPrimitiva::Sequence,   // sigma -- execution sequence
            LexPrimitiva::Sum,        // Sigma -- state alternation
            LexPrimitiva::Comparison, // kappa -- context guards
            LexPrimitiva::Existence,  // exists -- value creation
            LexPrimitiva::Causality,  // causality -- action effects
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // Experiment 1: Type-3 types should be T2-P (2-3 primitives)
    #[test]
    fn exp1_types_are_t2p_or_t1() {
        assert_eq!(TokenCategory::tier(), Tier::T1Universal);
        assert_eq!(StateSymbol::tier(), Tier::T2Primitive);
        assert_eq!(TransitionRule::tier(), Tier::T2Primitive);
        assert_eq!(TransitionRecognizer::tier(), Tier::T2Primitive);
    }

    // Experiment 2: Type-2 types feature rho
    #[test]
    fn exp2_types_have_recursion() {
        let comp = StateNode::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
    }

    // Experiment 3: Type-1 types feature kappa
    #[test]
    fn exp3_types_have_comparison() {
        let comp = ContextRule::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn conservation_validator_is_t2c() {
        assert_eq!(ConservationValidator::tier(), Tier::T2Composite);
    }

    // Experiment 4: Type-0 types feature exists
    #[test]
    fn exp4_types_have_existence() {
        let comp = Belief::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
    }

    #[test]
    fn interpreter_is_t3() {
        assert_eq!(StateMachineInterpreter::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            StateMachineInterpreter::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
    }

    #[test]
    fn chomsky_hierarchy_primitive_count_increases() {
        // Type-3: 2 primitives
        let exp1 = TransitionRecognizer::primitive_composition().unique().len();
        // Type-2: 3 primitives
        let exp2 = StateNode::primitive_composition().unique().len();
        // Type-1: 4 primitives
        let exp3 = ConservationValidator::primitive_composition()
            .unique()
            .len();
        // Type-0: 6 primitives
        let exp4 = StateMachineInterpreter::primitive_composition()
            .unique()
            .len();

        assert!(
            exp2 > exp1,
            "Type-2 should have more primitives than Type-3"
        );
        assert!(
            exp3 > exp2,
            "Type-1 should have more primitives than Type-2"
        );
        assert!(
            exp4 > exp3,
            "Type-0 should have more primitives than Type-1"
        );
    }
}
