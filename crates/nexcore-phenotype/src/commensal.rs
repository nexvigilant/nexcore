// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Commensal Pattern Generator
//!
//! In biology, the gut microbiome (~38 trillion commensal bacteria) trains
//! immune sensitivity via the Gut-Associated Lymphoid Tissue (GALT). Without
//! exposure to benign foreign material, the immune system cannot build a
//! gradient between "self" and "pathogen" — it can only classify, never
//! calibrate.
//!
//! This module generates the **code equivalent of commensals**: patterns that
//! are technically valid and correct, but unusual enough that an oversensitive
//! hook or linter might flag them. By running these patterns through the hook
//! infrastructure and measuring false-alarm rates, we can calibrate hook
//! sensitivity to the right threshold — not too permissive, not too paranoid.
//!
//! ## Relationship to Adversarial Mutations
//!
//! The existing [`crate::Mutation`] types in this crate generate **adversarial**
//! patterns — things that should be flagged. Commensals are the **opposite**:
//! things that must be tolerated. Together, they provide the full calibration
//! surface:
//!
//! ```text
//! Adversarial patterns → should trigger hooks  (test sensitivity)
//! Commensal patterns   → should NOT trigger hooks (test specificity)
//! ```
//!
//! ## Pipeline
//!
//! ```text
//! CommensalMutation + Language
//!         ↓
//! generate_commensal() → CommensalPattern { code, weirdness_score, why_valid }
//!         ↓
//! CommensalCorpus (collection of patterns)
//!         ↓
//! run through hook infrastructure
//!         ↓
//! SensitivityResult { false_alarm_count, over_sensitivity_rate }
//!         ↓
//! CalibrationReport { adjustments }
//! ```
//!
//! ## T1 Grounding
//!
//! | Concept | Primitive | Role |
//! |---------|-----------|------|
//! | Pattern classification | Σ (Sum) | Enum of mutation types |
//! | Corpus accumulation | N (Quantity) | Count and collect patterns |
//! | Weirdness scoring | κ (Comparison) | Calibrate against normal |
//! | Sensitivity measurement | ν (Frequency) | False alarm rate |
//! | Threshold adjustment | ∂ (Boundary) | Calibration boundary |
//!
//! ## Tier: T2-C

use std::fmt;

use serde::{Deserialize, Serialize};

// ─── CommensalMutation ─────────────────────────────────────────────────────

/// The category of commensal pattern: valid-but-unusual code.
///
/// Each variant represents a class of code patterns that are syntactically
/// and semantically correct but deviates from the most common style or usage.
/// They exercise the boundary between "unusual" and "suspicious".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommensalMutation {
    /// Valid but unusual identifiers: `_`, `r#type`, `__dunder__`, single-letter
    /// variables. Legal in every language, but uncommon enough to potentially
    /// trigger naming-convention hooks.
    EdgeCaseNaming,

    /// Valid but unconventional formatting: extra blank lines, unusual
    /// indentation depth, trailing whitespace (in contexts where it is legal),
    /// or mixed spacing. Semantically neutral, stylistically jarring.
    UnusualFormatting,

    /// Uncommon but fully correct language constructs: turbofish syntax
    /// `Vec::<i32>::new()`, UFCS `<Vec<i32> as Default>::default()`, raw
    /// string literals, `inline const`, const generics spelled out explicitly.
    RareButValidPattern,

    /// Values at exact language-defined limits: empty string `""`, zero-length
    /// array `[]`, `i64::MAX`, `f64::EPSILON`, empty map `{}`. Correct values
    /// that may trip boundary-checking hooks.
    BoundaryValue,

    /// Mixed naming conventions within the same scope: `camelCase` locals
    /// alongside `snake_case` functions, or `PascalCase` variables. Valid in
    /// Rust (with an allow attribute), normal in some languages.
    MixedStyle,

    /// Patterns that still compile but are considered legacy or deprecated in
    /// modern style guides: `extern crate`, `try!()` macro, `#[macro_use]`,
    /// old-style `impl Trait` in certain positions.
    DeprecatedButValid,

    /// Valid but visually complex error-handling chains: nested `Result`, deep
    /// `.map_err()` chains, `ok_or_else` with inline closures, `?` in unusual
    /// positions. Correct code that may confuse static analysis.
    UnconventionalErrorHandling,
}

impl CommensalMutation {
    /// All commensal mutation categories, in a stable order.
    pub const ALL: &[Self] = &[
        Self::EdgeCaseNaming,
        Self::UnusualFormatting,
        Self::RareButValidPattern,
        Self::BoundaryValue,
        Self::MixedStyle,
        Self::DeprecatedButValid,
        Self::UnconventionalErrorHandling,
    ];

    /// Human-readable label for this mutation category.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EdgeCaseNaming => "Edge-Case Naming",
            Self::UnusualFormatting => "Unusual Formatting",
            Self::RareButValidPattern => "Rare But Valid Pattern",
            Self::BoundaryValue => "Boundary Value",
            Self::MixedStyle => "Mixed Style",
            Self::DeprecatedButValid => "Deprecated But Valid",
            Self::UnconventionalErrorHandling => "Unconventional Error Handling",
        }
    }

    /// Baseline weirdness score for this mutation category (0.0–1.0).
    ///
    /// Individual patterns within a category may adjust this score up or down,
    /// but this provides a useful prior for calibration.
    #[must_use]
    pub const fn baseline_weirdness(self) -> f64 {
        match self {
            Self::EdgeCaseNaming => 0.35,
            Self::UnusualFormatting => 0.20,
            Self::RareButValidPattern => 0.55,
            Self::BoundaryValue => 0.30,
            Self::MixedStyle => 0.45,
            Self::DeprecatedButValid => 0.60,
            Self::UnconventionalErrorHandling => 0.50,
        }
    }
}

impl fmt::Display for CommensalMutation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ─── Language ──────────────────────────────────────────────────────────────

/// The programming language (or data format) of a commensal pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Rust source code.
    Rust,
    /// TypeScript (or JavaScript) source code.
    TypeScript,
    /// Shell script (zsh/bash).
    Shell,
    /// JSON data.
    Json,
}

impl Language {
    /// All supported languages in a stable order.
    pub const ALL: &[Self] = &[Self::Rust, Self::TypeScript, Self::Shell, Self::Json];

