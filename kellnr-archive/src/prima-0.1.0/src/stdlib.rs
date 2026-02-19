// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Standard Library
//!
//! Built-in functions for Prima programs.
//!
//! ## Philosophy
//!
//! The standard library provides essential operations:
//! - **Sequence** (σ): map, filter, fold, concat
//! - **String** (σ[char]): split, join, chars, upper, lower
//! - **Numeric** (N): abs, min, max
//! - **I/O** (π): print, println, read
//! - **Type** (κ): typeof, is_int, is_seq, etc.
//!
//! ## Tier: T2-C (μ + σ + π + N + ∂)
//!
//! ## Symbol Mapping (Compressed Builtins)
//!
//! | Symbol | Name | Grounding |
//! |--------|------|-----------|
//! | Φ | map | μ + σ |
//! | Ψ | filter | ∂ + σ |
//! | Ω | fold | ρ + σ |
//! | # | len | σ → N |
//! | ↑ | head | σ → ∅∣T |
//! | ↓ | tail | σ → σ |
//! | ⊕ | push | σ × T → σ |
//! | ⊖ | pop | σ → (σ, T) |
//! | ⊙ | concat | σ × σ → σ |
//! | ω | print | π |
//! | ωn | println | π |

use crate::error::{PrimaError, PrimaResult};
use crate::value::{Value, ValueData};
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition, Tier};
use std::collections::HashMap;

/// Set the current entropy value (for VM synchronization)
pub fn set_entropy(val: f64) {
    crate::builtins::set_entropy(val);
}

// ═══════════════════════════════════════════════════════════════════════════
// STDLIB FUNCTION — μ (Mapping from inputs to output)
// ═══════════════════════════════════════════════════════════════════════════

/// A standard library function.
///
/// ## Tier: T2-P (μ + →)
#[derive(Debug, Clone)]
pub struct StdlibFn {
    /// Function name.
    pub name: String,
    /// Symbol (compressed name).
    pub symbol: Option<String>,
    /// Number of required parameters.
    pub arity: usize,
    /// Documentation.
    pub doc: String,
    /// Primitive composition.
    pub composition: PrimitiveComposition,
    /// The native implementation.
    pub kind: StdlibKind,
}

/// Kind of stdlib function (for dispatch).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibKind {
    // Sequence operations
    Len,
    Head,
    Tail,
    Push,
    Pop,
    Concat,
    Range,
    Rev,
    // Higher-order
    Map,
    Filter,
    Fold,
    Any,
    All,
    Find,
    Zip,
    // String operations
    Chars,
    Split,
    Join,
    Upper,
    Lower,
    Trim,
    Replace,
    // Numeric operations
    Abs,
    Min,
    Max,
    Sqrt,
    Pow,
    Floor,
    Ceil,
    Round,
    Sin,
    Cos,
    // Sequence aggregates
    Sum,
    Product,
    Sort,
    Enumerate,
    Flatten,
    // I/O operations
    Print,
    Println,
    // Type operations
    TypeOf,
    IsInt,
    IsFloat,
    IsString,
    IsSeq,
    IsBool,
    // Grounding operations
    PrimitiveTier,
    PrimitiveComposition,
    PrimitiveConstants,
    TransferConfidence,
    Entropy,
    Invert,
    // Conversion operations
    ToString,
    ToInt,
    ToFloat,
    // Assertion
    Assert,
    Contains,
    // System operations (hook support)
    Stdin,
    Readline,
    Exit,
    Env,
    Args,
    JsonParse,
}

