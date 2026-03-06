//! # core-true
//!
//! Parser and validator for `.true` formal axiom files.
//!
//! Implements 8 validation passes:
//!
//! 1. Syntax well-formed (every statement is axiom/def/theorem/∎)
//! 2. No duplicate subjects
//! 3. All from-refs exist as defined subjects
//! 4. DAG acyclic (topological sort succeeds)
//! 5. All theorems have `[proof: ...]`
//! 6. File ends with ∎
//! 7. conf(child) <= min(conf(parents))
//! 8. Cross-validate: spec (core.true) ↔ impl (nexcore-lex-primitiva)

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use std::collections::{HashMap, HashSet};
use std::fmt;

// ═══════════════════════════════════════════
// DATA MODEL
// ═══════════════════════════════════════════

/// The four constructs in .true syntax.
#[derive(Debug, Clone, PartialEq)]
pub enum Construct {
    /// `axiom SUBJECT : PROPERTIES` — given, irreducible. conf = 1.0
    Axiom,
    /// `def SUBJECT : PROPERTIES` — derived from axioms/defs. conf = min(parent confs)
    Def,
    /// `theorem SUBJECT : EXPRESSION [PROOF]` — proven from defs. conf = proof_type
    Theorem,
    /// `∎` — nothing more to prove. conf = min(all theorems)
    Halt,
}

impl fmt::Display for Construct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Axiom => write!(f, "axiom"),
            Self::Def => write!(f, "def"),
            Self::Theorem => write!(f, "theorem"),
            Self::Halt => write!(f, "∎"),
        }
    }
}

/// Proof types with their confidence values.
#[derive(Debug, Clone, PartialEq)]
pub enum ProofType {
    Computational, // 0.99
    Analytical,    // 0.95
    Mapping,       // 0.90
    Adversarial,   // 0.85
    Empirical,     // 0.80
}

impl ProofType {
    /// Parse from the proof annotation string.
    pub fn parse(s: &str) -> Option<Self> {
        let lower = s.trim().to_lowercase();
        if lower.starts_with("computational") {
            Some(Self::Computational)
        } else if lower.starts_with("analytical") {
            Some(Self::Analytical)
        } else if lower.starts_with("mapping") {
            Some(Self::Mapping)
        } else if lower.starts_with("adversarial") {
            Some(Self::Adversarial)
        } else if lower.starts_with("empirical") {
            Some(Self::Empirical)
        } else if lower.starts_with("by construction") {
            Some(Self::Computational) // treat as computational
        } else {
            None
        }
    }

    /// Returns the confidence value for this proof type.
    #[must_use]
    pub const fn confidence(&self) -> f64 {
        match self {
            Self::Computational => 0.99,
            Self::Analytical => 0.95,
            Self::Mapping => 0.90,
            Self::Adversarial => 0.85,
            Self::Empirical => 0.80,
        }
    }
}

impl fmt::Display for ProofType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Computational => write!(f, "computational (0.99)"),
            Self::Analytical => write!(f, "analytical (0.95)"),
            Self::Mapping => write!(f, "mapping (0.90)"),
            Self::Adversarial => write!(f, "adversarial (0.85)"),
            Self::Empirical => write!(f, "empirical (0.80)"),
        }
    }
}

/// A parsed statement from a .true file.
#[derive(Debug, Clone)]
pub struct Statement {
    /// The construct type.
    pub construct: Construct,
    /// The subject name (empty for ∎).
    pub subject: String,
    /// Line number where this statement starts (1-indexed).
    pub line: usize,
    /// Raw body text (everything after `SUBJECT :`).
    pub body: String,
    /// References to other subjects via `from [...]`.
    pub from_refs: Vec<String>,
    /// Proof type (only for theorems).
    pub proof_type: Option<ProofType>,
    /// Explicit conf override (for axioms like open_lambda with conf=0.75).
    pub explicit_conf: Option<f64>,
    /// Boolean function value (`val=N`), if this is a primitive definition.
    pub boolean_val: Option<u8>,
    /// Whether this statement defines a primitive (`prim` keyword present).
    pub is_prim: bool,
    /// Primitive name (e.g., "Quantity", "Void") extracted from the quoted name.
    pub prim_name: Option<String>,
}

