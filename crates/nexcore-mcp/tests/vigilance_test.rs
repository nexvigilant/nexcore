//! Integration tests for Vigilance tools
//!
//! Covers safety margin, risk scoring, ToV mapping, and harm types.

use nexcore_mcp::params::{
    MapToTovParams, RiskScoreGeometricParams, RiskScoreParams, SafetyMarginParams,
};
use nexcore_mcp::tools::vigilance;

#[test]
fn test_safety_margin_signal() {
    // Setup - High PRR and high case count (Actionable Signal)
    let params = SafetyMarginParams {
        prr: 4.5,
        ror_lower: 2.1,
        ic025: 1.8,
        eb05: 2.5,
        n: 120,
    };

    // Execute
    let result = vigilance::safety_margin(params);

    // Assert
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap();
    assert!(text.text.contains("Confirmed Axiomatic Violation"));
    assert!(text.text.contains("\"distance\":"));
}

#[test]
fn test_safety_margin_weak_signal() {
    // Setup - Low PRR and low case count (No Signal)
    let params = SafetyMarginParams {
        prr: 1.2,
        ror_lower: 0.5,
        ic025: -0.1,
        eb05: 0.8,
        n: 2,
    };

    // Execute
    let result = vigilance::safety_margin(params);

    // Assert
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap();
    assert!(text.text.contains("Safe"));
}

#[test]
fn test_risk_score_calculation() {
    // Setup
    let params = RiskScoreParams {
        drug: "NexDrug-Alpha".to_string(),
        event: "Acute Hepatotoxicity".to_string(),
        prr: 5.0,
        ror_lower: 2.5,
        ic025: 1.5,
        eb05: 2.0,
        n: 50,
    };

    // Execute
    let result = vigilance::risk_score(params);

    // Assert
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap();
    assert!(text.text.contains("NexDrug-Alpha"));
    assert!(text.text.contains("Acute Hepatotoxicity"));
    assert!(text.text.contains("\"score\":"));
    assert!(text.text.contains("\"level\":"));
}

#[test]
fn test_harm_types_listing() {
    // Execute
    let result = vigilance::harm_types();

    // Assert
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap();
    assert!(text.text.contains("harm_types"));
    assert!(text.text.contains("\"count\":8"));
    assert!(text.text.contains("Temporal"));
    assert!(text.text.contains("Scope"));
}

#[test]
fn test_map_to_tov_valid() {
    // Setup - Level 5 (System)
    let params = MapToTovParams { level: 5 };

    // Execute
    let result = vigilance::map_to_tov(params);

    // Assert
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap();
    assert!(text.text.contains("System"));
    assert!(text.text.contains("\"tov_level\":"));
}

#[test]
fn test_map_to_tov_invalid() {
    // Setup - Level 9 (Invalid)
    let params = MapToTovParams { level: 9 };

    // Execute
    let result = vigilance::map_to_tov(params);

    // Assert
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap();
    assert!(text.text.contains("error"));
    assert!(text.text.contains("Invalid level"));
}

// =============================================================================
// Live Proof — 4 cases through dual-mode scorer
// =============================================================================

#[test]
fn proof_dual_mode_live() {
    let cases: Vec<(&str, &str, f64, f64, f64, f64, u64, &str)> = vec![
        (
            "Metformin",
            "Lactic Acidosis",
            71.42,
            72.86,
            4.80,
            4.80,
            18372,
            "TRUE POSITIVE",
        ),
        (
            "TestDrug-A",
            "Hepatotoxicity",
            8.0,
            4.0,
            -2.0,
            0.5,
            100,
            "MASKING",
        ),
        ("TestDrug-B", "Rash", 2.5, 1.2, 0.3, 1.5, 15, "MARGINAL"),
        (
            "Placebo",
            "Headache",
            0.5,
            0.3,
            -1.5,
            0.5,
            50,
            "TRUE NEGATIVE",
        ),
    ];

    for (drug, event, prr, ror, ic, eb, n, label) in cases {
        let params = RiskScoreGeometricParams {
            drug: drug.to_string(),
            event: event.to_string(),
            prr,
            ror_lower: ror,
            ic025: ic,
            eb05: eb,
            n,
            mode: "dual".to_string(),
            weights: None,
        };
        let result = vigilance::risk_score_geometric(params).unwrap();
        let text = &result.content[0].as_text().unwrap().text;
        let json: serde_json::Value = serde_json::from_str(text).unwrap();

        eprintln!("\n>>> {drug} + {event} [{label}]");
        eprintln!(
            "  Additive:  {} [{}]",
            json["additive"]["score"], json["additive"]["level"]
        );
        eprintln!(
            "  Geometric: {} [{}]",
            json["geometric"]["composite_score"], json["geometric"]["level"]
        );
        eprintln!("  Signals:   {}", json["geometric"]["signals_detected"]);
        eprintln!(
            "  Divergence: {} | Masking: {}",
            json["divergence"], json["compensatory_masking"]
        );
        eprintln!("  {}", json["divergence_explanation"]);

        // Structural assertions
        assert!(json["additive"]["score"].is_object() || json["additive"]["score"].is_number());
        assert!(
            json["geometric"]["composite_score"].is_object()
                || json["geometric"]["composite_score"].is_number()
        );
    }
}

