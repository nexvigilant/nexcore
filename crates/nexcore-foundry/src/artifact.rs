//! Builder pipeline artifact types for The Foundry assembly line architecture.
//!
//! Each artifact represents the typed output of a specific builder station
//! in the B1 → B2 → B3 pipeline:
//!
//! - [`DesignSpec`] — produced by B1 (Architect), consumed by B2 (Coder)
//! - [`SourceArtifact`] — produced by B2 (Coder), consumed by B3 (Validator)
//! - [`ValidatedDeliverable`] — produced by B3 (Validator), gating release
//! - [`ShippableArtifact`] — produced by B3 upon a green gate, ready for dispatch
//!
//! # Example
//!
//! ```
//! use nexcore_foundry::artifact::{
//!     ValidatedDeliverable, ShippableArtifact,
//! };
//!
//! let vd = ValidatedDeliverable {
//!     build_pass: true,
//!     test_count: 10,
//!     tests_passed: 10,
//!     lint_pass: true,
//!     coverage_percent: 92.5,
//!     failures: vec![],
//! };
//! assert!(vd.is_green());
//!
//! let ship = ShippableArtifact::from_validated(vd);
//! assert!(ship.ready);
//! ```

use nexcore_lex_primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Classifies the kind of component represented by a [`Component`] entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    /// A Rust library or binary crate.
    RustCrate,
    /// An HTTP API route definition.
    ApiRoute,
    /// A frontend UI component.
    UiComponent,
    /// A database migration script.
    Migration,
    /// A test harness or test module.
    Test,
}

/// A single named component described in a [`DesignSpec`].
///
/// Each component carries a [`ComponentKind`] discriminant and a filesystem
/// path relative to the workspace root so the coder station knows exactly
/// where to materialise the output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// The kind of component (crate, route, UI, migration, or test).
    pub kind: ComponentKind,
    /// Human-readable identifier for this component.
    pub name: String,
    /// Workspace-relative path at which the component should be created.
    pub path: String,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// An API contract describing the expected input and output types of an
/// interface boundary.
///
/// Type strings are intentionally free-form to accommodate Rust, TypeScript,
/// JSON Schema, and OpenAPI references in a single representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Canonical type name (or schema ref) for the contract's input.
    pub input_type: String,
    /// Canonical type name (or schema ref) for the contract's output.
    pub output_type: String,
}

// ---------------------------------------------------------------------------
// DesignSpec — B1 output
// ---------------------------------------------------------------------------

/// The typed output artifact produced by the B1 Architect station.
///
/// A `DesignSpec` captures the full design intent for a feature: which
/// components must be built, which API contracts must be honoured, which
/// tests must pass, and which Lex Primitiva ground the design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSpec {
    /// Human-readable feature or module name.
    pub name: String,
    /// Ordered list of components to create or modify.
    pub components: Vec<Component>,
    /// API contracts that bound the public interface.
    pub contracts: Vec<Contract>,
    /// Ordered test plan entries describing required test cases.
    pub test_plan: Vec<String>,
    /// Lex Primitiva that ground the design (traceability).
    pub primitives: Vec<LexPrimitiva>,
    /// Additional design constraints (e.g. "no heap allocation in hot path").
    pub constraints: Vec<String>,
}

// ---------------------------------------------------------------------------
// FileEntry
// ---------------------------------------------------------------------------

/// A file recorded in a [`SourceArtifact`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Workspace-relative path to the file.
    pub path: String,
    /// SHA-256 hex digest of the file content at the time of recording.
    pub content_hash: String,
    /// Lines of code (non-blank, non-comment) in this file.
    pub loc: u64,
}

// ---------------------------------------------------------------------------
// SourceArtifact — B2 input/output
// ---------------------------------------------------------------------------

/// The typed artifact produced (and incrementally updated) by the B2 Coder
/// station.
///
/// A `SourceArtifact` records every file that was created or modified along
/// with the order in which crates should be built, enabling the B3 Validator
/// to drive `cargo build` deterministically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceArtifact {
    /// All files that form part of this deliverable.
    pub files: Vec<FileEntry>,
    /// Ordered list of crate (or package) names for `cargo build -p`.
    pub build_order: Vec<String>,
    /// Whether the coder station has finished implementing all components.
    pub implemented: bool,
}