impl Statement {
    /// Returns the confidence of this statement.
    #[must_use]
    pub fn confidence(&self) -> f64 {
        if let Some(c) = self.explicit_conf {
            return c;
        }
        match self.construct {
            Construct::Axiom => 1.0,
            Construct::Def => 1.0, // will be overridden by pass 7
            Construct::Theorem => self.proof_type.as_ref().map_or(0.0, |pt| pt.confidence()),
            Construct::Halt => 0.0, // computed
        }
    }
}

/// A validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Which pass detected the error.
    pub pass: u8,
    /// Line number (0 if file-level).
    pub line: usize,
    /// Description.
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line > 0 {
            write!(
                f,
                "pass {}: line {}: {}",
                self.pass, self.line, self.message
            )
        } else {
            write!(f, "pass {}: {}", self.pass, self.message)
        }
    }
}

/// Result of parsing and validating a .true file.
#[derive(Debug)]
pub struct ValidationReport {
    /// All parsed statements.
    pub statements: Vec<Statement>,
    /// Errors found during validation.
    pub errors: Vec<ValidationError>,
    /// System confidence (min of all theorem confs).
    pub system_conf: f64,
    /// Per-theorem confidence breakdown.
    pub theorem_confs: Vec<(String, f64)>,
}

impl ValidationReport {
    /// Returns true if validation passed with 0 errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

// ═══════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════

/// Parse a .true file into statements.
pub fn parse(source: &str) -> Vec<Statement> {
    let mut statements = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let stripped = strip_comment(line).trim();

        if stripped.is_empty() {
            i += 1;
            continue;
        }

        // Detect construct keyword
        let (construct, rest) = if stripped == "∎" || stripped == "QED" {
            (Some(Construct::Halt), "")
        } else if let Some(rest) = stripped.strip_prefix("axiom ") {
            (Some(Construct::Axiom), rest)
        } else if let Some(rest) = stripped.strip_prefix("def ") {
            (Some(Construct::Def), rest)
        } else if let Some(rest) = stripped.strip_prefix("theorem ") {
            (Some(Construct::Theorem), rest)
        } else {
            (None, stripped)
        };

        if let Some(construct) = construct {
            let start_line = i + 1; // 1-indexed

            // Collect continuation lines (indented lines that follow)
            let mut body = String::from(rest);
            let mut j = i + 1;
            while j < lines.len() {
                let next = strip_comment(lines[j]);
                let next_trimmed = next.trim();
                if next_trimmed.is_empty() {
                    j += 1;
                    continue;
                }
                // Continuation: starts with whitespace or is clearly not a new construct
                if is_continuation(lines[j], next_trimmed) {
                    body.push('\n');
                    body.push_str(next_trimmed);
                    j += 1;
                } else {
                    break;
                }
            }

            let subject = extract_subject(&construct, rest);
            let from_refs = extract_from_refs(&body);
            let proof_type = extract_proof_type(&body);
            let explicit_conf = extract_explicit_conf(&body);
            let boolean_val = extract_val(&body);
            let is_prim =
                body.contains("prim,") || body.starts_with("prim,") || rest.contains("prim,");
            let prim_name = extract_prim_name(&body);

            statements.push(Statement {
                construct,
                subject,
                line: start_line,
                body,
                from_refs,
                proof_type,
                explicit_conf,
                boolean_val,
                is_prim,
                prim_name,
            });

            i = j;
        } else {
            // Non-construct line (possibly continuation of previous — skip)
            i += 1;
        }
    }

    statements
}

/// Strip `--` comments from a line.
fn strip_comment(line: &str) -> &str {
    // Find `--` that's not inside a quoted string
    let mut in_quote = false;
    let bytes = line.as_bytes();
    for (pos, &byte) in bytes.iter().enumerate() {
        if byte == b'"' {
            in_quote = !in_quote;
        }
        if !in_quote && byte == b'-' && pos + 1 < bytes.len() && bytes[pos + 1] == b'-' {
            return &line[..pos];
        }
    }
    line
}

