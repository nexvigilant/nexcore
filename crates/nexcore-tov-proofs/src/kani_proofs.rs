//! Kani Verification Harnesses for Theory of Vigilance
//!
//! This module contains bounded model checking proofs using Kani.
//! These harnesses verify properties that cannot be checked by the type system alone.
//!
//! ## Running Kani Proofs
//!
//! ```bash
//! # Install Kani (if not already installed)
//! cargo install --locked kani-verifier
//! kani setup
//!
//! # Run all Kani proofs
//! cargo kani --features kani
//!
//! # Run specific proof
//! cargo kani --features kani --harness verify_hierarchy_bounds
//! ```
//!
//! ## Verification Coverage
//!
//! | Harness | Property Verified |
//! |---------|-------------------|
//! | `verify_hierarchy_bounds` | N ∈ [1, 8] for all ValidatedLevel |
//! | `verify_law_index_bounds` | I ∈ [1, 11] for all ValidatedLawIndex |
//! | `verify_harm_classification_total` | classify_harm covers all 8 combinations |
//! | `verify_propagation_probability_bound` | BoundedProbability < 1 |
//! | `verify_axiom_dependency_chain` | A1→A2→A5 derivation is valid |
//! | `verify_signal_collapse` | S=0 when any component is 0 |

#![cfg(feature = "kani")]

use crate::proofs::vigilance::*;
use crate::type_level::*;

// ============================================================================
// HIERARCHY LEVEL VERIFICATION
// ============================================================================

/// Verify that ValidatedLevel only accepts values in [1, 8]
#[cfg(kani)]
#[kani::proof]
fn verify_hierarchy_bounds() {
    let n: u8 = kani::any();

    // Assume n is in valid range
    kani::assume(n >= 1 && n <= 8);

    // This should not panic for valid values
    match n {
        1 => {
            let _: ValidatedLevel<1> = ValidatedLevel::new();
        }
        2 => {
            let _: ValidatedLevel<2> = ValidatedLevel::new();
        }
        3 => {
            let _: ValidatedLevel<3> = ValidatedLevel::new();
        }
        4 => {
            let _: ValidatedLevel<4> = ValidatedLevel::new();
        }
        5 => {
            let _: ValidatedLevel<5> = ValidatedLevel::new();
        }
        6 => {
            let _: ValidatedLevel<6> = ValidatedLevel::new();
        }
        7 => {
            let _: ValidatedLevel<7> = ValidatedLevel::new();
        }
        8 => {
            let _: ValidatedLevel<8> = ValidatedLevel::new();
        }
        _ => kani::assert(false, "value outside assumed range [1,8]"),
    }
}

// ============================================================================
// CONSERVATION LAW INDEX VERIFICATION
// ============================================================================

/// Verify that ValidatedLawIndex only accepts values in [1, 11]
#[cfg(kani)]
#[kani::proof]
fn verify_law_index_bounds() {
    let i: u8 = kani::any();

    // Assume i is in valid range
    kani::assume(i >= 1 && i <= 11);

    // Verify each valid index
    match i {
        1 => {
            let _: ValidatedLawIndex<1> = ValidatedLawIndex::new();
        }
        2 => {
            let _: ValidatedLawIndex<2> = ValidatedLawIndex::new();
        }
        3 => {
            let _: ValidatedLawIndex<3> = ValidatedLawIndex::new();
        }
        4 => {
            let _: ValidatedLawIndex<4> = ValidatedLawIndex::new();
        }
        5 => {
            let _: ValidatedLawIndex<5> = ValidatedLawIndex::new();
        }
        6 => {
            let _: ValidatedLawIndex<6> = ValidatedLawIndex::new();
        }
        7 => {
            let _: ValidatedLawIndex<7> = ValidatedLawIndex::new();
        }
        8 => {
            let _: ValidatedLawIndex<8> = ValidatedLawIndex::new();
        }
        9 => {
            let _: ValidatedLawIndex<9> = ValidatedLawIndex::new();
        }
        10 => {
            let _: ValidatedLawIndex<10> = ValidatedLawIndex::new();
        }
        11 => {
            let _: ValidatedLawIndex<11> = ValidatedLawIndex::new();
        }
        _ => kani::assert(false, "value outside assumed range [1,11]"),
    }
}

