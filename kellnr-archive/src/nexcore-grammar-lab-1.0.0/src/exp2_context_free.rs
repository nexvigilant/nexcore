//! # Experiment 2: Type-2 Context-Free Grammar — Nested State Parser
//!
//! **Chomsky Level:** Type-2 (Context-Free)
//! **Primitives:** σ (Sequence) + Σ (Sum) + ρ (Recursion)
//! **STEM Traits:** Infer, Associate, Transit (from Exp1)
//!
//! ## What Changes From Experiment 1
//!
//! We add ONE primitive: ρ (Recursion).
//! This single addition gives us a pushdown automaton — a STACK.
//! With a stack we can match braces, nest states, build trees.
//!
//! ## Grammar (Context-Free)
//!
//! ```text
//! Machine  → StateDecl Machine | ε
//! StateDecl → StateName '{' Machine '}' | StateName
//! Machine  → StateDecl '→' Machine | StateDecl
//! ```
//!
//! This is context-free because productions have a single nonterminal
//! on the left side. The recursive `Machine → StateDecl '{' Machine '}'`
//! is what requires the stack.
//!
//! ## Practical Output
//!
//! `NestedStateParser` — parses hierarchical state machine definitions
//! like `Active { Processing { Step1 → Step2 } → Review } → Done`

#![allow(dead_code)]

use stem_core::Infer;
use stem_math::Associate;

use std::fmt;

// ============================================================================
// AST Types — The Parse Tree (T2-C: σ + Σ + ρ)
// ============================================================================

/// A state node in the AST — can contain nested child states
/// Tier: T2-C (σ Sequence + Σ Sum + ρ Recursion — recursive tree structure)
#[derive(Debug, Clone, PartialEq)]
pub struct StateNode {
    pub name: String,
    /// Children — THIS is where ρ (Recursion) lives.
    /// A Type-3 grammar cannot have children. Adding ρ enables nesting.
    pub children: Vec<StateNode>,
    /// Transitions to sibling states at the same level
    pub transitions_to: Vec<String>,
}

impl StateNode {
    #[must_use]
    pub fn leaf(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
            transitions_to: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_children(name: &str, children: Vec<StateNode>) -> Self {
        Self {
            name: name.to_string(),
            children,
            transitions_to: Vec::new(),
        }
    }

    /// Depth of nesting — measures how much ρ we're using
    #[must_use]
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self
                .children
                .iter()
                .map(StateNode::depth)
                .max()
                .unwrap_or(0)
        }
    }

    /// Total node count (recursive)
    #[must_use]
    pub fn node_count(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(StateNode::node_count)
            .sum::<usize>()
    }
}

impl fmt::Display for StateNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.children.is_empty() {
            write!(f, " {{ ")?;
            for (i, child) in self.children.iter().enumerate() {
                if i > 0 {
                    write!(f, " → ")?;
                }
                write!(f, "{child}")?;
            }
            write!(f, " }}")?;
        }
        Ok(())
    }
}

/// Parse result
/// Tier: T2-P (Σ Sum — success | failure alternation)
#[derive(Debug, Clone)]
pub enum ParseResult {
    Success(Vec<StateNode>),
    Error { position: usize, message: String },
}

// ============================================================================
// Token types for the CFG
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Arrow,
    LBrace,
    RBrace,
    End,
}

// ============================================================================
// STEM Trait Implementations
// ============================================================================

/// State composition — implements Associate (T1: RECURSION ρ)
///
/// Associativity: (A compose B) compose C = A compose (B compose C)
/// This is critical for grammar parsing — parsing should be unambiguous
/// regardless of grouping order.
pub struct StateComposer;

impl Associate for StateComposer {
    type Operand = StateNode;

    /// Compose two state nodes by making the second a child of the first
    fn op(&self, a: &StateNode, b: &StateNode) -> StateNode {
        let mut result = a.clone();
        result.children.push(b.clone());
        result
    }

    /// Associativity check: (a·b)·c has same structure as a·(b·c)
    /// For tree composition this checks structural equivalence
    fn is_associative(&self, a: &StateNode, b: &StateNode, c: &StateNode) -> bool
    where
        Self::Operand: PartialEq,
    {
        // Tree composition is NOT generally associative
        // (a compose b) compose c ≠ a compose (b compose c) in general
        // This is an important insight: grammar parse trees depend on derivation order!
        let ab_c = self.op(&self.op(a, b), c);
        let a_bc = self.op(a, &self.op(b, c));
        ab_c == a_bc
    }
}

/// Grammar inference — implements Infer (T1: RECURSION ρ)
///
/// This is the core insight: PARSING IS INFERENCE.
/// Pattern = grammar rules, Data = token stream, Prediction = parse tree.
pub struct GrammarInference {
    /// Known valid state names (vocabulary)
    vocabulary: Vec<String>,
}

