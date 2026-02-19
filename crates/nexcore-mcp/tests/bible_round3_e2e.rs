//! End-to-end tests for AI Engineering Bible Round 3 tools.
//!
//! Tests: security_posture (4), security_threat_readiness (3),
//!        security_compliance_gap (3), observability_record_latency (3),
//!        observability_query (3), observability_freshness (3)
//! Total: 19 tests

use nexcore_mcp::params::*;
use nexcore_mcp::tools::{observability, security_posture};

// ============================================================================
// Helper
// ============================================================================

fn extract_text(result: &rmcp::model::CallToolResult) -> String {
    result.content[0]
        .as_text()
        .map(|t| t.text.clone())
        .unwrap_or_default()
}

// ============================================================================
// Security Posture Assess Tests
// ============================================================================

#[test]
fn test_posture_assess_all_frameworks() {
    let params = SecurityPostureAssessParams {
        target: "test-system".to_string(),
        frameworks: None,
        existing_controls: None,
    };
    let result = security_posture::security_posture_assess(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    assert!(
        parsed["overall_posture"].is_object(),
        "Expected overall_posture: {}",
        text
    );
    assert!(
        parsed["frameworks"].is_array(),
        "Expected frameworks array: {}",
        text
    );

    let frameworks = parsed["frameworks"].as_array().expect("frameworks array");
    assert_eq!(
        frameworks.len(),
        6,
        "Expected 6 frameworks, got {}",
        frameworks.len()
    );

    let grade = parsed["overall_posture"]["grade"].as_str().expect("grade");
    assert_eq!(
        grade, "F",
        "Expected grade F with no controls, got {}",
        grade
    );
}

#[test]
fn test_posture_assess_single_framework() {
    let params = SecurityPostureAssessParams {
        target: "test-soc2-only".to_string(),
        frameworks: Some(vec!["soc2".to_string()]),
        existing_controls: None,
    };
    let result = security_posture::security_posture_assess(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let frameworks = parsed["frameworks"].as_array().expect("frameworks array");
    assert_eq!(
        frameworks.len(),
        1,
        "Expected 1 framework for SOC2-only, got {}",
        frameworks.len()
    );
    assert_eq!(
        frameworks[0]["code"].as_str().unwrap_or_default(),
        "soc2",
        "Expected soc2 framework code"
    );
}

#[test]
fn test_posture_assess_with_controls() {
    let params = SecurityPostureAssessParams {
        target: "test-with-controls".to_string(),
        frameworks: Some(vec!["soc2".to_string()]),
        existing_controls: Some(vec!["encryption".to_string(), "access control".to_string()]),
    };
    let result = security_posture::security_posture_assess(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let fw = &parsed["frameworks"][0];
    let controls_met = fw["controls_met"].as_u64().expect("controls_met");
    assert!(
        controls_met > 0,
        "Expected some controls met with encryption + access control, got {}",
        controls_met
    );

    let score = fw["score_pct"].as_f64().expect("score_pct");
    assert!(
        score > 0.0,
        "Expected score > 0 with controls, got {}",
        score
    );
}

#[test]
fn test_posture_assess_invalid_framework() {
    let params = SecurityPostureAssessParams {
        target: "test-invalid".to_string(),
        frameworks: Some(vec!["nonexistent".to_string()]),
        existing_controls: None,
    };
    let result = security_posture::security_posture_assess(params);
    assert!(result.is_err(), "Expected error for unknown framework");
}

// ============================================================================
// Security Threat Readiness Tests
// ============================================================================

#[test]
fn test_threat_readiness_no_defenses() {
    let params = SecurityThreatReadinessParams {
        target: "test-undefended".to_string(),
        threats: None,
        defenses: None,
    };
    let result = security_posture::security_threat_readiness(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let threats = parsed["threats"].as_array().expect("threats array");
    assert_eq!(threats.len(), 5, "Expected all 5 AI threats");

    // With no defenses, all threats should be "exposed"
    for threat in threats {
        let status = threat["status"].as_str().expect("status");
        assert_eq!(
            status, "exposed",
            "Expected all threats exposed with no defenses, got {} for {}",
            status, threat["threat"]
        );
    }
}

#[test]
fn test_threat_readiness_with_defenses() {
    let params = SecurityThreatReadinessParams {
        target: "test-partial-defense".to_string(),
        threats: None,
        defenses: Some(vec![
            "input_validation".to_string(),
            "rate_limiting".to_string(),
            "sandboxing".to_string(),
        ]),
    };
    let result = security_posture::security_threat_readiness(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let threats = parsed["threats"].as_array().expect("threats array");
    // At least some threats should have improved status (not all exposed)
    let non_exposed = threats
        .iter()
        .filter(|t| t["status"].as_str().unwrap_or("exposed") != "exposed")
        .count();
    assert!(
        non_exposed > 0,
        "Expected some threats improved with defenses deployed, all still exposed"
    );
}

#[test]
fn test_threat_readiness_specific_threat() {
    let params = SecurityThreatReadinessParams {
        target: "test-prompt-injection".to_string(),
        threats: Some(vec!["prompt injection".to_string()]),
        defenses: None,
    };
    let result = security_posture::security_threat_readiness(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let threats = parsed["threats"].as_array().expect("threats array");
    assert_eq!(
        threats.len(),
        1,
        "Expected 1 threat for prompt_injection filter, got {}",
        threats.len()
    );
    assert!(
        threats[0]["threat"]
            .as_str()
            .unwrap_or_default()
            .contains("Prompt Injection"),
        "Expected Prompt Injection threat"
    );
}

// ============================================================================
// Security Compliance Gap Tests
// ============================================================================

#[test]
fn test_compliance_gap_all_gaps() {
    let params = SecurityComplianceGapParams {
        framework: "gdpr".to_string(),
        implemented_controls: None,
    };
    let result = security_posture::security_compliance_gap(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let gaps = parsed["gaps"].as_u64().expect("gaps count");
    assert_eq!(
        gaps, 10,
        "Expected 10 gaps for GDPR with no controls, got {}",
        gaps
    );

    let implemented = parsed["implemented"].as_u64().expect("implemented count");
    assert_eq!(
        implemented, 0,
        "Expected 0 implemented with no controls, got {}",
        implemented
    );
}

#[test]
fn test_compliance_gap_some_implemented() {
    let params = SecurityComplianceGapParams {
        framework: "nist".to_string(),
        implemented_controls: Some(vec![
            "risk assessment".to_string(),
            "data security".to_string(),
        ]),
    };
    let result = security_posture::security_compliance_gap(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let gaps = parsed["gaps"].as_u64().expect("gaps count");
    let implemented = parsed["implemented"].as_u64().expect("implemented count");

    assert!(
        gaps < 10,
        "Expected fewer than 10 gaps with controls, got {}",
        gaps
    );
    assert!(
        implemented > 0,
        "Expected some implemented controls, got {}",
        implemented
    );
}

#[test]
fn test_compliance_gap_invalid_framework() {
    let params = SecurityComplianceGapParams {
        framework: "fake_framework".to_string(),
        implemented_controls: None,
    };
    let result = security_posture::security_compliance_gap(params);
    assert!(result.is_err(), "Expected error for unknown framework");
}

// ============================================================================
// Observability Record Latency Tests
// ============================================================================

#[test]
fn test_record_latency_basic() {
    let params = ObservabilityRecordLatencyParams {
        endpoint_id: "test-model-basic".to_string(),
        latency_ms: 150.0,
        success: None,
        tags: None,
    };
    let result = observability::observability_record_latency(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    assert_eq!(
        parsed["endpoint_id"].as_str().unwrap_or_default(),
        "test-model-basic",
        "Expected endpoint_id"
    );
    assert!(
        parsed["recorded"].is_object(),
        "Expected recorded object: {}",
        text
    );
    assert_eq!(
        parsed["recorded"]["latency_ms"].as_f64().unwrap_or(0.0),
        150.0,
        "Expected recorded latency 150.0"
    );
}

#[test]
fn test_record_latency_with_tags() {
    let params = ObservabilityRecordLatencyParams {
        endpoint_id: "test-model-tagged".to_string(),
        latency_ms: 200.0,
        success: Some(true),
        tags: Some(vec!["model:gpt4".to_string(), "region:us".to_string()]),
    };
    let result = observability::observability_record_latency(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    assert!(parsed["endpoint_id"].is_string(), "Expected endpoint_id");
    assert!(
        parsed["recorded"]["success"].as_bool().unwrap_or(false),
        "Expected success=true"
    );
}

#[test]
fn test_record_latency_failure() {
    // First record a success
    let _ = observability::observability_record_latency(ObservabilityRecordLatencyParams {
        endpoint_id: "test-model-failure".to_string(),
        latency_ms: 100.0,
        success: Some(true),
        tags: None,
    });

    // Now record a failure
    let params = ObservabilityRecordLatencyParams {
        endpoint_id: "test-model-failure".to_string(),
        latency_ms: 5000.0,
        success: Some(false),
        tags: None,
    };
    let result = observability::observability_record_latency(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let error_count = parsed["endpoint_stats"]["error_count"]
        .as_u64()
        .expect("error_count");
    assert!(
        error_count > 0,
        "Expected error_count > 0 after recording failure, got {}",
        error_count
    );
}

// ============================================================================
// Observability Query Tests
// ============================================================================

#[test]
fn test_query_nonexistent_endpoint() {
    let params = ObservabilityQueryParams {
        endpoint_id: Some("nonexistent-endpoint-xyz-12345".to_string()),
        window_secs: None,
    };
    let result = observability::observability_query(params);
    assert!(result.is_err(), "Expected error for nonexistent endpoint");
}

#[test]
fn test_query_all_endpoints() {
    // Ensure at least one endpoint exists
    let _ = observability::observability_record_latency(ObservabilityRecordLatencyParams {
        endpoint_id: "test-query-all-seed".to_string(),
        latency_ms: 50.0,
        success: Some(true),
        tags: None,
    });

    let params = ObservabilityQueryParams {
        endpoint_id: None,
        window_secs: None,
    };
    let result = observability::observability_query(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    assert!(
        parsed["endpoints"].is_array(),
        "Expected endpoints array: {}",
        text
    );
    assert!(
        parsed["endpoint_count"].is_number(),
        "Expected endpoint_count: {}",
        text
    );
}

#[test]
fn test_query_after_recording() {
    let endpoint = "e2e-test-obs-query";

    // Record 3 latency measurements
    for latency in [100.0, 200.0, 300.0] {
        let _ = observability::observability_record_latency(ObservabilityRecordLatencyParams {
            endpoint_id: endpoint.to_string(),
            latency_ms: latency,
            success: Some(true),
            tags: None,
        });
    }

    let params = ObservabilityQueryParams {
        endpoint_id: Some(endpoint.to_string()),
        window_secs: Some(300),
    };
    let result = observability::observability_query(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let endpoints = parsed["endpoints"].as_array().expect("endpoints array");
    assert!(!endpoints.is_empty(), "Expected at least 1 endpoint result");

    let ep = &endpoints[0];
    assert!(
        ep["latency"]["p50_ms"].is_number(),
        "Expected p50_ms: {}",
        text
    );
    assert!(
        ep["latency"]["p95_ms"].is_number(),
        "Expected p95_ms: {}",
        text
    );
    assert!(
        ep["latency"]["p99_ms"].is_number(),
        "Expected p99_ms: {}",
        text
    );
}

// ============================================================================
// Observability Freshness Tests
// ============================================================================

#[test]
fn test_freshness_fresh_source() {
    let params = ObservabilityFreshnessParams {
        source_id: "test-fresh-source".to_string(),
        last_updated: None, // defaults to now
        max_age_secs: Some(3600),
    };
    let result = observability::observability_freshness(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let status = parsed["status"].as_str().expect("status");
    assert_eq!(
        status, "FRESH",
        "Expected FRESH status for just-updated source, got {}",
        status
    );

    let stale = parsed["stale"].as_bool().expect("stale flag");
    assert!(!stale, "Expected stale=false for fresh source");
}

#[test]
fn test_freshness_stale_source() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let params = ObservabilityFreshnessParams {
        source_id: "test-stale-source".to_string(),
        last_updated: Some(now - 1_000_000.0), // far in the past
        max_age_secs: Some(60),
    };
    let result = observability::observability_freshness(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    let status = parsed["status"].as_str().expect("status");
    assert_eq!(
        status, "STALE",
        "Expected STALE status for old source, got {}",
        status
    );

    let stale = parsed["stale"].as_bool().expect("stale flag");
    assert!(stale, "Expected stale=true for stale source");
}

#[test]
fn test_freshness_approaching() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    let max_age: u64 = 1000;
    // Set last_updated so that ~80% of max_age has elapsed => freshness_pct ~20% (< 25%)
    let last_updated = now - (max_age as f64 * 0.8);

    let params = ObservabilityFreshnessParams {
        source_id: "test-approaching-source".to_string(),
        last_updated: Some(last_updated),
        max_age_secs: Some(max_age),
    };
    let result = observability::observability_freshness(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");

    // Should still be FRESH (not yet exceeded max_age)
    let status = parsed["status"].as_str().expect("status");
    assert_eq!(
        status, "FRESH",
        "Expected FRESH (not yet stale), got {}",
        status
    );

    // But freshness_pct should be < 25, triggering "approaching" recommendation
    let freshness_pct = parsed["freshness_pct"].as_f64().expect("freshness_pct");
    assert!(
        freshness_pct < 25.0,
        "Expected freshness_pct < 25 for approaching staleness, got {}",
        freshness_pct
    );

    let recommendation = parsed["recommendation"].as_str().expect("recommendation");
    assert!(
        recommendation.contains("approaching"),
        "Expected recommendation about approaching staleness, got: {}",
        recommendation
    );
}
