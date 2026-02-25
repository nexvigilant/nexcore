//! Integration tests for nexcore-stoichiometry.
//!
//! These tests exercise the full public API surface through the prelude,
//! verifying encode/decode round-trips, seed term balance, custom encoding,
//! sister detection, and mass state analysis.

use nexcore_stoichiometry::prelude::*;
use nexcore_stoichiometry::{LexPrimitiva, balance::Balancer, equation::BalanceProof};

/// Encode a builtin term, verify it balances, decode it back, verify the
/// Jeopardy question contains the original term name.
#[test]
fn test_full_encode_decode_roundtrip() {
    let codec = StoichiometricCodec::builtin();

    // Pick "Pharmacovigilance" — the canonical seed term
    let term = codec
        .dictionary()
        .lookup("Pharmacovigilance")
        .expect("Pharmacovigilance must exist in builtin dictionary");

    // Verify balanced
    assert!(
        term.equation.balance.is_balanced,
        "Pharmacovigilance equation must be balanced"
    );
    assert!(
        Balancer::verify(&term.equation.balance),
        "Pharmacovigilance balance proof must verify"
    );

    // Decode back
    let answer = codec.decode(&term.equation);
    assert!(answer.is_some(), "decode must return a JeopardyAnswer");

    let answer = answer.unwrap();
    assert!(
        answer.question.contains("Pharmacovigilance"),
        "decoded question '{}' must contain 'Pharmacovigilance'",
        answer.question
    );
    assert!(
        answer.question.starts_with("What is "),
        "Jeopardy format: must start with 'What is '"
    );
    assert!(
        answer.question.ends_with('?'),
        "Jeopardy format: must end with '?'"
    );
    assert!(
        (answer.confidence - 1.0).abs() < f64::EPSILON,
        "exact match should have confidence 1.0"
    );
}

/// Iterate ALL seed terms from the builtin dictionary. Assert every single
/// one has `is_balanced == true` AND `Balancer::verify` passes.
#[test]
fn test_all_seed_terms_balance() {
    let codec = StoichiometricCodec::builtin();
    let terms = codec.dictionary().all_terms();

    assert!(
        terms.len() >= 10,
        "builtin dictionary must have at least 10 seed terms, got {}",
        terms.len()
    );

    for term in &terms {
        assert!(
            term.equation.balance.is_balanced,
            "term '{}' equation is NOT balanced (delta={:.4})",
            term.name, term.equation.balance.delta
        );
        assert!(
            Balancer::verify(&term.equation.balance),
            "term '{}' balance proof does NOT verify",
            term.name
        );

        // Additional: reactant mass must equal product mass within tolerance
        let delta = term.equation.balance.delta.abs();
        assert!(
            delta < 0.001,
            "term '{}' mass delta {:.6} exceeds tolerance",
            term.name,
            delta
        );
    }
}

/// Encode a brand-new custom term using words already in the decomposer
/// vocabulary. Verify the resulting equation balances.
#[test]
fn test_encode_custom_term() {
    let mut codec = StoichiometricCodec::builtin();

    // "Drug Safety" uses words "drug" and "safety" — both in seed vocabulary
    let eq = codec
        .encode(
            "Drug Safety",
            "monitoring and assessment of drug adverse effects",
            DefinitionSource::Custom("integration-test".to_string()),
        )
        .expect("encoding with known vocabulary should succeed");

    assert!(
        eq.balance.is_balanced,
        "custom term 'Drug Safety' must balance"
    );
    assert!(
        Balancer::verify(&eq.balance),
        "custom term balance proof must verify"
    );
    assert!(
        !eq.reactants.is_empty(),
        "custom term must have at least one reactant"
    );
    assert_eq!(eq.concept.name, "Drug Safety", "concept name must match");

    // Decode the custom term back
    let answer = codec.decode(&eq);
    assert!(
        answer.is_some(),
        "custom term must be decodable after registration"
    );
    let answer = answer.unwrap();
    assert!(
        answer.question.contains("Drug Safety"),
        "decoded question must reference 'Drug Safety'"
    );
}

/// Find sisters for "Adverse Event" in the builtin dictionary.
/// "Serious Adverse Event" (SAE) shares significant primitive overlap.
#[test]
fn test_sister_detection_across_dictionary() {
    let codec = StoichiometricCodec::builtin();

    let ae = codec
        .dictionary()
        .lookup("Adverse Event")
        .expect("Adverse Event must exist in builtin dictionary");

    // Threshold 0.3 — moderate overlap
    let sisters = codec.find_sisters(&ae.equation, 0.3);

    assert!(
        !sisters.is_empty(),
        "Adverse Event must have at least one sister at threshold 0.3"
    );

    // Every sister must have valid fields
    for sister in &sisters {
        assert!(
            sister.similarity >= 0.3,
            "sister '{}' similarity {:.3} below threshold",
            sister.name,
            sister.similarity
        );
        assert!(
            sister.similarity <= 1.0,
            "sister '{}' similarity {:.3} exceeds 1.0",
            sister.name,
            sister.similarity
        );
        assert!(
            !sister.shared_primitives.is_empty(),
            "sister '{}' must share at least one primitive",
            sister.name
        );
        // Must not include self
        assert_ne!(
            sister.name, "Adverse Event",
            "sisters must not include self"
        );
    }
}

