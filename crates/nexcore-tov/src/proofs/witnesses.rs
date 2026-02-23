//! # Constructive Witness Generation
//!
//! This module implements constructive witness generation — the computational
//! dual of absence reasoning. Where `∅` (Void) encodes meaningful absence,
//! `∃` (Existence) encodes demonstrated presence via a concrete witness.
//!
//! Together, absence (`∅`) and witness generation (`∃`) form the
//! **∅/∃ duality**: two poles of the same epistemic act. To rule out
//! existence is to prove `∅`; to establish existence is to produce a witness.
//!
//! ## T1 Primitive Grounding
//!
//! | Primitive  | Symbol | Role in this module                               |
//! |------------|--------|---------------------------------------------------|
//! | Existence  | ∃      | Dominant — the core act of providing a witness    |
//! | Comparison | κ      | Predicate evaluation against each candidate       |
//! | Sequence   | σ      | Iteration order through the candidate set         |
//!
//! ## PV Transfer
//!
//! **"Does any drug in this class exhibit this ADR?"**
//!
//! Constructive witness generation maps directly onto pharmacovigilance
//! evidence search: given a class of compounds and an adverse drug reaction
//! (ADR) predicate, search the candidate population for any substance that
//! satisfies the safety signal threshold. A single confirmed witness is
//! sufficient to establish class-level association risk.
//!
//! ## Usage
//!
//! ```
//! use nexcore_tov::proofs::witnesses::{
//!     search_witness, validate_witness, existence_proof,
//!     WitnessSearchConfig,
//! };
//!
//! let candidates = vec![1u32, 2, 3, 4, 5];
//! let config = WitnessSearchConfig::default();
//!
//! // Find first even number
//! let result = search_witness(&candidates, |n| n % 2 == 0, &config, "even number").unwrap();
//! assert!(result.is_some());
//! assert_eq!(result.unwrap().value, 2);
//! ```

use nexcore_error::Error;
use serde::{Deserialize, Serialize};

// ============================================================================
// ERROR TYPE
// ============================================================================

/// Errors that can occur during witness search and validation.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum WitnessError {
    /// Returned when no candidates are provided to search.
    ///
    /// Searching an empty set is epistemically undefined: there is nothing
    /// to examine, so neither existence nor absence can be established.
    #[error("no candidates provided for witness search")]
    EmptyCandidates,

    /// Returned by [`validate_witness`] when the specific candidate does
    /// not satisfy the predicate.
    #[error("candidate does not satisfy predicate")]
    PredicateNotSatisfied,

    /// Returned when the search exhausted the configured iteration budget
    /// without finding a witness. The inner value is the limit that was reached.
    #[error("search exhausted maximum iterations ({0})")]
    SearchExhausted(usize),

    /// Returned when the `property` description string is empty.
    ///
    /// An unnamed property cannot meaningfully label a witness — witnesses
    /// must carry a human-readable description of what they prove.
    #[error("property description must not be empty")]
    EmptyProperty,
}

// ============================================================================
// CORE TYPES
// ============================================================================

/// A concrete value that proves existence of a named property.
///
/// A `Witness<T>` carries:
/// - `value`: the concrete element drawn from the candidate set
/// - `property`: a human-readable description of what the value proves
/// - `confidence`: a `[0.0, 1.0]` score; deterministic predicate checks
///   always yield `1.0`
///
/// # Primitive Grounding
///
/// `Witness<T>` grounds to `∃` (Existence, dominant) + `κ` (Comparison)
/// + `π` (Persistence). It materialises the existential claim by holding the
/// evidence that justifies it.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::{validate_witness, WitnessError};
///
/// let result = validate_witness(&42u32, |n| *n > 40, "greater than 40");
/// assert!(result.is_ok());
/// let witness = result.unwrap();
/// assert_eq!(witness.value, 42);
/// assert_eq!(witness.property, "greater than 40");
/// assert_eq!(witness.confidence, 1.0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Witness<T> {
    /// The concrete value that satisfies the property.
    pub value: T,
    /// Human-readable description of the property proven.
    pub property: String,
    /// Confidence in the witness verdict. Deterministic predicate checks
    /// always yield `1.0`.
    pub confidence: f64,
}