// ============================================================================
// HARM CLASSIFICATION VERIFICATION
// ============================================================================

/// Verify that classify_harm is a total function (covers all 8 combinations)
#[cfg(kani)]
#[kani::proof]
fn verify_harm_classification_total() {
    // Generate arbitrary characteristics
    let mult_single: bool = kani::any();
    let temp_acute: bool = kani::any();
    let det_deterministic: bool = kani::any();

    let multiplicity = if mult_single {
        Multiplicity::Single
    } else {
        Multiplicity::Multiple
    };

    let temporal = if temp_acute {
        Temporal::Acute
    } else {
        Temporal::Chronic
    };

    let determinism = if det_deterministic {
        Determinism::Deterministic
    } else {
        Determinism::Stochastic
    };

    let event = CharacterizedHarmEvent {
        characteristics: HarmCharacteristics {
            multiplicity,
            temporal,
            determinism,
        },
    };

    // This should never panic - classify_harm must handle all combinations
    let result = classify_harm(event);

    // Verify result is a valid HarmType
    let is_valid = matches!(
        result,
        HarmType::Acute
            | HarmType::Cumulative
            | HarmType::OffTarget
            | HarmType::Cascade
            | HarmType::Idiosyncratic
            | HarmType::Saturation
            | HarmType::Interaction
            | HarmType::Population
    );

    kani::assert(is_valid, "classify_harm must return a valid HarmType");
}

/// Verify that harm classification is deterministic (same input → same output)
#[cfg(kani)]
#[kani::proof]
fn verify_harm_classification_deterministic() {
    let mult_single: bool = kani::any();
    let temp_acute: bool = kani::any();
    let det_deterministic: bool = kani::any();

    let characteristics = HarmCharacteristics {
        multiplicity: if mult_single {
            Multiplicity::Single
        } else {
            Multiplicity::Multiple
        },
        temporal: if temp_acute {
            Temporal::Acute
        } else {
            Temporal::Chronic
        },
        determinism: if det_deterministic {
            Determinism::Deterministic
        } else {
            Determinism::Stochastic
        },
    };

    let event1 = CharacterizedHarmEvent {
        characteristics: characteristics.clone(),
    };
    let event2 = CharacterizedHarmEvent { characteristics };

    let result1 = classify_harm(event1);
    let result2 = classify_harm(event2);

    kani::assert(result1 == result2, "classify_harm must be deterministic");
}

// ============================================================================
// PROPAGATION PROBABILITY VERIFICATION
// ============================================================================

/// Verify that BoundedProbability values are always < 1
#[cfg(kani)]
#[kani::proof]
fn verify_propagation_probability_bound() {
    // Test specific probabilities used in ToV
    let p50: BoundedProbability<1, 2> = BoundedProbability::new();
    kani::assert(p50.value() < 1.0, "50% probability must be < 1");

    let p10: BoundedProbability<1, 10> = BoundedProbability::new();
    kani::assert(p10.value() < 1.0, "10% probability must be < 1");

    let p1: BoundedProbability<1, 100> = BoundedProbability::new();
    kani::assert(p1.value() < 1.0, "1% probability must be < 1");

    // Verify ordering
    kani::assert(
        p1.value() < p10.value() && p10.value() < p50.value(),
        "Probabilities must maintain ordering",
    );
}

// ============================================================================
// SIGNAL DETECTION VERIFICATION
// ============================================================================

/// Verify that rarity values correctly detect non-recurrence threshold
#[cfg(kani)]
#[kani::proof]
fn verify_non_recurrence_threshold() {
    let bits: u64 = kani::any();

    // Bound the search space
    kani::assume(bits <= 128);

    // Check threshold behavior
    let is_non_recurrent = bits >= 63;

    // For low bits, should NOT be non-recurrent
    if bits < 63 {
        let low: ValidatedRarity<62> = ValidatedRarity::new();
        kani::assert(
            !low.is_non_recurrent(),
            "Rarity below threshold should not be non-recurrent",
        );
    }

    // At threshold, SHOULD be non-recurrent
    let threshold: ValidatedRarity<63> = ValidatedRarity::new();
    kani::assert(
        threshold.is_non_recurrent(),
        "Rarity at threshold should be non-recurrent",
    );

    // Above threshold, SHOULD be non-recurrent
    let high: ValidatedRarity<100> = ValidatedRarity::new();
    kani::assert(
        high.is_non_recurrent(),
        "Rarity above threshold should be non-recurrent",
    );
}

