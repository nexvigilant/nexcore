//! Chomsky grammar classification for PV subsystems.
//!
//! Each PV subsystem maps to a minimum Chomsky level based on its
//! computational requirements. The WHO definition's 4 verbs ascend
//! the hierarchy: Detection(3) → Assessment(2) → Understanding(1) → Prevention(0).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Chomsky hierarchy level for a PV subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChomskyLevel {
    /// Type-3: Regular grammar. Finite automaton. Flat pipelines, threshold gates.
    Type3Regular,
    /// Type-2: Context-free grammar. Pushdown automaton. Recursive parsers, trees.
    Type2ContextFree,
    /// Type-1: Context-sensitive grammar. Linear bounded automaton. Validators.
    Type1ContextSensitive,
    /// Type-0: Unrestricted grammar. Turing machine. Full computation.
    Type0Unrestricted,
}

impl ChomskyLevel {
    /// Computational model name.
    #[must_use]
    pub const fn automaton(&self) -> &'static str {
        match self {
            Self::Type3Regular => "Finite Automaton",
            Self::Type2ContextFree => "Pushdown Automaton",
            Self::Type1ContextSensitive => "Linear Bounded Automaton",
            Self::Type0Unrestricted => "Turing Machine",
        }
    }

    /// Generators required: σ,Σ → +ρ → +κ → +∃.
    #[must_use]
    pub const fn generator_count(&self) -> u8 {
        match self {
            Self::Type3Regular => 2,
            Self::Type2ContextFree => 3,
            Self::Type1ContextSensitive => 4,
            Self::Type0Unrestricted => 5,
        }
    }
}

impl fmt::Display for ChomskyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type3Regular => write!(f, "Type-3 (Regular)"),
            Self::Type2ContextFree => write!(f, "Type-2 (Context-Free)"),
            Self::Type1ContextSensitive => write!(f, "Type-1 (Context-Sensitive)"),
            Self::Type0Unrestricted => write!(f, "Type-0 (Unrestricted)"),
        }
    }
}

/// A PV subsystem with its Chomsky classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvSubsystem {
    /// Data entry: fixed fields, linear sequence.
    CaseIntake,
    /// Hierarchical terminology: SOC > HLGT > HLT > PT > LLT.
    MeddraCoding,
    /// PRR/ROR/IC/EBGM: arithmetic on contingency tables.
    DisproportionalityAnalysis,
    /// Branching decision tree with nested conditionals.
    NaranjoCausality,
    /// 9 criteria weighted by context of all others.
    BradfordHillEvaluation,
    /// State machine with context-dependent transitions.
    SignalManagement,
    /// Human judgment on existence of new evidence and value tradeoffs.
    BenefitRiskEvaluation,
    /// Cross-referencing content between sections.
    PbrerAuthoring,
    /// Political, ethical, economic factors. Non-algorithmic.
    RegulatoryDecision,
    /// Well-formed nested XML with schema validation.
    E2bXmlTransmission,
}