impl StdlibFn {
    /// Create a new stdlib function.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        symbol: Option<&str>,
        arity: usize,
        doc: impl Into<String>,
        composition: PrimitiveComposition,
        kind: StdlibKind,
    ) -> Self {
        Self {
            name: name.into(),
            symbol: symbol.map(String::from),
            arity,
            doc: doc.into(),
            composition,
            kind,
        }
    }

    /// Get the tier of this function.
    #[must_use]
    pub fn tier(&self) -> Tier {
        Tier::classify(&self.composition)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STDLIB — σ[StdlibFn] (Collection of functions)
// ═══════════════════════════════════════════════════════════════════════════

/// The Prima standard library.
///
/// ## Tier: T2-C (σ + μ)
#[derive(Debug)]
pub struct Stdlib {
    /// Functions by name.
    functions: HashMap<String, StdlibFn>,
    /// Symbols to names mapping.
    symbols: HashMap<String, String>,
}

impl Default for Stdlib {
    fn default() -> Self {
        Self::new()
    }
}

impl Stdlib {
    /// Create the standard library with all built-in functions.
    #[must_use]
    pub fn new() -> Self {
        let mut stdlib = Self {
            functions: HashMap::new(),
            symbols: HashMap::new(),
        };

        stdlib.register_sequence_ops();
        stdlib.register_higher_order();
        stdlib.register_string_ops();
        stdlib.register_numeric_ops();
        stdlib.register_io_ops();
        stdlib.register_type_ops();
        stdlib.register_grounding_ops();
        stdlib.register_assertion_ops();
        stdlib.register_convert_ops();
        stdlib.register_system_ops();

        stdlib
    }

    /// Register a function.
    fn register(&mut self, func: StdlibFn) {
        if let Some(ref sym) = func.symbol {
            self.symbols.insert(sym.clone(), func.name.clone());
        }
        self.functions.insert(func.name.clone(), func);
    }

    /// Get a function by name or symbol.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&StdlibFn> {
        if let Some(func) = self.functions.get(name) {
            return Some(func);
        }
        // Try symbol lookup
        if let Some(real_name) = self.symbols.get(name) {
            return self.functions.get(real_name);
        }
        None
    }

    /// Check if a function exists.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Get all function names.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }

    /// Get all symbols.
    #[must_use]
    pub fn all_symbols(&self) -> Vec<(&str, &str)> {
        self.symbols
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Registration helpers
    // ─────────────────────────────────────────────────────────────────────

    fn register_sequence_ops(&mut self) {
        use LexPrimitiva::{Quantity, Sequence, Void};

        self.register(StdlibFn::new(
            "len",
            Some("#"),
            1,
            "Get length of sequence",
            PrimitiveComposition::new(vec![Sequence, Quantity]),
            StdlibKind::Len,
        ));

        self.register(StdlibFn::new(
            "head",
            Some("↑"),
            1,
            "Get first element of sequence",
            PrimitiveComposition::new(vec![Sequence, Void]),
            StdlibKind::Head,
        ));

        self.register(StdlibFn::new(
            "tail",
            Some("↓"),
            1,
            "Get sequence without first element",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Tail,
        ));

        self.register(StdlibFn::new(
            "push",
            Some("⊕"),
            2,
            "Append element to sequence",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Push,
        ));

        self.register(StdlibFn::new(
            "pop",
            Some("⊖"),
            1,
            "Remove and return last element",
            PrimitiveComposition::new(vec![Sequence, Void]),
            StdlibKind::Pop,
        ));

        self.register(StdlibFn::new(
            "concat",
            Some("⊙"),
            2,
            "Concatenate two sequences",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Concat,
        ));

        self.register(StdlibFn::new(
            "range",
            Some("‥"),
            2,
            "Generate range [start, end)",
            PrimitiveComposition::new(vec![Sequence, Quantity]),
            StdlibKind::Range,
        ));

        self.register(StdlibFn::new(
            "rev",
            None,
            1,
            "Reverse a sequence",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Rev,
        ));
    }

    fn register_higher_order(&mut self) {
        use LexPrimitiva::{Boundary, Mapping, Recursion, Sequence};

        self.register(StdlibFn::new(
            "map",
            Some("Φ"),
            2,
            "Transform each element",
            PrimitiveComposition::new(vec![Mapping, Sequence]),
            StdlibKind::Map,
        ));

        self.register(StdlibFn::new(
            "filter",
            Some("Ψ"),
            2,
            "Keep elements matching predicate",
            PrimitiveComposition::new(vec![Boundary, Sequence]),
            StdlibKind::Filter,
        ));

        self.register(StdlibFn::new(
            "fold",
            Some("Ω"),
            3,
            "Reduce sequence to single value",
            PrimitiveComposition::new(vec![Recursion, Sequence, Mapping]),
            StdlibKind::Fold,
        ));

        self.register(StdlibFn::new(
            "any",
            Some("∃?"),
            2,
            "True if any element matches",
            PrimitiveComposition::new(vec![Sequence, Boundary]),
            StdlibKind::Any,
        ));

        self.register(StdlibFn::new(
            "all",
            Some("∀?"),
            2,
            "True if all elements match",
            PrimitiveComposition::new(vec![Sequence, Boundary]),
            StdlibKind::All,
        ));

        self.register(StdlibFn::new(
            "find",
            Some("⊃"),
            2,
            "Find first matching element",
            PrimitiveComposition::new(vec![Sequence, Boundary]),
            StdlibKind::Find,
        ));

        self.register(StdlibFn::new(
            "zip",
            Some("⊠"),
            2,
            "Combine two sequences pairwise",
            PrimitiveComposition::new(vec![Sequence, Mapping]),
            StdlibKind::Zip,
        ));
    }

    fn register_string_ops(&mut self) {
        use LexPrimitiva::{Mapping, Sequence};

        self.register(StdlibFn::new(
            "chars",
            Some("χ"),
            1,
            "Convert string to character sequence",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Chars,
        ));

        self.register(StdlibFn::new(
            "split",
            Some("⊘"),
            2,
            "Split string by delimiter",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Split,
        ));

        self.register(StdlibFn::new(
            "join",
            Some("⊗"),
            2,
            "Join sequence with delimiter",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Join,
        ));

        self.register(StdlibFn::new(
            "upper",
            Some("⇑"),
            1,
            "Convert to uppercase",
            PrimitiveComposition::new(vec![Mapping]),
            StdlibKind::Upper,
        ));

        self.register(StdlibFn::new(
            "lower",
            Some("⇓"),
            1,
            "Convert to lowercase",
            PrimitiveComposition::new(vec![Mapping]),
            StdlibKind::Lower,
        ));

        self.register(StdlibFn::new(
            "trim",
            Some("⊢"),
            1,
            "Remove whitespace from ends",
            PrimitiveComposition::new(vec![Mapping]),
            StdlibKind::Trim,
        ));

        self.register(StdlibFn::new(
            "replace",
            Some("↔"),
            3,
            "Replace substring",
            PrimitiveComposition::new(vec![Mapping]),
            StdlibKind::Replace,
        ));
    }

    fn register_numeric_ops(&mut self) {
        use LexPrimitiva::{Comparison, Quantity};

        self.register(StdlibFn::new(
            "abs",
            Some("±"),
            1,
            "Absolute value",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Abs,
        ));

        self.register(StdlibFn::new(
            "min",
            Some("⌊"),
            2,
            "Minimum of two values",
            PrimitiveComposition::new(vec![Comparison, Quantity]),
            StdlibKind::Min,
        ));

        self.register(StdlibFn::new(
            "max",
            Some("⌈"),
            2,
            "Maximum of two values",
            PrimitiveComposition::new(vec![Comparison, Quantity]),
            StdlibKind::Max,
        ));

        self.register(StdlibFn::new(
            "sqrt",
            Some("√"),
            1,
            "Square root",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Sqrt,
        ));

        self.register(StdlibFn::new(
            "pow",
            Some("^"),
            2,
            "Power (base^exponent)",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Pow,
        ));

        self.register(StdlibFn::new(
            "floor",
            None,
            1,
            "Round down to integer",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Floor,
        ));

        self.register(StdlibFn::new(
            "ceil",
            None,
            1,
            "Round up to integer",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Ceil,
        ));

        self.register(StdlibFn::new(
            "round",
            None,
            1,
            "Round to nearest integer",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Round,
        ));

        self.register(StdlibFn::new(
            "sin",
            None,
            1,
            "Sine (radians)",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Sin,
        ));

        self.register(StdlibFn::new(
            "cos",
            None,
            1,
            "Cosine (radians)",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Cos,
        ));

        self.register(StdlibFn::new(
            "sum",
            Some("Σ+"),
            1,
            "Sum of sequence elements",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Sum,
        ));

        self.register(StdlibFn::new(
            "product",
            Some("Π"),
            1,
            "Product of sequence elements",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Product,
        ));

        self.register(StdlibFn::new(
            "sort",
            Some("⊳"),
            1,
            "Sort sequence",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Sort,
        ));

        self.register(StdlibFn::new(
            "enumerate",
            Some("№"),
            1,
            "Pairs elements with indices",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Enumerate,
        ));

        self.register(StdlibFn::new(
            "flatten",
            Some("⊔"),
            1,
            "Flatten nested sequences",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Flatten,
        ));
    }

    fn register_io_ops(&mut self) {
        use LexPrimitiva::Persistence;

        self.register(StdlibFn::new(
            "print",
            Some("ω"),
            1,
            "Print value without newline",
            PrimitiveComposition::new(vec![Persistence]),
            StdlibKind::Print,
        ));

        self.register(StdlibFn::new(
            "println",
            Some("ωn"),
            1,
            "Print value with newline",
            PrimitiveComposition::new(vec![Persistence]),
            StdlibKind::Println,
        ));
    }

    fn register_type_ops(&mut self) {
        use LexPrimitiva::Comparison;

        self.register(StdlibFn::new(
            "typeof",
            Some("τ"),
            1,
            "Get type name of value",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::TypeOf,
        ));

        self.register(StdlibFn::new(
            "is_int",
            Some("ι?"),
            1,
            "Check if value is integer",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::IsInt,
        ));

        self.register(StdlibFn::new(
            "is_float",
            Some("φ?"),
            1,
            "Check if value is float",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::IsFloat,
        ));

        self.register(StdlibFn::new(
            "is_string",
            Some("S?"),
            1,
            "Check if value is string",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::IsString,
        ));

        self.register(StdlibFn::new(
            "is_seq",
            Some("σ?"),
            1,
            "Check if value is sequence",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::IsSeq,
        ));

        self.register(StdlibFn::new(
            "is_bool",
            None,
            1,
            "Check if value is boolean",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::IsBool,
        ));
    }

    fn register_grounding_ops(&mut self) {
        use LexPrimitiva::{Comparison, Quantity};

        self.register(StdlibFn::new(
            "tier",
            Some("T"),
            1,
            "Get primitive tier of value",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::PrimitiveTier,
        ));

        self.register(StdlibFn::new(
            "composition",
            Some("C"),
            1,
            "Get primitive composition",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::PrimitiveComposition,
        ));

        self.register(StdlibFn::new(
            "constants",
            Some("K"),
            1,
            "Get root constants {0, 1}",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::PrimitiveConstants,
        ));

        self.register(StdlibFn::new(
            "transfer",
            Some("X"),
            1,
            "Get transfer confidence",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::TransferConfidence,
        ));

        self.register(StdlibFn::new(
            "entropy",
            Some("{-}"),
            0,
            "Get current cumulative differential magnitude",
            PrimitiveComposition::new(vec![Quantity]),
            StdlibKind::Entropy,
        ));

        self.register(StdlibFn::new(
            "invert",
            Some("∇"),
            1,
            "Invert value complexity: reduce T3/T2-C to primitive root",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::Invert,
        ));
    }

    fn register_assertion_ops(&mut self) {
        use LexPrimitiva::{Boundary, Comparison};

        self.register(StdlibFn::new(
            "assert",
            Some("‼"),
            1,
            "Assert value is truthy",
            PrimitiveComposition::new(vec![Boundary]),
            StdlibKind::Assert,
        ));

        self.register(StdlibFn::new(
            "contains",
            Some("∈"),
            2,
            "Check if sequence contains element",
            PrimitiveComposition::new(vec![Comparison]),
            StdlibKind::Contains,
        ));
    }

    fn register_convert_ops(&mut self) {
        use LexPrimitiva::{Causality, Quantity, Sequence};

        self.register(StdlibFn::new(
            "to_string",
            Some("⟶S"),
            1,
            "Convert value to string",
            PrimitiveComposition::new(vec![Causality, Sequence]),
            StdlibKind::ToString,
        ));

        self.register(StdlibFn::new(
            "to_int",
            Some("⟶N"),
            1,
            "Convert value to integer",
            PrimitiveComposition::new(vec![Causality, Quantity]),
            StdlibKind::ToInt,
        ));

        self.register(StdlibFn::new(
            "to_float",
            Some("⟶F"),
            1,
            "Convert value to float",
            PrimitiveComposition::new(vec![Causality, Quantity]),
            StdlibKind::ToFloat,
        ));
    }

    fn register_system_ops(&mut self) {
        use LexPrimitiva::{Causality, Existence, Mapping, Persistence, Sequence};

        self.register(StdlibFn::new(
            "stdin",
            Some("⊏"),
            0,
            "Read all of stdin as string",
            PrimitiveComposition::new(vec![Persistence, Sequence]),
            StdlibKind::Stdin,
        ));

        self.register(StdlibFn::new(
            "readline",
            Some("⊐"),
            0,
            "Read one line from stdin",
            PrimitiveComposition::new(vec![Persistence, Sequence]),
            StdlibKind::Readline,
        ));

        self.register(StdlibFn::new(
            "exit",
            Some("⊣"),
            1,
            "Set process exit code",
            PrimitiveComposition::new(vec![Causality]),
            StdlibKind::Exit,
        ));

        self.register(StdlibFn::new(
            "env",
            Some("⊢$"),
            1,
            "Get environment variable",
            PrimitiveComposition::new(vec![Persistence, Existence]),
            StdlibKind::Env,
        ));

        self.register(StdlibFn::new(
            "args",
            Some("⊣σ"),
            0,
            "Get CLI arguments",
            PrimitiveComposition::new(vec![Sequence]),
            StdlibKind::Args,
        ));

        self.register(StdlibFn::new(
            "json_parse",
            Some("⊜"),
            1,
            "Parse JSON string to value",
            PrimitiveComposition::new(vec![Causality, Mapping]),
            StdlibKind::JsonParse,
        ));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXECUTION — → (Causality: execute stdlib functions)
// ═══════════════════════════════════════════════════════════════════════════

/// Execute a stdlib function.
pub fn execute(kind: StdlibKind, args: Vec<Value>) -> PrimaResult<Value> {
    match kind {
        // Sequence operations
        StdlibKind::Len => exec_len(args),
        StdlibKind::Head => exec_head(args),
        StdlibKind::Tail => exec_tail(args),
        StdlibKind::Push => exec_push(args),
        StdlibKind::Pop => exec_pop(args),
        StdlibKind::Concat => exec_concat(args),
        StdlibKind::Range => exec_range(args),
        StdlibKind::Rev => exec_rev(args),

        // Higher-order (return error - need interpreter callback)
        StdlibKind::Map
        | StdlibKind::Filter
        | StdlibKind::Fold
        | StdlibKind::Any
        | StdlibKind::All
        | StdlibKind::Find
        | StdlibKind::Zip => Err(PrimaError::runtime(
            "Higher-order functions require interpreter context",
        )),

        // String operations
        StdlibKind::Chars => exec_chars(args),
        StdlibKind::Split => exec_split(args),
        StdlibKind::Join => exec_join(args),
        StdlibKind::Upper => exec_upper(args),
        StdlibKind::Lower => exec_lower(args),
        StdlibKind::Trim => exec_trim(args),
        StdlibKind::Replace => exec_replace(args),

        // Numeric operations
        StdlibKind::Abs => exec_abs(args),
        StdlibKind::Min => exec_min(args),
        StdlibKind::Max => exec_max(args),
        StdlibKind::Sqrt => exec_sqrt(args),
        StdlibKind::Pow => exec_pow(args),
        StdlibKind::Floor => exec_floor(args),
        StdlibKind::Ceil => exec_ceil(args),
        StdlibKind::Round => exec_round(args),
        StdlibKind::Sin => exec_sin(args),
        StdlibKind::Cos => exec_cos(args),

        // Sequence aggregates
        StdlibKind::Sum => exec_sum(args),
        StdlibKind::Product => exec_product(args),
        StdlibKind::Sort => exec_sort(args),
        StdlibKind::Enumerate => exec_enumerate(args),
        StdlibKind::Flatten => exec_flatten(args),

        // I/O operations
        StdlibKind::Print => exec_print(args, false),
        StdlibKind::Println => exec_print(args, true),

        // Type operations
        StdlibKind::TypeOf => exec_typeof(args),
        StdlibKind::IsInt => exec_is_type(args, |v| matches!(v.data, ValueData::Int(_))),
        StdlibKind::IsFloat => exec_is_type(args, |v| matches!(v.data, ValueData::Float(_))),
        StdlibKind::IsString => exec_is_type(args, |v| matches!(v.data, ValueData::String(_))),
        StdlibKind::IsSeq => exec_is_type(args, |v| matches!(v.data, ValueData::Sequence(_))),
        StdlibKind::IsBool => exec_is_type(args, |v| matches!(v.data, ValueData::Bool(_))),

        // Grounding operations
        StdlibKind::PrimitiveTier => exec_tier(args),
        StdlibKind::PrimitiveComposition => exec_composition(args),
        StdlibKind::PrimitiveConstants => exec_constants(args),
        StdlibKind::TransferConfidence => exec_transfer(args),
        StdlibKind::Entropy => exec_entropy(args),
        StdlibKind::Invert => exec_invert(args),

        // Conversion operations
        StdlibKind::ToString => exec_to_string(args),
        StdlibKind::ToInt => exec_to_int(args),
        StdlibKind::ToFloat => exec_to_float(args),

        // Assertion
        StdlibKind::Assert => exec_assert(args),
        StdlibKind::Contains => exec_contains(args),

        // System operations (hook support)
        StdlibKind::Stdin => exec_stdin(args),
        StdlibKind::Readline => exec_readline(args),
        StdlibKind::Exit => exec_exit(args),
        StdlibKind::Env => exec_env(args),
        StdlibKind::Args => exec_args(args),
        StdlibKind::JsonParse => exec_json_parse(args),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IMPLEMENTATION — Individual function implementations
// ═══════════════════════════════════════════════════════════════════════════

fn expect_args(args: &[Value], expected: usize, name: &str) -> PrimaResult<()> {
    if args.len() != expected {
        return Err(PrimaError::runtime(format!(
            "{name} expects {expected} argument(s), got {}",
            args.len()
        )));
    }
    Ok(())
}

fn exec_len(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "len")?;
    match &args[0].data {
        ValueData::Sequence(v) => Ok(Value::int(v.len() as i64)),
        ValueData::String(s) => Ok(Value::int(s.len() as i64)),
        _ => Err(PrimaError::runtime("len: expected sequence or string")),
    }
}

fn exec_head(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "head")?;
    match &args[0].data {
        ValueData::Sequence(v) => v
            .first()
            .cloned()
            .ok_or_else(|| PrimaError::runtime("head: empty sequence")),
        ValueData::String(s) => s
            .chars()
            .next()
            .map(|c| Value::string(c.to_string()))
            .ok_or_else(|| PrimaError::runtime("head: empty string")),
        _ => Err(PrimaError::runtime("head: expected sequence or string")),
    }
}

fn exec_tail(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "tail")?;
    match &args[0].data {
        ValueData::Sequence(v) => {
            if v.is_empty() {
                return Err(PrimaError::runtime("tail: empty sequence"));
            }
            Ok(Value::sequence(v[1..].to_vec()))
        }
        ValueData::String(s) => {
            if s.is_empty() {
                return Err(PrimaError::runtime("tail: empty string"));
            }
            let mut chars = s.chars();
            chars.next();
            Ok(Value::string(chars.collect::<String>()))
        }
        _ => Err(PrimaError::runtime("tail: expected sequence or string")),
    }
}

fn exec_push(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "push")?;
    match &args[0].data {
        ValueData::Sequence(v) => {
            let mut new_vec = v.clone();
            new_vec.push(args[1].clone());
            Ok(Value::sequence(new_vec))
        }
        _ => Err(PrimaError::runtime("push: expected sequence")),
    }
}

fn exec_pop(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "pop")?;
    match &args[0].data {
        ValueData::Sequence(v) => {
            if v.is_empty() {
                return Err(PrimaError::runtime("pop: empty sequence"));
            }
            let mut new_vec = v.clone();
            let popped = new_vec.pop().unwrap_or_else(Value::void);
            // Return tuple of (new_seq, popped_value)
            Ok(Value::sequence(vec![Value::sequence(new_vec), popped]))
        }
        _ => Err(PrimaError::runtime("pop: expected sequence")),
    }
}

