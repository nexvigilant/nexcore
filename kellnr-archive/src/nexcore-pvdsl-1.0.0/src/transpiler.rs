//! # GVP-to-Axiom Transpiler
//!
//! Converts complex regulatory guidelines (GVP, FDA) into executable PVDSL axioms.

use serde::{Deserialize, Serialize};

/// A single condition within a regulatory rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Signal metric name (e.g., "PRR", "ROR", "IC")
    pub metric: String,
    /// Comparison operator (e.g., ">", ">=", "<")
    pub operator: String,
    /// Numeric threshold for the condition
    pub threshold: f64,
}

/// A complex regulatory rule with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryRule {
    /// Unique rule identifier
    pub id: String,
    /// Human-readable rule name
    pub name: String,
    /// Conditions that must be met for the rule to trigger
    pub conditions: Vec<RuleCondition>,
    /// Action to take when rule is triggered
    pub action: String,
    /// ToV harm type mapping (if applicable)
    pub tov_harm_type: Option<String>,
    /// Source regulatory guideline (e.g., "GVP-IX", "FDA-21CFR")
    pub guideline_source: String,
}

/// Transpiler for converting guidelines to PVDSL scripts.
pub struct GvpTranspiler {
    /// Collection of regulatory rules to transpile
    pub rules: Vec<RegulatoryRule>,
}

impl Default for GvpTranspiler {
    fn default() -> Self {
        Self::new()
    }
}

impl GvpTranspiler {
    /// Create a new transpiler.
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the transpiler.
    pub fn add_rule(&mut self, rule: RegulatoryRule) {
        self.rules.push(rule);
    }

    /// Transpile a single rule condition to its PVDSL equivalent (L1 Atom).
    fn transpile_condition(condition: &RuleCondition) -> String {
        let pvdsl_metric = match condition.metric.to_lowercase().as_str() {
            "prr" => "signal::prr(a, b, c, d)",
            "ror" => "signal::ror(a, b, c, d)",
            "chi_square" => "signal::chi_square(a, b, c, d)",
            "ic" => "signal::ic(a, b, c, d)",
            "ebgm" => "signal::ebgm(a, b, c, d)",
            _ => "0.0",
        };

        format!(
            "{} {} {:.4}",
            pvdsl_metric, condition.operator, condition.threshold
        )
    }

    /// Transpile all rules into a single PVDSL script (L2 Molecule).
    pub fn emit_pvdsl(&self) -> String {
        let mut script = String::new();
        script.push_str("// Auto-generated PVDSL Axioms from Regulatory Guidelines\n");
        script.push_str(&format!(
            "// Source: Combined Kernel Policy ({} rules)\n\n",
            self.rules.len()
        ));

        for rule in &self.rules {
            script.push_str(&format!("// Rule {}: {}\n", rule.id, rule.name));
            if let Some(harm) = &rule.tov_harm_type {
                script.push_str(&format!("// ToV Harm Type: {}\n", harm));
            }
            script.push_str(&format!("// Guideline: {}\n", rule.guideline_source));

            let combined_conditions = rule
                .conditions
                .iter()
                .map(Self::transpile_condition)
                .collect::<Vec<_>>()
                .join(" && ");

            script.push_str(&format!(
                "if {} {{    return \"{}\"\n}}\n\n",
                combined_conditions, rule.action
            ));
        }

        script.push_str("return \"compliant\"");
        script
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_transpilation() {
        let mut transpiler = GvpTranspiler::new();
        transpiler.add_rule(RegulatoryRule {
            id: "GVP-IX-02".into(),
            name: "Evans Standard".into(),
            conditions: vec![
                RuleCondition {
                    metric: "prr".into(),
                    operator: ">=".into(),
                    threshold: 2.0,
                },
                RuleCondition {
                    metric: "chi_square".into(),
                    operator: ">=".into(),
                    threshold: 3.841,
                },
            ],
            action: "signal_detected".into(),
            tov_harm_type: Some("Acute".into()),
            guideline_source: "EMA GVP Module IX".into(),
        });

        let script = transpiler.emit_pvdsl();
        assert!(script.contains("signal::prr"));
        assert!(script.contains("signal::chi_square"));
        assert!(script.contains("&& "));
        assert!(script.contains("GVP Module IX"));
    }
}
