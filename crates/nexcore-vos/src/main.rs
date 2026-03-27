#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

//! VOS Compute — Native Rust pharmacovigilance computation engine.
//!
//! Wraps nexcore-pv-core algorithms as JSON-in/JSON-out CLI commands.
//! Part of the Vigilance Operating System (VOS) toolchain.
//!
//! Usage: vos-compute <command> '<json_args>'
//!
//! Commands:
//!   signal     - Full signal detection (PRR/ROR/IC/BCPNN/EBGM)
//!   prr        - PRR only
//!   ror        - ROR only
//!   ic         - IC only
//!   ebgm       - EBGM only
//!   naranjo    - Naranjo causality assessment
//!   naranjo-quick - Quick Naranjo (5 questions)
//!   qbri       - Quantitative Benefit-Risk Index
//!   fisher     - Fisher exact test
//!   chi-square - Chi-square test

use nexcore_pv_core::benefit_risk;
use nexcore_pv_core::causality;
use nexcore_pv_core::signals;
use serde_json::{Value, json};
use std::process;

fn parse_table(v: &Value) -> Option<signals::core::types::ContingencyTable> {
    let a = v.get("a")?.as_u64()?;
    let b = v.get("b")?.as_u64()?;
    let c = v.get("c")?.as_u64()?;
    let d = v.get("d")?.as_u64()?;
    Some(signals::core::types::ContingencyTable::new(a, b, c, d))
}

fn run_signal(args: &Value) -> Value {
    let Some(table) = parse_table(args) else {
        return json!({"error": "requires a, b, c, d (u64)"});
    };

    let criteria = signals::core::types::SignalCriteria::evans();

    match signals::evaluate_signal_all(&table, &criteria) {
        Ok(results) => {
            let mut output = json!({
                "contingency_table": {
                    "a": table.a, "b": table.b, "c": table.c, "d": table.d,
                    "N": table.total()
                },
                "criteria": "Evans (PRR>=2, chi2>=3.841, N>=3)"
            });

            let mut any_signal = false;
            for r in &results {
                let method_name = format!("{:?}", r.method).to_lowercase();
                output[&method_name] = json!({
                    "value": r.point_estimate,
                    "ci_lower": r.lower_ci,
                    "ci_upper": r.upper_ci,
                    "chi_square": r.chi_square,
                    "signal": r.is_signal,
                });
                if r.is_signal {
                    any_signal = true;
                }
            }
            output["any_signal"] = json!(any_signal);
            output["n_methods"] = json!(results.len());
            output
        }
        Err(e) => json!({"error": format!("{e}")}),
    }
}

fn run_single_method(args: &Value, method: &str) -> Value {
    let Some(table) = parse_table(args) else {
        return json!({"error": "requires a, b, c, d (u64)"});
    };

    let criteria = signals::core::types::SignalCriteria::evans();

    let result = match method {
        "prr" => {
            signals::disproportionality::prr::calculate_prr(&table, &criteria).map(|r| r.into())
        }
        "ror" => {
            signals::disproportionality::ror::calculate_ror(&table, &criteria).map(|r| r.into())
        }
        "ic" => signals::bayesian::ic::calculate_ic(&table, &criteria).map(|r| r.into()),
        "ebgm" => signals::bayesian::ebgm::calculate_ebgm(&table, &criteria).map(|r| r.into()),
        _ => return json!({"error": format!("unknown method: {method}")}),
    };

    match result {
        Ok(r) => {
            let r: signals::DisproportionalityResult = r;
            json!({
                "method": format!("{:?}", r.method),
                "value": r.point_estimate,
                "ci_lower": r.lower_ci,
                "ci_upper": r.upper_ci,
                "chi_square": r.chi_square,
                "signal": r.is_signal,
                "contingency_table": {
                    "a": table.a, "b": table.b, "c": table.c, "d": table.d,
                    "N": table.total()
                }
            })
        }
        Err(e) => json!({"error": format!("{e}")}),
    }
}

fn run_naranjo(args: &Value) -> Value {
    let input = causality::NaranjoInput {
        previous_reports: args
            .get("previous_reports")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        after_drug: args
            .get("after_drug")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        improved_on_dechallenge: args
            .get("improved_on_dechallenge")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        recurred_on_rechallenge: args
            .get("recurred_on_rechallenge")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        alternative_causes: args
            .get("alternative_causes")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        reaction_on_placebo: args
            .get("reaction_on_placebo")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        detected_in_fluids: args
            .get("detected_in_fluids")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        dose_response: args
            .get("dose_response")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        previous_similar_reaction: args
            .get("previous_similar_reaction")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
        objective_evidence: args
            .get("objective_evidence")
            .and_then(|v| v.as_i64())
            .map_or(0, |v| v as i8),
    };

    let result = causality::calculate_naranjo(&input);
    json!({
        "score": result.score,
        "category": format!("{:?}", result.category),
        "question_scores": result.question_scores,
    })
}

