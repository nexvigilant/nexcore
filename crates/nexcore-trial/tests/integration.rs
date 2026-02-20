//! Integration test: full TRIAL pipeline (T-R-I-A-L)
//!
//! Registers a protocol, randomizes, runs interim, evaluates endpoint,
//! adjusts for multiplicity, and generates a CONSORT-style report.
//! Uses a software A/B testing domain to demonstrate cross-domain universality.

use nexcore_trial::{
    block_randomize, check_safety_boundary, determine_verdict, evaluate_interim,
    evaluate_two_proportions, generate_report, holm_adjust, register_protocol, safety_event_rate,
    types::{
        Adaptation, Arm, BlindingLevel, Endpoint, EndpointDirection, InterimData, InterimDecision,
        ProtocolRequest, SafetyRule,
    },
};

fn control_arm() -> Arm {
    Arm {
        name: "current_checkout".into(),
        description: "Existing checkout flow".into(),
        is_control: true,
    }
}

fn treatment_arm() -> Arm {
    Arm {
        name: "new_checkout".into(),
        description: "Redesigned checkout with single-page flow".into(),
        is_control: false,
    }
}

fn primary_endpoint() -> Endpoint {
    Endpoint {
        name: "conversion_rate".into(),
        metric: "proportion of users completing purchase".into(),
        direction: EndpointDirection::Higher,
        threshold: 0.05,
    }
}

fn secondary_endpoint() -> Endpoint {
    Endpoint {
        name: "cart_abandonment".into(),
        metric: "proportion of users abandoning cart".into(),
        direction: EndpointDirection::Lower,
        threshold: 0.03,
    }
}

#[test]
fn test_full_trial_pipeline_software_domain() {
    // ================================================================
    // T — TARGET: Register protocol
    // ================================================================
    let protocol = register_protocol(ProtocolRequest {
        hypothesis: "New single-page checkout flow increases conversion rate by >= 5pp".into(),
        population: "All users on pricing page with items in cart".into(),
        primary_endpoint: primary_endpoint(),
        secondary_endpoints: vec![secondary_endpoint()],
        arms: vec![control_arm(), treatment_arm()],
        sample_size: 400,
        power: 0.80,
        alpha: 0.05,
        duration_days: 30,
        safety_boundary: SafetyRule {
            metric: "error_rate".into(),
            threshold: 0.02,
            description: "Stop if checkout errors exceed 2%".into(),
        },
        adaptation_rules: vec![Adaptation {
            adaptation_type: "sample_reestimate".into(),
            conditions: "At 50% enrollment if effect size smaller than expected".into(),
            allowed_changes: "Increase sample size up to 600".into(),
        }],
        blinding: BlindingLevel::Open,
    });
    assert!(
        protocol.is_ok(),
        "Protocol registration failed: {:?}",
        protocol.err()
    );
    let protocol = protocol.unwrap();
    assert!(!protocol.id.is_empty(), "Protocol ID should be non-empty");
    assert!(protocol.power >= 0.80, "Power below minimum");
    assert_eq!(protocol.arms.len(), 2);

    // ================================================================
    // R — REGIMENT: Randomize subjects
    // ================================================================
    let assignments = block_randomize(400, 2, 4, Some(42));
    assert!(
        assignments.is_ok(),
        "Randomization failed: {:?}",
        assignments.err()
    );
    let assignments = assignments.unwrap();
    assert_eq!(assignments.len(), 400, "All subjects should be assigned");

    // Verify balance
    let arm0_count = assignments.iter().filter(|a| a.arm_index == 0).count();
    let arm1_count = assignments.iter().filter(|a| a.arm_index == 1).count();
    assert_eq!(
        arm0_count, 200,
        "Block randomization should be perfectly balanced"
    );
    assert_eq!(arm1_count, 200);

    // ================================================================
    // I — INTERIM: Analyze at 50% enrollment
    // ================================================================
    let interim_data = InterimData {
        information_fraction: 0.50,
        treatment_successes: 60,
        treatment_n: 100,
        control_successes: 45,
        control_n: 100,
        safety_events: 0,
    };

    // Safety check first
    let safety_rate = safety_event_rate(0, 200);
    let safety = check_safety_boundary(
        &SafetyRule {
            metric: "error_rate".into(),
            threshold: 0.02,
            description: "Checkout error threshold".into(),
        },
        safety_rate,
    );
    assert!(safety.is_safe, "No safety events → should be safe");

    // Interim decision
    let interim_result = evaluate_interim(&interim_data, &protocol);
    assert!(
        interim_result.is_ok(),
        "Interim analysis failed: {:?}",
        interim_result.err()
    );
    let interim = interim_result.unwrap();
    assert_eq!(
        interim.decision,
        InterimDecision::Continue,
        "Expected Continue at 50% interim, got {:?}",
        interim.decision
    );

    // ================================================================
    // A — ASSAY: Final endpoint evaluation at 100%
    // ================================================================
    // Primary: 120/200 treatment vs 95/200 control (60% vs 47.5%)
    let primary_result = evaluate_two_proportions(120, 200, 95, 200, 0.05);
    assert!(primary_result.is_ok());
    let primary = primary_result.unwrap();
    assert!(
        primary.significant,
        "Primary endpoint should be significant"
    );
    assert!(primary.effect_size > 0.10, "Effect size should be > 10pp");
    assert!(primary.ci_lower > 0.0, "CI should exclude 0");

    // Secondary: 30/200 treatment vs 50/200 control (15% vs 25% abandonment)
    let secondary_result = evaluate_two_proportions(50, 200, 30, 200, 0.05);
    assert!(secondary_result.is_ok());
    let secondary = secondary_result.unwrap();

    // Multiplicity: Holm adjustment for 2 endpoints
    let p_values = vec![primary.p_value, secondary.p_value];
    let adjusted = holm_adjust(&p_values, 0.05);
    assert!(
        adjusted[0].significant,
        "Primary should survive Holm adjustment"
    );

    // ================================================================
    // L — LIFECYCLE: Generate report
    // ================================================================
    let all_results = vec![primary.clone(), secondary];
    let report = generate_report(&protocol, &all_results);

    // Verify report structure
    assert!(
        report.contains("Protocol Summary"),
        "Report missing Protocol Summary"
    );
    assert!(
        report.contains("CONSORT Flow"),
        "Report missing CONSORT Flow"
    );
    assert!(
        report.contains("Primary Endpoint"),
        "Report missing Primary Endpoint"
    );
    assert!(
        report.contains("Safety Summary"),
        "Report missing Safety Summary"
    );

    // Verify verdict
    let verdict = determine_verdict(&[primary]);
    assert!(
        format!("{verdict:?}").contains("Positive"),
        "Verdict should be Positive for significant primary endpoint"
    );
}

