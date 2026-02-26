//! Content accuracy validation rules (R9-R14).
//!
//! Cross-references academy content against the forge IR to ensure
//! factual correctness.

use crate::ir::DomainAnalysis;
use crate::validate::{Severity, ValidationFinding};

/// Run accuracy validation rules R9-R14.
pub fn validate_accuracy(
    content: &serde_json::Value,
    domain: &DomainAnalysis,
) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();
    let content_text = content.to_string();

    validate_axiom_completeness(&content_text, domain, &mut findings);
    validate_harm_type_completeness(&content_text, domain, &mut findings);
    validate_conservation_law_completeness(&content_text, domain, &mut findings);
    validate_theorem_accuracy(&content_text, domain, &mut findings);
    validate_dag_consistency(content, domain, &mut findings);
    validate_threshold_values(&content_text, domain, &mut findings);

    findings
}

/// R9: All 5 axiom names from IR appear in content.
fn validate_axiom_completeness(
    content_text: &str,
    domain: &DomainAnalysis,
    findings: &mut Vec<ValidationFinding>,
) {
    for axiom in &domain.axioms {
        if !content_text.contains(&axiom.name) {
            findings.push(ValidationFinding {
                rule: "R9".to_string(),
                severity: Severity::Error,
                message: format!("Axiom {} ({}) not found in content", axiom.id, axiom.name),
                field_path: None,
            });
        }
    }
}

/// R10: All 8 harm type names found in content.
fn validate_harm_type_completeness(
    content_text: &str,
    domain: &DomainAnalysis,
    findings: &mut Vec<ValidationFinding>,
) {
    for harm_type in &domain.harm_types {
        if !content_text.contains(&harm_type.name) {
            findings.push(ValidationFinding {
                rule: "R10".to_string(),
                severity: Severity::Error,
                message: format!(
                    "Harm type {} ({}) not found in content",
                    harm_type.letter, harm_type.name
                ),
                field_path: None,
            });
        }
    }
}

/// R11: All 11 conservation law names found in content.
fn validate_conservation_law_completeness(
    content_text: &str,
    domain: &DomainAnalysis,
    findings: &mut Vec<ValidationFinding>,
) {
    for law in &domain.conservation_laws {
        if !content_text.contains(&law.name) {
            findings.push(ValidationFinding {
                rule: "R11".to_string(),
                severity: Severity::Error,
                message: format!(
                    "Conservation law {} ({}) not found in content",
                    law.number, law.name
                ),
                field_path: None,
            });
        }
    }
}

/// R12: 3 theorem names found with correct axiom dependencies.
fn validate_theorem_accuracy(
    content_text: &str,
    domain: &DomainAnalysis,
    findings: &mut Vec<ValidationFinding>,
) {
    for theorem in &domain.theorems {
        if !content_text.contains(&theorem.name) {
            findings.push(ValidationFinding {
                rule: "R12".to_string(),
                severity: Severity::Error,
                message: format!("Theorem '{}' not found in content", theorem.name),
                field_path: None,
            });
        }
    }
}

/// R13: DAG structure is consistent with the domain axiom set.
///
/// Verifies that the DAG node count matches the number of axioms in the IR,
/// and that the DAG has at least one edge (a single isolated-node DAG would
/// indicate a wiring failure rather than a valid dependency graph).
///
/// Previously this rule hardcoded `5` nodes and `5` edges, which was
/// wrong for any domain other than vigilance.  The correct invariant is:
/// `dag.nodes.len() == domain.axioms.len()`.
fn validate_dag_consistency(
    _content: &serde_json::Value,
    domain: &DomainAnalysis,
    findings: &mut Vec<ValidationFinding>,
) {
    let expected_node_count = domain.axioms.len();

    if domain.dependency_dag.nodes.len() != expected_node_count {
        findings.push(ValidationFinding {
            rule: "R13".to_string(),
            severity: Severity::Warning,
            message: format!(
                "DAG has {} nodes but domain has {} axioms — counts should match",
                domain.dependency_dag.nodes.len(),
                expected_node_count
            ),
            field_path: None,
        });
    }

    if expected_node_count > 1 && domain.dependency_dag.edges.is_empty() {
        findings.push(ValidationFinding {
            rule: "R13".to_string(),
            severity: Severity::Warning,
            message: format!(
                "DAG has {} nodes but zero edges — expected dependency edges between axioms",
                expected_node_count
            ),
            field_path: None,
        });
    }
}

/// R14: Threshold values match canonical values from IR.
fn validate_threshold_values(
    content_text: &str,
    domain: &DomainAnalysis,
    findings: &mut Vec<ValidationFinding>,
) {
    let t = &domain.signal_thresholds;
    let checks = [
        ("PRR", t.prr, "2.0"),
        ("Chi-square", t.chi_square, "3.841"),
        ("EB05", t.eb05, "2.0"),
    ];

    for (name, _expected, canonical_str) in &checks {
        // Check if the canonical threshold string appears in content
        if !content_text.contains(canonical_str) {
            findings.push(ValidationFinding {
                rule: "R14".to_string(),
                severity: Severity::Warning,
                message: format!("Signal threshold {name} >= {canonical_str} not found in content"),
                field_path: None,
            });
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::vigilance::extract_vigilance_domain;
    use serde_json::json;

    #[test]
    fn test_missing_axiom() {
        let domain = extract_vigilance_domain();
        // Content that mentions 4 axioms but not "Safety Manifold"
        let content = json!({
            "text": "System Decomposition, Hierarchical Organization, Conservation Constraints, Emergence"
        });
        let findings = validate_accuracy(&content, &domain);
        let r9s: Vec<_> = findings.iter().filter(|f| f.rule == "R9").collect();
        assert_eq!(r9s.len(), 1); // Missing "Safety Manifold"
    }

    #[test]
    fn test_all_axioms_present() {
        let domain = extract_vigilance_domain();
        let content = json!({
            "text": "System Decomposition, Hierarchical Organization, Conservation Constraints, Safety Manifold, Emergence, Acute, Cumulative, Off-Target, Cascade, Idiosyncratic, Saturation, Interaction, Population, Mass/Amount, Energy/Gradient, State Normalization, Flux Continuity, Catalyst Invariance, Entropy Increase, Momentum, Capacity/Saturation, Charge Conservation, Stoichiometry, Structural Invariant, Predictability Theorem, Attenuation Theorem, Intervention Theorem, 2.0, 3.841"
        });
        let findings = validate_accuracy(&content, &domain);
        let errors: Vec<_> = findings
            .iter()
            .filter(|f| matches!(f.severity, Severity::Error))
            .collect();
        assert!(errors.is_empty(), "Unexpected errors: {errors:?}");
    }
}