// =============================================================================
// Non-Compensatory Geometric Scoring (ASDF v2.0)
// =============================================================================

#[test]
fn test_geometric_dual_mode_true_positive() {
    // All 4 metrics above threshold — both modes should agree
    let params = RiskScoreGeometricParams {
        drug: "Metformin".to_string(),
        event: "Lactic Acidosis".to_string(),
        prr: 5.0,
        ror_lower: 3.0,
        ic025: 1.5,
        eb05: 4.0,
        n: 100,
        mode: "dual".to_string(),
        weights: None,
    };

    let result = vigilance::risk_score_geometric(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;

    // Must contain dual comparison structure
    assert!(
        text.contains("dual_comparison"),
        "Expected dual mode, got: {text}"
    );
    assert!(text.contains("Metformin"));
    assert!(text.contains("Lactic Acidosis"));
    assert!(text.contains("compensatory_masking"));
    assert!(text.contains("divergence_explanation"));

    // Parse and verify no masking for balanced signals
    let json: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(json["compensatory_masking"], false);
}

#[test]
fn test_geometric_detects_masking() {
    // PRR strong, Bayesian metrics absent — geometric should be much lower
    let params = RiskScoreGeometricParams {
        drug: "TestDrug".to_string(),
        event: "TestEvent".to_string(),
        prr: 8.0,
        ror_lower: 4.0,
        ic025: -2.0, // below threshold
        eb05: 0.5,   // below threshold
        n: 100,
        mode: "dual".to_string(),
        weights: None,
    };

    let result = vigilance::risk_score_geometric(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;

    let json: serde_json::Value = serde_json::from_str(text).unwrap();

    // Additive score should be higher than geometric (the masking)
    let additive_level = json["additive"]["level"].as_str().unwrap();
    let geometric_level = json["geometric"]["level"].as_str().unwrap();

    // Additive gives credit for 2/4 strong metrics; geometric penalizes the 2 absent
    assert!(
        json["divergence"].as_f64().unwrap() >= 1.0,
        "Expected divergence >= 1.0 for imbalanced evidence, got {}. Additive={additive_level}, Geometric={geometric_level}",
        json["divergence"]
    );
}

#[test]
fn test_geometric_mode_standalone() {
    // Pure geometric mode
    let params = RiskScoreGeometricParams {
        drug: "Semaglutide".to_string(),
        event: "Pancreatitis".to_string(),
        prr: 3.0,
        ror_lower: 2.0,
        ic025: 0.5,
        eb05: 3.0,
        n: 50,
        mode: "geometric".to_string(),
        weights: None,
    };

    let result = vigilance::risk_score_geometric(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;

    assert!(text.contains("geometric_noncompensatory"));
    assert!(text.contains("composite_score"));
    assert!(text.contains("dimensions"));
    assert!(text.contains("signals_detected"));

    let json: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(json["signals_detected"], "4/4");
    // Bayesian-heavy weights should be in output
    assert!(text.contains("0.35"));
}

#[test]
fn test_geometric_custom_weights() {
    // Override with equal weights
    let params = RiskScoreGeometricParams {
        drug: "TestDrug".to_string(),
        event: "TestEvent".to_string(),
        prr: 5.0,
        ror_lower: 3.0,
        ic025: 1.0,
        eb05: 3.0,
        n: 50,
        mode: "geometric".to_string(),
        weights: Some(vec![0.25, 0.25, 0.25, 0.25]),
    };

    let result = vigilance::risk_score_geometric(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;

    // Verify custom weights appear in output
    let json: serde_json::Value = serde_json::from_str(text).unwrap();
    let weights = json["weights_used"].as_array().unwrap();
    assert_eq!(weights[0].as_f64().unwrap(), 0.25);
    assert_eq!(weights[3].as_f64().unwrap(), 0.25);
}

#[test]
fn test_geometric_additive_mode_fallback() {
    // Pure additive mode through the new tool
    let params = RiskScoreGeometricParams {
        drug: "TestDrug".to_string(),
        event: "TestEvent".to_string(),
        prr: 5.0,
        ror_lower: 3.0,
        ic025: 1.0,
        eb05: 3.0,
        n: 50,
        mode: "additive".to_string(),
        weights: None,
    };

    let result = vigilance::risk_score_geometric(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;

    assert!(text.contains("\"mode\":\"additive\""));
    assert!(text.contains("\"score\":"));
    assert!(text.contains("\"level\":"));
}