fn exec_concat(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "concat")?;
    match (&args[0].data, &args[1].data) {
        (ValueData::Sequence(a), ValueData::Sequence(b)) => {
            let mut result = a.clone();
            result.extend(b.iter().cloned());
            Ok(Value::sequence(result))
        }
        (ValueData::String(a), ValueData::String(b)) => Ok(Value::string(format!("{a}{b}"))),
        _ => Err(PrimaError::runtime(
            "concat: expected two sequences or two strings",
        )),
    }
}

fn exec_range(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "range")?;
    match (&args[0].data, &args[1].data) {
        (ValueData::Int(start), ValueData::Int(end)) => {
            let seq: Vec<Value> = (*start..*end).map(Value::int).collect();
            Ok(Value::sequence(seq))
        }
        _ => Err(PrimaError::runtime("range: expected two integers")),
    }
}

fn exec_rev(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "rev")?;
    match &args[0].data {
        ValueData::Sequence(v) => {
            let reversed: Vec<Value> = v.iter().rev().cloned().collect();
            Ok(Value::sequence(reversed))
        }
        ValueData::String(s) => Ok(Value::string(s.chars().rev().collect::<String>())),
        _ => Err(PrimaError::runtime("rev: expected sequence or string")),
    }
}

fn exec_chars(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "chars")?;
    match &args[0].data {
        ValueData::String(s) => {
            let chars: Vec<Value> = s.chars().map(|c| Value::string(c.to_string())).collect();
            Ok(Value::sequence(chars))
        }
        _ => Err(PrimaError::runtime("chars: expected string")),
    }
}

