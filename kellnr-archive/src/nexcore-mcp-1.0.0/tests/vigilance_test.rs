//! Integration tests for Vigilance tools
//!
//! Covers safety margin, risk scoring, ToV mapping, and harm types.

use nexcore_mcp::params::{MapToTovParams, RiskScoreParams, SafetyMarginParams};
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