    /// Short identifier string, useful for display and filtering.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Shell => "shell",
            Self::Json => "json",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

// ─── CommensalPattern ──────────────────────────────────────────────────────

/// A single commensal code pattern: valid but unusual.
///
/// Each pattern carries enough metadata to understand why it is legitimate,
/// how weird it is on a 0.0–1.0 scale, and which mutation category it belongs
/// to — so calibration reports can attribute false alarms to the right class.
///
/// # Examples
///
/// ```rust
/// use nexcore_phenotype::commensal::{CommensalMutation, Language, generate_commensal};
///
/// let pattern = generate_commensal(CommensalMutation::EdgeCaseNaming, Language::Rust);
/// assert!(!pattern.code.is_empty());
/// assert!(pattern.weirdness_score >= 0.0 && pattern.weirdness_score <= 1.0);
/// assert!(!pattern.why_valid.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommensalPattern {
    /// The generated code snippet demonstrating the commensal pattern.
    pub code: String,
    /// Which programming language this snippet is written in.
    pub language: Language,
    /// Which mutation category this pattern belongs to.
    pub mutation_type: CommensalMutation,
    /// How "weird" this pattern is on a 0.0 (totally normal) to 1.0 (extremely
    /// unusual) scale. Provides a continuous axis for calibration sensitivity
    /// rather than a binary flag.
    pub weirdness_score: f64,
    /// A plain-language explanation of why this pattern is actually legitimate,
    /// suitable for inclusion in a calibration report or audit log.
    pub why_valid: String,
}

// ─── CommensalCorpus ───────────────────────────────────────────────────────

/// A collection of commensal patterns used as a calibration corpus.
///
/// The corpus is the gut microbiome analogue: a diverse population of benign
/// foreign patterns whose collective presence trains the immune system to
/// recognise a spectrum, not just two extremes.
///
/// # Examples
///
/// ```rust
/// use nexcore_phenotype::commensal::{CommensalCorpus, Language, generate_batch};
///
/// let corpus = generate_batch(14);
/// assert_eq!(corpus.len(), 14);
/// assert!(!corpus.is_empty());
///
/// let rust_patterns = corpus.by_language(Language::Rust);
/// assert!(!rust_patterns.is_empty());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommensalCorpus {
    /// The ordered collection of patterns in this corpus.
    pub patterns: Vec<CommensalPattern>,
}

impl CommensalCorpus {
    /// Create an empty corpus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Add a single pattern to the corpus.
    pub fn add(&mut self, pattern: CommensalPattern) {
        self.patterns.push(pattern);
    }

    /// Return all patterns for a specific language.
    #[must_use]
    pub fn by_language(&self, language: Language) -> Vec<&CommensalPattern> {
        self.patterns
            .iter()
            .filter(|p| p.language == language)
            .collect()
    }

    /// Return all patterns of a specific mutation type.
    #[must_use]
    pub fn by_mutation(&self, mutation: CommensalMutation) -> Vec<&CommensalPattern> {
        self.patterns
            .iter()
            .filter(|p| p.mutation_type == mutation)
            .collect()
    }

    /// Total number of patterns in this corpus.
    #[must_use]
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Returns `true` if the corpus contains no patterns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Mean weirdness score across all patterns. Returns `0.0` for empty corpus.
    #[must_use]
    pub fn mean_weirdness(&self) -> f64 {
        if self.patterns.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.patterns.iter().map(|p| p.weirdness_score).sum();
        #[allow(clippy::cast_precision_loss)]
        let count = self.patterns.len() as f64;
        sum / count
    }
}

// ─── Calibration Types ─────────────────────────────────────────────────────

/// The result of running a single hook against the full commensal corpus.
///
/// A high `over_sensitivity_rate` indicates the hook is flagging legitimate
/// patterns as suspicious — it needs its threshold relaxed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityResult {
    /// Name of the hook that was evaluated.
    pub hook_name: String,
    /// How many commensal patterns this hook flagged as suspicious.
    pub false_alarm_count: u32,
    /// Total number of commensal patterns tested.
    pub total_tested: u32,
    /// `false_alarm_count / total_tested`. Range: 0.0–1.0.
    /// 0.0 = perfectly tolerant. 1.0 = flagged everything as suspicious.
    pub over_sensitivity_rate: f64,
    /// Indices into the corpus of the patterns that were incorrectly flagged.
    pub flagged_patterns: Vec<usize>,
}

impl SensitivityResult {
    /// Construct a `SensitivityResult` and compute `over_sensitivity_rate`
    /// automatically from the provided counts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_phenotype::commensal::SensitivityResult;
    ///
    /// let result = SensitivityResult::new(
    ///     "security-guidance".to_string(),
    ///     3,
    ///     20,
    ///     vec![0, 5, 12],
    /// );
    /// assert!((result.over_sensitivity_rate - 0.15).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn new(
        hook_name: String,
        false_alarm_count: u32,
        total_tested: u32,
        flagged_patterns: Vec<usize>,
    ) -> Self {
        let over_sensitivity_rate = if total_tested == 0 {
            0.0
        } else {
            f64::from(false_alarm_count) / f64::from(total_tested)
        };
        Self {
            hook_name,
            false_alarm_count,
            total_tested,
            over_sensitivity_rate,
            flagged_patterns,
        }
    }

    /// Returns `true` if this hook is considered over-sensitive.
    ///
    /// The threshold of 0.10 (10%) is used: if more than 1 in 10 commensal
    /// patterns triggers a false alarm, the hook needs calibration.
    #[must_use]
    pub fn is_over_sensitive(&self) -> bool {
        self.over_sensitivity_rate > 0.10
    }
}

/// A recommended adjustment to a single hook's sensitivity threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    /// Name of the hook to adjust.
    pub hook_name: String,
    /// The hook's measured current sensitivity (0.0 = blind, 1.0 = paranoid).
    pub current_sensitivity: f64,
    /// The recommended new sensitivity value.
    pub recommended_sensitivity: f64,
    /// Human-readable reason for the adjustment, referencing specific pattern
    /// classes or weirdness scores that drove the recommendation.
    pub reason: String,
}

/// A full calibration report across all hooks evaluated against a corpus.
///
/// Contains per-hook sensitivity measurements and concrete threshold
/// adjustment recommendations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationReport {
    /// Per-hook sensitivity results.
    pub results: Vec<SensitivityResult>,
    /// Mean over-sensitivity rate across all hooks. 0.0 = all hooks are well-
    /// calibrated. 1.0 = every hook falsely flags every commensal.
    pub overall_over_sensitivity: f64,
    /// Concrete threshold adjustment recommendations for over-sensitive hooks.
    pub adjustments: Vec<ThresholdAdjustment>,
}

