// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Runtime Values
//!
//! Values with primitive compositions for runtime tracking.
//!
//! ## Mathematical Foundation
//!
//! Every value carries its `PrimitiveComposition`, enabling:
//! - Runtime tier classification
//! - Transfer confidence computation
//! - Grounding verification to {0, 1}
//!
//! ## Tier: T2-C (Σ + ς + σ)

use crate::ast::{Block, Param};
use nexcore_lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition, Tier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A Prima runtime value with composition tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    /// The actual data variant.
    pub data: ValueData,
    /// The primitive composition of the value.
    pub composition: PrimitiveComposition,
}

/// Value data variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueData {
    /// ∅ — Void/unit
    Void,
    /// N — Integer quantity
    Int(i64),
    /// N — Float quantity
    Float(f64),
    /// Σ(0, 1) — Boolean
    Bool(bool),
    /// σ[N] — String (sequence of char codes)
    String(String),
    /// σ[T] — Sequence
    Sequence(Vec<Value>),
    /// μ[K → V] — Mapping
    Mapping(HashMap<String, Value>),
    /// → — Function
    #[serde(skip)]
    Function(FnValue),
    /// Built-in function
    #[serde(skip)]
    Builtin(String),
    /// λ — Symbol (interned identifier for homoiconicity)
    /// `:name` syntax, evaluates to itself
    Symbol(String),
    /// ρ — Quoted expression (AST as data for homoiconicity)
    /// `'expr` syntax, contains the unevaluated AST
    #[serde(skip)]
    Quoted(Box<crate::ast::Expr>),
}

/// Function value.
#[derive(Debug, Clone)]
pub struct FnValue {
    /// The function name.
    pub name: String,
    /// The function parameters.
    pub params: Vec<Param>,
    /// The function body.
    pub body: Block,
    /// The closure (captured environment).
    pub closure: HashMap<String, Value>,
}

impl Value {
    /// Create void value.
    #[must_use]
    pub fn void() -> Self {
        Self {
            data: ValueData::Void,
            composition: PrimitiveComposition::new(vec![LexPrimitiva::Void]),
        }
    }

    /// Create integer value.
    #[must_use]
    pub fn int(n: i64) -> Self {
        Self {
            data: ValueData::Int(n),
            composition: PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
        }
    }

    /// Create float value.
    #[must_use]
    pub fn float(n: f64) -> Self {
        Self {
            data: ValueData::Float(n),
            composition: PrimitiveComposition::new(vec![LexPrimitiva::Quantity]),
        }
    }

    /// Create boolean value.
    #[must_use]
    pub fn bool(b: bool) -> Self {
        Self {
            data: ValueData::Bool(b),
            composition: PrimitiveComposition::new(vec![LexPrimitiva::Sum]),
        }
    }

    /// Create string value.
    #[must_use]
    pub fn string(s: impl Into<String>) -> Self {
        Self {
            data: ValueData::String(s.into()),
            composition: PrimitiveComposition::new(vec![
                LexPrimitiva::Sequence,
                LexPrimitiva::Quantity,
            ]),
        }
    }

    /// Create sequence value.
    #[must_use]
    pub fn sequence(elements: Vec<Value>) -> Self {
        let mut prims = vec![LexPrimitiva::Sequence];
        for elem in &elements {
            for p in &elem.composition.primitives {
                if !prims.contains(p) {
                    prims.push(*p);
                }
            }
        }
        Self {
            data: ValueData::Sequence(elements),
            composition: PrimitiveComposition::new(prims),
        }
    }

    /// Create mapping value.
    #[must_use]
    pub fn mapping(pairs: HashMap<String, Value>) -> Self {
        let mut prims = vec![LexPrimitiva::Mapping];
        for v in pairs.values() {
            for p in &v.composition.primitives {
                if !prims.contains(p) {
                    prims.push(*p);
                }
            }
        }
        Self {
            data: ValueData::Mapping(pairs),
            composition: PrimitiveComposition::new(prims),
        }
    }

    /// Create function value.
    #[must_use]
    pub fn function(
        name: String,
        params: Vec<Param>,
        body: Block,
        closure: HashMap<String, Value>,
    ) -> Self {
        Self {
            data: ValueData::Function(FnValue {
                name,
                params,
                body,
                closure,
            }),
            composition: PrimitiveComposition::new(vec![LexPrimitiva::Causality]),
        }
    }

    /// Create builtin function value.
    #[must_use]
    pub fn builtin(name: impl Into<String>) -> Self {
        Self {
            data: ValueData::Builtin(name.into()),
            composition: PrimitiveComposition::new(vec![LexPrimitiva::Causality]),
        }
    }

    /// Create symbol value — λ (Location) primitive.
    /// Symbols evaluate to themselves, enabling reflection.
    #[must_use]
    pub fn symbol(name: impl Into<String>) -> Self {
        Self {
            data: ValueData::Symbol(name.into()),
            composition: PrimitiveComposition::new(vec![LexPrimitiva::Location]),
        }
    }