impl PvSubsystem {
    /// All 10 subsystems.
    pub const ALL: &'static [Self] = &[
        Self::CaseIntake,
        Self::MeddraCoding,
        Self::DisproportionalityAnalysis,
        Self::NaranjoCausality,
        Self::BradfordHillEvaluation,
        Self::SignalManagement,
        Self::BenefitRiskEvaluation,
        Self::PbrerAuthoring,
        Self::RegulatoryDecision,
        Self::E2bXmlTransmission,
    ];

    /// Minimum Chomsky grammar level for this subsystem.
    #[must_use]
    pub const fn chomsky_level(&self) -> ChomskyLevel {
        match self {
            // Type-3: flat pipelines, no recursion
            Self::CaseIntake | Self::DisproportionalityAnalysis => ChomskyLevel::Type3Regular,
            // Type-2: recursive structure (hierarchy, branching)
            Self::MeddraCoding | Self::NaranjoCausality | Self::E2bXmlTransmission => {
                ChomskyLevel::Type2ContextFree
            }
            // Type-1: context-dependent evaluation
            Self::BradfordHillEvaluation | Self::SignalManagement | Self::PbrerAuthoring => {
                ChomskyLevel::Type1ContextSensitive
            }
            // Type-0: requires human judgment / full computation
            Self::BenefitRiskEvaluation | Self::RegulatoryDecision => {
                ChomskyLevel::Type0Unrestricted
            }
        }
    }

    /// Human-readable justification for the classification.
    #[must_use]
    pub const fn justification(&self) -> &'static str {
        match self {
            Self::CaseIntake => {
                "Fixed-field forms, linear data sequence. No nesting or context dependency."
            }
            Self::MeddraCoding => {
                "5-level hierarchy (SOC→LLT). Recursive tree. Pushdown automaton required."
            }
            Self::DisproportionalityAnalysis => {
                "Pure arithmetic on 2x2 tables. No recursion. Finite automaton sufficient."
            }
            Self::NaranjoCausality => {
                "Branching decision tree with nested conditionals. Needs stack."
            }
            Self::BradfordHillEvaluation => {
                "9 criteria evaluated in context of each other. Weight of each depends on values of others."
            }
            Self::SignalManagement => {
                "State machine with context-dependent transitions: escalation depends on severity + novelty + volume."
            }
            Self::BenefitRiskEvaluation => {
                "Requires human judgment on existence of new evidence and value tradeoffs. Not computable."
            }
            Self::PbrerAuthoring => {
                "Content of each section depends on content of other sections. Cross-referencing required."
            }
            Self::RegulatoryDecision => {
                "Political, ethical, economic factors beyond algorithmic determination."
            }
            Self::E2bXmlTransmission => {
                "Well-formed nested XML with schema validation. Context-free grammar describes structure."
            }
        }
    }

    /// Which WHO definition verb this subsystem primarily serves.
    #[must_use]
    pub const fn who_pillar(&self) -> &'static str {
        match self {
            Self::CaseIntake | Self::DisproportionalityAnalysis => "Detection",
            Self::MeddraCoding | Self::NaranjoCausality => "Assessment",
            Self::BradfordHillEvaluation | Self::SignalManagement | Self::PbrerAuthoring => {
                "Understanding"
            }
            Self::BenefitRiskEvaluation | Self::RegulatoryDecision => "Prevention",
            Self::E2bXmlTransmission => "Detection",
        }
    }
}

impl fmt::Display for PvSubsystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} — {} ({})",
            self,
            self.chomsky_level(),
            self.chomsky_level().automaton()
        )
    }
}

/// WHO definition verb complexity — the 4 pillars ascend the Chomsky hierarchy.
///
/// Detection → Assessment → Understanding → Prevention
/// Type-3    → Type-2     → Type-1        → Type-0
#[must_use]
pub fn who_pillar_complexity() -> [(&'static str, ChomskyLevel); 4] {
    [
        ("Detection", ChomskyLevel::Type3Regular),
        ("Assessment", ChomskyLevel::Type2ContextFree),
        ("Understanding", ChomskyLevel::Type1ContextSensitive),
        ("Prevention", ChomskyLevel::Type0Unrestricted),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_subsystems() {
        assert_eq!(PvSubsystem::ALL.len(), 10);
    }

    #[test]
    fn case_intake_is_type3() {
        assert_eq!(
            PvSubsystem::CaseIntake.chomsky_level(),
            ChomskyLevel::Type3Regular
        );
    }

    #[test]
    fn bradford_hill_is_type1() {
        assert_eq!(
            PvSubsystem::BradfordHillEvaluation.chomsky_level(),
            ChomskyLevel::Type1ContextSensitive
        );
    }

    #[test]
    fn benefit_risk_is_type0() {
        assert_eq!(
            PvSubsystem::BenefitRiskEvaluation.chomsky_level(),
            ChomskyLevel::Type0Unrestricted
        );
    }

    #[test]
    fn levels_are_ordered() {
        assert!(ChomskyLevel::Type3Regular < ChomskyLevel::Type2ContextFree);
        assert!(ChomskyLevel::Type2ContextFree < ChomskyLevel::Type1ContextSensitive);
        assert!(ChomskyLevel::Type1ContextSensitive < ChomskyLevel::Type0Unrestricted);
    }

    #[test]
    fn who_pillars_ascend() {
        let pillars = who_pillar_complexity();
        for i in 0..3 {
            assert!(
                pillars[i].1 < pillars[i + 1].1,
                "{} should be less complex than {}",
                pillars[i].0,
                pillars[i + 1].0
            );
        }
    }

    #[test]
    fn all_have_justifications() {
        for s in PvSubsystem::ALL {
            assert!(
                !s.justification().is_empty(),
                "{:?} missing justification",
                s
            );
        }
    }

    #[test]
    fn display_includes_automaton() {
        let s = format!("{}", PvSubsystem::MeddraCoding);
        assert!(s.contains("Pushdown Automaton"));
    }
}
