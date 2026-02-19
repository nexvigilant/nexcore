//! CTVP Phase Definitions

use serde::{Deserialize, Serialize};

/// The five CTVP validation phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Phase {
    /// Phase 0: Preclinical - Mechanism validity
    Preclinical = 0,
    /// Phase 1: Safety - Failure mode validation
    Safety = 1,
    /// Phase 2: Efficacy - Capability achievement
    Efficacy = 2,
    /// Phase 3: Confirmation - Scale validation
    Confirmation = 3,
    /// Phase 4: Surveillance - Ongoing correctness
    Surveillance = 4,
}

impl Phase {
    /// Returns pharmaceutical equivalent name
    ///
    /// # Returns
    /// Static string with pharma phase name
    pub fn pharma_name(&self) -> &'static str {
        match self {
            Self::Preclinical => "Preclinical",
            Self::Safety => "Phase 1",
            Self::Efficacy => "Phase 2",
            Self::Confirmation => "Phase 3",
            Self::Surveillance => "Phase 4",
        }
    }

    /// Returns software equivalent description
    ///
    /// # Returns
    /// Static string describing software testing stage
    pub fn software_equivalent(&self) -> &'static str {
        match self {
            Self::Preclinical => "Unit tests, mocks, property-based tests",
            Self::Safety => "Chaos engineering, fault injection",
            Self::Efficacy => "Real data validation, SLO measurement",
            Self::Confirmation => "Shadow/canary deployment, A/B testing",
            Self::Surveillance => "Drift detection, continuous validation",
        }
    }

    /// Returns all phases in order
    ///
    /// # Returns
    /// Array of all 5 phases
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
    /// In progress
    InProgress {
        /// Progress percentage (0-100)
        progress_pct: u8,
    },
    /// Validated successfully
    Validated,
    /// Failed validation
    Failed {
        /// Reason for failure
        reason: String,
    },
    /// Not applicable
    NotApplicable,
}

impl PhaseStatus {
    /// Returns emoji representation
    ///
    /// # Returns
    /// Emoji string for status
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::NotStarted => "⏳",
            Self::InProgress { .. } => "🔄",
            Self::Validated => "✅",
            Self::Failed { .. } => "❌",
            Self::NotApplicable => "➖",
        }
    }

    /// Returns true if validated
    ///
    /// # Returns
    /// True if phase is validated
    pub fn is_validated(&self) -> bool {
        matches!(self, Self::Validated)
    }
}

/// Quality of validation evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceQuality {
    /// No evidence
    None = 0,
    /// Weak evidence (mocks, fixtures)
    Weak = 1,
    /// Moderate evidence (some real integration)
    Moderate = 2,
    /// Strong evidence (real deps, comprehensive)
    Strong = 3,
}

impl EvidenceQuality {
    /// Returns emoji representation
    ///
    /// # Returns
    /// Emoji for quality level
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
    /// Phase this applies to
    pub phase: Phase,
    /// Quality level
    pub quality: EvidenceQuality,
    /// What this proves
    pub proves: Vec<String>,
    /// What this does NOT prove
    pub does_not_prove: Vec<String>,
    /// Identified gaps
    pub gaps: Vec<String>,
}

impl ValidationEvidence {
    /// Creates new evidence for a phase
    ///
    /// # Arguments
    /// * `phase` - The phase this evidence is for
    ///
    /// # Returns
    /// New ValidationEvidence with defaults
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            quality: EvidenceQuality::None,
            proves: Vec::new(),
            does_not_prove: Vec::new(),
            gaps: Vec::new(),
        }
    }

    /// Sets quality level
    ///
    /// # Arguments
    /// * `quality` - The quality level
    ///
    /// # Returns
    /// Self for chaining
    pub fn with_quality(mut self, quality: EvidenceQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Adds what this proves
    ///
    /// # Arguments
    /// * `what` - Description of what is proven
    ///
    /// # Returns
    /// Self for chaining
    pub fn proves(mut self, what: &str) -> Self {
        self.proves.push(what.to_string());
        self
    }
}

/// Complete validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Name of capability
    pub name: String,
    /// Evidence for each phase
    pub phases: Vec<ValidationEvidence>,
    /// Highest validated phase
    pub evidence_stops_at: Phase,
    /// Overall status
    pub status: PhaseStatus,
    /// Timestamp
    pub timestamp: f64,
}

impl ValidationSummary {
    /// Creates new validation summary
    ///
    /// # Arguments
    /// * `name` - Name of the capability
    ///
    /// # Returns
    /// New ValidationSummary
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            phases: Phase::all()
                .iter()
                .map(|p| ValidationEvidence::new(*p))
                .collect(),
            evidence_stops_at: Phase::Preclinical,
            status: PhaseStatus::NotStarted,
            timestamp: super::now(),
        }
    }

    /// Updates evidence for a phase
    ///
    /// # Arguments
    /// * `evidence` - The evidence to set
    pub fn set_evidence(&mut self, evidence: ValidationEvidence) {
        let idx = evidence.phase as usize;
        if idx < self.phases.len() {
            self.phases[idx] = evidence;
        }
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let mut highest = Phase::Preclinical;
        for e in &self.phases {
            if e.quality >= EvidenceQuality::Moderate {
                highest = e.phase;
            } else {
                break;
            }
        }
        self.evidence_stops_at = highest;

        let validated = self
            .phases
            .iter()
            .filter(|e| e.quality >= EvidenceQuality::Moderate)
            .count();
        self.status = if validated == 5 {
            PhaseStatus::Validated
        } else if validated > 0 {
            PhaseStatus::InProgress {
                progress_pct: (validated * 20) as u8,
            }
        } else {
            PhaseStatus::NotStarted
        };
    }

    /// Generates text report
    ///
    /// # Returns
    /// Formatted report string
    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str(&format!(
            "\n╔══════════════════════════════════════════════════════╗\n"
        ));
        r.push_str(&format!("║  🔬 CTVP: {:<42}║\n", self.name));
        r.push_str(&format!(
            "╠══════════════════════════════════════════════════════╣\n"
        ));
        for e in &self.phases {
            let name = match e.phase {
                Phase::Preclinical => "P0 Mechanism",
                Phase::Safety => "P1 Safety",
                Phase::Efficacy => "P2 Efficacy",
                Phase::Confirmation => "P3 Confirm",
                Phase::Surveillance => "P4 Surveil",
            };
            r.push_str(&format!(
                "║  {} {:<12} {:<30}║\n",
                e.quality.emoji(),
                name,
                format!("{:?}", e.quality)
            ));
        }
        r.push_str(&format!(
            "╠══════════════════════════════════════════════════════╣\n"
        ));
        r.push_str(&format!(
            "║  Evidence stops at: Phase {}                          ║\n",
            self.evidence_stops_at as u8
        ));
        r.push_str(&format!(
            "║  Status: {} {:?}                               ║\n",
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
    }

    #[test]
    fn test_evidence_quality_ordering() {
        assert!(EvidenceQuality::None < EvidenceQuality::Weak);
        assert!(EvidenceQuality::Weak < EvidenceQuality::Moderate);
    }
}