    /// Create quoted value — ρ (Recursion) primitive.
    /// Quoted expressions are AST nodes as data (homoiconicity).
    #[must_use]
    pub fn quoted(expr: crate::ast::Expr) -> Self {
        Self {
            data: ValueData::Quoted(Box::new(expr)),
            composition: PrimitiveComposition::new(vec![
                LexPrimitiva::Recursion, // ρ — self-reference
                LexPrimitiva::Sequence,  // σ — AST is a tree (nested sequences)
            ]),
        }
    }

    /// Get the tier of this value.
    #[must_use]
    pub fn tier(&self) -> Tier {
        Tier::classify(&self.composition)
    }

    /// Check if value is truthy.
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match &self.data {
            ValueData::Void => false,
            ValueData::Bool(b) => *b,
            ValueData::Int(n) => *n != 0,
            ValueData::Float(n) => *n != 0.0,
            ValueData::String(s) => !s.is_empty(),
            ValueData::Sequence(v) => !v.is_empty(),
            ValueData::Mapping(m) => !m.is_empty(),
            ValueData::Function(_) | ValueData::Builtin(_) => true,
            // Symbols are always truthy (they exist as references)
            ValueData::Symbol(_) => true,
            // Quoted expressions are always truthy (they hold AST data)
            ValueData::Quoted(_) => true,
        }
    }

    /// Get transfer confidence.
    #[must_use]
    pub fn transfer_confidence(&self) -> f64 {
        self.tier().transfer_multiplier() * self.composition.confidence
    }

    /// Check if this is the 0 constant.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        match &self.data {
            ValueData::Int(0) => true,
            ValueData::Float(n) if *n == 0.0 => true,
            _ => false,
        }
    }

    /// Check if this is the 1 constant.
    #[must_use]
    pub fn is_one(&self) -> bool {
        match &self.data {
            ValueData::Int(1) => true,
            ValueData::Float(n) if (*n - 1.0).abs() < f64::EPSILON => true,
            _ => false,
        }
    }

    /// Get grounding constants this value reaches.
    #[must_use]
    pub fn grounding_constants(&self) -> Vec<&'static str> {
        // All values ultimately ground to {0, 1}
        vec!["0", "1"]
    }
}

/// Format a sequence as σ[...].
fn fmt_sequence(v: &[Value], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "σ[")?;
    for (i, elem) in v.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", elem)?;
    }
    write!(f, "]")
}

/// Format a mapping as μ(...).
fn fmt_mapping(m: &HashMap<String, Value>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "μ(")?;
    for (i, (k, v)) in m.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "{} → {}", k, v)?;
    }
    write!(f, ")")
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            ValueData::Void => write!(f, "∅"),
            ValueData::Int(n) => write!(f, "{}", n),
            ValueData::Float(n) => write!(f, "{}", n),
            ValueData::Bool(b) => write!(f, "{}", if *b { "⊤" } else { "⊥" }),
            ValueData::String(s) => write!(f, "\"{}\"", s),
            ValueData::Sequence(v) => fmt_sequence(v, f),
            ValueData::Mapping(m) => fmt_mapping(m, f),
            ValueData::Function(fv) => write!(f, "fn {}(...) → ...", fv.name),
            ValueData::Builtin(name) => write!(f, "<builtin:{}>", name),
            ValueData::Symbol(name) => write!(f, ":{}", name),
            ValueData::Quoted(expr) => write!(f, "'{:?}", expr),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (&self.data, &other.data) {
            (ValueData::Void, ValueData::Void) => true,
            (ValueData::Int(a), ValueData::Int(b)) => a == b,
            (ValueData::Float(a), ValueData::Float(b)) => (a - b).abs() < f64::EPSILON,
            (ValueData::Bool(a), ValueData::Bool(b)) => a == b,
            (ValueData::String(a), ValueData::String(b)) => a == b,
            (ValueData::Sequence(a), ValueData::Sequence(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_constants() {
        let zero = Value::int(0);
        let one = Value::int(1);
        assert!(zero.is_zero());
        assert!(one.is_one());
        assert!(zero.grounding_constants().contains(&"0"));
    }

    #[test]
    fn test_value_tiers() {
        let int = Value::int(42);
        assert_eq!(int.tier(), Tier::T1Universal);

        let string = Value::string("hello");
        assert_eq!(string.tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_truthiness() {
        assert!(!Value::void().is_truthy());
        assert!(Value::bool(true).is_truthy());
        assert!(!Value::bool(false).is_truthy());
        assert!(Value::int(1).is_truthy());
        assert!(!Value::int(0).is_truthy());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Value::void()), "∅");
        assert_eq!(format!("{}", Value::bool(true)), "⊤");
        assert_eq!(format!("{}", Value::bool(false)), "⊥");
        assert_eq!(format!("{}", Value::int(42)), "42");
    }

    #[test]
    fn test_sequence_composition() {
        let seq = Value::sequence(vec![Value::int(1), Value::int(2)]);
        assert!(seq.composition.primitives.contains(&LexPrimitiva::Sequence));
        assert!(seq.composition.primitives.contains(&LexPrimitiva::Quantity));
    }
}