/// A complete existence proof with provenance metadata.
///
/// Wraps a [`Witness<T>`] with audit information: how large was the
/// search space, how many candidates were examined before the witness
/// was found, and whether the search ran to exhaustion.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::{existence_proof, WitnessSearchConfig};
///
/// let candidates = vec![10u32, 20, 30];
/// let config = WitnessSearchConfig::default();
/// let proof = existence_proof(&candidates, |n| *n == 20, "equals twenty", &config).unwrap();
/// let proof = proof.unwrap();
/// assert_eq!(proof.witness.value, 20);
/// assert_eq!(proof.candidates_examined, 2); // 10 failed, 20 succeeded
/// assert_eq!(proof.search_space_size, 3);
/// assert!(!proof.search_exhaustive); // did not check 30
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExistenceProof<T> {
    /// The witness that establishes existence.
    pub witness: Witness<T>,
    /// Total number of candidates available in the search space.
    pub search_space_size: usize,
    /// Number of candidates examined before the witness was found.
    pub candidates_examined: usize,
    /// `true` if every candidate in the search space was checked (either
    /// the witness was the last element, or no witness was found at all).
    pub search_exhaustive: bool,
}

/// Configuration for bounded witness search.
///
/// Prevents unbounded iteration on pathological or adversarially large
/// candidate sets. The `timeout_ms` field is carried as documentation
/// metadata — timeout enforcement is the caller's responsibility.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::WitnessSearchConfig;
///
/// // Exhaustive search (default)
/// let config = WitnessSearchConfig::default();
/// assert_eq!(config.max_iterations, usize::MAX);
///
/// // Bounded search
/// let limited = WitnessSearchConfig::limited(500);
/// assert_eq!(limited.max_iterations, 500);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessSearchConfig {
    /// Maximum number of candidates to examine before stopping.
    ///
    /// Defaults to [`usize::MAX`] (exhaustive — examine every candidate).
    pub max_iterations: usize,
    /// Optional wall-clock timeout hint in milliseconds.
    ///
    /// Not enforced at the function level; callers are responsible for
    /// wrapping searches with their own deadline logic. Carried here for
    /// documentation, logging, and future executor integration.
    pub timeout_ms: Option<u64>,
}

impl Default for WitnessSearchConfig {
    /// Returns an exhaustive configuration: no iteration cap, no timeout hint.
    ///
    /// Use [`WitnessSearchConfig::limited`] for bounded searches.
    fn default() -> Self {
        Self {
            max_iterations: usize::MAX,
            timeout_ms: None,
        }
    }
}

