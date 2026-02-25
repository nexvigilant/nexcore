//! # Primitive Extraction
//!
//! Extract Lex Primitiva from text, code, or domain concepts.
//!
//! ## Tier: T2-C (Sequence + Mapping + Comparison)

use crate::primitiva::{LexPrimitiva, PrimitiveComposition};
use crate::tier::Tier;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Result of extracting primitives from input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExtractionResult {
    /// The input that was analyzed.
    pub input: String,
    /// Extracted primitives with confidence scores.
    pub primitives: Vec<ExtractedPrimitive>,
    /// The resulting composition.
    pub composition: PrimitiveComposition,
    /// Overall extraction confidence.
    pub confidence: f64,
    /// Tier classification.
    pub tier: Tier,
}

/// A single extracted primitive with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExtractedPrimitive {
    /// The primitive that was extracted.
    pub primitive: LexPrimitiva,
    /// Confidence in this extraction (0.0-1.0).
    pub confidence: f64,
    /// Evidence that led to this extraction.
    pub evidence: Vec<String>,
    /// Whether this is the dominant primitive.
    pub is_dominant: bool,
}

/// Keyword patterns for primitive detection.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PrimitivePatterns {
    /// Keyword patterns keyed by primitive variant.
    pub patterns: BTreeMap<LexPrimitiva, Vec<&'static str>>,
}

impl Default for PrimitivePatterns {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimitivePatterns {
    /// Creates the default pattern set.
    #[must_use]
    pub fn new() -> Self {
        let mut patterns: BTreeMap<LexPrimitiva, Vec<&'static str>> = BTreeMap::new();
        Self::add_sequence_patterns(&mut patterns);
        Self::add_mapping_patterns(&mut patterns);
        Self::add_state_patterns(&mut patterns);
        Self::add_recursion_patterns(&mut patterns);
        Self::add_void_patterns(&mut patterns);
        Self::add_boundary_patterns(&mut patterns);
        Self::add_frequency_patterns(&mut patterns);
        Self::add_existence_patterns(&mut patterns);
        Self::add_persistence_patterns(&mut patterns);
        Self::add_causality_patterns(&mut patterns);
        Self::add_comparison_patterns(&mut patterns);
        Self::add_quantity_patterns(&mut patterns);
        Self::add_location_patterns(&mut patterns);
        Self::add_irreversibility_patterns(&mut patterns);
        Self::add_sum_patterns(&mut patterns);
        Self::add_product_patterns(&mut patterns);
        Self { patterns }
    }