impl Infer for GrammarInference {
    /// The grammar rules (pattern to match against)
    type Pattern = Vec<String>; // valid state names
    /// The input token stream
    type Data = String;
    /// The predicted parse tree
    type Prediction = ParseResult;

    /// Infer a parse tree from the input string using the grammar
    fn infer(&self, _pattern: &Self::Pattern, data: &Self::Data) -> Self::Prediction {
        let mut parser = RecursiveDescentParser::new(data);
        parser.parse_machine()
    }
}

impl GrammarInference {
    #[must_use]
    pub fn new(vocabulary: Vec<String>) -> Self {
        Self { vocabulary }
    }

    /// Parse input using grammar inference
    pub fn parse(&self, input: &str) -> ParseResult {
        self.infer(&self.vocabulary, &input.to_string())
    }
}

// ============================================================================
// Recursive Descent Parser (the ρ-powered engine)
// ============================================================================

/// A recursive descent parser — the simplest CFG recognizer.
///
/// The key difference from Experiment 1: this parser calls ITSELF.
/// `parse_state_decl` calls `parse_machine` which calls `parse_state_decl`.
/// This mutual recursion IS the ρ primitive in action.
struct RecursiveDescentParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl RecursiveDescentParser {
    fn new(input: &str) -> Self {
        Self {
            tokens: Self::tokenize(input),
            pos: 0,
        }
    }

    fn tokenize(input: &str) -> Vec<Token> {
        let normalized = input
            .replace("→", " → ")
            .replace("->", " -> ")
            .replace('{', " { ")
            .replace('}', " } ");

        let mut tokens = Vec::new();
        for tok in normalized.split_whitespace() {
            match tok {
                "->" | "→" => tokens.push(Token::Arrow),
                "{" => tokens.push(Token::LBrace),
                "}" => tokens.push(Token::RBrace),
                s => tokens.push(Token::Ident(s.to_string())),
            }
        }
        tokens.push(Token::End);
        tokens
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::End)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::End);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(format!(
                "Expected {expected:?}, got {tok:?} at position {}",
                self.pos - 1
            ))
        }
    }

    /// Parse a machine (list of state declarations connected by arrows)
    ///
    /// Grammar: Machine → StateDecl ('→' StateDecl)*
    fn parse_machine(&mut self) -> ParseResult {
        let mut nodes = Vec::new();

        match self.parse_state_decl() {
            Ok(node) => nodes.push(node),
            Err(msg) => {
                return ParseResult::Error {
                    position: self.pos,
                    message: msg,
                };
            }
        }

        // Continue parsing transitions: → StateDecl → StateDecl ...
        while matches!(self.peek(), Token::Arrow) {
            self.advance(); // consume arrow
            match self.parse_state_decl() {
                Ok(node) => {
                    // Record transition from previous node
                    if let Some(prev) = nodes.last_mut() {
                        prev.transitions_to.push(node.name.clone());
                    }
                    nodes.push(node);
                }
                Err(msg) => {
                    return ParseResult::Error {
                        position: self.pos,
                        message: msg,
                    };
                }
            }
        }

        ParseResult::Success(nodes)
    }

    /// Parse a state declaration (possibly with nested children)
    ///
    /// Grammar: StateDecl → Ident '{' Machine '}' | Ident
    ///
    /// THIS IS WHERE ρ LIVES. When we see '{', we RECURSE into parse_machine.
    /// The Experiment 1 recognizer could NEVER do this — it had no stack.
    fn parse_state_decl(&mut self) -> Result<StateNode, String> {
        // Expect an identifier
        let name = match self.advance() {
            Token::Ident(s) => s,
            other => {
                return Err(format!(
                    "Expected state name, got {other:?} at position {}",
                    self.pos - 1
                ));
            }
        };

        // Check for nested children
        if matches!(self.peek(), Token::LBrace) {
            self.advance(); // consume '{'

            // RECURSIVE CALL — this is the pushdown automaton's stack push
            let children = match self.parse_machine() {
                ParseResult::Success(nodes) => nodes,
                ParseResult::Error { position, message } => {
                    return Err(format!("In nested state '{name}': {message} at {position}"));
                }
            };

            // Expect closing brace — this is the stack pop
            if let Err(e) = self.expect(&Token::RBrace) {
                return Err(format!("In nested state '{name}': {e}"));
            }

            Ok(StateNode::with_children(&name, children))
        } else {
            Ok(StateNode::leaf(&name))
        }
    }
}