fn run_naranjo_quick(args: &Value) -> Value {
    let temporal = args
        .get("temporal")
        .and_then(|v| v.as_i64())
        .map_or(0, |v| v as i32);
    let dechallenge = args
        .get("dechallenge")
        .and_then(|v| v.as_i64())
        .map_or(0, |v| v as i32);
    let rechallenge = args
        .get("rechallenge")
        .and_then(|v| v.as_i64())
        .map_or(0, |v| v as i32);
    let alternatives = args
        .get("alternatives")
        .and_then(|v| v.as_i64())
        .map_or(0, |v| v as i32);
    let previous = args
        .get("previous")
        .and_then(|v| v.as_i64())
        .map_or(0, |v| v as i32);

    let result = causality::calculate_naranjo_quick(
        temporal,
        dechallenge,
        rechallenge,
        alternatives,
        previous,
    );
    json!({
        "score": result.score,
        "category": format!("{:?}", result.category),
    })
}

fn run_qbri(args: &Value) -> Value {
    let effect_size = args
        .get("effect_size")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let p_value = args.get("p_value").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let unmet_need = args
        .get("unmet_need")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5);
    let signal_strength = args
        .get("signal_strength")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let probability = args
        .get("probability")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let severity = args.get("severity").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
    let reversible = args
        .get("reversible")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let benefit = benefit_risk::BenefitAssessment::from_trial(effect_size, p_value, unmet_need);
    let risk = benefit_risk::RiskAssessment::from_signal(
        signal_strength,
        probability,
        severity,
        reversible,
    );

    let thresholds = benefit_risk::QbriThresholds::new(2.0, 0.5, 0.0);
    let result = benefit_risk::compute_qbri(&benefit, &risk, &thresholds);

    json!({
        "qbri": result.index,
        "benefit_score": result.benefit_score,
        "risk_score": result.risk_score,
        "decision": format!("{:?}", result.decision),
        "confidence": result.confidence,
        "thresholds": {
            "approve": 2.0,
            "monitor": 0.5,
            "uncertain": 0.0,
        }
    })
}

fn run_fisher(args: &Value) -> Value {
    let Some(table) = parse_table(args) else {
        return json!({"error": "requires a, b, c, d (u64)"});
    };

    let result = signals::fisher::fisher_exact_test(&table);
    json!({
        "p_value_one_tailed": result.p_value_one_tailed,
        "p_value_two_tailed": result.p_value_two_tailed,
        "signal": result.is_signal,
        "contingency_table": {
            "a": table.a, "b": table.b, "c": table.c, "d": table.d,
        }
    })
}

fn run_chi_square(args: &Value) -> Value {
    let Some(table) = parse_table(args) else {
        return json!({"error": "requires a, b, c, d (u64)"});
    };

    let chi2 = signals::chi_square::calculate_chi_square(&table);
    let yates = signals::chi_square::calculate_yates_corrected_chi_square(&table);
    let p_value = signals::chi_square::calculate_chi_square_p_value(chi2);

    json!({
        "chi_square": chi2,
        "yates_corrected": yates,
        "p_value": p_value,
        "significant": chi2 >= 3.841,
        "contingency_table": {
            "a": table.a, "b": table.b, "c": table.c, "d": table.d,
        }
    })
}

fn print_usage() {
    eprintln!("vos-compute — NexCore PV computation engine");
    eprintln!();
    eprintln!("Usage: vos-compute <command> [json_args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  signal         Full signal detection (PRR/ROR/IC/BCPNN/EBGM)");
    eprintln!("  prr            Proportional Reporting Ratio");
    eprintln!("  ror            Reporting Odds Ratio");
    eprintln!("  ic             Information Component");
    eprintln!("  ebgm           Empirical Bayes Geometric Mean");
    eprintln!("  naranjo        Full Naranjo causality (10 questions)");
    eprintln!("  naranjo-quick  Quick Naranjo (5 questions)");
    eprintln!("  qbri           Quantitative Benefit-Risk Index");
    eprintln!("  fisher         Fisher exact test");
    eprintln!("  chi-square     Chi-square test");
    eprintln!();
    eprintln!("All commands accept JSON input and produce JSON output.");
    eprintln!("Signal commands require: {{\"a\":N,\"b\":N,\"c\":N,\"d\":N}}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];
    let input: Value = if args.len() > 2 {
        match serde_json::from_str(&args[2]) {
            Ok(v) => v,
            Err(e) => {
                let output = json!({"error": format!("invalid JSON: {e}")});
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output).unwrap_or_default()
                );
                process::exit(1);
            }
        }
    } else {
        json!({})
    };

    let result = match command.as_str() {
        "signal" => run_signal(&input),
        "prr" => run_single_method(&input, "prr"),
        "ror" => run_single_method(&input, "ror"),
        "ic" => run_single_method(&input, "ic"),
        "ebgm" => run_single_method(&input, "ebgm"),
        "naranjo" => run_naranjo(&input),
        "naranjo-quick" => run_naranjo_quick(&input),
        "qbri" => run_qbri(&input),
        "fisher" => run_fisher(&input),
        "chi-square" => run_chi_square(&input),
        "help" | "--help" | "-h" => {
            print_usage();
            process::exit(0);
        }
        _ => json!({"error": format!("unknown command: {command}. Run 'vos-compute help'")}),
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap_or_default()
    );
}
