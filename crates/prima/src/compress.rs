// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Compression System
//!
//! Bidirectional word↔symbol translation for token efficiency.
//!
//! ## Mathematical Foundation
//!
//! Compression is grounded in:
//! - **μ** (Mapping): word → symbol transformation
//! - **σ** (Sequence): token stream compression
//! - **κ** (Comparison): lookup matching
//!
//! ## Chemistry Notation
//!
//! Types are represented as molecular formulas:
//! - `σN` = sequence of quantities (like H₂O)
//! - `μ[N→N]` = mapping compound
//! - `Σ(N,∅)` = sum type (Option<N>)
//!
//! ## Tier: T2-P (μ + σ + κ)

use crate::vocabulary;
use std::collections::HashMap;

/// Word to symbol mapping.
#[derive(Debug, Clone)]
pub struct Lexicon {
    /// Word → Symbol mappings
    to_symbol: HashMap<String, String>,
    /// Symbol → Word mappings (reverse)
    to_word: HashMap<String, String>,
}

impl Default for Lexicon {
    fn default() -> Self {
        Self::new()
    }
}

impl Lexicon {
    /// Create lexicon with all Prima mappings.
    ///
    /// Loads from [`vocabulary`] — the single source of truth.
    #[must_use]
    pub fn new() -> Self {
        let mut lex = Self {
            to_symbol: HashMap::new(),
            to_word: HashMap::new(),
        };
        // Load all vocabulary groups (canonical + aliases)
        for group in vocabulary::ALL_COMPRESS {
            for (word, sym) in *group {
                lex.add(word, sym);
            }
        }
        lex
    }

    /// Add a bidirectional mapping.
    fn add(&mut self, word: &str, symbol: &str) {
        self.to_symbol
            .insert(word.to_lowercase(), symbol.to_string());
        self.to_word.insert(symbol.to_string(), word.to_string());
    }

    /// Translate word to symbol.
    #[must_use]
    pub fn compress(&self, word: &str) -> Option<&str> {
        self.to_symbol.get(&word.to_lowercase()).map(String::as_str)
    }

    /// Translate symbol to word.
    #[must_use]
    pub fn expand(&self, symbol: &str) -> Option<&str> {
        self.to_word.get(symbol).map(String::as_str)
    }

    /// Compress a sentence to symbols.
    #[must_use]
    pub fn compress_text(&self, text: &str) -> String {
        text.split_whitespace()
            .map(|w| self.compress(w).unwrap_or(w))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Count tokens saved by compression.
    #[must_use]
    pub fn compression_ratio(&self, text: &str) -> f64 {
        let original = text.len();
        let compressed = self.compress_text(text).len();
        if original == 0 {
            return 1.0;
        }
        compressed as f64 / original as f64
    }
}

// ============================================================================
// Molecular Formula System
// ============================================================================

/// A molecular formula representing a type composition.
///
/// Uses chemistry notation:
/// - `σN` = sequence of N (like H₂)
/// - `σ²N` = nested sequence
/// - `μ[N→N]` = mapping compound
/// - `Σ(N,∅)` = sum (Option)
#[derive(Debug, Clone)]
pub struct MolecularFormula {
    /// Elements in the formula.
    pub elements: Vec<Element>,
}

/// An element in a molecular formula.
#[derive(Debug, Clone)]
pub struct Element {
    /// The primitive symbol.
    pub symbol: String,
    /// Subscript (count/multiplicity).
    pub subscript: u32,
    /// Superscript (nesting level).
    pub superscript: u32,
    /// Inner formula (for compounds).
    pub inner: Option<Box<MolecularFormula>>,
}

impl Element {
    /// Create a simple element.
    #[must_use]
    pub fn simple(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            subscript: 1,
            superscript: 0,
            inner: None,
        }
    }

    /// Create with subscript.
    #[must_use]
    pub fn with_subscript(symbol: &str, n: u32) -> Self {
        Self {
            symbol: symbol.to_string(),
            subscript: n,
            superscript: 0,
            inner: None,
        }
    }

    /// Create a compound element.
    #[must_use]
    pub fn compound(symbol: &str, inner: MolecularFormula) -> Self {
        Self {
            symbol: symbol.to_string(),
            subscript: 1,
            superscript: 0,
            inner: Some(Box::new(inner)),
        }
    }
}

impl MolecularFormula {
    /// Create empty formula.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Create from elements.
    #[must_use]
    pub fn from_elements(elements: Vec<Element>) -> Self {
        Self { elements }
    }