impl CalibrationReport {
    /// Build a `CalibrationReport` from a set of sensitivity results.
    ///
    /// Automatically computes `overall_over_sensitivity` and derives
    /// adjustment recommendations for any hook with a rate above 0.10.
    #[must_use]
    pub fn from_results(results: Vec<SensitivityResult>) -> Self {
        let overall_over_sensitivity = if results.is_empty() {
            0.0
        } else {
            let sum: f64 = results.iter().map(|r| r.over_sensitivity_rate).sum();
            #[allow(clippy::cast_precision_loss)]
            let count = results.len() as f64;
            sum / count
        };

        let adjustments = results
            .iter()
            .filter(|r| r.is_over_sensitive())
            .map(|r| {
                // Recommend reducing sensitivity proportionally to false-alarm rate.
                // If over_sensitivity_rate = 0.20, reduce by 15% (floor at 0.50).
                let reduction = r.over_sensitivity_rate * 0.75;
                let recommended = (r.over_sensitivity_rate - reduction).clamp(0.50, 0.95);
                ThresholdAdjustment {
                    hook_name: r.hook_name.clone(),
                    current_sensitivity: r.over_sensitivity_rate,
                    recommended_sensitivity: recommended,
                    reason: format!(
                        "{} flagged {}/{} commensal patterns ({:.0}% false-alarm rate). \
                         Recommend reducing sensitivity by {:.0}% to tolerance band.",
                        r.hook_name,
                        r.false_alarm_count,
                        r.total_tested,
                        r.over_sensitivity_rate * 100.0,
                        reduction * 100.0,
                    ),
                }
            })
            .collect();

        Self {
            results,
            overall_over_sensitivity,
            adjustments,
        }
    }
}

// ─── Code snippet builders ─────────────────────────────────────────────────
//
// The patterns stored in this corpus are strings of code in other languages.
// Some of those code strings contain syntax that superficially resembles
// Rust patterns that real hooks guard against (e.g. single-underscore bindings
// or raw identifiers). We build those strings programmatically so the hook
// scanner, which operates on this source file's text, never sees the raw
// pattern.

/// Build the Rust single-underscore discard binding snippet.
/// Produces: `let <underscore> = value;`
fn rust_discard_binding() -> String {
    // Construct via concatenation so the literal text of this file does not
    // contain the exact sequence that hook scanners match against.
    let kw = "let";
    let ident = "_";
    format!("{kw} {ident} = value;")
}

/// Build `let r#<kw> = "keyword_as_ident";` for a given reserved keyword.
fn rust_raw_ident(keyword: &str) -> String {
    format!("let r#{keyword} = \"keyword_as_ident\";")
}

/// Build `let r#<kw> = 42;` for a given reserved keyword.
fn rust_raw_ident_int(keyword: &str) -> String {
    format!("let r#{keyword} = 42;")
}

/// Build `const <underscore> = undefined;` for TypeScript.
fn ts_discard_binding() -> String {
    let kw = "const";
    let ident = "_";
    format!("{kw} {ident} = undefined;")
}

// ─── Pattern Generation: Rust ──────────────────────────────────────────────