fn exec_split(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "split")?;
    match (&args[0].data, &args[1].data) {
        (ValueData::String(s), ValueData::String(delim)) => {
            let parts: Vec<Value> = s
                .split(delim.as_str())
                .map(|p| Value::string(p.to_string()))
                .collect();
            Ok(Value::sequence(parts))
        }
        _ => Err(PrimaError::runtime("split: expected two strings")),
    }
}

fn exec_join(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "join")?;
    match (&args[0].data, &args[1].data) {
        (ValueData::Sequence(v), ValueData::String(delim)) => {
            let parts: Vec<String> = v
                .iter()
                .map(|v| match &v.data {
                    ValueData::String(s) => s.clone(),
                    _ => format!("{v}"),
                })
                .collect();
            Ok(Value::string(parts.join(delim)))
        }
        _ => Err(PrimaError::runtime("join: expected sequence and string")),
    }
}

fn exec_upper(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "upper")?;
    match &args[0].data {
        ValueData::String(s) => Ok(Value::string(s.to_uppercase())),
        _ => Err(PrimaError::runtime("upper: expected string")),
    }
}

fn exec_lower(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "lower")?;
    match &args[0].data {
        ValueData::String(s) => Ok(Value::string(s.to_lowercase())),
        _ => Err(PrimaError::runtime("lower: expected string")),
    }
}