    /// Add an element.
    pub fn add(&mut self, element: Element) {
        self.elements.push(element);
    }

    /// Format as chemical formula string.
    #[must_use]
    pub fn to_formula(&self) -> String {
        self.elements.iter().map(format_element).collect()
    }

    /// Calculate molecular weight (primitive count).
    #[must_use]
    pub fn weight(&self) -> u32 {
        self.elements.iter().map(element_weight).sum()
    }

    /// Get tier based on weight.
    #[must_use]
    pub fn tier(&self) -> &'static str {
        match self.weight() {
            1 => "T1",
            2..=3 => "T2-P",
            4..=5 => "T2-C",
            _ => "T3",
        }
    }
}

impl Default for MolecularFormula {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a single element.
fn format_element(elem: &Element) -> String {
    let mut s = elem.symbol.clone();
    if elem.superscript > 0 {
        s.push_str(&superscript_str(elem.superscript));
    }
    if let Some(inner) = &elem.inner {
        s.push('[');
        s.push_str(&inner.to_formula());
        s.push(']');
    }
    if elem.subscript > 1 {
        s.push_str(&subscript_str(elem.subscript));
    }
    s
}

/// Calculate element weight.
fn element_weight(elem: &Element) -> u32 {
    let base = elem.subscript;
    let inner = elem.inner.as_ref().map_or(0, |f| f.weight());
    base + inner
}

/// Convert to subscript characters.
fn subscript_str(n: u32) -> String {
    const SUBS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
    n.to_string()
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| SUBS[d as usize]))
        .collect()
}

/// Convert to superscript characters.
fn superscript_str(n: u32) -> String {
    const SUPS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    n.to_string()
        .chars()
        .filter_map(|c| c.to_digit(10).map(|d| SUPS[d as usize]))
        .collect()
}

// ============================================================================
// Common Molecular Formulas
// ============================================================================

/// Create formula for σ[N] (sequence of quantities).
#[must_use]
pub fn seq_n() -> MolecularFormula {
    MolecularFormula::from_elements(vec![Element::compound(
        "σ",
        MolecularFormula::from_elements(vec![Element::simple("N")]),
    )])
}

/// Create formula for μ[A→B] (mapping).
#[must_use]
pub fn mapping(from: &str, to: &str) -> MolecularFormula {
    let inner = MolecularFormula::from_elements(vec![
        Element::simple(from),
        Element::simple("→"),
        Element::simple(to),
    ]);
    MolecularFormula::from_elements(vec![Element::compound("μ", inner)])
}

/// Create formula for Σ(A,B) (sum type).
#[must_use]
pub fn sum_type(a: &str, b: &str) -> MolecularFormula {
    let inner = MolecularFormula::from_elements(vec![Element::simple(a), Element::simple(b)]);
    MolecularFormula::from_elements(vec![Element::compound("Σ", inner)])
}

/// Create formula for Option<T> = Σ(T,∅).
#[must_use]
pub fn option_type(t: &str) -> MolecularFormula {
    sum_type(t, "∅")
}

/// Create formula for Result<T,E> = Σ(T,∂).
#[must_use]
pub fn result_type(t: &str) -> MolecularFormula {
    sum_type(t, "∂")
}

// ============================================================================
// Token Transfer Calculations
// ============================================================================

/// Calculate token savings from compression.
#[derive(Debug, Clone)]
pub struct TokenMetrics {
    /// Original token count (words).
    pub original_tokens: usize,
    /// Compressed token count (symbols).
    pub compressed_tokens: usize,
    /// Bytes saved.
    pub bytes_saved: usize,
    /// Compression ratio.
    pub ratio: f64,
}

impl TokenMetrics {
    /// Calculate metrics for text.
    #[must_use]
    pub fn calculate(lexicon: &Lexicon, text: &str) -> Self {
        let original = text.split_whitespace().count();
        let compressed_text = lexicon.compress_text(text);
        let compressed = compressed_text.split_whitespace().count();
        let bytes_saved = text.len().saturating_sub(compressed_text.len());
        let ratio = if original > 0 {
            compressed as f64 / original as f64
        } else {
            1.0
        };
        Self {
            original_tokens: original,
            compressed_tokens: compressed,
            bytes_saved,
            ratio,
        }
    }

