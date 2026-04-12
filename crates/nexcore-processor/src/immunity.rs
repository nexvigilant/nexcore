//! Immunity bridge — auto-generate `Boundary` gates from antibody patterns.
//!
//! Connects `nexcore-immunity` antibodies to the processor framework.
//! Each antibody's detection pattern becomes a `Boundary` that rejects
//! inputs matching the antipattern.
//!
//! T1 composition: ∂(Boundary) + κ(Comparison) + ∃(Existence)
//!
//! This module does NOT depend on nexcore-immunity directly — it takes
//! regex patterns as strings, keeping the dependency unidirectional.
//! The caller bridges: `antibody.detection.code_patterns[i].pattern → here`.

use crate::boundary::PredicateBoundary;
use crate::error::ProcessorError;
use crate::pipeline::{Bounded, OpenBoundary};
use crate::processor::Processor;

/// An antibody-derived boundary gate.
///
/// Rejects inputs (strings) that match any of the provided antipattern
/// regexes. Patterns are compiled once at construction time.
pub struct AntibodyBoundary {
    name: String,
    patterns: Vec<(String, regex::Regex)>,
}

impl AntibodyBoundary {
    /// Create a boundary from antibody detection patterns.
    ///
    /// Each pattern is a regex string. If a pattern fails to compile,
    /// it is skipped (logged but not fatal — defense in depth).
    ///
    /// # Arguments
    /// - `name`: antibody name (for diagnostics)
    /// - `patterns`: list of (description, regex_string) pairs
    pub fn new(name: impl Into<String>, patterns: Vec<(String, String)>) -> Self {
        let name = name.into();
        let compiled: Vec<(String, regex::Regex)> = patterns
            .into_iter()
            .filter_map(|(desc, pat)| regex::Regex::new(&pat).ok().map(|r| (desc, r)))
            .collect();

        Self {
            name,
            patterns: compiled,
        }
    }

    /// Check if input matches any antipattern.
    ///
    /// Returns `Ok(())` if the input is CLEAN (no antipatterns found).
    /// Returns `Err` with the first matching pattern's description.
    pub fn check(&self, input: &str) -> Result<(), ProcessorError> {
        for (desc, re) in &self.patterns {
            if re.is_match(input) {
                return Err(ProcessorError::BoundaryRejection {
                    boundary: format!("antibody:{}", self.name),
                    reason: format!("matched antipattern: {desc}"),
                });
            }
        }
        Ok(())
    }

    /// Number of compiled patterns.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Antibody name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Wrap a string-processing `Processor` with an antibody boundary.
///
/// The boundary rejects any input string that matches an antipattern
/// BEFORE the inner processor runs. This is `∂(μ)` where ∂ comes
/// from the immune system.
pub fn guarded_by_antibody<P>(
    processor: P,
    antibody_name: impl Into<String>,
    patterns: Vec<(String, String)>,
) -> impl Processor<Input = String, Output = P::Output>
where
    P: Processor<Input = String>,
{
    let ab = AntibodyBoundary::new(antibody_name, patterns);
    let boundary =
        PredicateBoundary::new(format!("antibody:{}", ab.name()), move |input: &String| {
            ab.check(input).is_ok()
        });
    Bounded::new(processor, boundary, OpenBoundary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FnProcessor, Processor, ProcessorError};

    #[test]
    fn antibody_rejects_unwrap() {
        let ab = AntibodyBoundary::new(
            "unwrap-eliminator",
            vec![
                ("unwrap() call".into(), r"\.unwrap\(\)".into()),
                ("expect() call".into(), r"\.expect\(".into()),
            ],
        );

        // Clean code passes
        assert!(ab.check("let x = foo()?;").is_ok());

        // Antipattern rejected
        let result = ab.check("let x = foo().unwrap();");
        assert!(result.is_err());
    }

    #[test]
    fn antibody_rejects_unsafe() {
        let ab = AntibodyBoundary::new(
            "unsafe-blocker",
            vec![("unsafe block".into(), r"unsafe\s*\{".into())],
        );

        assert!(ab.check("fn safe_fn() {}").is_ok());
        assert!(ab.check("unsafe { ptr::read(p) }").is_err());
    }

    #[test]
    fn guarded_processor_blocks_antipattern() {
        let upper = FnProcessor::new("uppercase", |s: String| -> Result<String, ProcessorError> {
            Ok(s.to_uppercase())
        });

        let guarded = guarded_by_antibody(
            upper,
            "sql-injection",
            vec![(
                "SQL injection".into(),
                r"(?i)(DROP|DELETE|INSERT)\s+".into(),
            )],
        );

        // Clean input passes through
        let result = guarded.process("hello world".into());
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some("HELLO WORLD".into()));

        // SQL injection attempt blocked
        let result = guarded.process("DROP TABLE users".into());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_regex_skipped_gracefully() {
        let ab = AntibodyBoundary::new(
            "test",
            vec![
                ("valid".into(), r"foo".into()),
                ("invalid".into(), r"[invalid".into()), // broken regex
            ],
        );

        // Only the valid pattern compiled
        assert_eq!(ab.pattern_count(), 1);
        assert!(ab.check("foo").is_err()); // valid pattern still works
        assert!(ab.check("bar").is_ok());
    }

    #[test]
    fn empty_patterns_pass_everything() {
        let ab = AntibodyBoundary::new("empty", vec![]);
        assert_eq!(ab.pattern_count(), 0);
        assert!(ab.check("anything").is_ok());
    }
}