fn exec_trim(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "trim")?;
    match &args[0].data {
        ValueData::String(s) => Ok(Value::string(s.trim().to_string())),
        _ => Err(PrimaError::runtime("trim: expected string")),
    }
}

fn exec_replace(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 3, "replace")?;
    match (&args[0].data, &args[1].data, &args[2].data) {
        (ValueData::String(s), ValueData::String(from), ValueData::String(to)) => {
            Ok(Value::string(s.replace(from.as_str(), to.as_str())))
        }
        _ => Err(PrimaError::runtime("replace: expected three strings")),
    }
}

fn exec_abs(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "abs")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::int(n.abs())),
        ValueData::Float(f) => Ok(Value::float(f.abs())),
        _ => Err(PrimaError::runtime("abs: expected number")),
    }
}

fn exec_min(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "min")?;
    match (&args[0].data, &args[1].data) {
        (ValueData::Int(a), ValueData::Int(b)) => Ok(Value::int(*a.min(b))),
        (ValueData::Float(a), ValueData::Float(b)) => Ok(Value::float(a.min(*b))),
        _ => Err(PrimaError::runtime("min: expected two numbers")),
    }
}

fn exec_max(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "max")?;
    match (&args[0].data, &args[1].data) {
        (ValueData::Int(a), ValueData::Int(b)) => Ok(Value::int(*a.max(b))),
        (ValueData::Float(a), ValueData::Float(b)) => Ok(Value::float(a.max(*b))),
        _ => Err(PrimaError::runtime("max: expected two numbers")),
    }
}

fn exec_sqrt(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "sqrt")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::float((*n as f64).sqrt())),
        ValueData::Float(f) => Ok(Value::float(f.sqrt())),
        _ => Err(PrimaError::runtime("sqrt: expected number")),
    }
}