impl WitnessSearchConfig {
    /// Create a bounded search configuration with a fixed iteration cap.
    ///
    /// Useful when the candidate set may be large and an exact witness is
    /// not required — a partial search suffices for early-evidence use cases.
    ///
    /// # Example
    ///
    /// ```
    /// use nexcore_tov::proofs::witnesses::WitnessSearchConfig;
    ///
    /// let config = WitnessSearchConfig::limited(100);
    /// assert_eq!(config.max_iterations, 100);
    /// assert_eq!(config.timeout_ms, None);
    /// ```
    #[must_use]
    pub const fn limited(max: usize) -> Self {
        Self {
            max_iterations: max,
            timeout_ms: None,
        }
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Search for the first witness satisfying a predicate among candidates.
///
/// Iterates through `candidates` in order (σ — Sequence), evaluating
/// `predicate` (κ — Comparison) for each element, up to
/// `min(candidates.len(), config.max_iterations)`. The first element for
/// which the predicate returns `true` is returned as a [`Witness`] with
/// `confidence = 1.0`.
///
/// # Arguments
///
/// - `candidates` — the ordered set of values to examine
/// - `predicate`  — a function returning `true` when a candidate qualifies
/// - `config`     — search bounds configuration
/// - `property`   — a non-empty description of what the witness proves
///
/// # Returns
///
/// - `Ok(Some(witness))` — a qualifying element was found
/// - `Ok(None)`          — no qualifying element exists within the examined
///   range (exhaustive non-finding or budget exhaustion without a match)
/// - `Err(WitnessError::EmptyCandidates)` — `candidates` is empty
/// - `Err(WitnessError::EmptyProperty)`   — `property` is an empty string
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::{search_witness, WitnessSearchConfig};
///
/// let nums = vec![1u32, 3, 5, 8, 11];
/// let config = WitnessSearchConfig::default();
/// let w = search_witness(&nums, |n| n % 2 == 0, &config, "even number")
///     .unwrap()
///     .unwrap();
/// assert_eq!(w.value, 8);
/// ```
#[must_use = "the search result carries existence evidence; ignoring it discards the proof"]
pub fn search_witness<T, F>(
    candidates: &[T],
    predicate: F,
    config: &WitnessSearchConfig,
    property: &str,
) -> Result<Option<Witness<T>>, WitnessError>
where
    T: Clone,
    F: Fn(&T) -> bool,
{
    if candidates.is_empty() {
        return Err(WitnessError::EmptyCandidates);
    }
    if property.is_empty() {
        return Err(WitnessError::EmptyProperty);
    }

    let limit = candidates.len().min(config.max_iterations);

    for candidate in candidates.iter().take(limit) {
        if predicate(candidate) {
            return Ok(Some(Witness {
                value: candidate.clone(),
                property: property.to_owned(),
                confidence: 1.0,
            }));
        }
    }

    Ok(None)
}

/// Validate that a specific candidate is a valid witness for a property.
///
/// Unlike [`search_witness`], this function evaluates a single pre-selected
/// candidate. It is useful when the caller already has a value and wants to
/// confirm it satisfies a predicate before wrapping it as an existence claim.
///
/// # Arguments
///
/// - `candidate` — the value to test
/// - `predicate` — a function returning `true` when the candidate qualifies
/// - `property`  — a non-empty description of the property being proven
///
/// # Returns
///
/// - `Ok(witness)` — the candidate satisfies the predicate
/// - `Err(WitnessError::PredicateNotSatisfied)` — the candidate fails
/// - `Err(WitnessError::EmptyProperty)` — `property` is an empty string
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::{validate_witness, WitnessError};
///
/// // Valid witness
/// let ok = validate_witness(&10u32, |n| *n % 2 == 0, "even number");
/// assert!(ok.is_ok());
///
/// // Invalid witness
/// let err = validate_witness(&7u32, |n| *n % 2 == 0, "even number");
/// assert_eq!(err.unwrap_err(), WitnessError::PredicateNotSatisfied);
/// ```
#[must_use = "validation result carries the proof; ignoring it loses the witness"]
pub fn validate_witness<T, F>(
    candidate: &T,
    predicate: F,
    property: &str,
) -> Result<Witness<T>, WitnessError>
where
    T: Clone,
    F: Fn(&T) -> bool,
{
    if property.is_empty() {
        return Err(WitnessError::EmptyProperty);
    }

    if predicate(candidate) {
        Ok(Witness {
            value: candidate.clone(),
            property: property.to_owned(),
            confidence: 1.0,
        })
    } else {
        Err(WitnessError::PredicateNotSatisfied)
    }
}

/// Construct a full existence proof with provenance metadata.
///
/// Calls [`search_witness`] internally and, if a witness is found, wraps it
/// in an [`ExistenceProof`] that records exactly how many candidates were
/// examined and whether the search ran to exhaustion.
///
/// ## `search_exhaustive` semantics
///
/// `search_exhaustive` is `true` when **all** candidates in the search
/// space were examined — either because no witness was found after checking
/// every element, or because the matching element happened to be the last
/// candidate. It is `false` when a witness was found before reaching the
/// end of the candidate list.
///
/// ## Confidence semantics
///
/// `witness.confidence` is always `1.0`: the predicate is deterministic, so
/// any found witness is a certain proof of ∃.
///
/// # Arguments
///
/// - `candidates` — the ordered set of values to examine
/// - `predicate`  — a function returning `true` when a candidate qualifies
/// - `property`   — a non-empty description of what the witness proves
/// - `config`     — search bounds configuration
///
/// # Returns
///
/// - `Ok(Some(proof))` — existence is established; proof carries metadata
/// - `Ok(None)`        — no witness was found within the examined range
/// - `Err(…)`          — propagated from validation guards
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::{existence_proof, WitnessSearchConfig};
///
/// let candidates = vec![1u32, 2, 3, 4, 5];
/// let config = WitnessSearchConfig::default();
/// let proof = existence_proof(&candidates, |n| *n > 3, "greater than 3", &config)
///     .unwrap()
///     .unwrap();
/// assert_eq!(proof.witness.value, 4);
/// assert_eq!(proof.candidates_examined, 4); // 1, 2, 3 fail; 4 succeeds
/// assert!(!proof.search_exhaustive);
/// ```
#[must_use = "existence proof carries provenance metadata; ignoring it discards audit evidence"]
pub fn existence_proof<T, F>(
    candidates: &[T],
    predicate: F,
    property: &str,
    config: &WitnessSearchConfig,
) -> Result<Option<ExistenceProof<T>>, WitnessError>
where
    T: Clone,
    F: Fn(&T) -> bool,
{
    if candidates.is_empty() {
        return Err(WitnessError::EmptyCandidates);
    }
    if property.is_empty() {
        return Err(WitnessError::EmptyProperty);
    }

    let search_space_size = candidates.len();
    let limit = search_space_size.min(config.max_iterations);

    for (idx, candidate) in candidates.iter().take(limit).enumerate() {
        if predicate(candidate) {
            let candidates_examined = idx + 1;
            let search_exhaustive = candidates_examined == search_space_size;
            return Ok(Some(ExistenceProof {
                witness: Witness {
                    value: candidate.clone(),
                    property: property.to_owned(),
                    confidence: 1.0,
                },
                search_space_size,
                candidates_examined,
                search_exhaustive,
            }));
        }
    }

    // No witness found within the iteration budget.
    Ok(None)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helper: exhaustive default config, used throughout for brevity.
    // -----------------------------------------------------------------------
    fn cfg() -> WitnessSearchConfig {
        WitnessSearchConfig::default()
    }

    // -----------------------------------------------------------------------
    // WitnessSearchConfig construction
    // -----------------------------------------------------------------------

    #[test]
    fn config_default_is_exhaustive() {
        let config = WitnessSearchConfig::default();
        assert_eq!(config.max_iterations, usize::MAX);
        assert_eq!(config.timeout_ms, None);
    }

    #[test]
    fn config_limited_sets_max_iterations() {
        let config = WitnessSearchConfig::limited(500);
        assert_eq!(config.max_iterations, 500);
        assert_eq!(config.timeout_ms, None);
    }

    #[test]
    fn config_limited_zero_creates_zero_budget() {
        let config = WitnessSearchConfig::limited(0);
        assert_eq!(config.max_iterations, 0);
    }

    // -----------------------------------------------------------------------
    // search_witness — placement tests
    // -----------------------------------------------------------------------

    #[test]
    fn search_witness_found_at_first_element() {
        let candidates = vec![10u32, 20, 30];
        let w = search_witness(&candidates, |&n| n == 10, &cfg(), "equals ten")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 10);
        assert_eq!(w.property, "equals ten");
        assert_eq!(w.confidence, 1.0);
    }

    #[test]
    fn search_witness_found_at_middle_element() {
        let candidates = vec![1u32, 42, 99];
        let w = search_witness(&candidates, |&n| n == 42, &cfg(), "the answer")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 42);
    }