// ---------------------------------------------------------------------------
// ValidatedDeliverable — B3 input
// ---------------------------------------------------------------------------

/// The quality gate report produced by the B3 Validator station.
///
/// All fields are populated from a concrete CI run. Call [`is_green`] to
/// determine whether the deliverable may proceed to release.
///
/// [`is_green`]: ValidatedDeliverable::is_green
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedDeliverable {
    /// Whether `cargo build` (or equivalent) exited with success.
    pub build_pass: bool,
    /// Total number of tests discovered.
    pub test_count: u32,
    /// Number of tests that passed.
    pub tests_passed: u32,
    /// Whether `cargo clippy -- -D warnings` exited with success.
    pub lint_pass: bool,
    /// Line-coverage percentage (0.0 – 100.0).
    pub coverage_percent: f64,
    /// Human-readable descriptions of any failures encountered.
    pub failures: Vec<String>,
}

impl ValidatedDeliverable {
    /// Returns `true` when every quality gate has passed.
    ///
    /// A deliverable is green when:
    /// - the build succeeded,
    /// - every discovered test passed,
    /// - the linter found no warnings, and
    /// - there are no recorded failure messages.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::artifact::ValidatedDeliverable;
    ///
    /// let green = ValidatedDeliverable {
    ///     build_pass: true,
    ///     test_count: 5,
    ///     tests_passed: 5,
    ///     lint_pass: true,
    ///     coverage_percent: 88.0,
    ///     failures: vec![],
    /// };
    /// assert!(green.is_green());
    ///
    /// let red = ValidatedDeliverable {
    ///     build_pass: true,
    ///     test_count: 5,
    ///     tests_passed: 4,
    ///     lint_pass: true,
    ///     coverage_percent: 80.0,
    ///     failures: vec!["test_foo panicked".to_string()],
    /// };
    /// assert!(!red.is_green());
    /// ```
    #[must_use]
    pub fn is_green(&self) -> bool {
        self.build_pass
            && self.tests_passed == self.test_count
            && self.lint_pass
            && self.failures.is_empty()
    }
}

// ---------------------------------------------------------------------------
// ShippableArtifact — B3 output
// ---------------------------------------------------------------------------

/// The final artifact emitted by B3 when the quality gate is green.
///
/// A `ShippableArtifact` wraps the full [`ValidatedDeliverable`] together
/// with a changelog entry and a `ready` flag.  The `ready` flag is derived
/// from [`ValidatedDeliverable::is_green`] at construction time via
/// [`ShippableArtifact::from_validated`] and should not be mutated after
/// the fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippableArtifact {
    /// Whether this artifact is cleared for release.
    pub ready: bool,
    /// The full validation report that produced this artifact.
    pub validated: ValidatedDeliverable,
    /// Changelog entry to be appended to `CHANGELOG.md` on release.
    pub changelog_entry: String,
}

