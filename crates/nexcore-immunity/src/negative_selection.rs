// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Thymic negative selection — pre-deployment false positive testing for hook patterns.
//!
//! ## Biology Analog
//!
//! T-cells undergo negative selection in the thymus before entering circulation.
//! 95–98% are eliminated because they would attack self-antigens (the body's own
//! tissues). Only T-cells with appropriate reactivity thresholds survive and enter
//! the bloodstream as mature, functional immune cells.
//!
//! This module mirrors that process for hook patterns:
//!
//! - **Self** = a corpus of known-good, legitimate code patterns
//! - **Hook pattern** = a candidate T-cell (regex to detect bad code)
//! - **False positive rate** = how often the pattern fires on legitimate code
//! - **Selection threshold** = the maximum acceptable FPR before the pattern
//!   is rejected from deployment
//!
//! A pattern that triggers on 5% of legitimate Rust code will cause more harm than
//! good. Negative selection eliminates these noisy patterns before they reach
//! production hooks.
//!
//! ## Primitive Grounding: κ(Comparison) + ν(Frequency) + ∂(Boundary)
//!
//! - **κ**: Pattern matching — compare regex against each corpus entry
//! - **ν**: FPR measurement — count frequency of false positives
//! - **∂**: Selection boundary — the FPR threshold separating pass from fail
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_immunity::negative_selection::{
//!     HookCorpus, HookPattern, NegativeSelector, SelectionConfig,
//!     SelectionVerdict, Severity,
//! };
//!
//! let corpus = HookCorpus::default_corpus();
//! let config = SelectionConfig::default();
//! let selector = NegativeSelector::new(corpus, config);
//!
//! let pattern = HookPattern {
//!     name: "detect-unwrap".to_string(),
//!     regex_pattern: r"\.unwrap\(\)".to_string(),
//!     severity: Severity::High,
//! };
//!
//! let result = selector.test_pattern(&pattern);
//! let verdict = selector.verdict(&result);
//! match verdict {
//!     SelectionVerdict::Pass => println!("Pattern approved"),
//!     SelectionVerdict::Fail { fpr } => println!("Rejected: FPR={fpr:.2}"),
//!     SelectionVerdict::InsufficientCorpus { size, minimum } => {
//!         println!("Corpus too small: {size}/{minimum}");
//!     }
//! }
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// LANGUAGE ENUM
// ═══════════════════════════════════════════════════════════════════════════════

/// The programming language of a corpus pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// Rust source code.
    Rust,
    /// TypeScript / TSX source code.
    TypeScript,
    /// Shell script (bash/zsh).
    Shell,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::TypeScript => write!(f, "TypeScript"),
            Self::Shell => write!(f, "Shell"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEGITIMATE PATTERN
// ═══════════════════════════════════════════════════════════════════════════════

/// A known-good code pattern that represents safe, idiomatic code.
///
/// The corpus of `LegitimatePattern`s forms the "self" that hook patterns must
/// not react against. If a hook fires on these patterns, it is a false positive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegitimatePattern {
    /// The actual code snippet (may span multiple lines).
    pub code: String,
    /// The programming language of the snippet.
    pub language: Language,
    /// Human-readable description of what this pattern represents.
    pub context: String,
}

impl LegitimatePattern {
    /// Construct a new legitimate pattern.
    #[must_use]
    pub fn new(code: impl Into<String>, language: Language, context: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            language,
            context: context.into(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOOK CORPUS
// ═══════════════════════════════════════════════════════════════════════════════

/// Collection of legitimate code patterns used as the "self" corpus for
/// negative selection.
///
/// The corpus must be large enough (≥ `SelectionConfig::min_corpus_size`) before
/// selection results are considered statistically valid.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookCorpus {
    patterns: Vec<LegitimatePattern>,
}

impl HookCorpus {
    /// Create an empty corpus.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a single pattern to the corpus.
    pub fn add_pattern(&mut self, pattern: LegitimatePattern) {
        self.patterns.push(pattern);
    }

    /// Iterate over all Rust patterns in the corpus.
    pub fn rust_patterns(&self) -> impl Iterator<Item = &LegitimatePattern> {
        self.patterns
            .iter()
            .filter(|p| p.language == Language::Rust)
    }

    /// Iterate over all TypeScript patterns in the corpus.
    pub fn typescript_patterns(&self) -> impl Iterator<Item = &LegitimatePattern> {
        self.patterns
            .iter()
            .filter(|p| p.language == Language::TypeScript)
    }

    /// Iterate over all Shell patterns in the corpus.
    pub fn shell_patterns(&self) -> impl Iterator<Item = &LegitimatePattern> {
        self.patterns
            .iter()
            .filter(|p| p.language == Language::Shell)
    }

    /// Total number of patterns in the corpus.
    #[must_use]
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Returns `true` if the corpus contains no patterns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Build a pre-populated corpus with 60+ realistic legitimate code patterns
    /// spanning Rust, TypeScript, and Shell.
    ///
    /// This corpus is used as the default "self" for negative selection. It is
    /// deliberately conservative — every pattern is clearly idiomatic and safe.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn default_corpus() -> Self {
        let mut corpus = Self::new();

        // ── Rust patterns (25) ───────────────────────────────────────────────
        let rust = [
            (
                "let result = operation()?;",
                "Error propagation with the ? operator",
            ),
            (
                "match value { Some(v) => v, None => return Err(e) }",
                "Match expression for Option handling",
            ),
            (
                "impl Display for MyType { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, \"{}\", self.value) } }",
                "Display trait implementation",
            ),
            (
                "#[derive(Debug, Clone, Serialize, Deserialize)]",
                "Common derive macros",
            ),
            (
                "pub fn new() -> Self { Self::default() }",
                "Constructor returning Self::default",
            ),
            (
                "if let Some(val) = optional { process(val) }",
                "if-let pattern for optional value",
            ),
            (
                "vec![1, 2, 3].iter().map(|x| x * 2).collect::<Vec<_>>()",
                "Iterator map and collect idiom",
            ),
            (
                "assert!((result - expected).abs() < f64::EPSILON);",
                "Floating-point equality assertion",
            ),
            (
                "use std::collections::HashMap;",
                "Standard library HashMap import",
            ),
            (
                "#[cfg(test)] mod tests { use super::*; }",
                "Test module with wildcard import",
            ),
            (
                "let x = value.unwrap_or_default();",
                "unwrap_or_default for Option/Result",
            ),
            (
                "let x = value.unwrap_or(0);",
                "unwrap_or with a fallback value",
            ),
            (
                "fn process<T: Display + Clone>(item: T) -> String { item.to_string() }",
                "Generic function with trait bounds",
            ),
            (
                "impl From<IoError> for AppError { fn from(e: IoError) -> Self { Self::Io(e) } }",
                "From trait implementation for error conversion",
            ),
            (
                "let entries = fs::read_dir(path).map_err(AppError::Io)?;",
                "Fallible filesystem operation with map_err",
            ),
            (
                "#[tokio::test] async fn test_something() { let result = async_fn().await; assert!(result.is_ok()); }",
                "Async test function",
            ),
            (
                "pub struct Config { pub host: String, pub port: u16 }",
                "Simple public struct definition",
            ),
            (
                "impl Default for Config { fn default() -> Self { Self { host: String::from(\"localhost\"), port: 8080 } } }",
                "Default trait implementation",
            ),
            (
                "let filtered: Vec<_> = items.iter().filter(|&&x| x > 0).copied().collect();",
                "Iterator filter and collect",
            ),
            (
                "tracing::info!(\"Starting server on {}:{}\", host, port);",
                "Structured logging with tracing",
            ),
            (
                "let guard = mutex.lock().map_err(|_| AppError::PoisonedLock)?;",
                "Mutex lock with error mapping",
            ),
            (
                "let handle = tokio::spawn(async move { worker(data).await });",
                "Tokio task spawning",
            ),
            (
                "let bytes = serde_json::to_vec(&value)?;",
                "JSON serialization with error propagation",
            ),
            (
                "impl Iterator for MyIter { type Item = u32; fn next(&mut self) -> Option<u32> { self.data.next() } }",
                "Iterator trait implementation",
            ),
            (
                "let path = std::path::Path::new(\"/tmp/data.json\");",
                "Path construction from string literal",
            ),
        ];

        for (code, context) in rust {
            corpus.add_pattern(LegitimatePattern::new(code, Language::Rust, context));
        }

        // ── TypeScript patterns (22) ─────────────────────────────────────────
        let typescript = [
            (
                "const handler = async (req: Request) => { return Response.json({ ok: true }) }",
                "Async request handler arrow function",
            ),
            (
                "export default function Page() { return <div>Hello</div> }",
                "Next.js default page component",
            ),
            (
                "interface Props { name: string; value: number; }",
                "TypeScript interface definition",
            ),
            (
                "const [state, setState] = useState<string>('')",
                "React useState hook with type parameter",
            ),
            (
                "try { await fetch(url) } catch (e) { console.error(e) }",
                "Async fetch with error handling",
            ),
            (
                "export type { User, Session } from './types'",
                "Named type re-export",
            ),
            (
                "const result = await Promise.all([fetchA(), fetchB()]);",
                "Concurrent async operations with Promise.all",
            ),
            (
                "const config = { ...defaults, ...overrides };",
                "Object spread for config merging",
            ),
            (
                "function isError(val: unknown): val is Error { return val instanceof Error; }",
                "Type guard function",
            ),
            (
                "const id = crypto.randomUUID();",
                "UUID generation with Web Crypto API",
            ),
            (
                "import { z } from 'zod'; const schema = z.object({ name: z.string() });",
                "Zod schema definition",
            ),
            (
                "const data = schema.parse(raw);",
                "Zod schema parsing — safe validation",
            ),
            (
                "type Result<T> = { ok: true; data: T } | { ok: false; error: string };",
                "Discriminated union Result type",
            ),
            (
                "useEffect(() => { const sub = store.subscribe(handler); return () => sub(); }, []);",
                "React useEffect with cleanup",
            ),
            (
                "const Component = React.memo(function Component({ value }: Props) { return <span>{value}</span>; });",
                "Memoized React component",
            ),
            (
                "export const GET = async (req: Request) => { const url = new URL(req.url); return Response.json({}); }",
                "Next.js App Router route handler",
            ),
            (
                "const env = process.env.NODE_ENV ?? 'development';",
                "Nullish coalescing for environment variable",
            ),
            (
                "import type { Metadata } from 'next'; export const metadata: Metadata = { title: 'Page' };",
                "Next.js static metadata export",
            ),
            (
                "const items = array.filter(Boolean).map((x) => x.toString());",
                "Filter falsy values then map",
            ),
            (
                "logger.info({ userId, action }, 'User performed action');",
                "Structured logging with pino-style logger",
            ),
            (
                "const tx = await db.transaction().execute(async (trx) => { return trx.selectFrom('users').selectAll().execute(); });",
                "Database transaction with Kysely",
            ),
            (
                "if (typeof window !== 'undefined') { localStorage.setItem('key', value); }",
                "SSR-safe localStorage access guard",
            ),
        ];

        for (code, context) in typescript {
            corpus.add_pattern(LegitimatePattern::new(code, Language::TypeScript, context));
        }

        // ── Shell patterns (22) ──────────────────────────────────────────────
        let shell = [
            (
                "set -euo pipefail",
                "Strict mode: exit on error, unset vars, pipe failures",
            ),
            (
                "if [[ -f \"$file\" ]]; then",
                "File existence check with double-bracket",
            ),
            (
                "local result=$(command_here)",
                "Local variable from command substitution",
            ),
            (
                "echo \"Processing ${item}\"",
                "Echo with brace-quoted variable expansion",
            ),
            ("for f in *.rs; do", "Glob-based for loop over Rust files"),
            (
                "readonly CONFIG_DIR=\"${HOME}/.config/myapp\"",
                "Readonly constant directory path",
            ),
            (
                "mkdir -p \"${output_dir}\"",
                "Create directory tree idempotently",
            ),
            (
                "if [[ $# -lt 2 ]]; then echo \"Usage: $0 <src> <dst>\" >&2; exit 1; fi",
                "Argument count validation with usage message",
            ),
            (
                "log_info() { echo \"[INFO] $*\" >&2; }",
                "Simple logging helper function",
            ),
            (
                "source \"${HOME}/.claude/lib/claude-lib.sh\"",
                "Sourcing a shared library",
            ),
            (
                "while IFS= read -r line; do process \"$line\"; done < \"$input_file\"",
                "Safe line-by-line file reading",
            ),
            (
                "trap 'rm -f \"${tmpfile}\"' EXIT",
                "Cleanup trap on script exit",
            ),
            ("tmpfile=$(mktemp)", "Create a secure temporary file"),
            (
                "if command -v cargo &>/dev/null; then",
                "Check if a command exists",
            ),
            (
                "for dir in crates/*/; do echo \"Checking ${dir}\"; done",
                "Iterate over subdirectories",
            ),
            ("status=$?", "Capture exit status of previous command"),
            (
                "local -r name=\"${1:?'name required'}\"",
                "Readonly local with required-argument default",
            ),
            (
                "echo \"Build succeeded\" | tee -a \"${LOG_FILE}\"",
                "Tee output to both stdout and log file",
            ),
            (
                "git log --oneline -10",
                "Display last 10 git commits in short form",
            ),
            (
                "cargo build -p nexcore-mcp --release 2>&1 | tail -20",
                "Build a cargo crate and capture output",
            ),
            (
                "if [[ \"${ENVIRONMENT:-}\" == \"production\" ]]; then",
                "Check optional environment variable against value",
            ),
            (
                concat!(
                    r#"case "${1:-}" in start) start_service "#,
                    r#";; stop) stop_service ;; *) usage ;; esac"#
                ),
                "Case statement for CLI subcommand dispatch",
            ),
        ];

        for (code, context) in shell {
            corpus.add_pattern(LegitimatePattern::new(code, Language::Shell, context));
        }

        corpus
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SEVERITY
// ═══════════════════════════════════════════════════════════════════════════════

/// The severity level of a hook's detection target.
///
/// Higher severity hooks warrant stricter false positive thresholds — a
/// `Critical` hook that fires on 2% of legitimate code causes enormous friction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Minor style or convenience issue.
    Low,
    /// Moderate quality concern.
    Medium,
    /// Significant correctness or safety risk.
    High,
    /// Security, memory safety, or data integrity violation.
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOOK PATTERN
// ═══════════════════════════════════════════════════════════════════════════════

/// A hook's detection pattern — the candidate to be tested against the corpus.
///
/// A hook pattern must survive negative selection (FPR below threshold) before
/// being trusted in a production deployment gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPattern {
    /// Human-readable name identifying the hook.
    pub name: String,
    /// The regular expression that the hook uses to detect problems.
    pub regex_pattern: String,
    /// The severity of problems this hook is designed to catch.
    pub severity: Severity,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SELECTION CONFIG
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration controlling the negative selection process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionConfig {
    /// Maximum acceptable false positive rate (0.0–1.0).
    ///
    /// Patterns that match more than this fraction of legitimate corpus entries
    /// are rejected. Default: 0.02 (2%).
    pub fpr_threshold: f64,
    /// Minimum number of corpus patterns required for a statistically valid test.
    ///
    /// If the corpus is smaller than this, `SelectionVerdict::InsufficientCorpus`
    /// is returned instead of Pass/Fail. Default: 50.
    pub min_corpus_size: usize,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            fpr_threshold: 0.02,
            min_corpus_size: 50,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RESULT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// A single false positive match found during pattern testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    /// Index of the corpus pattern that was matched.
    pub pattern_idx: usize,
    /// Name of the hook whose pattern triggered.
    pub hook_name: String,
    /// The text within the corpus snippet that matched the regex.
    pub matched_text: String,
}

/// Full testing result for one hook pattern run against the entire corpus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    /// Name of the hook that was tested.
    pub hook_name: String,
    /// Fraction of corpus entries that the pattern matched (0.0–1.0).
    pub false_positive_rate: f64,
    /// Total number of corpus entries tested.
    pub total_tested: usize,
    /// Number of corpus entries where the pattern matched (false positives).
    pub false_positives: usize,
    /// All individual matches recorded during testing.
    pub matches: Vec<PatternMatch>,
}

/// The outcome of negative selection for a single hook pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectionVerdict {
    /// The pattern's FPR is within the acceptable threshold — approved for deployment.
    Pass,
    /// The pattern's FPR exceeded the threshold — rejected.
    Fail {
        /// The measured false positive rate.
        fpr: f64,
    },
    /// The corpus is too small to produce a statistically meaningful result.
    InsufficientCorpus {
        /// Actual corpus size.
        size: usize,
        /// Required minimum corpus size.
        minimum: usize,
    },
}

impl std::fmt::Display for SelectionVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pass => write!(f, "Pass"),
            Self::Fail { fpr } => write!(f, "Fail(fpr={fpr:.4})"),
            Self::InsufficientCorpus { size, minimum } => {
                write!(f, "InsufficientCorpus({size}/{minimum})")
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEGATIVE SELECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// The thymic negative selection engine.
///
/// Holds a corpus of legitimate code patterns and tests candidate hook patterns
/// against them, measuring the false positive rate. Patterns that match too many
/// legitimate snippets are rejected before they reach production hooks.
///
/// ## Example
///
/// ```rust
/// use nexcore_immunity::negative_selection::{
///     HookCorpus, HookPattern, NegativeSelector, SelectionConfig,
///     SelectionVerdict, Severity,
/// };
///
/// let corpus = HookCorpus::default_corpus();
/// let config = SelectionConfig::default();
/// let selector = NegativeSelector::new(corpus, config);
///
/// let tight_pattern = HookPattern {
///     name: "detect-bare-unwrap".to_string(),
///     regex_pattern: r"\.unwrap\(\)".to_string(),
///     severity: Severity::High,
/// };
///
/// let result = selector.test_pattern(&tight_pattern);
/// // A tight, well-targeted pattern should pass
/// println!("FPR: {:.2}%", result.false_positive_rate * 100.0);
/// ```
#[derive(Debug, Clone)]
pub struct NegativeSelector {
    corpus: HookCorpus,
    config: SelectionConfig,
}

impl NegativeSelector {
    /// Construct a new selector with the given corpus and configuration.
    #[must_use]
    pub fn new(corpus: HookCorpus, config: SelectionConfig) -> Self {
        Self { corpus, config }
    }

    /// Test a single hook pattern against the entire corpus.
    ///
    /// Each corpus entry is tested against the pattern's regex. Matches on
    /// legitimate code are counted as false positives. Returns a full
    /// [`SelectionResult`] including all individual match locations.
    ///
    /// If the regex fails to compile, the result will have `false_positive_rate`
    /// of `1.0` (worst case — the pattern is treated as maximally dangerous)
    /// and `total_tested` of `0`.
    #[must_use]
    pub fn test_pattern(&self, pattern: &HookPattern) -> SelectionResult {
        let Ok(regex) = Regex::new(&pattern.regex_pattern) else {
            // A pattern that cannot be compiled is treated as a total failure.
            return SelectionResult {
                hook_name: pattern.name.clone(),
                false_positive_rate: 1.0,
                total_tested: 0,
                false_positives: 0,
                matches: Vec::new(),
            };
        };

        let mut false_positives = 0usize;
        let mut all_matches = Vec::new();

        for (idx, corpus_entry) in self.corpus.patterns.iter().enumerate() {
            if let Some(m) = regex.find(&corpus_entry.code) {
                false_positives += 1;
                all_matches.push(PatternMatch {
                    pattern_idx: idx,
                    hook_name: pattern.name.clone(),
                    matched_text: m.as_str().to_string(),
                });
            }
        }

        let total_tested = self.corpus.len();

        #[allow(clippy::cast_precision_loss)]
        // usize values are bounded by corpus size — precision loss is acceptable here
        let false_positive_rate = if total_tested == 0 {
            0.0
        } else {
            false_positives as f64 / total_tested as f64
        };

        SelectionResult {
            hook_name: pattern.name.clone(),
            false_positive_rate,
            total_tested,
            false_positives,
            matches: all_matches,
        }
    }

    /// Test all provided hook patterns against the corpus.
    ///
    /// Returns one [`SelectionResult`] per pattern in input order.
    #[must_use]
    pub fn test_all(&self, patterns: &[HookPattern]) -> Vec<SelectionResult> {
        patterns.iter().map(|p| self.test_pattern(p)).collect()
    }

    /// Evaluate a [`SelectionResult`] and return the verdict.
    ///
    /// - [`SelectionVerdict::InsufficientCorpus`] if the corpus is smaller than
    ///   `config.min_corpus_size`.
    /// - [`SelectionVerdict::Fail`] if the measured FPR exceeds `config.fpr_threshold`.
    /// - [`SelectionVerdict::Pass`] otherwise.
    #[must_use]
    pub fn verdict(&self, result: &SelectionResult) -> SelectionVerdict {
        let corpus_size = self.corpus.len();
        if corpus_size < self.config.min_corpus_size {
            return SelectionVerdict::InsufficientCorpus {
                size: corpus_size,
                minimum: self.config.min_corpus_size,
            };
        }

        if result.false_positive_rate > self.config.fpr_threshold {
            SelectionVerdict::Fail {
                fpr: result.false_positive_rate,
            }
        } else {
            SelectionVerdict::Pass
        }
    }

    /// Return a reference to the corpus held by this selector.
    #[must_use]
    pub fn corpus(&self) -> &HookCorpus {
        &self.corpus
    }

    /// Return a reference to the configuration held by this selector.
    #[must_use]
    pub fn config(&self) -> &SelectionConfig {
        &self.config
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn make_selector() -> NegativeSelector {
        NegativeSelector::new(HookCorpus::default_corpus(), SelectionConfig::default())
    }

    // ── corpus ──────────────────────────────────────────────────────────────

    #[test]
    fn test_default_corpus_has_sufficient_patterns() {
        let corpus = HookCorpus::default_corpus();
        assert!(
            corpus.len() >= 60,
            "Expected >=60 patterns, got {}",
            corpus.len()
        );
    }

    #[test]
    fn test_corpus_language_breakdown() {
        let corpus = HookCorpus::default_corpus();
        let rust_count = corpus.rust_patterns().count();
        let ts_count = corpus.typescript_patterns().count();
        let shell_count = corpus.shell_patterns().count();

        assert!(
            rust_count >= 20,
            "Expected >=20 Rust patterns, got {rust_count}"
        );
        assert!(
            ts_count >= 20,
            "Expected >=20 TypeScript patterns, got {ts_count}"
        );
        assert!(
            shell_count >= 20,
            "Expected >=20 Shell patterns, got {shell_count}"
        );
        assert_eq!(
            rust_count + ts_count + shell_count,
            corpus.len(),
            "Language counts must sum to total"
        );
    }

    #[test]
    fn test_empty_corpus() {
        let corpus = HookCorpus::new();
        assert!(corpus.is_empty());
        assert_eq!(corpus.len(), 0);
        assert_eq!(corpus.rust_patterns().count(), 0);
    }

    #[test]
    fn test_add_pattern() {
        let mut corpus = HookCorpus::new();
        corpus.add_pattern(LegitimatePattern::new(
            "let x = 1;",
            Language::Rust,
            "Simple let binding",
        ));
        assert_eq!(corpus.len(), 1);
        assert!(!corpus.is_empty());
    }

    // ── language display ────────────────────────────────────────────────────

    #[test]
    fn test_language_display() {
        assert_eq!(Language::Rust.to_string(), "Rust");
        assert_eq!(Language::TypeScript.to_string(), "TypeScript");
        assert_eq!(Language::Shell.to_string(), "Shell");
    }

    // ── severity ordering ───────────────────────────────────────────────────

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    // ── tight pattern passes ─────────────────────────────────────────────────

    #[test]
    fn test_tight_pattern_passes() {
        // A well-targeted pattern for a rare construct — should not fire on
        // normal legitimate code.
        let selector = make_selector();
        let pattern = HookPattern {
            name: "detect-transmute".to_string(),
            // std::mem::transmute is extremely rare in normal code
            regex_pattern: r"std::mem::transmute".to_string(),
            severity: Severity::Critical,
        };
        let result = selector.test_pattern(&pattern);
        let verdict = selector.verdict(&result);
        assert_eq!(
            verdict,
            SelectionVerdict::Pass,
            "transmute pattern should pass negative selection (FPR: {:.4})",
            result.false_positive_rate
        );
    }

    // ── overly broad pattern fails ────────────────────────────────────────────

    #[test]
    fn test_broad_pattern_fails() {
        // Matching any whitespace is absurdly broad — fires on everything.
        let selector = make_selector();
        let pattern = HookPattern {
            name: "detect-all-whitespace".to_string(),
            regex_pattern: r"\s".to_string(),
            severity: Severity::Low,
        };
        let result = selector.test_pattern(&pattern);
        let verdict = selector.verdict(&result);
        assert!(
            matches!(verdict, SelectionVerdict::Fail { .. }),
            "Whitespace-matching pattern should fail negative selection"
        );
    }

    // ── insufficient corpus ──────────────────────────────────────────────────

    #[test]
    fn test_insufficient_corpus_verdict() {
        let mut tiny_corpus = HookCorpus::new();
        tiny_corpus.add_pattern(LegitimatePattern::new(
            "let x = 1;",
            Language::Rust,
            "Tiny corpus",
        ));
        let config = SelectionConfig::default(); // min_corpus_size = 50
        let selector = NegativeSelector::new(tiny_corpus, config);

        let pattern = HookPattern {
            name: "any-pattern".to_string(),
            regex_pattern: r"let".to_string(),
            severity: Severity::Low,
        };
        let result = selector.test_pattern(&pattern);
        let verdict = selector.verdict(&result);
        assert!(
            matches!(
                verdict,
                SelectionVerdict::InsufficientCorpus {
                    size: 1,
                    minimum: 50
                }
            ),
            "Should be InsufficientCorpus, got {verdict}"
        );
    }

    // ── invalid regex ────────────────────────────────────────────────────────

    #[test]
    fn test_invalid_regex_returns_worst_case() {
        let selector = make_selector();
        let pattern = HookPattern {
            name: "broken-regex".to_string(),
            regex_pattern: r"[invalid(regex".to_string(),
            severity: Severity::Low,
        };
        let result = selector.test_pattern(&pattern);
        // Worst-case FPR = 1.0, total_tested = 0
        assert!(
            (result.false_positive_rate - 1.0).abs() < f64::EPSILON,
            "Invalid regex should produce FPR=1.0"
        );
        assert_eq!(result.total_tested, 0);
    }

    // ── test_all ─────────────────────────────────────────────────────────────

    #[test]
    fn test_all_returns_one_result_per_pattern() {
        let selector = make_selector();
        let patterns = vec![
            HookPattern {
                name: "p1".to_string(),
                regex_pattern: r"std::mem::transmute".to_string(),
                severity: Severity::Critical,
            },
            HookPattern {
                name: "p2".to_string(),
                regex_pattern: r"std::mem::forget".to_string(),
                severity: Severity::High,
            },
        ];
        let results = selector.test_all(&patterns);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].hook_name, "p1");
        assert_eq!(results[1].hook_name, "p2");
    }

    // ── FPR boundary ─────────────────────────────────────────────────────────

    #[test]
    fn test_fpr_exactly_at_threshold_passes() {
        // Build a minimal corpus where 1 pattern matches out of 50 = 2% FPR,
        // which is exactly at the threshold.
        let mut corpus = HookCorpus::new();
        // One entry that contains the trigger word
        corpus.add_pattern(LegitimatePattern::new(
            "TRIGGER",
            Language::Rust,
            "The one match",
        ));
        // 49 entries that do not
        for i in 0..49 {
            corpus.add_pattern(LegitimatePattern::new(
                format!("safe_code_{i}"),
                Language::Rust,
                "Clean code",
            ));
        }
        assert_eq!(corpus.len(), 50);

        let config = SelectionConfig {
            fpr_threshold: 0.02,
            min_corpus_size: 50,
        };
        let selector = NegativeSelector::new(corpus, config);
        let pattern = HookPattern {
            name: "trigger-detector".to_string(),
            regex_pattern: r"TRIGGER".to_string(),
            severity: Severity::Medium,
        };
        let result = selector.test_pattern(&pattern);

        // 1/50 = 0.02, exactly at threshold → Pass
        assert_eq!(result.false_positives, 1);
        assert_eq!(result.total_tested, 50);
        assert!((result.false_positive_rate - 0.02).abs() < f64::EPSILON);
        assert_eq!(selector.verdict(&result), SelectionVerdict::Pass);
    }

    #[test]
    fn test_fpr_just_above_threshold_fails() {
        let mut corpus = HookCorpus::new();
        // 2 entries that contain the trigger word
        corpus.add_pattern(LegitimatePattern::new(
            "TRIGGER alpha",
            Language::Rust,
            "First match",
        ));
        corpus.add_pattern(LegitimatePattern::new(
            "TRIGGER beta",
            Language::Rust,
            "Second match",
        ));
        // 48 clean entries
        for i in 0..48 {
            corpus.add_pattern(LegitimatePattern::new(
                format!("clean_{i}"),
                Language::Rust,
                "Clean",
            ));
        }
        assert_eq!(corpus.len(), 50);

        let config = SelectionConfig {
            fpr_threshold: 0.02,
            min_corpus_size: 50,
        };
        let selector = NegativeSelector::new(corpus, config);
        let pattern = HookPattern {
            name: "trigger-detector".to_string(),
            regex_pattern: r"TRIGGER".to_string(),
            severity: Severity::Medium,
        };
        let result = selector.test_pattern(&pattern);

        // 2/50 = 0.04 > 0.02 → Fail
        assert_eq!(result.false_positives, 2);
        assert!(
            matches!(selector.verdict(&result), SelectionVerdict::Fail { fpr } if (fpr - 0.04).abs() < f64::EPSILON)
        );
    }

    // ── match recording ───────────────────────────────────────────────────────

    #[test]
    fn test_matches_are_recorded_correctly() {
        let mut corpus = HookCorpus::new();
        corpus.add_pattern(LegitimatePattern::new(
            "foo.unwrap_or(0)",
            Language::Rust,
            "Safe unwrap_or",
        ));
        for i in 0..49 {
            corpus.add_pattern(LegitimatePattern::new(
                format!("safe_{i}"),
                Language::Rust,
                "Safe",
            ));
        }

        let config = SelectionConfig {
            fpr_threshold: 0.02,
            min_corpus_size: 50,
        };
        let selector = NegativeSelector::new(corpus, config);
        let pattern = HookPattern {
            name: "unwrap-detector".to_string(),
            // This fires on unwrap_or too — intentionally broad for this test
            regex_pattern: r"unwrap".to_string(),
            severity: Severity::High,
        };
        let result = selector.test_pattern(&pattern);

        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].pattern_idx, 0);
        assert_eq!(result.matches[0].hook_name, "unwrap-detector");
        assert_eq!(result.matches[0].matched_text, "unwrap");
    }

    // ── verdict display ───────────────────────────────────────────────────────

    #[test]
    fn test_verdict_display() {
        assert_eq!(SelectionVerdict::Pass.to_string(), "Pass");
        assert!(
            SelectionVerdict::Fail { fpr: 0.05 }
                .to_string()
                .contains("Fail")
        );
        assert!(
            SelectionVerdict::InsufficientCorpus {
                size: 10,
                minimum: 50
            }
            .to_string()
            .contains("InsufficientCorpus")
        );
    }

    // ── accessor helpers ──────────────────────────────────────────────────────

    #[test]
    fn test_selector_accessors() {
        let corpus = HookCorpus::default_corpus();
        let config = SelectionConfig::default();
        let n = corpus.len();
        let selector = NegativeSelector::new(corpus, config);
        assert_eq!(selector.corpus().len(), n);
        assert!((selector.config().fpr_threshold - 0.02).abs() < f64::EPSILON);
        assert_eq!(selector.config().min_corpus_size, 50);
    }
}
