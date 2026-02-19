//! CTVP Phase Definitions
//!
//! Maps pharmaceutical trial phases to software validation stages.

use serde::{Deserialize, Serialize};

/// The five CTVP validation phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Phase {
    /// Phase 0: Preclinical - Mechanism validity (unit tests, mocks)
    Preclinical = 0,
    /// Phase 1: Safety - Failure mode validation (fault injection, chaos)
    Safety = 1,
    /// Phase 2: Efficacy - Capability achievement (real data, SLOs)
    Efficacy = 2,
    /// Phase 3: Confirmation - Scale validation (shadow, canary, A/B)
    Confirmation = 3,
    /// Phase 4: Surveillance - Ongoing correctness (drift, monitoring)
    Surveillance = 4,
}

impl Phase {
    /// Returns the pharmaceutical equivalent name
    pub fn pharma_name(&self) -> &'static str {
        match self {
            Self::Preclinical => "Preclinical (In-Vitro/In-Vivo)",
            Self::Safety => "Phase 1 (First-in-Human)",
            Self::Efficacy => "Phase 2 (Proof of Concept)",
            Self::Confirmation => "Phase 3 (Pivotal Trials)",
            Self::Surveillance => "Phase 4 (Post-Market)",
        }
    }

    /// Returns the software equivalent description
    pub fn software_equivalent(&self) -> &'static str {
        match self {
            Self::Preclinical => "Unit tests, mocks, property-based tests, static analysis",
            Self::Safety => "Chaos engineering, fault injection, boundary testing",
            Self::Efficacy => "Real data validation, SLO measurement, capability tracking",
            Self::Confirmation => "Shadow deployment, canary rollout, A/B testing",
            Self::Surveillance => "Drift detection, continuous validation, observability",
        }
    }

    /// Returns the primary question this phase answers
    pub fn primary_question(&self) -> &'static str {
        match self {
            Self::Preclinical => "Does the mechanism work under controlled conditions?",
            Self::Safety => "Does it fail gracefully under stress?",
            Self::Efficacy => "Does it achieve its intended purpose with real data?",
            Self::Confirmation => "Does it perform at least as well at scale?",
            Self::Surveillance => "Does it continue working correctly over time?",
        }
    }

    /// Returns all phases in order
    pub fn all() -> [Phase; 5] {
        [
            Self::Preclinical,
            Self::Safety,
            Self::Efficacy,
            Self::Confirmation,
            Self::Surveillance,
        ]
    }
}

/// Status of a validation phase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseStatus {
    /// Not yet attempted
    NotStarted,
    /// In progress, collecting data
    InProgress { progress_pct: u8 },
    /// Validated successfully
    Validated,
    /// Failed validation
    Failed { reason: String },
    /// Not applicable for this capability
    NotApplicable,
}

impl PhaseStatus {
    /// Returns emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::NotStarted => "⏳",
            Self::InProgress { .. } => "🔄",
            Self::Validated => "✅",
            Self::Failed { .. } => "❌",
            Self::NotApplicable => "➖",
        }
    }

    /// Returns true if phase is validated
    pub fn is_validated(&self) -> bool {
        matches!(self, Self::Validated)
    }
}

/// Quality of validation evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceQuality {
    /// No evidence exists
    None = 0,
    /// Evidence exists but is weak (mocks, fixtures, limited scope)
    Weak = 1,
    /// Moderate evidence (some real integration, limited scale)
    Moderate = 2,
    /// Strong evidence (real deps, comprehensive testing, production-scale)
    Strong = 3,
}

impl EvidenceQuality {
    /// Returns emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::None => "❌",
            Self::Weak => "⚠️",
            Self::Moderate => "🟡",
            Self::Strong => "✅",
        }
    }
}

/// Evidence for a specific phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationEvidence {
    /// Phase this evidence applies to
    pub phase: Phase,
    /// Quality of the evidence
    pub quality: EvidenceQuality,
    /// What this evidence proves
    pub proves: Vec<String>,
    /// What this evidence does NOT prove
    pub does_not_prove: Vec<String>,
    /// Gaps identified
    pub gaps: Vec<String>,
    /// Remediation actions needed
    pub remediation: Vec<String>,
}

impl ValidationEvidence {
    /// Creates new evidence for a phase
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            quality: EvidenceQuality::None,
            proves: Vec::new(),
            does_not_prove: Vec::new(),
            gaps: Vec::new(),
            remediation: Vec::new(),
        }
    }

    /// Sets evidence quality
    pub fn with_quality(mut self, quality: EvidenceQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Adds what this evidence proves
    pub fn proves(mut self, what: &str) -> Self {
        self.proves.push(what.to_string());
        self
    }

    /// Adds what this evidence does NOT prove
    pub fn does_not_prove(mut self, what: &str) -> Self {
        self.does_not_prove.push(what.to_string());
        self
    }

    /// Adds an identified gap
    pub fn gap(mut self, gap: &str) -> Self {
        self.gaps.push(gap.to_string());
        self
    }

    /// Adds a remediation action
    pub fn remediation(mut self, action: &str) -> Self {
        self.remediation.push(action.to_string());
        self
    }
}