impl ShippableArtifact {
    /// Constructs a `ShippableArtifact` from a [`ValidatedDeliverable`].
    ///
    /// The `ready` flag is set to `vd.is_green()`. A default (empty) changelog
    /// entry is provided; callers should populate it before persisting.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_foundry::artifact::{ValidatedDeliverable, ShippableArtifact};
    ///
    /// let vd = ValidatedDeliverable {
    ///     build_pass: true,
    ///     test_count: 3,
    ///     tests_passed: 3,
    ///     lint_pass: true,
    ///     coverage_percent: 95.0,
    ///     failures: vec![],
    /// };
    /// let ship = ShippableArtifact::from_validated(vd);
    /// assert!(ship.ready);
    /// ```
    #[must_use]
    pub fn from_validated(vd: ValidatedDeliverable) -> Self {
        let ready = vd.is_green();
        Self {
            ready,
            validated: vd,
            changelog_entry: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs a minimal [`DesignSpec`], round-trips it through JSON, and
    /// asserts that all fields survive serialisation and deserialisation intact.
    #[test]
    fn design_spec_roundtrip_json() {
        let spec = DesignSpec {
            name: "rate-limiter".to_string(),
            components: vec![
                Component {
                    kind: ComponentKind::RustCrate,
                    name: "nexcore-rate-limiter".to_string(),
                    path: "crates/nexcore-rate-limiter".to_string(),
                },
                Component {
                    kind: ComponentKind::Test,
                    name: "integration-tests".to_string(),
                    path: "crates/nexcore-rate-limiter/tests".to_string(),
                },
            ],
            contracts: vec![Contract {
                input_type: "RateLimitRequest".to_string(),
                output_type: "RateLimitDecision".to_string(),
            }],
            test_plan: vec![
                "token bucket allows burst".to_string(),
                "sliding window rejects excess".to_string(),
            ],
            primitives: vec![LexPrimitiva::Frequency, LexPrimitiva::Boundary],
            constraints: vec!["no heap allocation in hot path".to_string()],
        };

        let json = serde_json::to_string(&spec).unwrap();
        let restored: DesignSpec = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.name, spec.name);
        assert_eq!(restored.components.len(), 2);
        assert_eq!(restored.components[0].kind, ComponentKind::RustCrate);
        assert_eq!(restored.components[1].kind, ComponentKind::Test);
        assert_eq!(restored.contracts.len(), 1);
        assert_eq!(restored.contracts[0].input_type, "RateLimitRequest");
        assert_eq!(restored.contracts[0].output_type, "RateLimitDecision");
        assert_eq!(restored.test_plan.len(), 2);
        assert_eq!(restored.primitives, vec![LexPrimitiva::Frequency, LexPrimitiva::Boundary]);
        assert_eq!(restored.constraints.len(), 1);
        assert_eq!(restored.constraints[0], "no heap allocation in hot path");
    }

    /// Confirms that a [`SourceArtifact`] correctly records and exposes its
    /// ordered build list.
    #[test]
    fn source_artifact_tracks_build_order() {
        let artifact = SourceArtifact {
            files: vec![
                FileEntry {
                    path: "crates/nexcore-primitives/src/lib.rs".to_string(),
                    content_hash: "abc123".to_string(),
                    loc: 120,
                },
            ],
            build_order: vec![
                "nexcore-primitives".to_string(),
                "nexcore-lex-primitiva".to_string(),
                "nexcore-foundry".to_string(),
            ],
            implemented: true,
        };

        assert_eq!(artifact.build_order.len(), 3);
        assert_eq!(artifact.build_order[0], "nexcore-primitives");
        assert_eq!(artifact.build_order[1], "nexcore-lex-primitiva");
        assert_eq!(artifact.build_order[2], "nexcore-foundry");
    }

    /// A fully-passing [`ValidatedDeliverable`] must report `is_green() == true`.
    #[test]
    fn validated_deliverable_reports_pass() {
        let vd = ValidatedDeliverable {
            build_pass: true,
            test_count: 42,
            tests_passed: 42,
            lint_pass: true,
            coverage_percent: 91.3,
            failures: vec![],
        };

        assert!(vd.is_green());
    }

    /// A [`ValidatedDeliverable`] with recorded failures must report
    /// `is_green() == false`.
    #[test]
    fn validated_deliverable_reports_fail() {
        let vd = ValidatedDeliverable {
            build_pass: true,
            test_count: 10,
            tests_passed: 9,
            lint_pass: true,
            coverage_percent: 85.0,
            failures: vec!["test_overflow panicked at src/lib.rs:55".to_string()],
        };

        assert!(!vd.is_green());
    }

    /// [`ShippableArtifact::from_validated`] must set `ready = true` when the
    /// deliverable is green.
    #[test]
    fn shippable_artifact_created_from_validated() {
        let vd = ValidatedDeliverable {
            build_pass: true,
            test_count: 7,
            tests_passed: 7,
            lint_pass: true,
            coverage_percent: 96.0,
            failures: vec![],
        };

        let ship = ShippableArtifact::from_validated(vd);

        assert!(ship.ready);
        assert!(ship.validated.is_green());
        assert_eq!(ship.changelog_entry, "");
    }
}