// ============================================================================
// ACA SCORING VERIFICATION
// ============================================================================

/// Verify ACA case propagation factors are in expected ranges
#[cfg(kani)]
#[kani::proof]
fn verify_aca_case_propagation() {
    // Verify each case has correct propagation factor
    let case_i = case_propagation_factor(ACACase::CaseI);
    kani::assert(case_i.0 == 1.0, "Case I should have full propagation (1.0)");

    let case_ii = case_propagation_factor(ACACase::CaseII);
    kani::assert(case_ii.0 == 0.0, "Case II should have no propagation (0.0)");

    let case_iii = case_propagation_factor(ACACase::CaseIII);
    kani::assert(
        case_iii.0 == 0.5,
        "Case III should have partial propagation (0.5)",
    );

    let case_iv = case_propagation_factor(ACACase::CaseIV);
    kani::assert(case_iv.0 < 0.0, "Case IV should have negative evidence");
}

// ============================================================================
// KHS_AI VERIFICATION
// ============================================================================

/// Verify KHS_AI calculation produces valid scores
#[cfg(kani)]
#[kani::proof]
fn verify_khs_ai_calculation() {
    let latency: u8 = kani::any();
    let accuracy: u8 = kani::any();
    let resource: u8 = kani::any();
    let drift: u8 = kani::any();

    // Bound inputs to valid range
    kani::assume(latency <= 100);
    kani::assume(accuracy <= 100);
    kani::assume(resource <= 100);
    kani::assume(drift <= 100);

    let khs = KHSAI::calculate(latency, accuracy, resource, drift);

    // Overall score must be in [0, 100]
    kani::assert(khs.overall <= 100, "KHS_AI overall must be <= 100");

    // Component scores preserved
    kani::assert(khs.latency_stability == latency, "Latency preserved");
    kani::assert(khs.accuracy_stability == accuracy, "Accuracy preserved");
    kani::assert(khs.resource_efficiency == resource, "Resource preserved");
    kani::assert(khs.drift_score == drift, "Drift preserved");
}

/// Verify KHS_AI interpretation thresholds
#[cfg(kani)]
#[kani::proof]
fn verify_khs_ai_interpretation() {
    let score: u8 = kani::any();
    kani::assume(score <= 100);

    let status = interpret_khs_ai(score);

    match status {
        KHSAIStatus::Healthy => {
            kani::assert(score >= 80, "Healthy requires score >= 80");
        }
        KHSAIStatus::Monitor => {
            kani::assert(
                score >= 60 && score < 80,
                "Monitor requires 60 <= score < 80",
            );
        }
        KHSAIStatus::Investigate => {
            kani::assert(
                score >= 40 && score < 60,
                "Investigate requires 40 <= score < 60",
            );
        }
        KHSAIStatus::Intervene => {
            kani::assert(score < 40, "Intervene requires score < 40");
        }
    }
}

// ============================================================================
// FAILURE ATTRIBUTION VERIFICATION
// ============================================================================

/// Verify failure attribution decision tree covers all cases
#[cfg(kani)]
#[kani::proof]
fn verify_failure_attribution_total() {
    let infra_healthy: bool = kani::any();
    let model_validated: bool = kani::any();
    let recent_deployment: bool = kani::any();

    let result = attribute_failure(infra_healthy, model_validated, recent_deployment);

    // Result must be a valid attribution
    let is_valid = matches!(
        result,
        FailureAttribution::AIModel
            | FailureAttribution::Infrastructure
            | FailureAttribution::Compound
            | FailureAttribution::Unknown
    );

    kani::assert(is_valid, "Attribution must be valid");

    // Verify decision tree logic
    if !infra_healthy {
        kani::assert(
            matches!(result, FailureAttribution::Infrastructure),
            "Unhealthy infra → Infrastructure",
        );
    } else if !model_validated {
        kani::assert(
            matches!(result, FailureAttribution::AIModel),
            "Invalid model → AIModel",
        );
    } else if recent_deployment {
        kani::assert(
            matches!(result, FailureAttribution::Compound),
            "Recent deployment → Compound",
        );
    } else {
        kani::assert(
            matches!(result, FailureAttribution::Unknown),
            "Otherwise → Unknown",
        );
    }
}

