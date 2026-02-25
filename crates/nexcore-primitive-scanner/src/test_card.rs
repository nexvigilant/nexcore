//! Primitive test card - 3-test validation protocol.

use crate::types::PrimitiveTier;
use serde::{Deserialize, Serialize};

/// Result of a single test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TestResult {
    /// Test passed.
    Pass,
    /// Test failed.
    Fail,
}

/// Primitive test card output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PrimitiveTestCard {
    /// Term being tested.
    pub term: String,
    /// Definition.
    pub definition: String,
    /// Test 1: No domain-internal dependencies.
    pub test1: TestResult,
    /// Test 2: Grounds to external concepts.
    pub test2: TestResult,
    /// Test 3: Not merely a synonym.
    pub test3: TestResult,
    /// Final verdict.
    pub verdict: Verdict,
    /// Tier classification.
    pub tier: PrimitiveTier,
}

/// Verdict from primitive test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Verdict {
    /// Term is primitive.
    Primitive,
    /// Term is composite.
    Composite,
    /// Undetermined.
    Undetermined,
}

impl PrimitiveTestCard {
    /// Create a new test card.
    #[must_use]
    pub fn new(term: impl Into<String>, definition: impl Into<String>) -> Self {
        Self {
            term: term.into(),
            definition: definition.into(),
            test1: TestResult::Fail,
            test2: TestResult::Fail,
            test3: TestResult::Fail,
            verdict: Verdict::Undetermined,
            tier: PrimitiveTier::T3DomainSpecific,
        }
    }
}