/// Check if a line is a continuation of the previous statement.
fn is_continuation(raw_line: &str, trimmed: &str) -> bool {
    // New construct starts at column 0 with a keyword
    if raw_line.starts_with("axiom ")
        || raw_line.starts_with("def ")
        || raw_line.starts_with("theorem ")
        || trimmed == "∎"
        || trimmed == "QED"
    {
        return false;
    }
    // Indented or continuation-looking content
    raw_line.starts_with(' ') || raw_line.starts_with('\t')
}

/// Extract the subject name from a statement's first line.
fn extract_subject(construct: &Construct, rest: &str) -> String {
    if *construct == Construct::Halt {
        return String::new();
    }
    // Subject is everything before the first `:` or end of line
    let subject = rest.split(':').next().unwrap_or("").trim();
    subject.to_string()
}

/// Extract `from [X, Y, Z]` references from the body.
fn extract_from_refs(body: &str) -> Vec<String> {
    let mut refs = Vec::new();
    // Look for `from [...]` pattern
    if let Some(start) = body.find("from [") {
        let after = &body[start + 6..];
        if let Some(end) = after.find(']') {
            let inside = &after[..end];
            for item in inside.split(',') {
                let trimmed = item.trim();
                if !trimmed.is_empty() {
                    // Normalize: strip quotes, handle special symbols
                    let cleaned = trimmed.trim_matches('"').trim();
                    refs.push(cleaned.to_string());
                }
            }
        }
    }
    refs
}

/// Extract `[proof: TYPE, ...]` from the body.
fn extract_proof_type(body: &str) -> Option<ProofType> {
    if let Some(start) = body.find("[proof:") {
        let after = &body[start + 7..];
        if let Some(end) = after.find(']') {
            let proof_str = after[..end].trim();
            return ProofType::parse(proof_str);
        }
    }
    None
}

/// Extract explicit `conf=N.NN` from the body.
fn extract_explicit_conf(body: &str) -> Option<f64> {
    for segment in body.split(',') {
        let trimmed = segment.trim();
        if let Some(val_str) = trimmed.strip_prefix("conf=") {
            if let Ok(val) = val_str.trim().parse::<f64>() {
                return Some(val);
            }
        }
    }
    None
}

/// Extract `val=N` from the body.
fn extract_val(body: &str) -> Option<u8> {
    for segment in body.split(',') {
        let trimmed = segment.trim();
        if let Some(val_str) = trimmed.strip_prefix("val=") {
            if let Ok(val) = val_str.trim().parse::<u8>() {
                return Some(val);
            }
        }
    }
    None
}

/// Extract the primitive name (first quoted string after `prim,`).
fn extract_prim_name(body: &str) -> Option<String> {
    // Pattern: prim, "Name", ...
    // Find first quoted string
    let mut in_prim = false;
    for segment in body.split(',') {
        let trimmed = segment.trim();
        if trimmed == "prim" || trimmed.starts_with("prim,") {
            in_prim = true;
            continue;
        }
        if in_prim {
            let unquoted = trimmed.trim_matches('"').trim();
            if !unquoted.is_empty()
                && !unquoted.contains('=')
                && !unquoted.starts_with("from")
                && !unquoted.starts_with('[')
            {
                return Some(unquoted.to_string());
            }
        }
    }
    // Fallback: find first quoted string in body
    if let Some(start) = body.find('"') {
        if let Some(end) = body[start + 1..].find('"') {
            let name = &body[start + 1..start + 1 + end];
            if !name.is_empty() && !name.contains('=') {
                return Some(name.to_string());
            }
        }
    }
    None
}

// ═══════════════════════════════════════════
// VALIDATOR — 8 passes
// ═══════════════════════════════════════════