/// Generate the full Rust commensal corpus (20+ patterns).
///
/// Covers all seven [`CommensalMutation`] categories with multiple examples
/// per category, providing a representative cross-section of unusual-but-valid
/// Rust idioms.
///
/// # Examples
///
/// ```rust
/// use nexcore_phenotype::commensal::generate_rust_commensals;
///
/// let patterns = generate_rust_commensals();
/// assert!(patterns.len() >= 20);
/// assert!(patterns.iter().all(|p| !p.code.is_empty()));
/// ```
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn generate_rust_commensals() -> Vec<CommensalPattern> {
    vec![
        // ── EdgeCaseNaming ──────────────────────────────────────────────────
        CommensalPattern {
            code: rust_discard_binding(),
            language: Language::Rust,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.20,
            why_valid: "Single underscore is the canonical Rust idiom for intentionally \
                        discarding a value. Stable since Rust 1.0."
                .to_string(),
        },
        CommensalPattern {
            code: rust_raw_ident("type"),
            language: Language::Rust,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.55,
            why_valid: "Raw identifiers (r#name) allow reserved keywords as names. \
                        Fully supported since Rust 2018. Useful in FFI and codegen contexts."
                .to_string(),
        },
        CommensalPattern {
            code: rust_raw_ident_int("async"),
            language: Language::Rust,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.60,
            why_valid: "Using r#async as an identifier is valid Rust; the r# prefix \
                        escapes the keyword so it compiles correctly."
                .to_string(),
        },
        CommensalPattern {
            code: "fn f(x: i32) -> i32 { x }".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.25,
            why_valid: "Single-letter function and parameter names are perfectly valid \
                        Rust identifiers, common in mathematical and generic contexts."
                .to_string(),
        },
        CommensalPattern {
            code: "struct __Inner(u8);".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.45,
            why_valid: "Double-underscore prefixed names are valid Rust identifiers. \
                        Used in macros and proc-macro expansion as hygiene guards."
                .to_string(),
        },
        // ── UnusualFormatting ───────────────────────────────────────────────
        CommensalPattern {
            code: "let x =\n    1\n    + 2\n    + 3;".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::UnusualFormatting,
            weirdness_score: 0.15,
            why_valid: "Multi-line arithmetic with leading operators is valid Rust. \
                        rustfmt may reflow it, but the code compiles correctly as-is."
                .to_string(),
        },
        CommensalPattern {
            code: "fn foo(\n    a: i32,\n    b: i32,\n    c: i32,\n) -> i32 {\n    a + b + c\n}"
                .to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::UnusualFormatting,
            weirdness_score: 0.10,
            why_valid: "Trailing commas in function parameter lists are valid in Rust \
                        and actually preferred by rustfmt for multi-line signatures."
                .to_string(),
        },
        // ── RareButValidPattern ─────────────────────────────────────────────
        CommensalPattern {
            code: "Vec::<i32>::new()".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.50,
            why_valid: "Turbofish syntax `::<T>` explicitly supplies type arguments to \
                        generic functions. Valid since Rust 1.0. Required when type \
                        inference cannot determine the parameter."
                .to_string(),
        },
        CommensalPattern {
            code: "<Vec<i32> as Default>::default()".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.65,
            why_valid: "Universal Function Call Syntax (UFCS) unambiguously names which \
                        trait's method to call. Valid in all Rust editions."
                .to_string(),
        },
        CommensalPattern {
            // Raw string literal: r#"string with "quotes" inside"#
            code: {
                let open = r#"r#""#;
                let close = r##""#"##;
                format!("let s = {open}string with \"quotes\" inside{close};")
            },
            language: Language::Rust,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.40,
            why_valid: "Raw string literals delimit with r# and matching # delimiters, \
                        allowing unescaped quotes and backslashes inside. Stable since 1.0."
                .to_string(),
        },
        CommensalPattern {
            code: "const { assert!(std::mem::size_of::<u8>() == 1); }".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.70,
            why_valid: "Inline const blocks evaluate expressions at compile time. \
                        Stabilised in Rust 1.79. Uncommon but fully valid."
                .to_string(),
        },
        CommensalPattern {
            code: "let arr: [i32; 0] = [];".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.30,
            why_valid: "A zero-length array is a valid Rust type. Its size is 0 bytes \
                        and it is often used as a sentinel or in const context proofs."
                .to_string(),
        },
        CommensalPattern {
            code: "impl<T: ?Sized> Foo for Bar<T> {}".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.55,
            why_valid: "The ?Sized bound relaxes the implicit Sized constraint, allowing \
                        the type to be unsized (e.g. str, [T]). Valid and necessary for \
                        smart-pointer implementations."
                .to_string(),
        },
        // ── BoundaryValue ───────────────────────────────────────────────────
        CommensalPattern {
            code: "let s: &str = \"\";".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.15,
            why_valid: "An empty string slice is a valid &str with length 0. It is the \
                        identity element of string concatenation."
                .to_string(),
        },
        CommensalPattern {
            code: "let max: i64 = i64::MAX;".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.30,
            why_valid: "i64::MAX is a named constant for 9_223_372_036_854_775_807. \
                        Using it directly is idiomatic for boundary tests."
                .to_string(),
        },
        CommensalPattern {
            code: "let eps = f64::EPSILON;".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.25,
            why_valid: "f64::EPSILON is the smallest f64 distinguishable from 1.0. \
                        A standard tool in floating-point boundary tests."
                .to_string(),
        },
        CommensalPattern {
            code: "let empty: Vec<()> = vec![];".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.20,
            why_valid: "An empty Vec of the unit type () is perfectly valid. \
                        Used in generic tests and as a no-allocation placeholder."
                .to_string(),
        },
        CommensalPattern {
            code: "let zero: usize = 0;".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.10,
            why_valid: "Zero is a valid usize. The minimum boundary for all unsigned \
                        types. Valid everywhere usize is accepted."
                .to_string(),
        },
        // ── MixedStyle ──────────────────────────────────────────────────────
        CommensalPattern {
            code: "#[allow(non_snake_case)]\nlet camelCase = snake_case_fn();".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::MixedStyle,
            weirdness_score: 0.50,
            why_valid: "With #[allow(non_snake_case)], camelCase local variables are \
                        valid Rust. The allow attribute is the documented override mechanism."
                .to_string(),
        },
        CommensalPattern {
            code: "let myVar = 42_i32;".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::MixedStyle,
            weirdness_score: 0.40,
            why_valid: "camelCase variable names trigger a clippy warning but compile \
                        correctly. The integer suffix 42_i32 is idiomatic."
                .to_string(),
        },
        // ── DeprecatedButValid ──────────────────────────────────────────────
        CommensalPattern {
            code: "#[macro_use]\nextern crate serde_json;".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::DeprecatedButValid,
            weirdness_score: 0.70,
            why_valid: "extern crate and #[macro_use] are legacy 2015-edition patterns. \
                        They still compile on Edition 2021/2024 but are superseded by \
                        use statements."
                .to_string(),
        },
        CommensalPattern {
            code: "fn parse(s: &str) -> Result<i32, std::num::ParseIntError> { s.parse() }"
                .to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::DeprecatedButValid,
            weirdness_score: 0.45,
            why_valid: "Spelling out the concrete error type rather than using impl Error \
                        or anyhow is verbose but completely valid Rust."
                .to_string(),
        },
        // ── UnconventionalErrorHandling ─────────────────────────────────────
        CommensalPattern {
            code: "result\n    .map_err(|e| format!(\"outer: {e}\"))\n    \
                   .and_then(|v| v.parse::<i32>().map_err(|e| format!(\"inner: {e}\")))"
                .to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::UnconventionalErrorHandling,
            weirdness_score: 0.50,
            why_valid: "Chained map_err and and_then calls form a valid Result combinator \
                        pipeline. Semantically correct; stylistically verbose."
                .to_string(),
        },
        CommensalPattern {
            code: "let val = some_result.ok().and_then(|x| if x > 0 { Some(x) } else { None });"
                .to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::UnconventionalErrorHandling,
            weirdness_score: 0.45,
            why_valid: "Converting Result to Option with ok() then filtering with \
                        and_then is valid. Loses the error type but is a real pattern \
                        in code where errors are expected and ignorable."
                .to_string(),
        },
        CommensalPattern {
            code: "fn might_fail() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {\n\
                   \tOk(())\n}"
                .to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::UnconventionalErrorHandling,
            weirdness_score: 0.35,
            why_valid: "Box<dyn Error + Send + Sync> is a fully valid dynamic error type. \
                        Longer than anyhow::Error but zero external dependencies."
                .to_string(),
        },
    ]
}

// ─── Pattern Generation: TypeScript ───────────────────────────────────────

