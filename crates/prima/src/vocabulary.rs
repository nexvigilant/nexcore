// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Vocabulary — Single Source of Truth
//!
//! Every word↔symbol mapping in Prima is defined here **once**.
//!
//! ## Consumers
//!
//! | Module | Usage |
//! |--------|-------|
//! | `builtins.rs` | Runtime function registry (verbose + compressed names) |
//! | `compress.rs` | Text compression (word → symbol translation) |
//! | `lexer.rs` | Identifier character classification (`is_math_symbol`) |
//!
//! ## T1 Grounding
//!
//! - `μ` (Mapping): Each entry is a word→symbol mapping
//! - `σ` (Sequence): Categories are ordered collections of mappings
//! - `ν` (Invariant): All entries are compile-time constants
//!
//! ## Tier: T2-P (μ + σ + ν)

/// A vocabulary entry: (canonical_name, compressed_symbol).
///
/// The canonical name is the verbose form used in source code.
/// The compressed symbol is the Unicode equivalent.
pub type VocabEntry = (&'static str, &'static str);

// ═══════════════════════════════════════════════════════════════════════════
// THE 15 LEX PRIMITIVA — T1 Universals
// ═══════════════════════════════════════════════════════════════════════════

/// Primitive concept → symbol mappings.
/// These are the 15 irreducible building blocks.
pub const PRIMITIVES: &[VocabEntry] = &[
    ("sequence", "σ"),
    ("mapping", "μ"),
    ("state", "ς"),
    ("recursion", "ρ"),
    ("void", "∅"),
    ("boundary", "∂"),
    ("invariant", "ν"),
    ("existence", "∃"),
    ("persistence", "π"),
    ("causality", "→"),
    ("comparison", "κ"),
    ("quantity", "N"),
    ("location", "λ"),
    ("proportion", "∝"),
    ("sum", "Σ"),
];

// ═══════════════════════════════════════════════════════════════════════════
// KEYWORDS — Reserved words with primitive equivalents
// ═══════════════════════════════════════════════════════════════════════════

/// Keyword → primitive symbol mappings.
pub const KEYWORDS: &[VocabEntry] = &[
    ("let", "λ"),
    ("fn", "μ"),
    ("if", "∂"),
    ("for", "σ"),
    ("match", "Σ"),
    ("return", "→"),
    ("true", "1"),
    ("false", "0"),
];

/// Keyword aliases — alternate verbose forms that compress to the same symbols.
pub const KEYWORD_ALIASES: &[VocabEntry] = &[("function", "μ")];

// ═══════════════════════════════════════════════════════════════════════════
// OPERATORS — Comparison and logical
// ═══════════════════════════════════════════════════════════════════════════

/// Operator word → symbol mappings.
pub const OPERATORS: &[VocabEntry] = &[
    ("equals", "κ="),
    ("not_equals", "κ!="),
    ("less_than", "κ<"),
    ("greater_than", "κ>"),
    ("less_equal", "κ<="),
    ("greater_equal", "κ>="),
    ("and", "∧"),
    ("or", "∨"),
    ("not", "¬"),
    ("pipe", "|>"),
    ("arrow", "→"),
];

// ═══════════════════════════════════════════════════════════════════════════
// HIGHER-ORDER FUNCTIONS — σ + μ compositions
// ═══════════════════════════════════════════════════════════════════════════

/// Higher-order function mappings.
pub const HOFS: &[VocabEntry] = &[
    ("map", "Φ"),
    ("filter", "Ψ"),
    ("fold", "Ω"),
    ("any", "∃?"),
    ("all", "∀?"),
    ("find", "⊃"),
    ("zip", "⊠"),
];

/// HOF aliases.
pub const HOF_ALIASES: &[VocabEntry] = &[("reduce", "Ω")];

// ═══════════════════════════════════════════════════════════════════════════
// BUILTINS — Categorized by primitive grounding
// ═══════════════════════════════════════════════════════════════════════════

/// I/O builtins: → + π + ∅
pub const BUILTINS_IO: &[VocabEntry] = &[("print", "ω"), ("println", "ωn")];

/// Sequence builtins: σ operations.
pub const BUILTINS_SEQ: &[VocabEntry] = &[
    ("len", "#"),
    ("push", "⊕"),
    ("pop", "⊖"),
    ("head", "↑"),
    ("tail", "↓"),
    ("concat", "⊙"),
    ("range", "‥"),
];

