//! SkillContext: applies the GROUNDED feedback loop to skill improvement.
//!
//! Implements Context for the skill ecosystem — each iteration
//! hypothesizes a gap, tests for it, and learns from the result.
//!
//! Grounds to: ρ(Recursion) + κ(Comparison) + ∃(Existence)

use std::path::PathBuf;

use nexcore_chrono::DateTime;

use crate::GroundedError;
use crate::confidence::Confidence;
use crate::feedback::{Context, Experiment, Hypothesis, Learning, Outcome, Verdict};

/// Compliance checks ordered by priority (Bronze → Diamond).
const CHECKS: &[SkillCheck] = &[
    SkillCheck {
        name: "skill_md_exists",
        claim: "SKILL.md exists with content",
        command: "test -s \"$SKILL_PATH/SKILL.md\" && echo PASS || echo FAIL",
        tier: "Bronze",
        weight: 1.0,
    },
    SkillCheck {
        name: "frontmatter_valid",
        claim: "SKILL.md has valid YAML frontmatter",
        command: "head -1 \"$SKILL_PATH/SKILL.md\" | grep -q '^---' && echo PASS || echo FAIL",
        tier: "Bronze",
        weight: 0.9,
    },
    SkillCheck {
        name: "name_kebab_case",
        claim: "name field is kebab-case",
        command: "grep '^name:' \"$SKILL_PATH/SKILL.md\" | grep -qE '^name: [a-z][a-z0-9-]*$' && echo PASS || echo FAIL",
        tier: "Bronze",
        weight: 0.8,
    },
    SkillCheck {
        name: "references_exist",
        claim: "references/ directory exists with files",
        command: "test -d \"$SKILL_PATH/references\" && ls \"$SKILL_PATH/references/\" | grep -q . && echo PASS || echo FAIL",
        tier: "Silver",
        weight: 0.7,
    },
    SkillCheck {
        name: "scripts_exist",
        claim: "scripts/ directory exists",
        command: "test -d \"$SKILL_PATH/scripts\" && echo PASS || echo FAIL",
        tier: "Silver",
        weight: 0.7,
    },
    SkillCheck {
        name: "agent_persona",
        claim: "agent/persona.md exists",
        command: "test -f \"$SKILL_PATH/agent/persona.md\" && echo PASS || echo FAIL",
        tier: "Gold",
        weight: 0.6,
    },
    SkillCheck {
        name: "under_500_lines",
        claim: "SKILL.md is under 500 lines",
        command: "lines=$(wc -l < \"$SKILL_PATH/SKILL.md\" 2>/dev/null || echo 9999) && [ \"$lines\" -le 500 ] && echo PASS || echo FAIL",
        tier: "Gold",
        weight: 0.5,
    },
    SkillCheck {
        name: "three_plus_refs",
        claim: "3+ reference files exist",
        command: "count=$(ls \"$SKILL_PATH/references/\" 2>/dev/null | wc -l) && [ \"$count\" -ge 3 ] && echo PASS || echo FAIL",
        tier: "Gold",
        weight: 0.5,
    },
    SkillCheck {
        name: "scripts_executable",
        claim: "scripts are executable",
        command: "find \"$SKILL_PATH/scripts\" -name '*.sh' ! -executable 2>/dev/null | grep -q . && echo FAIL || echo PASS",
        tier: "Platinum",
        weight: 0.4,
    },
];

struct SkillCheck {
    name: &'static str,
    claim: &'static str,
    command: &'static str,
    tier: &'static str,
    weight: f64,
}

/// Context for improving a single skill through the GROUNDED loop.
pub struct SkillContext {
    /// Path to the skill directory.
    skill_path: PathBuf,
    /// Which check index we're currently testing.
    check_index: usize,
    /// Accumulated learnings from prior iterations.
    passed: Vec<String>,
    failed: Vec<String>,
}

impl SkillContext {
    /// Create a new SkillContext for the given skill path.
    pub fn new(skill_path: impl Into<PathBuf>) -> Self {
        Self {
            skill_path: skill_path.into(),
            check_index: 0,
            passed: Vec::new(),
            failed: Vec::new(),
        }
    }

    /// Get the skill name from the directory.
    pub fn skill_name(&self) -> &str {
        self.skill_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    /// Get summary of findings so far.
    pub fn summary(&self) -> SkillSummary {
        let total = self.passed.len() + self.failed.len();
        let score = if total > 0 {
            self.passed.len() as f64 / total as f64
        } else {
            0.0
        };
        SkillSummary {
            skill_name: self.skill_name().to_string(),
            skill_path: self.skill_path.clone(),
            checks_passed: self.passed.clone(),
            checks_failed: self.failed.clone(),
            compliance_score: score,
            tier: self.compute_tier(),
        }
    }

    fn compute_tier(&self) -> String {
        let has = |name: &str| self.passed.iter().any(|p| p == name);

        if has("skill_md_exists")
            && has("frontmatter_valid")
            && has("name_kebab_case")
            && has("references_exist")
            && has("scripts_exist")
            && has("agent_persona")
            && has("under_500_lines")
            && has("three_plus_refs")
            && has("scripts_executable")
        {
            "Diamond".into()
        } else if has("skill_md_exists")
            && has("frontmatter_valid")
            && has("references_exist")
            && has("scripts_exist")
            && has("agent_persona")
            && has("under_500_lines")
        {
            "Gold".into()
        } else if has("skill_md_exists")
            && has("frontmatter_valid")
            && (has("references_exist") || has("scripts_exist"))
        {
            "Silver".into()
        } else if has("skill_md_exists") {
            "Bronze".into()
        } else {
            "None".into()
        }
    }
}

/// Summary of a skill's grounded assessment.
#[derive(Debug, Clone)]
pub struct SkillSummary {
    pub skill_name: String,
    pub skill_path: PathBuf,
    pub checks_passed: Vec<String>,
    pub checks_failed: Vec<String>,
    pub compliance_score: f64,
    pub tier: String,
}

impl std::fmt::Display for SkillSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌─────────────────────────────────────────────┐")?;
        writeln!(f, "│ GROUNDED: {:<34}│", self.skill_name)?;
        writeln!(f, "├─────────────────────────────────────────────┤")?;
        writeln!(f, "│ Tier: {:<38}│", self.tier)?;
        let score_str = format!(
            "{:.0}% ({}/{})",
            self.compliance_score * 100.0,
            self.checks_passed.len(),
            self.checks_passed.len() + self.checks_failed.len()
        );
        let padding = 30usize.saturating_sub(score_str.len());
        writeln!(f, "│ Score: {score_str}{}│", " ".repeat(padding))?;
        if !self.checks_failed.is_empty() {
            writeln!(f, "├─────────────────────────────────────────────┤")?;
            writeln!(f, "│ GAPS:                                       │")?;
            for gap in &self.checks_failed {
                writeln!(f, "│  - {gap:<41}│")?;
            }
        }
        writeln!(f, "└─────────────────────────────────────────────┘")?;
        Ok(())
    }
}