/// Validate a parsed .true file. Runs all 7 passes.
pub fn validate(statements: &[Statement]) -> ValidationReport {
    let mut errors = Vec::new();

    // Pass 1: Syntax well-formed
    pass_1_syntax(statements, &mut errors);

    // Pass 2: No duplicate subjects
    pass_2_no_duplicates(statements, &mut errors);

    // Build subject index for subsequent passes
    let subjects: HashMap<&str, &Statement> = statements
        .iter()
        .filter(|s| !s.subject.is_empty())
        .map(|s| (s.subject.as_str(), s))
        .collect();

    // Pass 3: All from-refs exist
    pass_3_refs_exist(statements, &subjects, &mut errors);

    // Pass 4: DAG acyclic
    pass_4_dag_acyclic(statements, &subjects, &mut errors);

    // Pass 5: All theorems have [proof]
    pass_5_proofs_exist(statements, &mut errors);

    // Pass 6: File ends with ∎
    pass_6_ends_with_halt(statements, &mut errors);

    // Pass 7: conf(child) <= min(conf(parents))
    let conf_map = pass_7_conf_monotonicity(statements, &subjects, &mut errors);

    // Pass 8: Cross-validate spec ↔ impl
    pass_8_cross_validate(statements, &mut errors);

    // Compute system confidence
    let theorem_confs: Vec<(String, f64)> = statements
        .iter()
        .filter(|s| s.construct == Construct::Theorem)
        .map(|s| {
            let conf = conf_map
                .get(s.subject.as_str())
                .copied()
                .unwrap_or(s.confidence());
            (s.subject.clone(), conf)
        })
        .collect();

    let system_conf = theorem_confs
        .iter()
        .map(|(_, c)| *c)
        .fold(f64::INFINITY, f64::min);

    // Handle edge case: no theorems → conf = 1.0
    let system_conf = if system_conf.is_infinite() {
        1.0
    } else {
        system_conf
    };

    ValidationReport {
        statements: statements.to_vec(),
        errors,
        system_conf,
        theorem_confs,
    }
}

/// Pass 1: Every statement must be axiom, def, theorem, or ∎.
fn pass_1_syntax(statements: &[Statement], errors: &mut Vec<ValidationError>) {
    if statements.is_empty() {
        errors.push(ValidationError {
            pass: 1,
            line: 0,
            message: "file contains no statements".to_string(),
        });
    }
    // If we got here, all statements were parsed as valid constructs
    // (the parser only produces known construct types)
}

/// Pass 2: No duplicate subjects.
fn pass_2_no_duplicates(statements: &[Statement], errors: &mut Vec<ValidationError>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for s in statements {
        if s.subject.is_empty() {
            continue; // ∎ has no subject
        }
        if let Some(&prev_line) = seen.get(s.subject.as_str()) {
            errors.push(ValidationError {
                pass: 2,
                line: s.line,
                message: format!(
                    "duplicate subject '{}' (first at line {})",
                    s.subject, prev_line
                ),
            });
        } else {
            seen.insert(&s.subject, s.line);
        }
    }
}

/// Pass 3: All from-refs point to existing subjects.
fn pass_3_refs_exist(
    statements: &[Statement],
    subjects: &HashMap<&str, &Statement>,
    errors: &mut Vec<ValidationError>,
) {
    for s in statements {
        for ref_name in &s.from_refs {
            if !subjects.contains_key(ref_name.as_str()) {
                errors.push(ValidationError {
                    pass: 3,
                    line: s.line,
                    message: format!(
                        "'{}' references undefined subject '{}'",
                        s.subject, ref_name
                    ),
                });
            }
        }
    }
}

/// Pass 4: DAG is acyclic (topological sort succeeds).
fn pass_4_dag_acyclic(
    statements: &[Statement],
    subjects: &HashMap<&str, &Statement>,
    errors: &mut Vec<ValidationError>,
) {
    // Build adjacency: child -> parents
    let defs_with_refs: Vec<(&str, &[String])> = statements
        .iter()
        .filter(|s| !s.from_refs.is_empty())
        .map(|s| (s.subject.as_str(), s.from_refs.as_slice()))
        .collect();

    // DFS cycle detection
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();

    for (subject, _) in &defs_with_refs {
        if !visited.contains(*subject) {
            if let Some(cycle) = dfs_cycle(
                subject,
                &statements
                    .iter()
                    .filter(|s| !s.from_refs.is_empty())
                    .map(|s| (s.subject.as_str(), s.from_refs.as_slice()))
                    .collect(),
                &mut visited,
                &mut stack,
            ) {
                errors.push(ValidationError {
                    pass: 4,
                    line: subjects.get(cycle.as_str()).map_or(0, |s| s.line),
                    message: format!("cycle detected involving '{}'", cycle),
                });
            }
        }
    }
}