/// Sequence aliases.
pub const BUILTINS_SEQ_ALIASES: &[VocabEntry] = &[("length", "#"), ("first", "↑"), ("rest", "↓")];

/// String builtins: σ[char] operations.
pub const BUILTINS_STRING: &[VocabEntry] = &[
    ("chars", "χ"),
    ("split", "⊘"),
    ("join", "⊗"),
    ("upper", "⇑"),
    ("lower", "⇓"),
    ("trim", "⊢"),
    ("contains", "∈"),
    ("replace", "↔"),
];

/// Math builtins: N operations.
pub const BUILTINS_MATH: &[VocabEntry] =
    &[("abs", "±"), ("min", "⌊"), ("max", "⌈"), ("entropy", "{-}")];

/// Type introspection builtins: ∃ + τ.
pub const BUILTINS_TYPE: &[VocabEntry] = &[
    ("typeof", "τ"),
    ("is_int", "ι?"),
    ("is_float", "φ?"),
    ("is_string", "S?"),
    ("is_seq", "σ?"),
];

/// Grounding introspection builtins.
pub const BUILTINS_GROUNDING: &[VocabEntry] = &[
    ("tier", "T"),
    ("composition", "C"),
    ("constants", "K"),
    ("transfer", "X"),
];

/// Verification builtins: ∂ + ν.
pub const BUILTINS_VERIFY: &[VocabEntry] = &[("assert", "‼")];

/// Conversion builtins: → type coercion.
pub const BUILTINS_CONVERT: &[VocabEntry] =
    &[("to_string", "⟶S"), ("to_int", "⟶N"), ("to_float", "⟶F")];

/// System builtins: π + → + ∃ (CLI/process interaction).
/// Required for hook support: stdin, exit, env, args, json.
pub const BUILTINS_SYSTEM: &[VocabEntry] = &[
    ("stdin", "⊏"),      // Read full stdin → String
    ("readline", "⊐"),   // Read one line → String
    ("exit", "⊣"),       // Set exit code (N → ∅)
    ("env", "⊢$"),       // Get env var (String → String|∅)
    ("args", "⊣σ"),      // CLI args → σ[String]
    ("json_parse", "⊜"), // Parse JSON string → μ
];

// ═══════════════════════════════════════════════════════════════════════════
// DERIVED HELPERS — Computed from the above constants
// ═══════════════════════════════════════════════════════════════════════════

/// All builtin entries (canonical names only, no aliases).
/// Used by `builtins.rs` for registration.
pub const ALL_BUILTINS: &[&[VocabEntry]] = &[
    BUILTINS_IO,
    BUILTINS_SEQ,
    BUILTINS_STRING,
    BUILTINS_MATH,
    BUILTINS_TYPE,
    BUILTINS_GROUNDING,
    BUILTINS_VERIFY,
    BUILTINS_CONVERT,
    BUILTINS_SYSTEM,
];

/// All vocabulary entries for text compression (canonical + aliases).
/// Used by `compress.rs` Lexicon.
pub const ALL_COMPRESS: &[&[VocabEntry]] = &[
    PRIMITIVES,
    KEYWORDS,
    KEYWORD_ALIASES,
    OPERATORS,
    HOFS,
    HOF_ALIASES,
    BUILTINS_IO,
    BUILTINS_SEQ,
    BUILTINS_SEQ_ALIASES,
    BUILTINS_STRING,
    BUILTINS_MATH,
    BUILTINS_TYPE,
    BUILTINS_GROUNDING,
    BUILTINS_VERIFY,
    BUILTINS_CONVERT,
    BUILTINS_SYSTEM,
];