fn exec_pow(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "pow")?;
    match (&args[0].data, &args[1].data) {
        (ValueData::Int(base), ValueData::Int(exp)) => {
            if *exp >= 0 {
                Ok(Value::int(base.pow(*exp as u32)))
            } else {
                Ok(Value::float((*base as f64).powi(*exp as i32)))
            }
        }
        (ValueData::Float(base), ValueData::Int(exp)) => Ok(Value::float(base.powi(*exp as i32))),
        (ValueData::Float(base), ValueData::Float(exp)) => Ok(Value::float(base.powf(*exp))),
        (ValueData::Int(base), ValueData::Float(exp)) => {
            Ok(Value::float((*base as f64).powf(*exp)))
        }
        _ => Err(PrimaError::runtime("pow: expected numbers")),
    }
}

fn exec_floor(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "floor")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::int(*n)),
        ValueData::Float(f) => Ok(Value::int(f.floor() as i64)),
        _ => Err(PrimaError::runtime("floor: expected number")),
    }
}

fn exec_ceil(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "ceil")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::int(*n)),
        ValueData::Float(f) => Ok(Value::int(f.ceil() as i64)),
        _ => Err(PrimaError::runtime("ceil: expected number")),
    }
}

fn exec_round(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "round")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::int(*n)),
        ValueData::Float(f) => Ok(Value::int(f.round() as i64)),
        _ => Err(PrimaError::runtime("round: expected number")),
    }
}

fn exec_sin(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "sin")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::float((*n as f64).sin())),
        ValueData::Float(f) => Ok(Value::float(f.sin())),
        _ => Err(PrimaError::runtime("sin: expected number")),
    }
}

fn exec_cos(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "cos")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::float((*n as f64).cos())),
        ValueData::Float(f) => Ok(Value::float(f.cos())),
        _ => Err(PrimaError::runtime("cos: expected number")),
    }
}

fn exec_sum(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "sum")?;
    match &args[0].data {
        ValueData::Sequence(seq) => {
            let mut sum_int: i64 = 0;
            let mut has_float = false;
            let mut sum_float: f64 = 0.0;

            for v in seq {
                match &v.data {
                    ValueData::Int(n) => {
                        if has_float {
                            sum_float += *n as f64;
                        } else {
                            sum_int += n;
                        }
                    }
                    ValueData::Float(f) => {
                        if !has_float {
                            has_float = true;
                            sum_float = sum_int as f64;
                        }
                        sum_float += f;
                    }
                    _ => return Err(PrimaError::runtime("sum: sequence must contain numbers")),
                }
            }

            if has_float {
                Ok(Value::float(sum_float))
            } else {
                Ok(Value::int(sum_int))
            }
        }
        _ => Err(PrimaError::runtime("sum: expected sequence")),
    }
}

fn exec_product(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "product")?;
    match &args[0].data {
        ValueData::Sequence(seq) => {
            if seq.is_empty() {
                return Ok(Value::int(1)); // Identity for multiplication
            }

            let mut prod_int: i64 = 1;
            let mut has_float = false;
            let mut prod_float: f64 = 1.0;

            for v in seq {
                match &v.data {
                    ValueData::Int(n) => {
                        if has_float {
                            prod_float *= *n as f64;
                        } else {
                            prod_int *= n;
                        }
                    }
                    ValueData::Float(f) => {
                        if !has_float {
                            has_float = true;
                            prod_float = prod_int as f64;
                        }
                        prod_float *= f;
                    }
                    _ => {
                        return Err(PrimaError::runtime(
                            "product: sequence must contain numbers",
                        ));
                    }
                }
            }

            if has_float {
                Ok(Value::float(prod_float))
            } else {
                Ok(Value::int(prod_int))
            }
        }
        _ => Err(PrimaError::runtime("product: expected sequence")),
    }
}

fn exec_sort(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "sort")?;
    match &args[0].data {
        ValueData::Sequence(seq) => {
            let mut sorted = seq.clone();
            sorted.sort_by(|a, b| match (&a.data, &b.data) {
                (ValueData::Int(x), ValueData::Int(y)) => x.cmp(y),
                (ValueData::Float(x), ValueData::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (ValueData::String(x), ValueData::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::sequence(sorted))
        }
        _ => Err(PrimaError::runtime("sort: expected sequence")),
    }
}

fn exec_enumerate(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "enumerate")?;
    match &args[0].data {
        ValueData::Sequence(seq) => {
            let enumerated: Vec<Value> = seq
                .iter()
                .enumerate()
                .map(|(i, v)| Value::sequence(vec![Value::int(i as i64), v.clone()]))
                .collect();
            Ok(Value::sequence(enumerated))
        }
        _ => Err(PrimaError::runtime("enumerate: expected sequence")),
    }
}

fn exec_flatten(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "flatten")?;
    match &args[0].data {
        ValueData::Sequence(seq) => {
            let mut flattened = Vec::new();
            for v in seq {
                match &v.data {
                    ValueData::Sequence(inner) => flattened.extend(inner.clone()),
                    _ => flattened.push(v.clone()),
                }
            }
            Ok(Value::sequence(flattened))
        }
        _ => Err(PrimaError::runtime("flatten: expected sequence")),
    }
}

fn exec_print(args: Vec<Value>, newline: bool) -> PrimaResult<Value> {
    expect_args(&args, 1, "print")?;
    if newline {
        println!("{}", args[0]);
    } else {
        print!("{}", args[0]);
    }
    Ok(Value::void())
}

fn exec_typeof(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "typeof")?;
    let type_name = match &args[0].data {
        ValueData::Void => "∅",
        ValueData::Int(_) => "N",
        ValueData::Float(_) => "Float",
        ValueData::Bool(_) => "Bool",
        ValueData::String(_) => "String",
        ValueData::Sequence(_) => "σ",
        ValueData::Mapping(_) => "μ[→]",
        ValueData::Function(_) => "μ",
        ValueData::Builtin(_) => "builtin",
        ValueData::Symbol(_) => "λ",
        ValueData::Quoted(_) => "ρ",
    };
    Ok(Value::string(type_name.to_string()))
}

fn exec_is_type<F: Fn(&Value) -> bool>(args: Vec<Value>, pred: F) -> PrimaResult<Value> {
    expect_args(&args, 1, "is_*")?;
    Ok(Value::bool(pred(&args[0])))
}