impl Context for SkillContext {
    fn reason(&self) -> Result<Hypothesis, GroundedError> {
        if self.check_index >= CHECKS.len() {
            return Err(GroundedError::ExperimentFailed(
                "all checks exhausted".into(),
            ));
        }

        let check = &CHECKS[self.check_index];
        let prior = Confidence::new(check.weight)
            .map_err(|e| GroundedError::ExperimentFailed(e.to_string()))?;

        Ok(Hypothesis {
            claim: format!("[{}] {}: {}", check.tier, check.name, check.claim),
            prior,
            falsification_criteria: format!("command returns FAIL: {}", check.command),
            generated_at: DateTime::now(),
        })
    }

    fn design_test(&self, _hypothesis: &Hypothesis) -> Result<Experiment, GroundedError> {
        if self.check_index >= CHECKS.len() {
            return Err(GroundedError::ExperimentFailed(
                "all checks exhausted".into(),
            ));
        }

        let check = &CHECKS[self.check_index];
        let command = check
            .command
            .replace("$SKILL_PATH", &self.skill_path.to_string_lossy());

        Ok(Experiment {
            description: command,
            hypothesis_claim: check.claim.to_string(),
            expected_if_true: "PASS".into(),
            expected_if_false: "FAIL".into(),
        })
    }

    fn integrate(
        &self,
        hypothesis: &Hypothesis,
        outcome: &Outcome,
    ) -> Result<Learning, GroundedError> {
        let passed = outcome.observation.trim() == "PASS";
        let verdict = if passed {
            Verdict::Supported
        } else {
            Verdict::Refuted
        };

        let posterior = if passed {
            Confidence::new(0.95)
        } else {
            Confidence::new(0.1)
        }
        .map_err(|e| GroundedError::IntegrationFailed(e.to_string()))?;

        Ok(Learning {
            insight: if passed {
                format!("PASS: {}", hypothesis.claim)
            } else {
                format!("FAIL: {} — needs fix", hypothesis.claim)
            },
            posterior,
            verdict,
            hypothesis_claim: hypothesis.claim.clone(),
            observation: outcome.observation.clone(),
            learned_at: DateTime::now(),
        })
    }

    fn update(&mut self, learning: &Learning) -> Result<(), GroundedError> {
        if self.check_index < CHECKS.len() {
            let check_name = CHECKS[self.check_index].name.to_string();
            match learning.verdict {
                Verdict::Supported => self.passed.push(check_name),
                Verdict::Refuted | Verdict::Inconclusive => self.failed.push(check_name),
            }
        }
        self.check_index += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_context_creates_hypotheses() {
        let ctx = SkillContext::new("/tmp/fake-skill");
        let h = ctx.reason();
        assert!(h.is_ok());
        let h = h.unwrap_or_else(|_| Hypothesis {
            claim: String::new(),
            prior: Confidence::NONE,
            falsification_criteria: String::new(),
            generated_at: DateTime::now(),
        });
        assert!(h.claim.contains("SKILL.md"));
    }

    #[test]
    fn skill_context_designs_test() {
        let ctx = SkillContext::new("/tmp/fake-skill");
        let h = ctx.reason().unwrap_or_else(|_| Hypothesis {
            claim: String::new(),
            prior: Confidence::NONE,
            falsification_criteria: String::new(),
            generated_at: DateTime::now(),
        });
        let exp = ctx.design_test(&h);
        assert!(exp.is_ok());
        let exp = exp.unwrap_or_else(|_| Experiment {
            description: String::new(),
            hypothesis_claim: String::new(),
            expected_if_true: String::new(),
            expected_if_false: String::new(),
        });
        assert!(exp.description.contains("/tmp/fake-skill"));
    }

    #[test]
    fn skill_summary_display() {
        let summary = SkillSummary {
            skill_name: "test-skill".into(),
            skill_path: PathBuf::from("/tmp/test-skill"),
            checks_passed: vec!["skill_md_exists".into(), "frontmatter_valid".into()],
            checks_failed: vec!["references_exist".into()],
            compliance_score: 0.67,
            tier: "Bronze".into(),
        };
        let display = format!("{summary}");
        assert!(display.contains("test-skill"));
        assert!(display.contains("Bronze"));
        assert!(display.contains("references_exist"));
    }
}