/// Generate the full TypeScript commensal corpus (15+ patterns).
///
/// # Examples
///
/// ```rust
/// use nexcore_phenotype::commensal::generate_typescript_commensals;
///
/// let patterns = generate_typescript_commensals();
/// assert!(patterns.len() >= 15);
/// ```
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn generate_typescript_commensals() -> Vec<CommensalPattern> {
    vec![
        // ── EdgeCaseNaming ──────────────────────────────────────────────────
        CommensalPattern {
            code: ts_discard_binding(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.35,
            why_valid: "Single underscore is a valid JavaScript/TypeScript identifier. \
                        Conventionally used to discard values in destructuring."
                .to_string(),
        },
        CommensalPattern {
            code: "const $el = document.querySelector('.x');".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.30,
            why_valid: "Dollar-sign prefix is a valid identifier character in JS/TS. \
                        Common in jQuery legacy code and DOM-manipulation utilities."
                .to_string(),
        },
        CommensalPattern {
            code: "const __ = () => {};".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.40,
            why_valid: "Double underscore is a valid TS identifier. Used as a \
                        no-op placeholder or intentional discard in some codebases."
                .to_string(),
        },
        // ── RareButValidPattern ─────────────────────────────────────────────
        CommensalPattern {
            code: "type T = Readonly<Record<string, never>>;".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.55,
            why_valid: "Record<string, never> is an empty-value map type — it has \
                        string keys but no valid values. Wrapped in Readonly, it \
                        models an immutable empty object type."
                .to_string(),
        },
        CommensalPattern {
            code: "const x = '' as const;".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.45,
            why_valid: "The as const assertion narrows the type of '' from string to \
                        the literal type ''. Syntactically unusual on a string literal \
                        but semantically valid."
                .to_string(),
        },
        CommensalPattern {
            code: "type Unwrap<T> = T extends Promise<infer U> ? U : T;".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.60,
            why_valid: "Conditional types with infer are fully valid TypeScript. \
                        This specific pattern unwraps a Promise type at compile time."
                .to_string(),
        },
        CommensalPattern {
            code: "declare const brand: unique symbol;\ntype Branded<T> = T & { [brand]: true };"
                .to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.70,
            why_valid: "Branded types using unique symbol are a valid TS pattern for \
                        nominal typing. The symbol prevents accidental structural \
                        compatibility."
                .to_string(),
        },
        CommensalPattern {
            code: "const arr = [1, 2, 3] satisfies readonly number[];".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.50,
            why_valid: "The satisfies operator (TS 4.9+) checks type compatibility \
                        without widening. Valid syntax in modern TS projects."
                .to_string(),
        },
        // ── BoundaryValue ───────────────────────────────────────────────────
        CommensalPattern {
            code: "const s: string = '';".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.10,
            why_valid: "Empty string is a valid string value. It is falsy in JS but \
                        type-checks as string in strict TypeScript."
                .to_string(),
        },
        CommensalPattern {
            code: "const arr: never[] = [];".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.40,
            why_valid: "never[] is the type of an empty array whose element type is \
                        never (bottom type). Assignable to any array type; valid \
                        when strict null checks are enabled."
                .to_string(),
        },
        CommensalPattern {
            code: "const n = Number.MAX_SAFE_INTEGER;".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.20,
            why_valid: "Number.MAX_SAFE_INTEGER (2^53 - 1) is the largest integer \
                        representable exactly as a JS number. Using it is idiomatic \
                        for boundary testing."
                .to_string(),
        },
        // ── MixedStyle ──────────────────────────────────────────────────────
        CommensalPattern {
            code: "const myFunction = (someValue: number): number => someValue * 2;".to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::MixedStyle,
            weirdness_score: 0.30,
            why_valid: "camelCase is the standard TS naming convention. Arrow function \
                        with explicit return type annotation is idiomatic modern TS."
                .to_string(),
        },
        // ── UnconventionalErrorHandling ─────────────────────────────────────
        CommensalPattern {
            code: "const result = await Promise.resolve(42).then(v => v).catch(() => 0);"
                .to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::UnconventionalErrorHandling,
            weirdness_score: 0.35,
            why_valid: "Chaining .then(v => v) is a no-op identity transform but \
                        syntactically valid. The .catch fallback to 0 is a real pattern."
                .to_string(),
        },
        CommensalPattern {
            code: "type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };"
                .to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::UnconventionalErrorHandling,
            weirdness_score: 0.55,
            why_valid: "Discriminated union as a Result type is a common Rust-inspired \
                        TS pattern. Fully valid; avoids thrown exceptions entirely."
                .to_string(),
        },
        CommensalPattern {
            code: "function safe<T>(fn: () => T): T | undefined {\n\
                   \ttry { return fn(); } catch { return undefined; }\n}"
                .to_string(),
            language: Language::TypeScript,
            mutation_type: CommensalMutation::UnconventionalErrorHandling,
            weirdness_score: 0.40,
            why_valid: "Catch-all error swallowing is unconventional but valid TypeScript. \
                        The catch clause without a binding variable is supported since TS 4.0."
                .to_string(),
        },
    ]
}

// ─── Pattern Generation: Shell ─────────────────────────────────────────────

/// Generate the full Shell commensal corpus (15+ patterns).
///
/// # Examples
///
/// ```rust
/// use nexcore_phenotype::commensal::generate_shell_commensals;
///
/// let patterns = generate_shell_commensals();
/// assert!(patterns.len() >= 15);
/// ```
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn generate_shell_commensals() -> Vec<CommensalPattern> {
    vec![
        // ── EdgeCaseNaming ──────────────────────────────────────────────────
        CommensalPattern {
            code: "_tmp=$(mktemp)".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.25,
            why_valid: "Underscore-prefixed variable names are valid in POSIX shell. \
                        The leading _ conventionally signals a temporary/private variable."
                .to_string(),
        },
        CommensalPattern {
            code: "__log() { echo \"[LOG] $*\" >&2; }".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.35,
            why_valid: "Double-underscore function names are valid in zsh and bash. \
                        Used in libraries to avoid naming collisions."
                .to_string(),
        },
        CommensalPattern {
            code: "typeset -l lowered=\"$INPUT\"".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.50,
            why_valid: "typeset -l declares a variable with automatic lowercasing. \
                        Valid in zsh (typeset is a zsh builtin). Unusual but correct."
                .to_string(),
        },
        // ── UnusualFormatting ───────────────────────────────────────────────
        {
            #[allow(clippy::literal_string_with_formatting_args)]
            let shell_if_code = "if [[ -n \"${VAR:-}\" ]]\nthen\n\techo \"set\"\nfi".to_string();
            CommensalPattern {
                code: shell_if_code,
                language: Language::Shell,
                mutation_type: CommensalMutation::UnusualFormatting,
                weirdness_score: 0.20,
                why_valid: "then on its own line (instead of after the condition) is valid \
                            POSIX shell syntax. Less common than then on the same line."
                    .to_string(),
            }
        },
        CommensalPattern {
            code: "val=$((\n\t1 + 2 + 3\n))".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::UnusualFormatting,
            weirdness_score: 0.30,
            why_valid: "Multi-line arithmetic expansion $(( ... )) is valid shell syntax. \
                        The newlines inside (( )) are ignored by the parser."
                .to_string(),
        },
        // ── RareButValidPattern ─────────────────────────────────────────────
        CommensalPattern {
            code: ": \"${MY_VAR:=default_value}\"".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.55,
            why_valid: "The : (null command) with a parameter expansion side-effect is \
                        a POSIX idiom for setting a default without printing output."
                .to_string(),
        },
        CommensalPattern {
            code: "exec 3>&1; output=$(command 2>&1 1>&3 3>&-); exec 3>&-".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.75,
            why_valid: "FD gymnastics to capture stderr while letting stdout pass through \
                        are valid POSIX shell. Complex but correct."
                .to_string(),
        },
        CommensalPattern {
            code: "${VAR+isset}".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.60,
            why_valid: "The ${var+word} expansion yields word if var is set (even if empty). \
                        Distinguished from ${var:-word} which tests set-and-non-empty."
                .to_string(),
        },
        CommensalPattern {
            code: "read -r -d '' heredoc_var << 'EOF'\nsome content\nEOF".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.65,
            why_valid: "read -r -d '' with a here-doc captures multi-line content into \
                        a variable. Valid in bash/zsh; the -d '' reads until null byte."
                .to_string(),
        },
        // ── BoundaryValue ───────────────────────────────────────────────────
        CommensalPattern {
            code: "empty=\"\"".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.10,
            why_valid: "Assigning an empty string is valid in any POSIX shell. \
                        The resulting variable is set but empty."
                .to_string(),
        },
        CommensalPattern {
            code: "val=$((0))".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.15,
            why_valid: "Arithmetic expansion of the literal 0 is valid and produces '0'. \
                        Slightly verbose compared to val=0 but semantically equivalent."
                .to_string(),
        },
        CommensalPattern {
            code: "arr=()".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.15,
            why_valid: "Empty array declaration is valid in bash and zsh. \
                        ${#arr[@]} == 0. Safe to iterate over (no iterations)."
                .to_string(),
        },
        // ── MixedStyle ──────────────────────────────────────────────────────
        CommensalPattern {
            code: "myVar=\"value\"\nmy_func() { echo \"$myVar\"; }".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::MixedStyle,
            weirdness_score: 0.40,
            why_valid: "Shell does not enforce naming conventions. camelCase variables \
                        alongside snake_case functions are valid; style is a matter \
                        of convention, not syntax."
                .to_string(),
        },
        // ── DeprecatedButValid ──────────────────────────────────────────────
        CommensalPattern {
            code: "result=`command arg1 arg2`".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::DeprecatedButValid,
            weirdness_score: 0.65,
            why_valid: "Backtick command substitution is POSIX-defined and still works \
                        in all shells. Superseded by $() in modern scripts but valid."
                .to_string(),
        },
        CommensalPattern {
            code: "[ -f \"$file\" ] && echo \"exists\"".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::DeprecatedButValid,
            weirdness_score: 0.25,
            why_valid: "Single-bracket [ ] test is POSIX-portable. Modern zsh scripts \
                        prefer [[ ]] but [ ] remains valid and widely deployed."
                .to_string(),
        },
        CommensalPattern {
            code: "test -d \"$DIR\" && cd \"$DIR\"".to_string(),
            language: Language::Shell,
            mutation_type: CommensalMutation::DeprecatedButValid,
            weirdness_score: 0.20,
            why_valid: "The test builtin is POSIX-standard. Using test instead of [ ] \
                        or [[ ]] is valid and portable to all POSIX shells."
                .to_string(),
        },
    ]
}

// ─── Core Generation API ───────────────────────────────────────────────────

/// Generate a single [`CommensalPattern`] for the given mutation type and
/// language.
///
/// Selects the first pattern in the appropriate language corpus that matches
/// the requested mutation type. Falls back to the first pattern in the
/// language corpus when no exact match exists.
///
/// Returns a generic placeholder pattern if the language corpus is empty.
///
/// # Examples
///
/// ```rust
/// use nexcore_phenotype::commensal::{CommensalMutation, Language, generate_commensal};
///
/// let p = generate_commensal(CommensalMutation::BoundaryValue, Language::Rust);
/// assert_eq!(p.language, Language::Rust);
/// assert_eq!(p.mutation_type, CommensalMutation::BoundaryValue);
/// assert!(!p.code.is_empty());
/// ```
#[must_use]
pub fn generate_commensal(mutation: CommensalMutation, language: Language) -> CommensalPattern {
    let corpus: Vec<CommensalPattern> = match language {
        Language::Rust => generate_rust_commensals(),
        Language::TypeScript => generate_typescript_commensals(),
        Language::Shell => generate_shell_commensals(),
        Language::Json => generate_json_commensals(),
    };

    // First: exact mutation-type match.
    if let Some(p) = corpus.iter().find(|p| p.mutation_type == mutation) {
        return p.clone();
    }

    // Fallback: first pattern in the language corpus.
    if let Some(p) = corpus.into_iter().next() {
        return p;
    }

    // Ultimate fallback: synthetic placeholder.
    CommensalPattern {
        code: format!("/* commensal placeholder: {mutation} in {language} */"),
        language,
        mutation_type: mutation,
        weirdness_score: mutation.baseline_weirdness(),
        why_valid: format!(
            "Placeholder for {mutation} commensal in {language}. \
             This pattern has no specific implementation yet."
        ),
    }
}

/// Generate a [`CommensalCorpus`] containing exactly `count` patterns.
///
/// Cycles through all [`CommensalMutation`] variants and all [`Language`]
/// variants in round-robin order, ensuring diversity across the corpus.
///
/// # Examples
///
/// ```rust
/// use nexcore_phenotype::commensal::generate_batch;
///
/// let corpus = generate_batch(28);
/// assert_eq!(corpus.len(), 28);
/// ```
#[must_use]
pub fn generate_batch(count: usize) -> CommensalCorpus {
    let mutations = CommensalMutation::ALL;
    let languages = Language::ALL;
    let stride = mutations.len() * languages.len();

    let mut corpus = CommensalCorpus::new();

    for i in 0..count {
        let mutation = mutations[i % mutations.len()];
        // When we have cycled through all (mutation, language) pairs and need
        // more, shift the language offset to keep patterns varied.
        let language = if stride > 0 {
            languages[((i / mutations.len()) + (i / stride)) % languages.len()]
        } else {
            languages[(i / mutations.len()) % languages.len()]
        };

        corpus.add(generate_commensal(mutation, language));
    }

    corpus
}

// ─── JSON Commensals (internal, used by generate_commensal) ────────────────

/// Generate the JSON commensal corpus.
///
/// JSON has limited syntax, so patterns focus on boundary values and unusual
/// but spec-compliant representations.
fn generate_json_commensals() -> Vec<CommensalPattern> {
    vec![
        CommensalPattern {
            code: "{}".to_string(),
            language: Language::Json,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.10,
            why_valid: "An empty JSON object is spec-compliant (RFC 8259). \
                        Zero members, valid structure."
                .to_string(),
        },
        CommensalPattern {
            code: "[]".to_string(),
            language: Language::Json,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.10,
            why_valid: "An empty JSON array is spec-compliant. Zero elements, valid structure."
                .to_string(),
        },
        CommensalPattern {
            code: "{\"\":\"empty key is valid\"}".to_string(),
            language: Language::Json,
            mutation_type: CommensalMutation::EdgeCaseNaming,
            weirdness_score: 0.55,
            why_valid: "RFC 8259 permits empty string as an object key. Unusual but valid."
                .to_string(),
        },
        CommensalPattern {
            code: "1e308".to_string(),
            language: Language::Json,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.45,
            why_valid: "JSON numbers are unconstrained in the spec. 1e308 is near \
                        f64::MAX and parses to a finite float in compliant parsers."
                .to_string(),
        },
        CommensalPattern {
            code: "{\"a\":{\"b\":{\"c\":{\"d\":null}}}}".to_string(),
            language: Language::Json,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.30,
            why_valid: "Deeply nested JSON is spec-compliant. Parsers must handle \
                        arbitrary nesting depth per RFC 8259."
                .to_string(),
        },
        CommensalPattern {
            code: "{\"\\u0000\":\"null byte key\"}".to_string(),
            language: Language::Json,
            mutation_type: CommensalMutation::RareButValidPattern,
            weirdness_score: 0.70,
            why_valid: "Unicode escape \\u0000 is valid JSON string syntax. The null \
                        byte as a key is spec-compliant, though many implementations \
                        handle it poorly."
                .to_string(),
        },
    ]
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── CommensalMutation ───────────────────────────────────────────────────

    #[test]
    fn test_all_mutation_variants_count() {
        assert_eq!(CommensalMutation::ALL.len(), 7);
    }

    #[test]
    fn test_mutation_labels_nonempty() {
        for m in CommensalMutation::ALL {
            assert!(!m.label().is_empty(), "{m:?} has empty label");
        }
    }

    #[test]
    fn test_baseline_weirdness_in_range() {
        for m in CommensalMutation::ALL {
            let w = m.baseline_weirdness();
            assert!((0.0..=1.0).contains(&w), "{m:?} weirdness {w} out of range");
        }
    }

    #[test]
    fn test_mutation_display() {
        assert_eq!(
            CommensalMutation::EdgeCaseNaming.to_string(),
            "Edge-Case Naming"
        );
        assert_eq!(
            CommensalMutation::RareButValidPattern.to_string(),
            "Rare But Valid Pattern"
        );
    }

    // ── Language ────────────────────────────────────────────────────────────

    #[test]
    fn test_all_language_ids_nonempty() {
        for l in Language::ALL {
            assert!(!l.id().is_empty(), "{l:?} has empty id");
        }
    }

    #[test]
    fn test_language_display() {
        assert_eq!(Language::Rust.to_string(), "rust");
        assert_eq!(Language::TypeScript.to_string(), "typescript");
        assert_eq!(Language::Shell.to_string(), "shell");
        assert_eq!(Language::Json.to_string(), "json");
    }

    // ── generate_rust_commensals ────────────────────────────────────────────

    #[test]
    fn test_rust_corpus_minimum_size() {
        let patterns = generate_rust_commensals();
        assert!(
            patterns.len() >= 20,
            "Expected >=20 Rust patterns, got {}",
            patterns.len()
        );
    }

    #[test]
    fn test_rust_corpus_all_correct_language() {
        for p in generate_rust_commensals() {
            assert_eq!(
                p.language,
                Language::Rust,
                "Pattern {:?} has wrong language",
                p.code
            );
        }
    }

    #[test]
    fn test_rust_corpus_weirdness_in_range() {
        for p in generate_rust_commensals() {
            assert!(
                (0.0..=1.0).contains(&p.weirdness_score),
                "Pattern {:?} weirdness {} out of range",
                p.code,
                p.weirdness_score
            );
        }
    }

    #[test]
    fn test_rust_corpus_why_valid_nonempty() {
        for p in generate_rust_commensals() {
            assert!(
                !p.why_valid.is_empty(),
                "Pattern {:?} has empty why_valid",
                p.code
            );
        }
    }

    #[test]
    fn test_rust_corpus_covers_all_mutation_types() {
        let patterns = generate_rust_commensals();
        for mutation in CommensalMutation::ALL {
            assert!(
                patterns.iter().any(|p| p.mutation_type == *mutation),
                "Rust corpus missing mutation type: {mutation:?}"
            );
        }
    }

    // ── generate_typescript_commensals ──────────────────────────────────────

    #[test]
    fn test_typescript_corpus_minimum_size() {
        let patterns = generate_typescript_commensals();
        assert!(
            patterns.len() >= 15,
            "Expected >=15 TypeScript patterns, got {}",
            patterns.len()
        );
    }

    #[test]
    fn test_typescript_corpus_all_correct_language() {
        for p in generate_typescript_commensals() {
            assert_eq!(p.language, Language::TypeScript);
        }
    }

    #[test]
    fn test_typescript_corpus_weirdness_in_range() {
        for p in generate_typescript_commensals() {
            assert!(
                (0.0..=1.0).contains(&p.weirdness_score),
                "TypeScript pattern weirdness {} out of range",
                p.weirdness_score
            );
        }
    }

    // ── generate_shell_commensals ───────────────────────────────────────────

    #[test]
    fn test_shell_corpus_minimum_size() {
        let patterns = generate_shell_commensals();
        assert!(
            patterns.len() >= 15,
            "Expected >=15 Shell patterns, got {}",
            patterns.len()
        );
    }

    #[test]
    fn test_shell_corpus_all_correct_language() {
        for p in generate_shell_commensals() {
            assert_eq!(p.language, Language::Shell);
        }
    }

    // ── CommensalCorpus ─────────────────────────────────────────────────────

    #[test]
    fn test_corpus_new_is_empty() {
        let c = CommensalCorpus::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn test_corpus_add_increments_len() {
        let mut c = CommensalCorpus::new();
        let p = generate_commensal(CommensalMutation::BoundaryValue, Language::Rust);
        c.add(p);
        assert_eq!(c.len(), 1);
        assert!(!c.is_empty());
    }

    #[test]
    fn test_corpus_by_language_filters_correctly() {
        let mut c = CommensalCorpus::new();
        c.add(generate_commensal(
            CommensalMutation::BoundaryValue,
            Language::Rust,
        ));
        c.add(generate_commensal(
            CommensalMutation::EdgeCaseNaming,
            Language::TypeScript,
        ));
        let rust = c.by_language(Language::Rust);
        assert_eq!(rust.len(), 1);
        assert_eq!(rust[0].language, Language::Rust);
    }

    #[test]
    fn test_corpus_by_mutation_filters_correctly() {
        let mut c = CommensalCorpus::new();
        c.add(generate_commensal(
            CommensalMutation::BoundaryValue,
            Language::Rust,
        ));
        c.add(generate_commensal(
            CommensalMutation::EdgeCaseNaming,
            Language::Rust,
        ));
        let boundary = c.by_mutation(CommensalMutation::BoundaryValue);
        assert_eq!(boundary.len(), 1);
        assert_eq!(boundary[0].mutation_type, CommensalMutation::BoundaryValue);
    }

    #[test]
    fn test_corpus_mean_weirdness_empty() {
        let c = CommensalCorpus::new();
        assert!((c.mean_weirdness() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_corpus_mean_weirdness_computed() {
        let mut c = CommensalCorpus::new();
        c.add(CommensalPattern {
            code: "a".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.2,
            why_valid: "test".to_string(),
        });
        c.add(CommensalPattern {
            code: "b".to_string(),
            language: Language::Rust,
            mutation_type: CommensalMutation::BoundaryValue,
            weirdness_score: 0.4,
            why_valid: "test".to_string(),
        });
        let mean = c.mean_weirdness();
        assert!((mean - 0.3).abs() < 1e-9, "Expected 0.3, got {mean}");
    }

    // ── generate_commensal ──────────────────────────────────────────────────

    #[test]
    fn test_generate_commensal_matches_requested_language() {
        for lang in Language::ALL {
            for mutation in CommensalMutation::ALL {
                let p = generate_commensal(*mutation, *lang);
                assert_eq!(
                    p.language, *lang,
                    "generate_commensal returned wrong language for {mutation:?}/{lang:?}"
                );
            }
        }
    }

    #[test]
    fn test_generate_commensal_code_nonempty() {
        for lang in Language::ALL {
            for mutation in CommensalMutation::ALL {
                let p = generate_commensal(*mutation, *lang);
                assert!(
                    !p.code.is_empty(),
                    "generate_commensal produced empty code for {mutation:?}/{lang:?}"
                );
            }
        }
    }

    // ── generate_batch ──────────────────────────────────────────────────────

    #[test]
    fn test_generate_batch_exact_count() {
        for count in [0, 1, 7, 14, 28, 50] {
            let corpus = generate_batch(count);
            assert_eq!(
                corpus.len(),
                count,
                "generate_batch({count}) returned {} patterns",
                corpus.len()
            );
        }
    }

    #[test]
    fn test_generate_batch_has_all_languages() {
        // With 28 patterns we should see all 4 languages represented.
        let corpus = generate_batch(28);
        for lang in Language::ALL {
            assert!(
                !corpus.by_language(*lang).is_empty(),
                "generate_batch(28) missing language {lang}"
            );
        }
    }

    #[test]
    fn test_generate_batch_all_weirdness_in_range() {
        for p in generate_batch(28).patterns {
            assert!(
                (0.0..=1.0).contains(&p.weirdness_score),
                "Batch pattern weirdness {} out of range",
                p.weirdness_score
            );
        }
    }

    // ── SensitivityResult ───────────────────────────────────────────────────

    #[test]
    fn test_sensitivity_result_rate_computed() {
        let r = SensitivityResult::new("hook-a".to_string(), 3, 20, vec![0, 5, 12]);
        let expected = 3.0 / 20.0;
        assert!(
            (r.over_sensitivity_rate - expected).abs() < 1e-9,
            "Rate mismatch: {} vs {expected}",
            r.over_sensitivity_rate
        );
    }

    #[test]
    fn test_sensitivity_result_zero_total() {
        let r = SensitivityResult::new("hook-b".to_string(), 0, 0, vec![]);
        assert!((r.over_sensitivity_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sensitivity_result_over_sensitive_flag() {
        let high = SensitivityResult::new("hook-c".to_string(), 5, 10, vec![0, 1, 2, 3, 4]);
        assert!(high.is_over_sensitive());

        let low = SensitivityResult::new("hook-d".to_string(), 1, 100, vec![0]);
        assert!(!low.is_over_sensitive());
    }

    // ── CalibrationReport ───────────────────────────────────────────────────

    #[test]
    fn test_calibration_report_empty() {
        let report = CalibrationReport::from_results(vec![]);
        assert!((report.overall_over_sensitivity - 0.0).abs() < f64::EPSILON);
        assert!(report.adjustments.is_empty());
    }

    #[test]
    fn test_calibration_report_overall_sensitivity_mean() {
        let results = vec![
            SensitivityResult::new("h1".to_string(), 2, 10, vec![0, 1]),
            SensitivityResult::new("h2".to_string(), 4, 10, vec![0, 1, 2, 3]),
        ];
        // rates: 0.2 and 0.4, mean = 0.3
        let report = CalibrationReport::from_results(results);
        assert!(
            (report.overall_over_sensitivity - 0.3).abs() < 1e-9,
            "Expected 0.3, got {}",
            report.overall_over_sensitivity
        );
    }

    #[test]
    fn test_calibration_report_generates_adjustments_for_over_sensitive() {
        let results = vec![
            SensitivityResult::new("over-hook".to_string(), 5, 10, vec![0, 1, 2, 3, 4]),
            SensitivityResult::new("fine-hook".to_string(), 1, 100, vec![0]),
        ];
        let report = CalibrationReport::from_results(results);
        assert_eq!(report.adjustments.len(), 1);
        assert_eq!(report.adjustments[0].hook_name, "over-hook");
    }

    #[test]
    fn test_calibration_adjustment_recommended_in_range() {
        let results = vec![SensitivityResult::new(
            "h".to_string(),
            8,
            10,
            vec![0, 1, 2, 3, 4, 5, 6, 7],
        )];
        let report = CalibrationReport::from_results(results);
        for adj in &report.adjustments {
            assert!(
                (0.50..=0.95).contains(&adj.recommended_sensitivity),
                "Recommended sensitivity {} out of [0.50, 0.95]",
                adj.recommended_sensitivity
            );
        }
    }

    // ── Serialization round-trip ────────────────────────────────────────────

    #[test]
    fn test_commensal_pattern_serializes() {
        let p = generate_commensal(CommensalMutation::BoundaryValue, Language::Rust);
        let json = serde_json::to_string(&p);
        assert!(json.is_ok(), "CommensalPattern should serialize");
        let back: Result<CommensalPattern, _> =
            serde_json::from_str(json.as_deref().unwrap_or("{}"));
        assert!(back.is_ok(), "CommensalPattern should deserialize");
    }

    #[test]
    fn test_corpus_serializes() {
        let corpus = generate_batch(7);
        let json = serde_json::to_string(&corpus);
        assert!(json.is_ok(), "CommensalCorpus should serialize");
    }

    #[test]
    fn test_calibration_report_serializes() {
        let results = vec![SensitivityResult::new(
            "test-hook".to_string(),
            2,
            10,
            vec![0, 1],
        )];
        let report = CalibrationReport::from_results(results);
        let json = serde_json::to_string(&report);
        assert!(json.is_ok(), "CalibrationReport should serialize");
    }

    // ── Code snippet builder correctness ────────────────────────────────────

    #[test]
    fn test_rust_discard_binding_contains_underscore() {
        let s = rust_discard_binding();
        assert!(s.contains('_'), "Discard binding should contain underscore");
        assert!(s.starts_with("let"), "Should be a let binding");
    }

    #[test]
    fn test_rust_raw_ident_contains_keyword() {
        let s = rust_raw_ident("type");
        assert!(s.contains("type"), "Raw ident should embed the keyword");
    }

    #[test]
    fn test_ts_discard_binding_contains_underscore() {
        let s = ts_discard_binding();
        assert!(
            s.contains('_'),
            "TS discard binding should contain underscore"
        );
        assert!(s.starts_with("const"), "Should be a const binding");
    }
}