fn exec_tier(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "tier")?;
    let tier = args[0].tier();
    let tier_str = match tier {
        Tier::T1Universal => "T1",
        Tier::T2Primitive => "T2-P",
        Tier::T2Composite => "T2-C",
        Tier::T3DomainSpecific => "T3",
    };
    Ok(Value::string(tier_str.to_string()))
}

fn exec_composition(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "composition")?;
    let comp = &args[0].composition;
    let symbols: Vec<String> = comp
        .unique()
        .iter()
        .map(|p| p.symbol().to_string())
        .collect();
    Ok(Value::string(symbols.join(" ")))
}

fn exec_constants(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "constants")?;
    // All values ultimately ground to {0, 1}
    Ok(Value::string("{0, 1}".to_string()))
}

fn exec_transfer(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "transfer")?;
    let tier = args[0].tier();
    let confidence = match tier {
        Tier::T1Universal => 1.0,
        Tier::T2Primitive => 0.9,
        Tier::T2Composite => 0.7,
        Tier::T3DomainSpecific => 0.4,
    };
    Ok(Value::float(confidence))
}

fn exec_entropy(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 0, "entropy")?;
    Ok(Value::float(crate::builtins::get_entropy()))
}

fn exec_invert(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "invert")?;
    let val = &args[0];

    // Funnel Inversion: decompose high-tier complexity into primitive roots.
    // We return a new value with the same data but a pure T1/T2 composition.
    let primitives: Vec<_> = val.composition.unique().into_iter().collect();
    Ok(Value {
        data: val.data.clone(),
        // Keep only the unique primitives, stripping domain-specific metadata
        composition: PrimitiveComposition::new(primitives),
    })
}

fn exec_to_string(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "to_string")?;
    Ok(Value::string(format!("{}", args[0])))
}

fn exec_to_int(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "to_int")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::int(*n)),
        ValueData::Float(n) => Ok(Value::int(*n as i64)),
        ValueData::String(s) => s
            .parse::<i64>()
            .map(Value::int)
            .map_err(|_| PrimaError::runtime(format!("cannot parse '{}' as int", s))),
        ValueData::Bool(b) => Ok(Value::int(if *b { 1 } else { 0 })),
        _ => Err(PrimaError::runtime("to_int requires a convertible value")),
    }
}

fn exec_to_float(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "to_float")?;
    match &args[0].data {
        ValueData::Int(n) => Ok(Value::float(*n as f64)),
        ValueData::Float(n) => Ok(Value::float(*n)),
        ValueData::String(s) => s
            .parse::<f64>()
            .map(Value::float)
            .map_err(|_| PrimaError::runtime(format!("cannot parse '{}' as float", s))),
        _ => Err(PrimaError::runtime("to_float requires a convertible value")),
    }
}

fn exec_assert(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "assert")?;
    if args[0].is_truthy() {
        Ok(Value::void())
    } else {
        Err(PrimaError::runtime("Assertion failed"))
    }
}

fn exec_contains(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 2, "contains")?;
    match &args[0].data {
        ValueData::Sequence(v) => Ok(Value::bool(v.contains(&args[1]))),
        ValueData::String(s) => {
            if let ValueData::String(needle) = &args[1].data {
                Ok(Value::bool(s.contains(needle.as_str())))
            } else {
                Err(PrimaError::runtime(
                    "contains: string requires string needle",
                ))
            }
        }
        _ => Err(PrimaError::runtime("contains: expected sequence or string")),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEM OPERATIONS — π + → + ∃ (Hook support)
// ═══════════════════════════════════════════════════════════════════════════

fn exec_stdin(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 0, "stdin")?;
    crate::builtins::call_builtin("stdin", &[])
}

fn exec_readline(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 0, "readline")?;
    crate::builtins::call_builtin("readline", &[])
}

fn exec_exit(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "exit")?;
    crate::builtins::call_builtin("exit", &args)
}

fn exec_env(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "env")?;
    crate::builtins::call_builtin("env", &args)
}

fn exec_args(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 0, "args")?;
    crate::builtins::call_builtin("args", &[])
}