/// Complete CTVP validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Name of the system/capability being validated
    pub name: String,
    /// Evidence for each phase
    pub phases: Vec<ValidationEvidence>,
    /// Highest phase with validated evidence
    pub evidence_stops_at: Phase,
    /// Overall status
    pub status: PhaseStatus,
    /// Timestamp of this summary
    pub timestamp: f64,
}

impl ValidationSummary {
    /// Creates a new validation summary
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            phases: Phase::all()
                .iter()
                .map(|p| ValidationEvidence::new(*p))
                .collect(),
            evidence_stops_at: Phase::Preclinical,
            status: PhaseStatus::NotStarted,
            timestamp: crate::state::now(),
        }
    }

    /// Updates evidence for a specific phase
    pub fn set_evidence(&mut self, evidence: ValidationEvidence) {
        let phase_idx = evidence.phase as usize;
        if phase_idx < self.phases.len() {
            self.phases[phase_idx] = evidence;
        }
        self.recalculate_status();
    }

    /// Recalculates overall status based on phase evidence
    fn recalculate_status(&mut self) {
        // Find highest validated phase
        let mut highest = Phase::Preclinical;
        for evidence in &self.phases {
            if evidence.quality >= EvidenceQuality::Moderate {
                highest = evidence.phase;
            } else {
                break; // Phases must be sequential
            }
        }
        self.evidence_stops_at = highest;

        // Determine overall status
        self.status = if self
            .phases
            .iter()
            .all(|e| e.quality >= EvidenceQuality::Moderate)
        {
            PhaseStatus::Validated
        } else if self
            .phases
            .iter()
            .any(|e| e.quality >= EvidenceQuality::Weak)
        {
            let validated_count = self
                .phases
                .iter()
                .filter(|e| e.quality >= EvidenceQuality::Moderate)
                .count();
            PhaseStatus::InProgress {
                progress_pct: (validated_count * 20) as u8,
            }
        } else {
            PhaseStatus::NotStarted
        };
    }

    /// Generates a text report
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str(&format!(
            "\n╔══════════════════════════════════════════════════════╗\n"
        ));
        r.push_str(&format!("║  🔬 CTVP VALIDATION: {:<30}║\n", self.name));
        r.push_str(&format!(
            "╠══════════════════════════════════════════════════════╣\n"
        ));

        for evidence in &self.phases {
            let emoji = evidence.quality.emoji();
            let phase_name = match evidence.phase {
                Phase::Preclinical => "P0 Mechanism",
                Phase::Safety => "P1 Safety",
                Phase::Efficacy => "P2 Efficacy",
                Phase::Confirmation => "P3 Confirm",
                Phase::Surveillance => "P4 Surveil",
            };
            r.push_str(&format!(
                "║  {} {:<12} {:<30}║\n",
                emoji,
                phase_name,
                format!("{:?}", evidence.quality)
            ));
        }

        r.push_str(&format!(
            "╠══════════════════════════════════════════════════════╣\n"
        ));
        r.push_str(&format!(
            "║  Evidence stops at: Phase {:?}                       ║\n",
            self.evidence_stops_at as u8
        ));
        r.push_str(&format!(
            "║  Status: {} {:?}                                  ║\n",
            self.status.emoji(),
            self.status
        ));
        r.push_str(&format!(
            "╚══════════════════════════════════════════════════════╝\n"
        ));
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_ordering() {
        assert!(Phase::Preclinical < Phase::Safety);
        assert!(Phase::Safety < Phase::Efficacy);
        assert!(Phase::Efficacy < Phase::Confirmation);
        assert!(Phase::Confirmation < Phase::Surveillance);
    }

    #[test]
    fn test_evidence_quality_ordering() {
        assert!(EvidenceQuality::None < EvidenceQuality::Weak);
        assert!(EvidenceQuality::Weak < EvidenceQuality::Moderate);
        assert!(EvidenceQuality::Moderate < EvidenceQuality::Strong);
    }

    #[test]
    fn test_validation_summary() {
        let mut summary = ValidationSummary::new("Test System");
        assert_eq!(summary.status, PhaseStatus::NotStarted);

        // Add Phase 0 evidence
        let p0 = ValidationEvidence::new(Phase::Preclinical)
            .with_quality(EvidenceQuality::Strong)
            .proves("Unit tests pass");
        summary.set_evidence(p0);

        assert!(matches!(summary.status, PhaseStatus::InProgress { .. }));
    }
}