// ============================================================================
// Tests — Experimental Validation
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_sequence_still_works() {
        // Type-2 is a superset of Type-3 — flat sequences still parse
        let inference = GrammarInference::new(vec![]);
        let result = inference.parse("Idle → Active → Done");
        match result {
            ParseResult::Success(nodes) => {
                assert_eq!(nodes.len(), 3);
                assert_eq!(nodes[0].name, "Idle");
                assert_eq!(nodes[1].name, "Active");
                assert_eq!(nodes[2].name, "Done");
                // All leaf nodes — no recursion needed
                assert!(nodes.iter().all(|n| n.children.is_empty()));
            }
            ParseResult::Error { message, .. } => {
                assert!(false, "Unexpected error: {message}");
            }
        }
    }

    #[test]
    fn test_nested_states_parsed() {
        // THIS IS THE NEW CAPABILITY — ρ enables nesting
        let inference = GrammarInference::new(vec![]);
        let result = inference.parse("Active { Processing → Review } → Done");
        match result {
            ParseResult::Success(nodes) => {
                assert_eq!(nodes.len(), 2); // Active, Done
                assert_eq!(nodes[0].name, "Active");
                assert_eq!(nodes[0].children.len(), 2); // Processing, Review
                assert_eq!(nodes[0].children[0].name, "Processing");
                assert_eq!(nodes[0].children[1].name, "Review");
                assert_eq!(nodes[1].name, "Done");
            }
            ParseResult::Error { message, .. } => {
                assert!(false, "Unexpected error: {message}");
            }
        }
    }

    #[test]
    fn test_deeply_nested_states() {
        // Multiple levels of nesting — each level = one more stack frame
        let inference = GrammarInference::new(vec![]);
        let result =
            inference.parse("System { Active { Processing { Step1 → Step2 } → Review } → Idle }");
        match result {
            ParseResult::Success(nodes) => {
                assert_eq!(nodes.len(), 1);
                let system = &nodes[0];
                assert_eq!(system.name, "System");
                assert_eq!(system.depth(), 3); // 3 levels of nesting
                assert_eq!(system.node_count(), 7); // System > {Active > {Processing > {Step1, Step2}, Review}, Idle}
            }
            ParseResult::Error { message, .. } => {
                assert!(false, "Unexpected error: {message}");
            }
        }
    }

    #[test]
    fn test_unmatched_brace_error() {
        // Missing closing brace — the stack never pops
        let inference = GrammarInference::new(vec![]);
        let result = inference.parse("Active { Processing → Done");
        assert!(matches!(result, ParseResult::Error { .. }));
    }

    #[test]
    fn test_display_roundtrip() {
        // A parsed tree can be displayed back to (roughly) its original form
        let node = StateNode::with_children(
            "Active",
            vec![StateNode::leaf("Processing"), StateNode::leaf("Review")],
        );
        let display = format!("{node}");
        assert_eq!(display, "Active { Processing → Review }");
    }

    #[test]
    fn test_associativity_of_composition() {
        // CRITICAL INSIGHT: tree composition is NOT associative
        // This is why grammar derivation order matters!
        let composer = StateComposer;
        let a = StateNode::leaf("A");
        let b = StateNode::leaf("B");
        let c = StateNode::leaf("C");

        // (A compose B) compose C ≠ A compose (B compose C)
        // Because: first gives A{B, C}, second gives A{B{C}}
        assert!(!composer.is_associative(&a, &b, &c));
        // This non-associativity is WHY we need proper parsing — not just folding
    }

    #[test]
    fn test_depth_measures_recursion() {
        let flat = StateNode::leaf("Flat");
        assert_eq!(flat.depth(), 0); // No recursion

        let one_deep = StateNode::with_children("A", vec![StateNode::leaf("B")]);
        assert_eq!(one_deep.depth(), 1); // One level of ρ

        let two_deep = StateNode::with_children(
            "A",
            vec![StateNode::with_children("B", vec![StateNode::leaf("C")])],
        );
        assert_eq!(two_deep.depth(), 2); // Two levels of ρ
    }

    // ========================================================================
    // Key Experimental Observation
    // ========================================================================

    #[test]
    fn test_limitation_cannot_use_context() {
        // This is the TYPE-2 LIMITATION.
        // We can parse "Active { Processing → Review } → Done"
        // but we CANNOT enforce: "Processing is only valid inside Active"
        //
        // That requires κ (Comparison) — checking the context around a symbol.
        // Context-sensitive grammar: αAβ → αγβ (the α and β MATTER)
        //
        // Our CFG treats all state names as equally valid anywhere.
        // Experiment 3 adds the context-checking capability.
        let inference = GrammarInference::new(vec![]);
        // This parses fine even though "Escalated" might only be valid
        // inside a "Triage" parent — our grammar doesn't know that
        let result = inference.parse("Done { Escalated → Processing }");
        assert!(matches!(result, ParseResult::Success(_)));
        // A context-sensitive grammar would reject this!
    }
}