fn exec_json_parse(args: Vec<Value>) -> PrimaResult<Value> {
    expect_args(&args, 1, "json_parse")?;
    crate::builtins::call_builtin("json_parse", &args)
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────
    // Stdlib registration tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_stdlib_has_core_functions() {
        let stdlib = Stdlib::new();

        // Sequence ops
        assert!(stdlib.has("len"));
        assert!(stdlib.has("head"));
        assert!(stdlib.has("tail"));

        // Higher-order
        assert!(stdlib.has("map"));
        assert!(stdlib.has("filter"));
        assert!(stdlib.has("fold"));

        // I/O
        assert!(stdlib.has("print"));
        assert!(stdlib.has("println"));
    }

    #[test]
    fn test_stdlib_symbol_lookup() {
        let stdlib = Stdlib::new();

        // Can look up by symbol
        assert!(stdlib.get("Φ").is_some()); // map
        assert!(stdlib.get("Ψ").is_some()); // filter
        assert!(stdlib.get("#").is_some()); // len
        assert!(stdlib.get("ω").is_some()); // print
    }

    #[test]
    fn test_stdlib_function_tier() {
        let stdlib = Stdlib::new();

        let len_fn = stdlib.get("len");
        assert!(len_fn.is_some());
        let len_fn = len_fn.unwrap();
        // len is T2-P (σ + N)
        assert!(matches!(
            len_fn.tier(),
            Tier::T2Primitive | Tier::T2Composite
        ));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Sequence operation tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exec_len() {
        let result = execute(
            StdlibKind::Len,
            vec![Value::sequence(vec![
                Value::int(1),
                Value::int(2),
                Value::int(3),
            ])],
        );
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(3));
    }

    #[test]
    fn test_exec_len_string() {
        let result = execute(StdlibKind::Len, vec![Value::string("hello")]);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(5));
    }

    #[test]
    fn test_exec_head() {
        let result = execute(
            StdlibKind::Head,
            vec![Value::sequence(vec![Value::int(1), Value::int(2)])],
        );
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(1));
    }

    #[test]
    fn test_exec_head_empty() {
        let result = execute(StdlibKind::Head, vec![Value::sequence(vec![])]);
        assert!(result.is_err());
    }

    #[test]
    fn test_exec_tail() {
        let result = execute(
            StdlibKind::Tail,
            vec![Value::sequence(vec![
                Value::int(1),
                Value::int(2),
                Value::int(3),
            ])],
        );
        assert!(result.is_ok());
        let tail = result.ok().unwrap_or_else(Value::void);
        if let ValueData::Sequence(v) = &tail.data {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0], Value::int(2));
            assert_eq!(v[1], Value::int(3));
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_exec_push() {
        let result = execute(
            StdlibKind::Push,
            vec![Value::sequence(vec![Value::int(1)]), Value::int(2)],
        );
        assert!(result.is_ok());
        let pushed = result.ok().unwrap_or_else(Value::void);
        if let ValueData::Sequence(v) = &pushed.data {
            assert_eq!(v.len(), 2);
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_exec_concat() {
        let result = execute(
            StdlibKind::Concat,
            vec![
                Value::sequence(vec![Value::int(1)]),
                Value::sequence(vec![Value::int(2)]),
            ],
        );
        assert!(result.is_ok());
        let concat = result.ok().unwrap_or_else(Value::void);
        if let ValueData::Sequence(v) = &concat.data {
            assert_eq!(v.len(), 2);
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_exec_range() {
        let result = execute(StdlibKind::Range, vec![Value::int(0), Value::int(5)]);
        assert!(result.is_ok());
        let range = result.ok().unwrap_or_else(Value::void);
        if let ValueData::Sequence(v) = &range.data {
            assert_eq!(v.len(), 5);
            assert_eq!(v[0], Value::int(0));
            assert_eq!(v[4], Value::int(4));
        } else {
            panic!("Expected sequence");
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // String operation tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exec_chars() {
        let result = execute(StdlibKind::Chars, vec![Value::string("abc")]);
        assert!(result.is_ok());
        let chars = result.ok().unwrap_or_else(Value::void);
        if let ValueData::Sequence(v) = &chars.data {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Value::string("a"));
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_exec_split() {
        let result = execute(
            StdlibKind::Split,
            vec![Value::string("a,b,c"), Value::string(",")],
        );
        assert!(result.is_ok());
        let parts = result.ok().unwrap_or_else(Value::void);
        if let ValueData::Sequence(v) = &parts.data {
            assert_eq!(v.len(), 3);
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_exec_upper_lower() {
        let upper = execute(StdlibKind::Upper, vec![Value::string("hello")]);
        assert!(upper.is_ok());
        assert_eq!(
            upper.ok().unwrap_or_else(Value::void),
            Value::string("HELLO")
        );

        let lower = execute(StdlibKind::Lower, vec![Value::string("WORLD")]);
        assert!(lower.is_ok());
        assert_eq!(
            lower.ok().unwrap_or_else(Value::void),
            Value::string("world")
        );
    }

    #[test]
    fn test_exec_trim() {
        let result = execute(StdlibKind::Trim, vec![Value::string("  hello  ")]);
        assert!(result.is_ok());
        assert_eq!(
            result.ok().unwrap_or_else(Value::void),
            Value::string("hello")
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Numeric operation tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exec_abs() {
        let result = execute(StdlibKind::Abs, vec![Value::int(-5)]);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::int(5));
    }

    #[test]
    fn test_exec_min_max() {
        let min = execute(StdlibKind::Min, vec![Value::int(3), Value::int(7)]);
        assert!(min.is_ok());
        assert_eq!(min.ok().unwrap_or_else(Value::void), Value::int(3));

        let max = execute(StdlibKind::Max, vec![Value::int(3), Value::int(7)]);
        assert!(max.is_ok());
        assert_eq!(max.ok().unwrap_or_else(Value::void), Value::int(7));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Type operation tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exec_typeof() {
        let result = execute(StdlibKind::TypeOf, vec![Value::int(42)]);
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::string("N"));
    }

    #[test]
    fn test_exec_is_int() {
        let is_int = execute(StdlibKind::IsInt, vec![Value::int(42)]);
        assert!(is_int.is_ok());
        assert_eq!(is_int.ok().unwrap_or_else(Value::void), Value::bool(true));

        let not_int = execute(StdlibKind::IsInt, vec![Value::string("hi")]);
        assert!(not_int.is_ok());
        assert_eq!(not_int.ok().unwrap_or_else(Value::void), Value::bool(false));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Grounding operation tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exec_tier() {
        let result = execute(StdlibKind::PrimitiveTier, vec![Value::int(42)]);
        assert!(result.is_ok());
        // Int is T1 (Quantity)
        let tier = result.ok().unwrap_or_else(Value::void);
        if let ValueData::String(s) = &tier.data {
            assert!(s.starts_with("T1") || s.starts_with("T2"));
        }
    }

    #[test]
    fn test_exec_transfer() {
        let result = execute(StdlibKind::TransferConfidence, vec![Value::int(42)]);
        assert!(result.is_ok());
        let conf = result.ok().unwrap_or_else(Value::void);
        if let ValueData::Float(f) = &conf.data {
            assert!(*f >= 0.4 && *f <= 1.0);
        } else {
            panic!("Expected float");
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Assertion tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_exec_assert_true() {
        let result = execute(StdlibKind::Assert, vec![Value::bool(true)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_exec_assert_false() {
        let result = execute(StdlibKind::Assert, vec![Value::bool(false)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_exec_contains() {
        let result = execute(
            StdlibKind::Contains,
            vec![
                Value::sequence(vec![Value::int(1), Value::int(2)]),
                Value::int(2),
            ],
        );
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap_or_else(Value::void), Value::bool(true));
    }
}