    /// Format as report.
    #[must_use]
    pub fn report(&self) -> String {
        format!(
            "Tokens: {} → {} ({:.0}% of original)\nBytes saved: {}",
            self.original_tokens,
            self.compressed_tokens,
            self.ratio * 100.0,
            self.bytes_saved
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexicon_compress() {
        let lex = Lexicon::new();
        assert_eq!(lex.compress("map"), Some("Φ"));
        assert_eq!(lex.compress("filter"), Some("Ψ"));
        assert_eq!(lex.compress("fold"), Some("Ω"));
    }

    #[test]
    fn test_lexicon_expand() {
        let lex = Lexicon::new();
        assert_eq!(lex.expand("Φ"), Some("map"));
        assert_eq!(lex.expand("Ψ"), Some("filter"));
        // "reduce" is added after "fold", so it's the canonical reverse
        assert_eq!(lex.expand("Ω"), Some("reduce"));
    }

    #[test]
    fn test_compress_text() {
        let lex = Lexicon::new();
        let text = "let x equals map sequence";
        let compressed = lex.compress_text(text);
        assert!(compressed.contains("λ"));
        assert!(compressed.contains("Φ"));
    }

    #[test]
    fn test_compression_ratio() {
        let lex = Lexicon::new();
        let ratio = lex.compression_ratio("map filter fold");
        assert!(ratio < 1.0); // Should compress
    }

    #[test]
    fn test_molecular_formula_simple() {
        let mut formula = MolecularFormula::new();
        formula.add(Element::simple("N"));
        assert_eq!(formula.to_formula(), "N");
        assert_eq!(formula.weight(), 1);
        assert_eq!(formula.tier(), "T1");
    }

    #[test]
    fn test_molecular_formula_subscript() {
        let formula = MolecularFormula::from_elements(vec![Element::with_subscript("N", 3)]);
        assert_eq!(formula.to_formula(), "N₃");
    }

    #[test]
    fn test_seq_n_formula() {
        let formula = seq_n();
        assert_eq!(formula.to_formula(), "σ[N]");
        assert_eq!(formula.tier(), "T2-P");
    }

    #[test]
    fn test_mapping_formula() {
        let formula = mapping("N", "N");
        assert_eq!(formula.to_formula(), "μ[N→N]");
    }

    #[test]
    fn test_option_formula() {
        let formula = option_type("N");
        assert_eq!(formula.to_formula(), "Σ[N∅]");
    }

    #[test]
    fn test_result_formula() {
        let formula = result_type("N");
        assert_eq!(formula.to_formula(), "Σ[N∂]");
    }

    #[test]
    fn test_token_metrics() {
        let lex = Lexicon::new();
        let metrics = TokenMetrics::calculate(&lex, "map filter fold");
        assert_eq!(metrics.original_tokens, 3);
        assert!(metrics.bytes_saved > 0);
    }

    #[test]
    fn test_subscript_str() {
        assert_eq!(subscript_str(0), "₀");
        assert_eq!(subscript_str(1), "₁");
        assert_eq!(subscript_str(12), "₁₂");
    }

    #[test]
    fn test_superscript_str() {
        assert_eq!(superscript_str(0), "⁰");
        assert_eq!(superscript_str(2), "²");
        assert_eq!(superscript_str(10), "¹⁰");
    }

    #[test]
    fn test_keywords_compress() {
        let lex = Lexicon::new();
        assert_eq!(lex.compress("let"), Some("λ"));
        assert_eq!(lex.compress("fn"), Some("μ"));
        assert_eq!(lex.compress("if"), Some("∂"));
    }

    #[test]
    fn test_builtins_compress() {
        let lex = Lexicon::new();
        assert_eq!(lex.compress("print"), Some("ω"));
        assert_eq!(lex.compress("len"), Some("#"));
        assert_eq!(lex.compress("head"), Some("↑"));
    }

    #[test]
    fn test_case_insensitive() {
        let lex = Lexicon::new();
        assert_eq!(lex.compress("MAP"), Some("Φ"));
        assert_eq!(lex.compress("Map"), Some("Φ"));
        assert_eq!(lex.compress("map"), Some("Φ"));
    }

    #[test]
    fn test_formula_weight_nested() {
        // σ[μ[N→N]] should have weight 4 (σ + μ + N + N + →)
        let inner = mapping("N", "N");
        let outer = MolecularFormula::from_elements(vec![Element::compound("σ", inner)]);
        assert!(outer.weight() >= 3);
    }
}
