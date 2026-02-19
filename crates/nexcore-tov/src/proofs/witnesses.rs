//! # Constructive Witness Generation
//!
//! This module implements constructive witness generation — the computational
//! dual of absence reasoning. Where `∅` (Void) encodes meaningful absence,
//! `∃` (Existence) encodes demonstrated presence via a concrete witness.
//!
//! Together, absence (`∅`) and witness generation (`∃`) form the
//! **∅/∃ duality**: two poles of the same epistemic act. To rule out
//! existence is to prove ∅; to establish existence is to produce a witness.
//!
//! ## T1 Primitive Grounding
//!
//! | Primitive | Symbol | Role in this module |
//! |-----------|--------|---------------------|
//! | Existence | ∃      | Dominant — the core act of providing a witness |
//! | Comparison | κ     | Predicate evaluation against each candidate |
//! | Sequence  | σ      | Iteration order through the candidate set |
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
//!     search_witness, validate_witness, existence_proof, count_witnesses,
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

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    NoCandidates,

    /// Returned by [`validate_witness`] when the specific candidate does
    /// not satisfy the predicate.
    #[error("candidate does not satisfy predicate")]
    PredicateNotSatisfied,

    /// Returned when the search loop would exceed the configured iteration
    /// budget. The inner value is the limit that was reached.
    #[error("search exceeded maximum iterations ({0})")]
    MaxIterationsExceeded(usize),

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
/// candidate sets.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::WitnessSearchConfig;
///
/// // Override the default iteration budget
/// let config = WitnessSearchConfig { max_iterations: 500 };
/// assert_eq!(config.max_iterations, 500);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessSearchConfig {
    /// Maximum number of candidates to examine before stopping.
    ///
    /// Defaults to `10_000`.
    pub max_iterations: usize,
}