    fn add_sequence_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Sequence,
            vec![
                "sequence", "order", "list", "array", "iterator", "chain", "step", "next",
                "previous", "first", "last", "index", "iterate", "loop", "for", "while",
                "pipeline", "stream",
            ],
        );
    }

    fn add_mapping_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Mapping,
            vec![
                "map",
                "transform",
                "convert",
                "translate",
                "from",
                "into",
                "function",
                "morphism",
                "functor",
                "apply",
                "project",
            ],
        );
    }

    fn add_state_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::State,
            vec![
                "state",
                "store",
                "variable",
                "mutable",
                "context",
                "session",
                "memory",
                "cache",
                "buffer",
                "container",
                "holder",
            ],
        );
    }

    fn add_recursion_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Recursion,
            vec![
                "recursive",
                "recursion",
                "self-reference",
                "induction",
                "fibonacci",
                "factorial",
                "tree",
                "nested",
                "fractal",
            ],
        );
    }

    fn add_void_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Void,
            vec![
                "void",
                "null",
                "none",
                "empty",
                "absent",
                "nothing",
                "nil",
                "undefined",
                "missing",
                "optional",
                "maybe",
            ],
        );
    }

    fn add_boundary_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Boundary,
            vec![
                "boundary",
                "limit",
                "constraint",
                "bound",
                "edge",
                "threshold",
                "error",
                "exception",
                "result",
                "validate",
                "check",
                "guard",
            ],
        );
    }

    fn add_frequency_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Frequency,
            vec![
                "frequency",
                "rate",
                "period",
                "cycle",
                "oscillate",
                "periodic",
                "hertz",
                "interval",
                "sampling",
                "throttle",
                "debounce",
            ],
        );
    }

    fn add_existence_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Existence,
            vec![
                "exist",
                "create",
                "new",
                "instantiate",
                "construct",
                "spawn",
                "allocate",
                "initialize",
                "witness",
                "prove",
                "assert",
            ],
        );
    }

    fn add_persistence_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Persistence,
            vec![
                "persist",
                "save",
                "store",
                "database",
                "file",
                "disk",
                "durable",
                "permanent",
                "log",
                "journal",
                "snapshot",
            ],
        );
    }

    fn add_causality_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Causality,
            vec![
                "cause",
                "effect",
                "trigger",
                "event",
                "callback",
                "handler",
                "if",
                "then",
                "when",
                "because",
                "therefore",
                "implies",
            ],
        );
    }

    fn add_comparison_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Comparison,
            vec![
                "compare",
                "equal",
                "match",
                "differ",
                "greater",
                "less",
                "sort",
                "order",
                "rank",
                "predicate",
                "filter",
                "select",
            ],
        );
    }

    fn add_quantity_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Quantity,
            vec![
                "quantity",
                "number",
                "count",
                "measure",
                "amount",
                "size",
                "length",
                "magnitude",
                "integer",
                "float",
                "numeric",
            ],
        );
    }

    fn add_location_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Location,
            vec![
                "location",
                "address",
                "position",
                "coordinate",
                "path",
                "pointer",
                "reference",
                "url",
                "uri",
                "index",
                "offset",
            ],
        );
    }

    fn add_irreversibility_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Irreversibility,
            vec![
                "irreversible",
                "consume",
                "drop",
                "destroy",
                "move",
                "take",
                "ownership",
                "linear",
                "affine",
                "once",
                "final",
            ],
        );
    }

    fn add_sum_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Sum,
            vec![
                "sum",
                "union",
                "variant",
                "enum",
                "either",
                "choice",
                "alternative",
                "case",
                "match",
                "discriminant",
                "tag",
            ],
        );
    }

    fn add_product_patterns(p: &mut BTreeMap<LexPrimitiva, Vec<&'static str>>) {
        p.insert(
            LexPrimitiva::Product,
            vec![
                "product",
                "struct",
                "tuple",
                "record",
                "pair",
                "combine",
                "field",
                "projection",
                "zip",
                "component",
                "arity",
            ],
        );
    }

    /// Get patterns for a primitive.
    #[must_use]
    pub fn get(&self, primitive: LexPrimitiva) -> &[&'static str] {
        self.patterns
            .get(&primitive)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Primitive extractor for analyzing text.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PrimitiveExtractor {
    patterns: PrimitivePatterns,
    /// Minimum confidence threshold for inclusion.
    pub min_confidence: f64,
}

impl Default for PrimitiveExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimitiveExtractor {
    /// Creates a new extractor with default patterns.
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: PrimitivePatterns::new(),
            min_confidence: 0.1,
        }
    }

    /// Creates an extractor with custom confidence threshold.
    #[must_use]
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            patterns: PrimitivePatterns::new(),
            min_confidence: threshold.clamp(0.0, 1.0),
        }
    }

    /// Extract primitives from text.
    #[must_use]
    pub fn extract(&self, input: &str) -> ExtractionResult {
        let lower = input.to_lowercase();
        #[allow(
            clippy::as_conversions,
            reason = "word_count bounded by input length; safe cast to f64"
        )]
        let word_count = lower.split_whitespace().count().max(1) as f64;
        let scores = self.score_all_primitives(&lower, word_count);
        let primitives = self.build_primitive_list(scores);
        self.build_result(input, primitives)
    }

    /// Score all primitives against input text.
    fn score_all_primitives(
        &self,
        lower: &str,
        word_count: f64,
    ) -> BTreeMap<LexPrimitiva, (f64, Vec<String>)> {
        let mut scores = BTreeMap::new();
        for primitive in LexPrimitiva::all() {
            if let Some(score) = self.score_primitive(primitive, lower, word_count) {
                scores.insert(primitive, score);
            }
        }
        scores
    }

    /// Score a single primitive.
    fn score_primitive(
        &self,
        primitive: LexPrimitiva,
        lower: &str,
        word_count: f64,
    ) -> Option<(f64, Vec<String>)> {
        let patterns = self.patterns.get(primitive);
        let matches: Vec<String> = patterns
            .iter()
            .filter(|p| lower.contains(*p))
            .map(|p| (*p).to_string())
            .collect();

        if matches.is_empty() {
            return None;
        }

        #[allow(
            clippy::as_conversions,
            reason = "matches.len() and patterns.len() bounded by small constants; safe cast to f64"
        )]
        let match_score = matches.len() as f64 / patterns.len().max(1) as f64;
        #[allow(
            clippy::as_conversions,
            reason = "matches.len() bounded by small constants; safe cast to f64"
        )]
        let density_score = matches.len() as f64 / word_count;
        let confidence = (match_score * 0.6 + density_score * 0.4).min(1.0);
        Some((confidence, matches))
    }

    /// Build sorted primitive list from scores.
    fn build_primitive_list(
        &self,
        scores: BTreeMap<LexPrimitiva, (f64, Vec<String>)>,
    ) -> Vec<ExtractedPrimitive> {
        let mut primitives: Vec<ExtractedPrimitive> = scores
            .iter()
            .filter(|(_, (conf, _))| *conf >= self.min_confidence)
            .map(|(prim, (conf, evidence))| ExtractedPrimitive {
                primitive: *prim,
                confidence: *conf,
                evidence: evidence.clone(),
                is_dominant: false,
            })
            .collect();

        primitives.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(first) = primitives.first_mut() {
            first.is_dominant = true;
        }
        primitives
    }

    /// Build final extraction result.
    fn build_result(&self, input: &str, primitives: Vec<ExtractedPrimitive>) -> ExtractionResult {
        let prim_list: Vec<LexPrimitiva> = primitives.iter().map(|p| p.primitive).collect();
        let dominant = primitives.first().map(|p| p.primitive);
        #[allow(
            clippy::as_conversions,
            reason = "primitives.len() bounded by 16; safe cast to f64"
        )]
        let confidence =
            primitives.iter().map(|p| p.confidence).sum::<f64>() / primitives.len().max(1) as f64;

        let composition = PrimitiveComposition::new(prim_list);
        let composition = match dominant {
            Some(dom) => composition.with_dominant(dom, confidence),
            None => composition,
        };
        let tier = Tier::classify(&composition);

        ExtractionResult {
            input: input.to_string(),
            primitives,
            composition,
            confidence,
            tier,
        }
    }

    /// Extract primitives from a Rust type name.
    #[must_use]
    pub fn extract_from_type(&self, type_name: &str) -> ExtractionResult {
        self.extract(&Self::enhance_type_name(type_name))
    }

    /// Enhance a type name with semantic context.
    fn enhance_type_name(type_name: &str) -> String {
        let mut result = type_name.to_string();
        let hints = [
            ("Vec", " sequence list array"),
            ("Array", " sequence list array"),
            ("HashMap", " mapping transform key value"),
            ("BTreeMap", " mapping transform"),
            ("Option", " void null maybe optional"),
            ("Result", " boundary error constraint"),
            ("Box", " location pointer reference"),
            ("Rc", " location pointer reference"),
            ("Arc", " location pointer reference"),
            ("Mutex", " state mutable concurrent"),
            ("RwLock", " state mutable concurrent"),
            ("Iterator", " sequence iterate stream"),
            ("Iter", " sequence iterate"),
            ("Fn", " mapping function causality"),
        ];
        for (pattern, hint) in hints {
            if type_name.contains(pattern) {
                result.push_str(hint);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sequence() {
        let extractor = PrimitiveExtractor::new();
        let result = extractor.extract("iterate through the list in order");
        assert!(
            result
                .primitives
                .iter()
                .any(|p| p.primitive == LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn test_extract_mapping() {
        let extractor = PrimitiveExtractor::new();
        let result = extractor.extract("transform the input into output");
        assert!(
            result
                .primitives
                .iter()
                .any(|p| p.primitive == LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn test_extract_void() {
        let extractor = PrimitiveExtractor::new();
        let result = extractor.extract("handle the null or empty case");
        assert!(
            result
                .primitives
                .iter()
                .any(|p| p.primitive == LexPrimitiva::Void)
        );
    }

    #[test]
    fn test_extract_from_type_vec() {
        let extractor = PrimitiveExtractor::new();
        let result = extractor.extract_from_type("Vec<String>");
        assert!(
            result
                .primitives
                .iter()
                .any(|p| p.primitive == LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn test_extract_from_type_option() {
        let extractor = PrimitiveExtractor::new();
        let result = extractor.extract_from_type("Option<i32>");
        assert!(
            result
                .primitives
                .iter()
                .any(|p| p.primitive == LexPrimitiva::Void)
        );
    }

    #[test]
    fn test_dominant_marking() {
        let extractor = PrimitiveExtractor::new();
        let result = extractor.extract("iterate iterate iterate through the sequence");
        let dominant_count = result.primitives.iter().filter(|p| p.is_dominant).count();
        assert_eq!(dominant_count, 1);
    }

    #[test]
    fn test_confidence_threshold() {
        let strict = PrimitiveExtractor::with_threshold(0.5);
        let lenient = PrimitiveExtractor::with_threshold(0.05);
        let strict_result = strict.extract("maybe consider the possibility");
        let lenient_result = lenient.extract("maybe consider the possibility");
        assert!(lenient_result.primitives.len() >= strict_result.primitives.len());
    }

    #[test]
    fn test_empty_input() {
        let extractor = PrimitiveExtractor::new();
        let result = extractor.extract("");
        assert!(result.primitives.is_empty());
        assert_eq!(result.tier, Tier::T1Universal);
    }

    #[test]
    fn test_patterns_coverage() {
        let patterns = PrimitivePatterns::new();
        for prim in LexPrimitiva::all() {
            assert!(!patterns.get(prim).is_empty(), "{:?} has no patterns", prim);
        }
    }
}
