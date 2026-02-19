//! Compliance Assessment DSL
//!
//! Domain-specific language for regulatory compliance assessments.
//! Generates findings from control evaluations.

use crate::oscal::{Control, ControlStatus};
use serde::{Deserialize, Serialize};

/// Severity of a compliance finding.
///
/// T2-P: Reusable severity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FindingSeverity {
    /// Informational only.
    Info,
    /// Low severity - minor deviation.
    Low,
    /// Medium severity - significant gap.
    Medium,
    /// High severity - critical deficiency.
    High,
    /// Critical - immediate action required.
    Critical,
}

/// A compliance finding from an assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Related control ID.
    pub control_id: String,
    /// Finding severity.
    pub severity: FindingSeverity,
    /// Finding title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Recommended remediation.
    pub remediation: Option<String>,
}

/// Result of a compliance assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceResult {
    /// All controls satisfied.
    Compliant,
    /// Some controls not satisfied.
    NonCompliant,
    /// Assessment could not be completed.
    Inconclusive,
}

/// A compliance assessment session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Assessment {
    /// Assessment identifier.
    pub id: String,
    /// Controls being assessed.
    pub controls: Vec<Control>,
    /// Generated findings.
    pub findings: Vec<Finding>,
    /// Overall result.
    pub result: Option<ComplianceResult>,
}

impl Assessment {
    /// Create a new assessment.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            controls: Vec::new(),
            findings: Vec::new(),
            result: None,
        }
    }

    /// Add a control to assess.
    pub fn add_control(&mut self, control: Control) {
        self.controls.push(control);
    }

    /// Record a finding.
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Evaluate and set the overall result.
    pub fn evaluate(&mut self) {
        let has_critical = self
            .findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Critical);
        let has_high = self
            .findings
            .iter()
            .any(|f| f.severity == FindingSeverity::High);
        let all_implemented = self.controls.iter().all(|c| {
            matches!(
                c.status,
                ControlStatus::Implemented | ControlStatus::NotApplicable
            )
        });

        self.result = Some(if has_critical || has_high {
            ComplianceResult::NonCompliant
        } else if all_implemented {
            ComplianceResult::Compliant
        } else {
            ComplianceResult::Inconclusive
        });
    }

    /// Get count of findings by severity.
    #[must_use]
    pub fn finding_count(&self, severity: FindingSeverity) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(severity: FindingSeverity) -> Finding {
        Finding {
            control_id: "TEST-001".to_string(),
            severity,
            title: format!("{severity:?} finding"),
            description: "Test finding".to_string(),
            remediation: None,
        }
    }

    fn make_control(status: ControlStatus) -> Control {
        Control {
            id: "CTL-001".to_string(),
            title: "Test Control".to_string(),
            description: "A test control".to_string(),
            catalog: "TEST".to_string(),
            status,
        }
    }

    #[test]
    fn new_assessment_has_no_result() {
        let assessment = Assessment::new("test-001");
        assert_eq!(assessment.id, "test-001");
        assert!(assessment.result.is_none());
        assert!(assessment.findings.is_empty());
        assert!(assessment.controls.is_empty());
    }

    #[test]
    fn evaluate_compliant_when_all_implemented() {
        let mut assessment = Assessment::new("assess-01");
        assessment.add_control(make_control(ControlStatus::Implemented));
        assessment.add_control(make_control(ControlStatus::NotApplicable));
        assessment.add_finding(make_finding(FindingSeverity::Info));

        assessment.evaluate();
        assert_eq!(assessment.result, Some(ComplianceResult::Compliant));
    }

    #[test]
    fn evaluate_non_compliant_with_critical_finding() {
        let mut assessment = Assessment::new("assess-02");
        assessment.add_control(make_control(ControlStatus::Implemented));
        assessment.add_finding(make_finding(FindingSeverity::Critical));

        assessment.evaluate();
        assert_eq!(assessment.result, Some(ComplianceResult::NonCompliant));
    }

    #[test]
    fn evaluate_non_compliant_with_high_finding() {
        let mut assessment = Assessment::new("assess-03");
        assessment.add_control(make_control(ControlStatus::Implemented));
        assessment.add_finding(make_finding(FindingSeverity::High));

        assessment.evaluate();
        assert_eq!(assessment.result, Some(ComplianceResult::NonCompliant));
    }

    #[test]
    fn evaluate_inconclusive_when_partial() {
        let mut assessment = Assessment::new("assess-04");
        assessment.add_control(make_control(ControlStatus::Partial));
        assessment.add_finding(make_finding(FindingSeverity::Low));

        assessment.evaluate();
        assert_eq!(assessment.result, Some(ComplianceResult::Inconclusive));
    }

    #[test]
    fn finding_count_by_severity() {
        let mut assessment = Assessment::new("assess-05");
        assessment.add_finding(make_finding(FindingSeverity::Low));
        assessment.add_finding(make_finding(FindingSeverity::Low));
        assessment.add_finding(make_finding(FindingSeverity::High));

        assert_eq!(assessment.finding_count(FindingSeverity::Low), 2);
        assert_eq!(assessment.finding_count(FindingSeverity::High), 1);
        assert_eq!(assessment.finding_count(FindingSeverity::Critical), 0);
    }

    #[test]
    fn finding_severity_ordering() {
        assert!(FindingSeverity::Critical > FindingSeverity::High);
        assert!(FindingSeverity::High > FindingSeverity::Medium);
        assert!(FindingSeverity::Medium > FindingSeverity::Low);
        assert!(FindingSeverity::Low > FindingSeverity::Info);
    }
}