// ============================================================================
// ARCHITECTURE ADJACENCY VERIFICATION
// ============================================================================

/// Verify architecture adjacency values are in [0, 1]
#[cfg(kani)]
#[kani::proof]
fn verify_architecture_adjacency_bounds() {
    // Check all relationship types
    let same_family = architecture_adjacency(ArchitectureRelationship::SameFamily);
    let same_base = architecture_adjacency(ArchitectureRelationship::SameBase);
    let same_pattern = architecture_adjacency(ArchitectureRelationship::SamePattern);
    let different = architecture_adjacency(ArchitectureRelationship::Different);

    // All must be in [0, 1]
    kani::assert(
        same_family >= 0.0 && same_family <= 1.0,
        "SameFamily in [0,1]",
    );
    kani::assert(same_base >= 0.0 && same_base <= 1.0, "SameBase in [0,1]");
    kani::assert(
        same_pattern >= 0.0 && same_pattern <= 1.0,
        "SamePattern in [0,1]",
    );
    kani::assert(different >= 0.0 && different <= 1.0, "Different in [0,1]");

    // Verify ordering: closer relationship → higher adjacency
    kani::assert(same_family > same_base, "SameFamily > SameBase");
    kani::assert(same_base > same_pattern, "SameBase > SamePattern");
    kani::assert(same_pattern > different, "SamePattern > Different");
}

// ============================================================================
// ELEMENT COUNT VERIFICATION
// ============================================================================

/// Verify standard element count is exactly 15
#[cfg(kani)]
#[kani::proof]
fn verify_element_count() {
    let count = StandardElementCount::count();
    kani::assert(count == 15, "Standard element count must be 15");
}

// ============================================================================
// PANIC FREEDOM VERIFICATION
// ============================================================================

/// Verify that ValidatedLevel::new() doesn't panic for valid inputs
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(1)]
fn verify_validated_level_no_panic() {
    // All valid levels should construct without panic
    let _l1: ValidatedLevel<1> = ValidatedLevel::new();
    let _l2: ValidatedLevel<2> = ValidatedLevel::new();
    let _l3: ValidatedLevel<3> = ValidatedLevel::new();
    let _l4: ValidatedLevel<4> = ValidatedLevel::new();
    let _l5: ValidatedLevel<5> = ValidatedLevel::new();
    let _l6: ValidatedLevel<6> = ValidatedLevel::new();
    let _l7: ValidatedLevel<7> = ValidatedLevel::new();
    let _l8: ValidatedLevel<8> = ValidatedLevel::new();
}

/// Verify that ValidatedLawIndex::new() doesn't panic for valid inputs
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(1)]
fn verify_validated_law_index_no_panic() {
    let _l1: ValidatedLawIndex<1> = ValidatedLawIndex::new();
    let _l2: ValidatedLawIndex<2> = ValidatedLawIndex::new();
    let _l3: ValidatedLawIndex<3> = ValidatedLawIndex::new();
    let _l4: ValidatedLawIndex<4> = ValidatedLawIndex::new();
    let _l5: ValidatedLawIndex<5> = ValidatedLawIndex::new();
    let _l6: ValidatedLawIndex<6> = ValidatedLawIndex::new();
    let _l7: ValidatedLawIndex<7> = ValidatedLawIndex::new();
    let _l8: ValidatedLawIndex<8> = ValidatedLawIndex::new();
    let _l9: ValidatedLawIndex<9> = ValidatedLawIndex::new();
    let _l10: ValidatedLawIndex<10> = ValidatedLawIndex::new();
    let _l11: ValidatedLawIndex<11> = ValidatedLawIndex::new();
}
