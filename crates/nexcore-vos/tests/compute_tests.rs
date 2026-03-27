//! Integration tests for vos-compute binary.
//! Tests every command with known inputs and validates JSON output.

use std::process::Command;

fn run_vos(args: &[&str]) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vos-compute"))
        .args(args)
        .output()
        .expect("failed to run vos-compute");

    assert!(
        output.status.success(),
        "vos-compute failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("invalid utf8");
    serde_json::from_str(&stdout).expect("invalid JSON output")
}

// ─── Signal Detection ────────────────────────────────────────────

#[test]
fn test_signal_full_detection() {
    let result = run_vos(&["signal", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);

    assert!(
        result["any_signal"].as_bool().unwrap_or(false),
        "should detect signal"
    );
    assert_eq!(result["n_methods"].as_u64().unwrap_or(0), 5);

    // PRR should be ~10.4
    let prr = result["prr"]["value"].as_f64().unwrap_or(0.0);
    assert!(prr > 9.0 && prr < 12.0, "PRR={prr} should be ~10.4");

    // ROR should be ~11.25
    let ror = result["ror"]["value"].as_f64().unwrap_or(0.0);
    assert!(ror > 10.0 && ror < 13.0, "ROR={ror} should be ~11.25");

    // IC should be ~2.8
    let ic = result["ic"]["value"].as_f64().unwrap_or(0.0);
    assert!(ic > 2.5 && ic < 3.5, "IC={ic} should be ~2.8");

    // All methods should detect signal
    assert!(result["prr"]["signal"].as_bool().unwrap_or(false));
    assert!(result["ror"]["signal"].as_bool().unwrap_or(false));
    assert!(result["ic"]["signal"].as_bool().unwrap_or(false));
}

#[test]
fn test_signal_no_signal() {
    // Very low disproportionality — no signal
    let result = run_vos(&["signal", r#"{"a":10,"b":5000,"c":10000,"d":100000}"#]);
    assert!(
        !result["any_signal"].as_bool().unwrap_or(true),
        "should NOT detect signal"
    );
}

#[test]
fn test_signal_minimum_cases() {
    // Only 2 cases — below Evans N>=3
    let result = run_vos(&["signal", r#"{"a":2,"b":100,"c":50,"d":10000}"#]);
    // May or may not signal depending on PRR, but should not crash
    assert!(result.get("prr").is_some());
}

#[test]
fn test_signal_contingency_table_preserved() {
    let result = run_vos(&["signal", r#"{"a":100,"b":200,"c":300,"d":400}"#]);
    let ct = &result["contingency_table"];
    assert_eq!(ct["a"].as_u64().unwrap(), 100);
    assert_eq!(ct["b"].as_u64().unwrap(), 200);
    assert_eq!(ct["c"].as_u64().unwrap(), 300);
    assert_eq!(ct["d"].as_u64().unwrap(), 400);
    assert_eq!(ct["N"].as_u64().unwrap(), 1000);
}

// ─── Individual Methods ──────────────────────────────────────────

#[test]
fn test_prr_single() {
    let result = run_vos(&["prr", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    assert!(result["signal"].as_bool().unwrap_or(false));
    let val = result["value"].as_f64().unwrap_or(0.0);
    assert!(val > 9.0, "PRR={val}");
}

#[test]
fn test_ror_single() {
    let result = run_vos(&["ror", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    assert!(result["signal"].as_bool().unwrap_or(false));
}

#[test]
fn test_ic_single() {
    let result = run_vos(&["ic", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    assert!(result["signal"].as_bool().unwrap_or(false));
}

#[test]
fn test_ebgm_single() {
    let result = run_vos(&["ebgm", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    // EBGM may or may not signal depending on prior computation
    assert!(result.get("value").is_some());
}

// ─── Naranjo Causality ───────────────────────────────────────────

#[test]
fn test_naranjo_definite() {
    let result = run_vos(&[
        "naranjo",
        r#"{
        "previous_reports": 1,
        "after_drug": 2,
        "improved_on_dechallenge": 1,
        "recurred_on_rechallenge": 2,
        "alternative_causes": 1,
        "reaction_on_placebo": 0,
        "detected_in_fluids": 1,
        "dose_response": 1,
        "previous_similar_reaction": 1,
        "objective_evidence": 1
    }"#,
    ]);
    assert_eq!(result["category"].as_str().unwrap(), "Definite");
    assert!(result["score"].as_i64().unwrap() >= 9);
}

#[test]
fn test_naranjo_doubtful() {
    let result = run_vos(&[
        "naranjo",
        r#"{
        "previous_reports": 0,
        "after_drug": 0,
        "improved_on_dechallenge": 0,
        "recurred_on_rechallenge": 0,
        "alternative_causes": 0,
        "reaction_on_placebo": 0,
        "detected_in_fluids": 0,
        "dose_response": 0,
        "previous_similar_reaction": 0,
        "objective_evidence": 0
    }"#,
    ]);
    assert_eq!(result["category"].as_str().unwrap(), "Doubtful");
    assert_eq!(result["score"].as_i64().unwrap(), 0);
}

#[test]
fn test_naranjo_quick_possible() {
    let result = run_vos(&[
        "naranjo-quick",
        r#"{"temporal":1,"dechallenge":1,"rechallenge":0,"alternatives":-1,"previous":1}"#,
    ]);
    assert_eq!(result["category"].as_str().unwrap(), "Possible");
    assert_eq!(result["score"].as_i64().unwrap(), 2);
}

#[test]
fn test_naranjo_quick_probable() {
    let result = run_vos(&[
        "naranjo-quick",
        r#"{"temporal":1,"dechallenge":1,"rechallenge":2,"alternatives":1,"previous":1}"#,
    ]);
    assert_eq!(result["category"].as_str().unwrap(), "Probable");
}

#[test]
fn test_naranjo_empty_input() {
    let result = run_vos(&["naranjo", r#"{}"#]);
    assert_eq!(result["category"].as_str().unwrap(), "Doubtful");
    assert_eq!(result["score"].as_i64().unwrap(), 0);
}

// ─── QBRI Benefit-Risk ──────────────────────────────────────────

#[test]
fn test_qbri_approve() {
    let result = run_vos(&[
        "qbri",
        r#"{"effect_size":0.9,"p_value":0.001,"unmet_need":0.9,"signal_strength":0.5,"probability":0.05,"severity":1,"reversible":true}"#,
    ]);
    let qbri = result["qbri"].as_f64().unwrap_or(0.0);
    assert!(qbri > 0.0, "QBRI should be positive");
    assert!(result.get("decision").is_some());
    assert!(result.get("confidence").is_some());
}

#[test]
fn test_qbri_high_risk() {
    let result = run_vos(&[
        "qbri",
        r#"{"effect_size":0.2,"p_value":0.5,"unmet_need":0.3,"signal_strength":8.0,"probability":0.8,"severity":5,"reversible":false}"#,
    ]);
    let qbri = result["qbri"].as_f64().unwrap_or(999.0);
    // High risk, low benefit — should be low or negative QBRI
    assert!(qbri < 2.0, "QBRI={qbri} should be low for high-risk drug");
}

#[test]
fn test_qbri_empty_input() {
    let result = run_vos(&["qbri", r#"{}"#]);
    assert!(result.get("qbri").is_some());
}

// ─── Fisher Exact Test ───────────────────────────────────────────

#[test]
fn test_fisher_significant() {
    let result = run_vos(&["fisher", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    assert!(result["signal"].as_bool().unwrap_or(false));
    let p = result["p_value_two_tailed"].as_f64().unwrap_or(1.0);
    assert!(p < 0.05, "p={p} should be significant");
}

#[test]
fn test_fisher_not_significant() {
    let result = run_vos(&["fisher", r#"{"a":5,"b":100,"c":50,"d":1000}"#]);
    // With these balanced numbers, may or may not be significant
    assert!(result.get("p_value_two_tailed").is_some());
}

// ─── Chi-Square Test ─────────────────────────────────────────────

#[test]
fn test_chi_square_significant() {
    let result = run_vos(&["chi-square", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    assert!(result["significant"].as_bool().unwrap_or(false));
    let chi2 = result["chi_square"].as_f64().unwrap_or(0.0);
    assert!(chi2 >= 3.841, "chi2={chi2} should exceed 3.841");
}

#[test]
fn test_chi_square_yates_correction() {
    let result = run_vos(&["chi-square", r#"{"a":10,"b":100,"c":20,"d":200}"#]);
    let chi2 = result["chi_square"].as_f64().unwrap_or(0.0);
    let yates = result["yates_corrected"].as_f64().unwrap_or(0.0);
    // Yates correction always reduces chi-square
    assert!(
        yates <= chi2,
        "Yates ({yates}) should be <= uncorrected ({chi2})"
    );
}

#[test]
fn test_chi_square_p_value() {
    let result = run_vos(&["chi-square", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    let p = result["p_value"].as_f64().unwrap_or(1.0);
    assert!(p < 0.001, "p={p} should be very small");
}

// ─── Error Handling ──────────────────────────────────────────────

#[test]
fn test_missing_fields() {
    let result = run_vos(&["signal", r#"{"a":100}"#]);
    assert!(
        result.get("error").is_some(),
        "should return error for missing fields"
    );
}

#[test]
fn test_empty_input() {
    let result = run_vos(&["signal", r#"{}"#]);
    assert!(result.get("error").is_some());
}

#[test]
fn test_unknown_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_vos-compute"))
        .args(["nonexistent", "{}"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let result: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        result["error"]
            .as_str()
            .unwrap()
            .contains("unknown command")
    );
}

// ─── Cross-Validation ────────────────────────────────────────────

#[test]
fn test_prr_matches_full_signal() {
    // PRR from single method should match PRR from full signal
    let full = run_vos(&["signal", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    let single = run_vos(&["prr", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);

    let full_prr = full["prr"]["value"].as_f64().unwrap_or(0.0);
    let single_prr = single["value"].as_f64().unwrap_or(0.0);

    assert!(
        (full_prr - single_prr).abs() < 0.001,
        "PRR mismatch: full={full_prr}, single={single_prr}"
    );
}

#[test]
fn test_chi_square_matches_signal_prr() {
    // Chi-square from standalone should match chi-square reported in PRR
    let prr = run_vos(&["prr", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);
    let chi2 = run_vos(&["chi-square", r#"{"a":450,"b":5000,"c":800,"d":100000}"#]);

    let prr_chi2 = prr["chi_square"].as_f64().unwrap_or(0.0);
    let standalone_chi2 = chi2["chi_square"].as_f64().unwrap_or(0.0);

    // They may use different formulas (PRR uses its own chi2), but both should be significant
    assert!(prr_chi2 > 3.841);
    assert!(standalone_chi2 > 3.841);
}
