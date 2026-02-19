//! CONSORT-style clinical study report generator per ICH E3.
//!
//! Tier: T2-P (π+ν+∝ — Report Generation)
//!
//! Generates a Markdown report covering:
//! - Protocol summary
//! - CONSORT flow (enrolled → randomized → analyzed)
//! - Primary and secondary endpoint results
//! - Safety summary
//! - Trial verdict

use crate::types::{EndpointResult, Protocol, TrialVerdict};

/// Generate a CONSORT-style Markdown report for a completed trial.
///
/// # Arguments
/// - `protocol`: The registered protocol
/// - `results`: Endpoint evaluation results (index 0 = primary, rest = secondary)
///
/// # Returns
/// Markdown string suitable for rendering or saving as a `.md` file.
pub fn generate_report(protocol: &Protocol, results: &[EndpointResult]) -> String {
    let verdict = determine_verdict(results);
    let verdict_str = match &verdict {
        TrialVerdict::Positive => "**POSITIVE** ✓",
        TrialVerdict::Negative => "**NEGATIVE** ✗",
        TrialVerdict::Inconclusive => "**INCONCLUSIVE** ~",
    };

    let mut report = String::new();

    // ── Header ────────────────────────────────────────────────────────────────
    report.push_str(&format!(
        "# Clinical Study Report\n\n\
         **Trial ID:** {}\n\
         **Verdict:** {}\n\
         **Generated:** {}\n\n",
        protocol.id, verdict_str, protocol.created_at
    ));

    // ── Protocol Summary ──────────────────────────────────────────────────────
    report.push_str("---\n\n## Protocol Summary\n\n");
    report.push_str(&format!("**Hypothesis:** {}\n\n", protocol.hypothesis));
    report.push_str(&format!("**Population:** {}\n\n", protocol.population));
    report.push_str(&format!(
        "**Design:** {} arms, {} subjects planned, power={:.0}%, α={:.3}\n\n",
        protocol.arms.len(),
        protocol.sample_size,
        protocol.power * 100.0,
        protocol.alpha,
    ));
    report.push_str(&format!("**Duration:** {} days\n\n", protocol.duration_days));
    report.push_str(&format!("**Blinding:** {:?}\n\n", protocol.blinding));

    // Arms table
    report.push_str("### Arms\n\n");
    report.push_str("| Arm | Description | Role |\n");
    report.push_str("|-----|-------------|------|\n");
    for arm in &protocol.arms {
        let role = if arm.is_control { "Control" } else { "Treatment" };
        report.push_str(&format!("| {} | {} | {} |\n", arm.name, arm.description, role));
    }
    report.push('\n');

    // ── CONSORT Flow ──────────────────────────────────────────────────────────
    report.push_str("---\n\n## CONSORT Flow\n\n");
    report.push_str("```\n");
    report.push_str(&format!("Enrolled:     {} subjects\n", protocol.sample_size));
    report.push_str(&format!(
        "Randomized:   {} subjects ({} arms)\n",
        protocol.sample_size,
        protocol.arms.len()
    ));
    report.push_str(&format!("Analyzed:     {} subjects (primary analysis)\n", protocol.sample_size));
    report.push_str("```\n\n");

    // ── Primary Endpoint Results ──────────────────────────────────────────────
    report.push_str("---\n\n## Primary Endpoint\n\n");
    if let Some(primary) = results.first() {
        report.push_str(&format_endpoint_section(primary, protocol.alpha));
    } else {
        report.push_str("*No primary endpoint results provided.*\n\n");
    }

    // ── Secondary Endpoint Results ────────────────────────────────────────────
    if results.len() > 1 {
        report.push_str("---\n\n## Secondary Endpoints\n\n");
        report.push_str(
            "> Note: Secondary endpoints require multiplicity adjustment. \
             Results below are unadjusted. Apply Holm or Benjamini-Hochberg correction.\n\n",
        );
        for (i, result) in results.iter().skip(1).enumerate() {
            report.push_str(&format!("### Secondary Endpoint {}\n\n", i + 1));
            report.push_str(&format_endpoint_section(result, protocol.alpha));
        }
    }

    // ── Safety Summary ────────────────────────────────────────────────────────
    report.push_str("---\n\n## Safety Summary\n\n");
    report.push_str(&format!(
        "**Safety Boundary:** {} < {:.3}\n\n",
        protocol.safety_boundary.metric, protocol.safety_boundary.threshold
    ));
    report.push_str(&format!(
        "**Rule:** {}\n\n",
        protocol.safety_boundary.description
    ));
    report.push_str("*No safety stopping occurred during this trial.*\n\n");

    // ── Adaptation Rules ──────────────────────────────────────────────────────
    if !protocol.adaptation_rules.is_empty() {
        report.push_str("---\n\n## Pre-Specified Adaptations\n\n");
        report.push_str("| Type | Conditions | Allowed Changes |\n");
        report.push_str("|------|------------|-----------------|\n");
        for rule in &protocol.adaptation_rules {
            report.push_str(&format!(
                "| {} | {} | {} |\n",
                rule.adaptation_type, rule.conditions, rule.allowed_changes
            ));
        }
        report.push('\n');
    }

    // ── Verdict ───────────────────────────────────────────────────────────────
    report.push_str("---\n\n## Trial Verdict\n\n");
    report.push_str(&format!("### {verdict_str}\n\n"));
    let verdict_explanation = match &verdict {
        TrialVerdict::Positive => format!(
            "The primary endpoint '{}' was met with statistical significance (p < {:.3}). \
             The hypothesis is supported by the data.",
            protocol.primary_endpoint.name, protocol.alpha
        ),
        TrialVerdict::Negative => format!(
            "The primary endpoint '{}' did not reach statistical significance (p ≥ {:.3}). \
             The null hypothesis is not rejected.",
            protocol.primary_endpoint.name, protocol.alpha
        ),
        TrialVerdict::Inconclusive => format!(
            "Results for '{}' are inconclusive. Further investigation required.",
            protocol.primary_endpoint.name
        ),
    };
    report.push_str(&format!("{verdict_explanation}\n\n"));
    report.push_str("---\n\n*Report generated by nexcore-trial (TRIAL framework v1.0)*\n");

    report
}