#[test]
fn test_negative_trial_pipeline() {
    // Register protocol with high bar
    let protocol = register_protocol(ProtocolRequest {
        hypothesis: "Dark mode increases session duration by 20%".into(),
        population: "Desktop users on main dashboard".into(),
        primary_endpoint: Endpoint {
            name: "session_duration".into(),
            metric: "average session minutes".into(),
            direction: EndpointDirection::Higher,
            threshold: 5.0,
        },
        secondary_endpoints: vec![],
        arms: vec![control_arm(), treatment_arm()],
        sample_size: 200,
        power: 0.80,
        alpha: 0.05,
        duration_days: 14,
        safety_boundary: SafetyRule {
            metric: "crash_rate".into(),
            threshold: 0.01,
            description: "Stop if app crashes exceed 1%".into(),
        },
        adaptation_rules: vec![],
        blinding: BlindingLevel::Single,
    });
    assert!(protocol.is_ok());
    let protocol = protocol.unwrap();

    // Final analysis: 52/100 vs 48/100 — not significant
    let result = evaluate_two_proportions(52, 100, 48, 100, 0.05);
    assert!(result.is_ok());
    let r = result.unwrap();
    assert!(
        !r.significant,
        "Borderline result should NOT be significant"
    );

    // Verdict: Negative
    let verdict = determine_verdict(&[r.clone()]);
    assert!(
        format!("{verdict:?}").contains("Negative"),
        "Non-significant primary → Negative verdict"
    );

    // Report still generates
    let report = generate_report(&protocol, &[r]);
    assert!(!report.is_empty());
}

#[test]
fn test_safety_stop_pipeline() {
    let protocol = register_protocol(ProtocolRequest {
        hypothesis: "New payment flow reduces friction".into(),
        population: "Mobile checkout users".into(),
        primary_endpoint: primary_endpoint(),
        secondary_endpoints: vec![],
        arms: vec![control_arm(), treatment_arm()],
        sample_size: 400,
        power: 0.80,
        alpha: 0.05,
        duration_days: 30,
        safety_boundary: SafetyRule {
            metric: "payment_error_rate".into(),
            threshold: 0.02,
            description: "Stop if payment errors exceed 2%".into(),
        },
        adaptation_rules: vec![],
        blinding: BlindingLevel::Open,
    });
    assert!(protocol.is_ok());
    let protocol = protocol.unwrap();

    // Interim with safety events: 5/100 = 5% >> 2% threshold
    let data = InterimData {
        information_fraction: 0.50,
        treatment_successes: 30,
        treatment_n: 50,
        control_successes: 28,
        control_n: 50,
        safety_events: 5,
    };

    let result = evaluate_interim(&data, &protocol);
    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(
        r.decision,
        InterimDecision::StopSafety,
        "Should stop for safety: 5% > 2% threshold"
    );
}