/// Characters that are valid in identifiers because they represent builtins.
///
/// Derived from the symbol column of all builtin entries.
/// Used by `lexer.rs` `is_math_symbol()`.
///
/// # Implementation Note
///
/// This is a `const fn` but returns a fixed set because const iteration
/// over slices is limited. When new builtins are added to the vocabulary,
/// add their lead character here.
pub const fn is_math_identifier_char(c: char) -> bool {
    matches!(
        c,
        '#'  | // len
        '±'  | // abs
        '↑'  | '↓'  | // head, tail
        '⊕'  | '⊖'  | // push, pop
        '⊙'  | '⊘'  | '⊗'  | // concat, split, join
        '‥'  | // range
        'χ'  | 'ω'  | // chars, print
        '⇑'  | '⇓'  | // upper, lower
        '⊢'  | '↔'  | // trim, replace
        '∈'  | // contains
        '⟶'  | // conversion arrow
        '⌊'  | '⌈'  | // min, max
        'τ'  | 'ι'  | 'φ'  | // typeof, is_int, is_float
        '‼'  | // assert
        '⊏'  | '⊐'  | // stdin, readline
        '⊣'  | // exit, args (⊣σ starts with ⊣)
        '⊜' // json_parse
    )
}

/// Total number of unique canonical builtin functions.
pub const fn builtin_count() -> usize {
    BUILTINS_IO.len()
        + BUILTINS_SEQ.len()
        + BUILTINS_STRING.len()
        + BUILTINS_MATH.len()
        + BUILTINS_TYPE.len()
        + BUILTINS_GROUNDING.len()
        + BUILTINS_VERIFY.len()
        + BUILTINS_CONVERT.len()
        + BUILTINS_SYSTEM.len()
}

/// Total number of vocabulary entries (including aliases).
pub const fn total_entries() -> usize {
    let mut total = 0;
    let mut i = 0;
    while i < ALL_COMPRESS.len() {
        total += ALL_COMPRESS[i].len();
        i += 1;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_count() {
        // The 15 Lex Primitiva
        assert_eq!(PRIMITIVES.len(), 15);
    }

    #[test]
    fn test_builtin_count() {
        // 34 original + 6 system builtins = 40
        assert_eq!(builtin_count(), 40);
    }

    #[test]
    fn test_total_entries() {
        // Should be > builtin_count (includes primitives, keywords, aliases, etc.)
        assert!(total_entries() > builtin_count());
    }

    #[test]
    fn test_no_duplicate_canonical_names() {
        let mut names = std::collections::HashSet::new();
        for group in ALL_BUILTINS {
            for (name, _) in *group {
                assert!(
                    names.insert(*name),
                    "Duplicate canonical builtin name: {}",
                    name
                );
            }
        }
    }

    #[test]
    fn test_hof_entries() {
        assert_eq!(HOFS.len(), 7);
        assert_eq!(HOFS[0], ("map", "Φ"));
        assert_eq!(HOFS[1], ("filter", "Ψ"));
        assert_eq!(HOFS[2], ("fold", "Ω"));
    }

    #[test]
    fn test_keyword_mappings() {
        assert_eq!(KEYWORDS.len(), 8);
        assert!(KEYWORDS.iter().any(|(k, s)| *k == "let" && *s == "λ"));
        assert!(KEYWORDS.iter().any(|(k, s)| *k == "fn" && *s == "μ"));
    }

    #[test]
    fn test_math_identifier_chars() {
        // Builtin math symbols recognized as identifier chars
        assert!(is_math_identifier_char('#'));
        assert!(is_math_identifier_char('↑'));
        assert!(is_math_identifier_char('ω'));
        assert!(is_math_identifier_char('‼'));

        // Greek HOF symbols (Φ, Ψ, Ω) are alphabetic, not math — handled by is_alphabetic()
        assert!(!is_math_identifier_char('Φ'));

        // Normal ASCII should not be math identifier chars
        assert!(!is_math_identifier_char('a'));
        assert!(!is_math_identifier_char('1'));
        assert!(!is_math_identifier_char('+'));
    }

    #[test]
    fn test_aliases_map_to_existing_symbols() {
        // Every alias symbol should exist in the canonical entries
        let canonical_symbols: std::collections::HashSet<&str> = BUILTINS_SEQ
            .iter()
            .chain(HOFS.iter())
            .chain(KEYWORDS.iter())
            .map(|(_, s)| *s)
            .collect();

        for (alias, symbol) in BUILTINS_SEQ_ALIASES
            .iter()
            .chain(HOF_ALIASES.iter())
            .chain(KEYWORD_ALIASES.iter())
        {
            assert!(
                canonical_symbols.contains(symbol),
                "Alias '{}' → '{}' maps to non-existent canonical symbol",
                alias,
                symbol
            );
        }
    }
}