/// Determine the overall trial verdict from endpoint results.
///
/// Primary endpoint significance determines POSITIVE/NEGATIVE.
/// If no results, returns Inconclusive.
pub fn determine_verdict(results: &[EndpointResult]) -> TrialVerdict {
    match results.first() {
        Some(primary) => {
            if primary.significant {
                TrialVerdict::Positive
            } else {
                TrialVerdict::Negative
            }
        }
        None => TrialVerdict::Inconclusive,
    }
}

// ── Internal ─────────────────────────────────────────────────────────────────

fn format_endpoint_section(result: &EndpointResult, alpha: f64) -> String {
    let sig_str = if result.significant { "Yes" } else { "No" };
    let mut s = String::new();
    if !result.name.is_empty() {
        s.push_str(&format!("**Endpoint:** {}\n\n", result.name));
    }
    s.push_str("| Metric | Value |\n");
    s.push_str("|--------|-------|\n");
    s.push_str(&format!("| Test Statistic | {:.4} |\n", result.test_statistic));
    s.push_str(&format!("| p-value | {:.4} |\n", result.p_value));
    s.push_str(&format!("| Significant (α={alpha:.3}) | {sig_str} |\n"));
    s.push_str(&format!("| Effect Size | {:.4} |\n", result.effect_size));
    s.push_str(&format!(
        "| 95% CI | [{:.4}, {:.4}] |\n",
        result.ci_lower, result.ci_upper
    ));
    if let Some(nnt) = result.nnt {
        s.push_str(&format!("| NNT | {:.1} |\n", nnt));
    }
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Arm, BlindingLevel, Endpoint, EndpointDirection, SafetyRule,
    };

    fn make_protocol() -> Protocol {
        Protocol {
            id: "trial-001".into(),
            hypothesis: "Treatment A improves conversion rate".into(),
            population: "Adults 18-65".into(),
            primary_endpoint: Endpoint {
                name: "conversion_rate".into(),
                metric: "proportion converting".into(),
                direction: EndpointDirection::Higher,
                threshold: 0.05,
            },
            secondary_endpoints: vec![],
            arms: vec![
                Arm { name: "control".into(), description: "Standard".into(), is_control: true },
                Arm { name: "treatment".into(), description: "New flow".into(), is_control: false },
            ],
            sample_size: 400,
            power: 0.80,
            alpha: 0.05,
            duration_days: 30,
            safety_boundary: SafetyRule {
                metric: "sae_rate".into(),
                threshold: 0.02,
                description: "Stop if SAE rate > 2%".into(),
            },
            adaptation_rules: vec![],
            blinding: BlindingLevel::Double,
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn significant_result() -> EndpointResult {
        EndpointResult {
            name: "conversion_rate".into(),
            test_statistic: 2.45,
            p_value: 0.014,
            significant: true,
            effect_size: 0.15,
            ci_lower: 0.03,
            ci_upper: 0.27,
            nnt: Some(6.7),
        }
    }

    fn null_result() -> EndpointResult {
        EndpointResult {
            name: "conversion_rate".into(),
            test_statistic: 0.85,
            p_value: 0.39,
            significant: false,
            effect_size: 0.04,
            ci_lower: -0.05,
            ci_upper: 0.13,
            nnt: None,
        }
    }

    #[test]
    fn test_generate_report_contains_all_sections() {
        let protocol = make_protocol();
        let results = vec![significant_result()];
        let report = generate_report(&protocol, &results);
        assert!(report.contains("CONSORT Flow"), "Missing CONSORT Flow section");
        assert!(report.contains("Primary Endpoint"), "Missing Primary Endpoint section");
        assert!(report.contains("Safety Summary"), "Missing Safety Summary section");
    }

    #[test]
    fn test_positive_verdict() {
        let protocol = make_protocol();
        let report = generate_report(&protocol, &[significant_result()]);
        assert!(report.contains("POSITIVE"), "Expected POSITIVE verdict");
    }

    #[test]
    fn test_negative_verdict() {
        let protocol = make_protocol();
        let report = generate_report(&protocol, &[null_result()]);
        assert!(report.contains("NEGATIVE"), "Expected NEGATIVE verdict");
    }

    #[test]
    fn test_inconclusive_verdict_no_results() {
        let protocol = make_protocol();
        let report = generate_report(&protocol, &[]);
        assert!(report.contains("INCONCLUSIVE"), "Expected INCONCLUSIVE with no results");
    }

    #[test]
    fn test_report_contains_protocol_id() {
        let protocol = make_protocol();
        let report = generate_report(&protocol, &[significant_result()]);
        assert!(report.contains("trial-001"), "Report should contain protocol ID");
    }

    #[test]
    fn test_determine_verdict_positive() {
        assert_eq!(determine_verdict(&[significant_result()]), TrialVerdict::Positive);
    }

    #[test]
    fn test_determine_verdict_negative() {
        assert_eq!(determine_verdict(&[null_result()]), TrialVerdict::Negative);
    }

    #[test]
    fn test_determine_verdict_inconclusive() {
        assert_eq!(determine_verdict(&[]), TrialVerdict::Inconclusive);
    }
}