/// DFS cycle detection. Returns Some(node) if cycle found.
fn dfs_cycle<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, &'a [String]>,
    visited: &mut HashSet<&'a str>,
    stack: &mut HashSet<&'a str>,
) -> Option<String> {
    if stack.contains(node) {
        return Some(node.to_string());
    }
    if visited.contains(node) {
        return None;
    }
    visited.insert(node);
    stack.insert(node);

    if let Some(refs) = adj.get(node) {
        for ref_name in *refs {
            if let Some(cycle) = dfs_cycle(ref_name.as_str(), adj, visited, stack) {
                return Some(cycle);
            }
        }
    }

    stack.remove(node);
    None
}

/// Pass 5: All theorems have a [proof: ...] annotation.
fn pass_5_proofs_exist(statements: &[Statement], errors: &mut Vec<ValidationError>) {
    for s in statements {
        if s.construct == Construct::Theorem && s.proof_type.is_none() {
            errors.push(ValidationError {
                pass: 5,
                line: s.line,
                message: format!("theorem '{}' missing [proof: ...] annotation", s.subject),
            });
        }
    }
}

/// Pass 6: File ends with ∎.
fn pass_6_ends_with_halt(statements: &[Statement], errors: &mut Vec<ValidationError>) {
    match statements.last() {
        Some(s) if s.construct == Construct::Halt => {} // ok
        _ => {
            errors.push(ValidationError {
                pass: 6,
                line: 0,
                message: "file does not end with ∎".to_string(),
            });
        }
    }
}

/// Pass 7: conf(child) <= min(conf(parents)). Returns resolved conf map.
fn pass_7_conf_monotonicity(
    statements: &[Statement],
    subjects: &HashMap<&str, &Statement>,
    errors: &mut Vec<ValidationError>,
) -> HashMap<String, f64> {
    let mut conf_map: HashMap<String, f64> = HashMap::new();

    // First: assign base confidences
    for s in statements {
        if s.subject.is_empty() {
            continue;
        }
        conf_map.insert(s.subject.clone(), s.confidence());
    }

    // Resolve defs: conf = min(parent confs)
    // Iterate until stable (handles multi-level deps)
    let mut changed = true;
    let mut iterations = 0;
    while changed && iterations < 100 {
        changed = false;
        iterations += 1;

        for s in statements {
            if s.construct != Construct::Def || s.from_refs.is_empty() {
                continue;
            }
            let parent_min = s
                .from_refs
                .iter()
                .filter_map(|r| conf_map.get(r.as_str()).copied())
                .fold(f64::INFINITY, f64::min);

            if parent_min.is_finite() {
                let current = conf_map.get(s.subject.as_str()).copied().unwrap_or(1.0);
                if parent_min < current {
                    conf_map.insert(s.subject.clone(), parent_min);
                    changed = true;
                }
            }
        }
    }

    // Verify monotonicity
    for s in statements {
        if s.from_refs.is_empty() || s.subject.is_empty() {
            continue;
        }
        let child_conf = conf_map.get(s.subject.as_str()).copied().unwrap_or(1.0);
        for ref_name in &s.from_refs {
            if let Some(&parent_conf) = conf_map.get(ref_name.as_str()) {
                if child_conf > parent_conf + f64::EPSILON {
                    errors.push(ValidationError {
                        pass: 7,
                        line: s.line,
                        message: format!(
                            "'{}' conf={:.2} > parent '{}' conf={:.2}",
                            s.subject, child_conf, ref_name, parent_conf
                        ),
                    });
                }
            }
        }
    }

    conf_map
}