/// Create MassState from a seed term equation. Verify total_mass > 0,
/// entropy > 0, and depleted returns some primitives (no seed term uses
/// all 15 operational primitives).
#[test]
fn test_mass_state_from_equation() {
    let codec = StoichiometricCodec::builtin();

    let term = codec
        .dictionary()
        .lookup("Pharmacovigilance")
        .expect("Pharmacovigilance must exist");

    let state = MassState::from_equation(&term.equation);

    // Total mass must be positive (non-empty equation)
    assert!(
        state.total_mass() > 0.0,
        "total mass must be > 0, got {:.4}",
        state.total_mass()
    );

    // Entropy must be positive (multiple distinct primitives)
    assert!(
        state.entropy() > 0.0,
        "entropy must be > 0 for a multi-primitive term, got {:.4}",
        state.entropy()
    );

    // Entropy must not exceed theoretical max
    let max_h = MassState::max_entropy();
    assert!(
        state.entropy() <= max_h + 0.001,
        "entropy {:.4} exceeds max {:.4}",
        state.entropy(),
        max_h
    );

    // Depleted: no seed term uses all 15 primitives
    let depleted = state.depleted();
    assert!(
        !depleted.is_empty(),
        "Pharmacovigilance should not use all 15 primitives — some must be depleted"
    );

    // Gibbs free energy must be finite
    let g = state.gibbs_free_energy();
    assert!(g.is_finite(), "Gibbs free energy must be finite");
}

/// Verify PrimitiveInventory correctly indexes all 15 operational primitives
/// and excludes Product.
#[test]
fn test_primitive_inventory_indexes_all_15() {
    let ops = PrimitiveInventory::operational_primitives();
    assert_eq!(
        ops.len(),
        15,
        "must have exactly 15 operational primitives, got {}",
        ops.len()
    );

    // Each must have a valid index
    for (i, &p) in ops.iter().enumerate() {
        let idx = PrimitiveInventory::to_index(p);
        assert!(
            idx.is_some(),
            "operational primitive {:?} at position {} has no index",
            p,
            i
        );
    }

    // Product must be excluded
    assert!(
        PrimitiveInventory::to_index(LexPrimitiva::Product).is_none(),
        "Product must be excluded from operational index"
    );
}

/// Serde round-trip on all public types that derive Serialize/Deserialize.
#[test]
fn test_serde_roundtrip_all_public_types() {
    let codec = StoichiometricCodec::builtin();
    let term = codec
        .dictionary()
        .lookup("Adverse Event")
        .expect("Adverse Event must exist");

    // BalancedEquation round-trip
    let eq_json = serde_json::to_string(&term.equation).expect("BalancedEquation must serialize");
    let eq_back: BalancedEquation =
        serde_json::from_str(&eq_json).expect("BalancedEquation must deserialize");
    assert_eq!(eq_back.concept.name, term.equation.concept.name);
    assert_eq!(
        eq_back.balance.is_balanced,
        term.equation.balance.is_balanced
    );

    // TermEntry round-trip
    let te_json = serde_json::to_string(term).expect("TermEntry must serialize");
    let te_back: TermEntry = serde_json::from_str(&te_json).expect("TermEntry must deserialize");
    assert_eq!(te_back.name, term.name);
    assert_eq!(te_back.definition, term.definition);

    // PrimitiveInventory round-trip
    let inv = &term.equation.balance.reactant_inventory;
    let inv_json = serde_json::to_string(inv).expect("PrimitiveInventory must serialize");
    let inv_back: PrimitiveInventory =
        serde_json::from_str(&inv_json).expect("PrimitiveInventory must deserialize");
    assert!(inv.is_equal(&inv_back));

    // BalanceProof round-trip
    let bp_json =
        serde_json::to_string(&term.equation.balance).expect("BalanceProof must serialize");
    let bp_back: BalanceProof =
        serde_json::from_str(&bp_json).expect("BalanceProof must deserialize");
    assert_eq!(bp_back.is_balanced, term.equation.balance.is_balanced);

    // JeopardyAnswer round-trip
    let answer = JeopardyAnswer {
        question: "What is Adverse Event?".to_string(),
        concept: "Adverse Event".to_string(),
        confidence: 1.0,
        equation_display: "test".to_string(),
    };
    let ja_json = serde_json::to_string(&answer).expect("JeopardyAnswer must serialize");
    let ja_back: JeopardyAnswer =
        serde_json::from_str(&ja_json).expect("JeopardyAnswer must deserialize");
    assert_eq!(ja_back.concept, "Adverse Event");

    // SisterMatch round-trip
    let sister = SisterMatch {
        name: "SAE".to_string(),
        similarity: 0.75,
        shared_primitives: vec![LexPrimitiva::Causality, LexPrimitiva::Boundary],
        unique_to_self: vec![LexPrimitiva::Existence],
        unique_to_other: vec![LexPrimitiva::Irreversibility],
        is_isomer: false,
    };
    let sm_json = serde_json::to_string(&sister).expect("SisterMatch must serialize");
    let sm_back: SisterMatch =
        serde_json::from_str(&sm_json).expect("SisterMatch must deserialize");
    assert_eq!(sm_back.name, "SAE");
    assert!((sm_back.similarity - 0.75).abs() < f64::EPSILON);

    // MassState round-trip
    let ms = MassState::from_equation(&term.equation);
    let ms_json = serde_json::to_string(&ms).expect("MassState must serialize");
    let ms_back: MassState = serde_json::from_str(&ms_json).expect("MassState must deserialize");
    assert!(
        (ms_back.total_mass() - ms.total_mass()).abs() < 0.001,
        "MassState mass mismatch after serde round-trip"
    );

    // StoichiometryError — not Serialize, skip (thiserror enum)
    // DefinitionSource round-trip
    let ds = DefinitionSource::IchGuideline("ICH E2A".to_string());
    let ds_json = serde_json::to_string(&ds).expect("DefinitionSource must serialize");
    let ds_back: DefinitionSource =
        serde_json::from_str(&ds_json).expect("DefinitionSource must deserialize");
    let ds_display = format!("{ds_back}");
    assert!(ds_display.contains("ICH"));
}