impl Default for WitnessSearchConfig {
    /// Returns a configuration with a 10 000-iteration budget.
    fn default() -> Self {
        Self {
            max_iterations: 10_000,
        }
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Search for the first witness satisfying a predicate among candidates.
///
/// Iterates through `candidates` in order, evaluating `predicate` for each
/// element, up to `min(candidates.len(), config.max_iterations)`. The first
/// element for which the predicate returns `true` is returned as a
/// [`Witness`] with `confidence = 1.0`.
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
///   range (either exhaustive non-finding or budget exhaustion)
/// - `Err(WitnessError::NoCandidates)` — `candidates` is empty
/// - `Err(WitnessError::EmptyProperty)` — `property` is an empty string
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
        return Err(WitnessError::NoCandidates);
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
/// - `Err(…)`          — propagated from [`search_witness`]
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
    // Guard: propagate pre-condition errors before counting.
    if candidates.is_empty() {
        return Err(WitnessError::NoCandidates);
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

    // No witness found — report exhaustive only if we examined everything.
    Ok(None)
}

/// Count how many candidates satisfy the predicate, bounded by `config`.
///
/// Walks the entire candidate slice up to `config.max_iterations`, counting
/// every element for which `predicate` returns `true`. Unlike
/// [`search_witness`], this does not stop at the first match.
///
/// # Arguments
///
/// - `candidates` — the ordered set of values to examine
/// - `predicate`  — a function returning `true` for qualifying candidates
/// - `config`     — search bounds configuration
///
/// # Returns
///
/// - `Ok(count)` — the number of satisfying candidates within the limit
/// - `Err(WitnessError::NoCandidates)` — `candidates` is empty
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::witnesses::{count_witnesses, WitnessSearchConfig};
///
/// let nums = vec![1u32, 2, 3, 4, 5, 6];
/// let config = WitnessSearchConfig::default();
/// let count = count_witnesses(&nums, |n| n % 2 == 0, &config).unwrap();
/// assert_eq!(count, 3); // 2, 4, 6
/// ```
pub fn count_witnesses<T, F>(
    candidates: &[T],
    predicate: F,
    config: &WitnessSearchConfig,
) -> Result<usize, WitnessError>
where
    F: Fn(&T) -> bool,
{
    if candidates.is_empty() {
        return Err(WitnessError::NoCandidates);
    }

    let limit = candidates.len().min(config.max_iterations);
    let count = candidates.iter().take(limit).filter(|c| predicate(c)).count();
    Ok(count)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Helper: default config for test brevity
    // ------------------------------------------------------------------
    fn cfg() -> WitnessSearchConfig {
        WitnessSearchConfig::default()
    }

    // ------------------------------------------------------------------
    // search_witness — placement tests
    // ------------------------------------------------------------------

    #[test]
    fn search_witness_found_at_first_element() {
        let candidates = vec![10u32, 20, 30];
        let result = search_witness(&candidates, |&n| n == 10, &cfg(), "equals ten");
        let w = result.unwrap().unwrap();
        assert_eq!(w.value, 10);
        assert_eq!(w.property, "equals ten");
        assert_eq!(w.confidence, 1.0);
    }

    #[test]
    fn search_witness_found_at_middle_element() {
        let candidates = vec![1u32, 42, 99];
        let result = search_witness(&candidates, |&n| n == 42, &cfg(), "the answer");
        let w = result.unwrap().unwrap();
        assert_eq!(w.value, 42);
    }

    #[test]
    fn search_witness_found_at_last_element() {
        let candidates = vec![1u32, 2, 3];
        let result = search_witness(&candidates, |&n| n == 3, &cfg(), "last element");
        let w = result.unwrap().unwrap();
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
        assert_eq!(err, WitnessError::NoCandidates);
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

    // ------------------------------------------------------------------
    // search_witness — max_iterations boundary
    // ------------------------------------------------------------------

    #[test]
    fn search_witness_max_iterations_limits_search() {
        // Witness is at index 5 but budget is only 3 — should not be found.
        let candidates: Vec<u32> = (0..10).collect();
        let tight_config = WitnessSearchConfig { max_iterations: 3 };
        let result = search_witness(&candidates, |&n| n == 5, &tight_config, "five");
        // Examined 0,1,2 only — 5 never checked.
        assert_eq!(result.unwrap(), None);
    }

    // ------------------------------------------------------------------
    // validate_witness
    // ------------------------------------------------------------------

    #[test]
    fn validate_witness_valid_candidate() {
        let w = validate_witness(&4u32, |&n| n % 2 == 0, "even number").unwrap();
        assert_eq!(w.value, 4);
        assert_eq!(w.property, "even number");
        assert_eq!(w.confidence, 1.0);
    }

    #[test]
    fn validate_witness_invalid_candidate_returns_error() {
        let err = validate_witness(&3u32, |&n| n % 2 == 0, "even number").unwrap_err();
        assert_eq!(err, WitnessError::PredicateNotSatisfied);
    }

    #[test]
    fn validate_witness_empty_property_returns_error() {
        let err = validate_witness(&1u32, |_| true, "").unwrap_err();
        assert_eq!(err, WitnessError::EmptyProperty);
    }

    // ------------------------------------------------------------------
    // existence_proof — basic correctness
    // ------------------------------------------------------------------

    #[test]
    fn existence_proof_found_carries_correct_metadata() {
        let candidates = vec![1u32, 2, 3, 4, 5];
        // Witness: 3 (index 2), so candidates_examined = 3
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
        let result =
            existence_proof(&candidates, |&n| n % 2 == 0, "even number", &cfg()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn existence_proof_search_exhaustive_flag_when_witness_is_last() {
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
    fn existence_proof_search_exhaustive_flag_when_witness_not_last() {
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
        assert_eq!(err, WitnessError::NoCandidates);
    }

    #[test]
    fn existence_proof_empty_property_returns_error() {
        let candidates = vec![1u32];
        let err = existence_proof(&candidates, |_| true, "", &cfg()).unwrap_err();
        assert_eq!(err, WitnessError::EmptyProperty);
    }

    // ------------------------------------------------------------------
    // count_witnesses
    // ------------------------------------------------------------------

    #[test]
    fn count_witnesses_zero_matches() {
        let candidates = vec![1u32, 3, 5];
        let count = count_witnesses(&candidates, |&n| n % 2 == 0, &cfg()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn count_witnesses_all_match() {
        let candidates = vec![2u32, 4, 6, 8];
        let count = count_witnesses(&candidates, |&n| n % 2 == 0, &cfg()).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn count_witnesses_partial_match() {
        let candidates: Vec<u32> = (1..=10).collect();
        let count = count_witnesses(&candidates, |&n| n % 2 == 0, &cfg()).unwrap();
        assert_eq!(count, 5); // 2, 4, 6, 8, 10
    }

    #[test]
    fn count_witnesses_empty_returns_error() {
        let candidates: Vec<u32> = vec![];
        let err = count_witnesses(&candidates, |_| true, &cfg()).unwrap_err();
        assert_eq!(err, WitnessError::NoCandidates);
    }

    // ------------------------------------------------------------------
    // Large candidate set
    // ------------------------------------------------------------------

    #[test]
    fn search_witness_large_candidate_set() {
        let candidates: Vec<u32> = (0..1_000).collect();
        let w = search_witness(&candidates, |&n| n == 999, &cfg(), "last of thousand")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 999);
    }

    // ------------------------------------------------------------------
    // Complex predicate (multiple conditions)
    // ------------------------------------------------------------------

    #[test]
    fn search_witness_complex_predicate() {
        // Must be even, divisible by 3, and greater than 10
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

    // ------------------------------------------------------------------
    // Integer domain: first even number
    // ------------------------------------------------------------------

    #[test]
    fn search_witness_integer_first_even() {
        let candidates = vec![1u32, 3, 7, 4, 9];
        let w = search_witness(&candidates, |&n| n % 2 == 0, &cfg(), "even number")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, 4);
    }

    // ------------------------------------------------------------------
    // String domain: first string of specific length
    // ------------------------------------------------------------------

    #[test]
    fn search_witness_string_with_specific_length() {
        let candidates = vec!["a", "bc", "def", "ghij"];
        let w = search_witness(&candidates, |s| s.len() == 3, &cfg(), "length-3 string")
            .unwrap()
            .unwrap();
        assert_eq!(w.value, "def");
        assert_eq!(w.property, "length-3 string");
    }

    // ------------------------------------------------------------------
    // Witness confidence is always 1.0 for deterministic predicates
    // ------------------------------------------------------------------

    #[test]
    fn witness_confidence_is_always_one() {
        let candidates = vec![42u32];
        let w = search_witness(&candidates, |_| true, &cfg(), "any value")
            .unwrap()
            .unwrap();
        assert!((w.confidence - 1.0).abs() < f64::EPSILON);
    }

    // ------------------------------------------------------------------
    // count_witnesses respects max_iterations budget
    // ------------------------------------------------------------------

    #[test]
    fn count_witnesses_respects_max_iterations() {
        // 10 even numbers in 0..20 but budget is only 6 → examines 0..5
        let candidates: Vec<u32> = (0..20).collect();
        let tight = WitnessSearchConfig { max_iterations: 6 };
        let count = count_witnesses(&candidates, |&n| n % 2 == 0, &tight).unwrap();
        // Examined: 0,1,2,3,4,5 — even ones: 0,2,4
        assert_eq!(count, 3);
    }

    // ------------------------------------------------------------------
    // WitnessSearchConfig Default
    // ------------------------------------------------------------------

    #[test]
    fn witness_search_config_default_is_ten_thousand() {
        let config = WitnessSearchConfig::default();
        assert_eq!(config.max_iterations, 10_000);
    }
}