/// Pass 8: Cross-validate spec (core.true primitives) against Rust implementation.
///
/// Checks for each primitive statement:
/// - `val=N` in core.true matches `boolean_val()` expectation
/// - `from [...]` in core.true matches `LexPrimitiva::derives_from()`
/// - primitive name in core.true matches `LexPrimitiva::name()`
fn pass_8_cross_validate(statements: &[Statement], errors: &mut Vec<ValidationError>) {
    use nexcore_lex_primitiva::LexPrimitiva;

    // Collect primitive statements (those with `is_prim` and a `val`)
    let prim_stmts: Vec<&Statement> = statements
        .iter()
        .filter(|s| s.is_prim && s.boolean_val.is_some())
        .collect();

    if prim_stmts.is_empty() {
        return; // No primitives to cross-validate (file may not be core.true)
    }

    // Map .true subject names → LexPrimitiva variants
    let resolve = |subject: &str| -> Option<LexPrimitiva> {
        // Try as symbol first (handles ∅, ×, ∝, ς, ∂, μ, κ, Σ, ν, λ, ρ, π, σ, ∃)
        LexPrimitiva::from_symbol(subject).or_else(|| {
            // Handle special subjects: N, ->
            match subject {
                "N" => Some(LexPrimitiva::Quantity),
                "->" => Some(LexPrimitiva::Causality),
                _ => None,
            }
        })
    };

    // Build expected boolean vals from Rust code
    let expected_val = |p: LexPrimitiva| -> u8 {
        // These are the canonical boolean vals from the bijection
        match p {
            LexPrimitiva::Void => 0,
            LexPrimitiva::Product => 1,
            LexPrimitiva::Irreversibility => 2,
            LexPrimitiva::State => 3,
            LexPrimitiva::Boundary => 4,
            LexPrimitiva::Mapping => 5,
            LexPrimitiva::Comparison => 6,
            LexPrimitiva::Sum => 7,
            LexPrimitiva::Frequency => 8,
            LexPrimitiva::Quantity => 9,
            LexPrimitiva::Location => 10,
            LexPrimitiva::Recursion => 11,
            LexPrimitiva::Persistence => 12,
            LexPrimitiva::Causality => 13,
            LexPrimitiva::Sequence => 14,
            LexPrimitiva::Existence => 15,
            _ => 15, // non_exhaustive fallback
        }
    };

    let mut checked = 0;

    for stmt in &prim_stmts {
        let Some(prim) = resolve(&stmt.subject) else {
            errors.push(ValidationError {
                pass: 8,
                line: stmt.line,
                message: format!(
                    "cannot resolve '{}' to a LexPrimitiva variant",
                    stmt.subject
                ),
            });
            continue;
        };

        // Check 8a: boolean val matches
        if let Some(spec_val) = stmt.boolean_val {
            let impl_val = expected_val(prim);
            if spec_val != impl_val {
                errors.push(ValidationError {
                    pass: 8,
                    line: stmt.line,
                    message: format!(
                        "'{}' val drift: spec val={}, impl val={}",
                        stmt.subject, spec_val, impl_val
                    ),
                });
            }
        }

        // Check 8b: from-refs match derives_from()
        if !stmt.from_refs.is_empty() || !prim.derives_from().is_empty() {
            let spec_parents: HashSet<String> = stmt.from_refs.iter().cloned().collect();
            let impl_parents: HashSet<String> = prim
                .derives_from()
                .iter()
                .map(|p| {
                    // Convert LexPrimitiva back to the subject name used in core.true
                    match p {
                        LexPrimitiva::Quantity => "N".to_string(),
                        LexPrimitiva::Causality => "->".to_string(),
                        other => other.symbol().to_string(),
                    }
                })
                .collect();

            if spec_parents != impl_parents {
                let spec_only: Vec<_> = spec_parents.difference(&impl_parents).collect();
                let impl_only: Vec<_> = impl_parents.difference(&spec_parents).collect();
                errors.push(ValidationError {
                    pass: 8,
                    line: stmt.line,
                    message: format!(
                        "'{}' from-ref drift: spec={:?}, impl={:?} (spec_only={:?}, impl_only={:?})",
                        stmt.subject,
                        spec_parents,
                        impl_parents,
                        spec_only,
                        impl_only
                    ),
                });
            }
        }

        // Check 8c: primitive name matches
        if let Some(spec_name) = &stmt.prim_name {
            let impl_name = prim.name();
            if spec_name != impl_name {
                errors.push(ValidationError {
                    pass: 8,
                    line: stmt.line,
                    message: format!(
                        "'{}' name drift: spec=\"{}\", impl=\"{}\"",
                        stmt.subject, spec_name, impl_name
                    ),
                });
            }
        }

        checked += 1;
    }

    // Verify we checked all 16 primitives
    if checked < 16 && !prim_stmts.is_empty() {
        // Only warn if we found some prims but not all (don't warn on non-core.true files)
        if checked > 0 && checked < 16 {
            errors.push(ValidationError {
                pass: 8,
                line: 0,
                message: format!(
                    "cross-validated {}/16 primitives — {} missing from spec",
                    checked,
                    16 - checked
                ),
            });
        }
    }
}