    #[test]
    fn search_witness_found_at_last_element() {
        let candidates = vec![1u32, 2, 3];
        let w = search_witness(&candidates, |&n| n == 3, &cfg(), "last element")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 3);
    }

    #[test]
    fn search_witness_no_match_returns_none() {
        let candidates = vec![1u32, 3, 5, 7];
        let result = search_witness(&candidates, |&n| n % 2 == 0, &cfg(), "even number");
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn search_witness_empty_candidates_returns_error() {
        let candidates: Vec<u32> = vec![];
        let err = search_witness(&candidates, |_| true, &cfg(), "any").unwrap_err();
        assert_eq!(err, WitnessError::EmptyCandidates);
    }

    #[test]
    fn search_witness_empty_property_returns_error() {
        let candidates = vec![1u32];
        let err = search_witness(&candidates, |_| true, &cfg(), "").unwrap_err();
        assert_eq!(err, WitnessError::EmptyProperty);
    }

    #[test]
    fn search_witness_always_true_returns_first_candidate() {
        let candidates = vec![100u32, 200, 300];
        let w = search_witness(&candidates, |_| true, &cfg(), "anything")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 100);
    }

    #[test]
    fn search_witness_always_false_returns_none() {
        let candidates = vec![1u32, 2, 3];
        let result = search_witness(&candidates, |_| false, &cfg(), "impossible");
        assert_eq!(result.unwrap(), None);
    }

    // -----------------------------------------------------------------------
    // search_witness — max_iterations boundary
    // -----------------------------------------------------------------------

    #[test]
    fn search_witness_max_iterations_zero_returns_none() {
        // Budget of zero: no candidates examined, even the first.
        let candidates = vec![1u32, 2, 3];
        let zero_config = WitnessSearchConfig::limited(0);
        let result = search_witness(&candidates, |_| true, &zero_config, "any");
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn search_witness_max_iterations_one_only_checks_first() {
        // Budget of 1: only index 0 is examined.
        let candidates = vec![1u32, 2, 3];
        let one_config = WitnessSearchConfig::limited(1);
        // Predicate: matches only 2 (index 1) — should not be found.
        let result = search_witness(&candidates, |&n| n == 2, &one_config, "two");
        assert_eq!(result.unwrap(), None);
        // But 1 (index 0) should be found.
        let w = search_witness(&candidates, |&n| n == 1, &one_config, "one")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 1);
    }

    #[test]
    fn search_witness_max_iterations_limits_search_to_budget() {
        // Witness is at index 5 but budget is only 3 — should not be found.
        let candidates: Vec<u32> = (0..10).collect();
        let tight_config = WitnessSearchConfig::limited(3);
        let result = search_witness(&candidates, |&n| n == 5, &tight_config, "five");
        assert_eq!(result.unwrap(), None);
    }

    // -----------------------------------------------------------------------
    // validate_witness
    // -----------------------------------------------------------------------

    #[test]
    fn validate_witness_valid_candidate() {
        let w = validate_witness(&4u32, |&n| n % 2 == 0, "even number").unwrap();
        assert_eq!(w.value, 4);
        assert_eq!(w.property, "even number");
        assert_eq!(w.confidence, 1.0);
    }

    #[test]
    fn validate_witness_invalid_candidate_returns_predicate_not_satisfied() {
        let err = validate_witness(&3u32, |&n| n % 2 == 0, "even number").unwrap_err();
        assert_eq!(err, WitnessError::PredicateNotSatisfied);
    }

    #[test]
    fn validate_witness_empty_property_returns_error() {
        let err = validate_witness(&1u32, |_| true, "").unwrap_err();
        assert_eq!(err, WitnessError::EmptyProperty);
    }

    #[test]
    fn validate_witness_string_type() {
        let w = validate_witness(&"hello", |s| s.len() == 5, "length-5 string").unwrap();
        assert_eq!(w.value, "hello");
        assert_eq!(w.confidence, 1.0);
    }

    // -----------------------------------------------------------------------
    // existence_proof — basic correctness
    // -----------------------------------------------------------------------

    #[test]
    fn existence_proof_found_carries_correct_metadata() {
        let candidates = vec![1u32, 2, 3, 4, 5];
        // Witness: 3 (index 2), so candidates_examined = 3.
        let proof = existence_proof(&candidates, |&n| n == 3, "equals three", &cfg())
            .unwrap()
            .unwrap();
        assert_eq!(proof.witness.value, 3);
        assert_eq!(proof.search_space_size, 5);
        assert_eq!(proof.candidates_examined, 3);
        assert!(!proof.search_exhaustive);
    }

    #[test]
    fn existence_proof_not_found_returns_none() {
        let candidates = vec![1u32, 3, 5];
        let result = existence_proof(&candidates, |&n| n % 2 == 0, "even number", &cfg()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn existence_proof_search_exhaustive_true_when_witness_is_last() {
        let candidates = vec![1u32, 3, 5, 8];
        // 8 is even and is the last element — exhaustive should be true.
        let proof = existence_proof(&candidates, |&n| n % 2 == 0, "even number", &cfg())
            .unwrap()
            .unwrap();
        assert_eq!(proof.witness.value, 8);
        assert_eq!(proof.candidates_examined, 4);
        assert!(proof.search_exhaustive);
    }

    #[test]
    fn existence_proof_search_exhaustive_false_when_witness_not_last() {
        let candidates = vec![2u32, 3, 5];
        // 2 is even and is at index 0 — examined 1, total 3 → not exhaustive.
        let proof = existence_proof(&candidates, |&n| n % 2 == 0, "even number", &cfg())
            .unwrap()
            .unwrap();
        assert_eq!(proof.candidates_examined, 1);
        assert!(!proof.search_exhaustive);
    }

    #[test]
    fn existence_proof_empty_candidates_returns_error() {
        let candidates: Vec<u32> = vec![];
        let err = existence_proof(&candidates, |_| true, "any", &cfg()).unwrap_err();
        assert_eq!(err, WitnessError::EmptyCandidates);
    }

    #[test]
    fn existence_proof_empty_property_returns_error() {
        let candidates = vec![1u32];
        let err = existence_proof(&candidates, |_| true, "", &cfg()).unwrap_err();
        assert_eq!(err, WitnessError::EmptyProperty);
    }

    #[test]
    fn existence_proof_confidence_is_always_one() {
        let candidates = vec![7u32];
        let proof = existence_proof(&candidates, |_| true, "any value", &cfg())
            .unwrap()
            .unwrap();
        assert!((proof.witness.confidence - 1.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Exhaustive vs partial search metadata
    // -----------------------------------------------------------------------

    #[test]
    fn search_exhaustive_single_element_set_is_exhaustive() {
        let candidates = vec![42u32];
        let proof = existence_proof(&candidates, |_| true, "sole element", &cfg())
            .unwrap()
            .unwrap();
        assert_eq!(proof.search_space_size, 1);
        assert_eq!(proof.candidates_examined, 1);
        assert!(proof.search_exhaustive);
    }

    // -----------------------------------------------------------------------
    // Various types: i32, String, custom struct
    // -----------------------------------------------------------------------

    #[test]
    fn search_witness_i32_negative_values() {
        let candidates = vec![-5i32, -3, -1, 0, 2, 4];
        let w = search_witness(&candidates, |&n| n > 0, &cfg(), "positive integer")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 2);
    }

    #[test]
    fn search_witness_string_type() {
        let candidates = vec!["alpha", "beta", "gamma"];
        let w = search_witness(&candidates, |s| s.starts_with('g'), &cfg(), "starts with g")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, "gamma");
    }

    #[test]
    fn search_witness_struct_type() {
        #[derive(Clone, PartialEq, Debug)]
        struct Drug {
            name: &'static str,
            signal_strength: f64,
        }
        let candidates = vec![
            Drug {
                name: "DrugA",
                signal_strength: 1.2,
            },
            Drug {
                name: "DrugB",
                signal_strength: 2.5,
            },
            Drug {
                name: "DrugC",
                signal_strength: 0.8,
            },
        ];
        let w = search_witness(
            &candidates,
            |d| d.signal_strength >= 2.0,
            &cfg(),
            "signal >= PRR threshold",
        )
        .unwrap()
        .unwrap();
        assert_eq!(w.value.name, "DrugB");
        assert_eq!(w.property, "signal >= PRR threshold");
    }

    // -----------------------------------------------------------------------
    // Large candidate set
    // -----------------------------------------------------------------------

    #[test]
    fn search_witness_large_candidate_set() {
        let candidates: Vec<u32> = (0..1_000).collect();
        let w = search_witness(&candidates, |&n| n == 999, &cfg(), "last of thousand")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 999);
    }

    // -----------------------------------------------------------------------
    // Complex predicate (multiple conditions)
    // -----------------------------------------------------------------------

    #[test]
    fn search_witness_complex_predicate() {
        // Must be even, divisible by 3, and greater than 10.
        let candidates: Vec<u32> = (1..=20).collect();
        let w = search_witness(
            &candidates,
            |&n| n % 2 == 0 && n % 3 == 0 && n > 10,
            &cfg(),
            "even, divisible by 3, greater than 10",
        )
        .unwrap()
        .unwrap();
        assert_eq!(w.value, 12);
        assert_eq!(w.property, "even, divisible by 3, greater than 10");
    }

    // -----------------------------------------------------------------------
    // Error variant display (smoke-test the derive macro)
    // -----------------------------------------------------------------------

    #[test]
    fn witness_error_display_empty_candidates() {
        let msg = WitnessError::EmptyCandidates.to_string();
        assert!(msg.contains("no candidates"));
    }

    #[test]
    fn witness_error_display_predicate_not_satisfied() {
        let msg = WitnessError::PredicateNotSatisfied.to_string();
        assert!(msg.contains("predicate"));
    }

    #[test]
    fn witness_error_display_search_exhausted() {
        let msg = WitnessError::SearchExhausted(42).to_string();
        assert!(msg.contains("42"));
    }

    #[test]
    fn witness_error_display_empty_property() {
        let msg = WitnessError::EmptyProperty.to_string();
        assert!(msg.contains("property"));
    }
}