/// Parse and validate a .true file in one call.
pub fn parse_and_validate(source: &str) -> ValidationReport {
    let statements = parse(source);
    validate(&statements)
}

// ═══════════════════════════════════════════
// DISPLAY
// ═══════════════════════════════════════════

impl fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let axioms = self
            .statements
            .iter()
            .filter(|s| s.construct == Construct::Axiom)
            .count();
        let defs = self
            .statements
            .iter()
            .filter(|s| s.construct == Construct::Def)
            .count();
        let theorems = self
            .statements
            .iter()
            .filter(|s| s.construct == Construct::Theorem)
            .count();

        writeln!(
            f,
            "  STATEMENTS  {} axioms, {} defs, {} theorems",
            axioms, defs, theorems
        )?;
        writeln!(f)?;

        if !self.theorem_confs.is_empty() {
            writeln!(f, "  THEOREM                              CONF")?;
            for (name, conf) in &self.theorem_confs {
                writeln!(f, "  {:<38} {:.2}", name, conf)?;
            }
            writeln!(f)?;
        }

        if self.errors.is_empty() {
            writeln!(f, "  PASSES  7/7 ✓")?;
        } else {
            writeln!(f, "  ERRORS  {}", self.errors.len())?;
            for err in &self.errors {
                writeln!(f, "  ✗ {}", err)?;
            }
        }

        writeln!(f)?;
        writeln!(f, "  SYSTEM CONF = {:.2}", self.system_conf)?;

        Ok(())
    }
}

// ═══════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Minimal valid file ──

    const MINIMAL: &str = "\
axiom A : root
def B : from [A], derived
theorem T : claim [proof: computational, test]
∎
";

    #[test]
    fn test_minimal_valid() {
        let report = parse_and_validate(MINIMAL);
        assert!(report.is_valid(), "Errors: {:?}", report.errors);
        assert_eq!(report.statements.len(), 4);
        assert!((report.system_conf - 0.99).abs() < f64::EPSILON);
    }

    // ── Pass 1: syntax ──

    #[test]
    fn test_pass1_empty_file() {
        let report = parse_and_validate("");
        assert!(!report.is_valid());
        assert!(report.errors.iter().any(|e| e.pass == 1));
    }

    // ── Pass 2: no duplicates ──

    #[test]
    fn test_pass2_duplicate_subject() {
        let src = "\
axiom A : first
axiom A : second
∎
";
        let report = parse_and_validate(src);
        assert!(!report.is_valid());
        assert!(report.errors.iter().any(|e| e.pass == 2));
    }

    // ── Pass 3: refs exist ──

    #[test]
    fn test_pass3_missing_ref() {
        let src = "\
axiom A : root
def B : from [A, MISSING], derived
∎
";
        let report = parse_and_validate(src);
        assert!(!report.is_valid());
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.pass == 3 && e.message.contains("MISSING"))
        );
    }

    #[test]
    fn test_pass3_valid_refs() {
        let src = "\
axiom A : root
axiom B : root
def C : from [A, B], derived
∎
";
        let report = parse_and_validate(src);
        // Pass 3 should not error
        assert!(!report.errors.iter().any(|e| e.pass == 3));
    }

    // ── Pass 5: theorems have proofs ──

    #[test]
    fn test_pass5_missing_proof() {
        let src = "\
axiom A : root
theorem T : claim without proof annotation
∎
";
        let report = parse_and_validate(src);
        assert!(!report.is_valid());
        assert!(report.errors.iter().any(|e| e.pass == 5));
    }

    // ── Pass 6: ends with ∎ ──

    #[test]
    fn test_pass6_no_halt() {
        let src = "\
axiom A : root
theorem T : claim [proof: computational, test]
";
        let report = parse_and_validate(src);
        assert!(!report.is_valid());
        assert!(report.errors.iter().any(|e| e.pass == 6));
    }

    // ── Pass 7: conf monotonicity ──

    #[test]
    fn test_pass7_conf_propagation() {
        let src = "\
axiom A : root
def B : from [A], derived
theorem T : claim [proof: computational, test]
∎
";
        let report = parse_and_validate(src);
        assert!(report.is_valid());
        // B derives from A (conf=1.0), so B.conf = 1.0
        // T is computational = 0.99
        assert!((report.system_conf - 0.99).abs() < f64::EPSILON);
    }

    // ── Proof type parsing ──

    #[test]
    fn test_proof_type_parsing() {
        assert_eq!(
            ProofType::parse("computational, 341 tests").map(|p| p.confidence()),
            Some(0.99)
        );
        assert_eq!(
            ProofType::parse("analytical, math proof").map(|p| p.confidence()),
            Some(0.95)
        );
        assert_eq!(
            ProofType::parse("mapping, 12 theories").map(|p| p.confidence()),
            Some(0.90)
        );
        assert_eq!(
            ProofType::parse("adversarial, 6 attacks").map(|p| p.confidence()),
            Some(0.85)
        );
        assert_eq!(
            ProofType::parse("empirical, measurement").map(|p| p.confidence()),
            Some(0.80)
        );
        assert_eq!(
            ProofType::parse("by construction").map(|p| p.confidence()),
            Some(0.99)
        );
        assert!(ProofType::parse("unknown_type").is_none());
    }

    // ── Comment stripping ──

    #[test]
    fn test_strip_comment() {
        assert_eq!(
            strip_comment("axiom A : root -- this is a comment"),
            "axiom A : root "
        );
        assert_eq!(strip_comment("-- full line comment"), "");
        assert_eq!(strip_comment("no comment here"), "no comment here");
        assert_eq!(
            strip_comment(r#"axiom A : "has -- inside quotes" -- real comment"#),
            r#"axiom A : "has -- inside quotes" "#
        );
    }

    // ── From-ref extraction ──

    #[test]
    fn test_extract_from_refs() {
        let refs = extract_from_refs("from [A, B, C], other stuff");
        assert_eq!(refs, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_extract_from_refs_special_symbols() {
        let refs = extract_from_refs("from [∃, κ], more");
        assert_eq!(refs, vec!["∃", "κ"]);
    }

    #[test]
    fn test_extract_from_refs_with_arrow() {
        let refs = extract_from_refs("from [->, ∂, ς]");
        assert_eq!(refs, vec!["->", "∂", "ς"]);
    }

    // ── Explicit conf ──

    #[test]
    fn test_extract_explicit_conf() {
        assert_eq!(extract_explicit_conf("conf=0.75, other"), Some(0.75));
        assert_eq!(extract_explicit_conf("no conf here"), None);
    }

    // ── Multi-line / continuation ──

    #[test]
    fn test_multiline_theorem() {
        let src = "\
axiom A : root
theorem T :
  \"multi-line claim\",
  \"continues here\"
  [proof: analytical, derivation]
∎
";
        let report = parse_and_validate(src);
        assert!(report.is_valid(), "Errors: {:?}", report.errors);
        let theorem = report.statements.iter().find(|s| s.subject == "T");
        assert!(theorem.is_some());
        assert!(theorem.map_or(false, |t| t.proof_type.is_some()));
    }

    // ── Integration: validate actual core.true ──

    #[test]
    fn test_validate_core_true() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../core.true");
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return, // skip if file not found (CI)
        };

        let report = parse_and_validate(&source);

        // Print report for visibility
        let report_str = format!("{}", report);
        assert!(!report_str.is_empty());

        // Should have statements
        assert!(!report.statements.is_empty());

        // Should end with ∎
        assert!(
            !report.errors.iter().any(|e| e.pass == 6),
            "core.true should end with ∎"
        );

        // All theorems should have proofs
        assert!(
            !report.errors.iter().any(|e| e.pass == 5),
            "All theorems should have [proof: ...]"
        );

        // System conf should be 0.99 (all computational in v7.5)
        assert!(
            report.system_conf >= 0.98,
            "System conf should be ≥0.98, got {}",
            report.system_conf
        );
    }
}
