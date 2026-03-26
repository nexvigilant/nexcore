// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Root Cause Diagnosis Engine
//!
//! Moves from observed symptoms to underlying organizational issues using
//! structured frameworks: 5 Whys decomposition, Ishikawa categorization,
//! and Theory of Constraints identification.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Causal chain (5 Whys) | Causality | → |
//! | Category comparison | Comparison | κ |
//! | Symptom inventory | State | ς |
//! | Void identification | Void | ∅ |
//! | Boundary of analysis | Boundary | ∂ |

use serde::{Deserialize, Serialize};
use std::fmt;

/// Ishikawa root cause categories (6M model adapted for software organizations).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IshikawaCategory {
    /// Skill gap, capacity, no owner, turnover
    People,
    /// Missing SOP, broken handoff, no quality gate
    Process,
    /// Missing tool, broken integration, capability gap
    Technology,
    /// Wrong priority, unclear direction, competing goals
    Strategy,
    /// External dependency, market timing, regulation
    Environment,
    /// Unmeasured, wrong metric, stale data
    Measurement,
}

impl fmt::Display for IshikawaCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::People => write!(f, "People"),
            Self::Process => write!(f, "Process"),
            Self::Technology => write!(f, "Technology"),
            Self::Strategy => write!(f, "Strategy"),
            Self::Environment => write!(f, "Environment"),
            Self::Measurement => write!(f, "Measurement"),
        }
    }
}

impl IshikawaCategory {
    /// All six categories.
    pub const ALL: [Self; 6] = [
        Self::People,
        Self::Process,
        Self::Technology,
        Self::Strategy,
        Self::Environment,
        Self::Measurement,
    ];

    /// Diagnostic question for this category.
    #[must_use]
    pub fn diagnostic_question(&self) -> &'static str {
        match self {
            Self::People => "Who should have done this and why didn't they?",
            Self::Process => "What workflow should have produced this and why doesn't it exist?",
            Self::Technology => "What capability is needed and why wasn't it built?",
            Self::Strategy => "Was this deprioritized? By what logic?",
            Self::Environment => "Is this blocked by something outside our control?",
            Self::Measurement => "Would we have caught this if we were measuring the right thing?",
        }
    }
}

/// An observed symptom — the starting point of diagnosis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symptom {
    /// What was observed
    pub description: String,
    /// Severity: 1 (low) to 5 (critical)
    pub severity: u8,
    /// Optional: which gate or system area
    pub area: Option<String>,
}

/// A single "Why" in the 5 Whys chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhyStep {
    /// The depth (1-5)
    pub depth: u8,
    /// The causal explanation at this depth
    pub explanation: String,
    /// Whether this step has been verified
    pub verified: bool,
}

/// A root cause identified through 5 Whys analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    /// The original symptom this root cause traces back to
    pub symptom_index: usize,
    /// The causal chain (5 Whys)
    pub why_chain: Vec<WhyStep>,
    /// Ishikawa category assignment
    pub category: IshikawaCategory,
    /// The root cause statement (deepest verified Why)
    pub statement: String,
    /// Confidence in this root cause (0.0-1.0)
    pub confidence: f64,
    /// Corrective action (stop the bleeding)
    pub corrective_action: String,
    /// Preventive action (ensure it can't recur)
    pub preventive_action: String,
}

/// The constraint — the single highest-leverage root cause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Which root cause is the constraint
    pub root_cause_index: usize,
    /// Leverage score: (downstream × severity) / effort
    pub leverage_score: f64,
    /// How many other root causes this unblocks
    pub downstream_count: usize,
    /// Immediate exploitation action
    pub exploit_action: String,
    /// Structural elevation action
    pub elevate_action: String,
}

/// Complete root cause diagnosis output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    /// Input symptoms
    pub symptoms: Vec<Symptom>,
    /// Identified root causes
    pub root_causes: Vec<RootCause>,
    /// Category distribution
    pub category_distribution: Vec<(IshikawaCategory, usize)>,
    /// The dominant category (most root causes)
    pub dominant_category: Option<IshikawaCategory>,
    /// The identified constraint
    pub constraint: Option<Constraint>,
    /// Conservation assessment
    pub conservation: ConservationAssessment,
}

/// Conservation law assessment of the diagnosis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationAssessment {
    /// ∅ — Are there voids (unmeasured, undefined things)?
    pub void_status: String,
    /// ς — Is state being tracked?
    pub state_status: String,
    /// ∂ — Are boundaries sharp?
    pub boundary_status: String,
    /// ∃ — Does the diagnosed output exist as a concrete artifact?
    pub existence_status: String,
}

/// Score a set of symptoms and root causes to identify the constraint.
///
/// Leverage = (downstream_gates_unblocked × severity) / effort_estimate
/// where effort_estimate is inverse of confidence (less confident = harder to fix).
#[must_use]
pub fn identify_constraint(root_causes: &[RootCause], symptoms: &[Symptom]) -> Option<Constraint> {
    if root_causes.is_empty() {
        return None;
    }

    let mut best_idx = 0;
    let mut best_leverage = 0.0_f64;

    for (i, rc) in root_causes.iter().enumerate() {
        // Count how many other root causes share a Why chain element with this one
        // (indicating downstream dependency)
        let downstream = root_causes
            .iter()
            .enumerate()
            .filter(|(j, other)| {
                *j != i
                    && other.why_chain.iter().any(|w| {
                        rc.why_chain
                            .iter()
                            .any(|rw| rw.explanation == w.explanation)
                    })
            })
            .count();

        let severity = symptoms
            .get(rc.symptom_index)
            .map_or(3.0, |s| f64::from(s.severity));

        // Effort is inverse of confidence — low confidence means harder to fix
        let effort = if rc.confidence > 0.0 {
            1.0 / rc.confidence
        } else {
            10.0
        };

        let leverage = (downstream as f64 + 1.0) * severity / effort;

        if leverage > best_leverage {
            best_leverage = leverage;
            best_idx = i;
        }
    }

    Some(Constraint {
        root_cause_index: best_idx,
        leverage_score: best_leverage,
        downstream_count: root_causes
            .iter()
            .enumerate()
            .filter(|(j, other)| {
                *j != best_idx
                    && other.why_chain.iter().any(|w| {
                        root_causes[best_idx]
                            .why_chain
                            .iter()
                            .any(|rw| rw.explanation == w.explanation)
                    })
            })
            .count(),
        exploit_action: root_causes[best_idx].corrective_action.clone(),
        elevate_action: root_causes[best_idx].preventive_action.clone(),
    })
}

/// Compute category distribution from root causes.
#[must_use]
pub fn category_distribution(root_causes: &[RootCause]) -> Vec<(IshikawaCategory, usize)> {
    let mut dist: Vec<(IshikawaCategory, usize)> = IshikawaCategory::ALL
        .iter()
        .map(|cat| {
            let count = root_causes.iter().filter(|rc| rc.category == *cat).count();
            (*cat, count)
        })
        .collect();

    // Sort by count descending
    dist.sort_by(|a, b| b.1.cmp(&a.1));
    dist
}

/// Find the dominant category (most root causes land in it).
#[must_use]
pub fn dominant_category(root_causes: &[RootCause]) -> Option<IshikawaCategory> {
    let dist = category_distribution(root_causes);
    dist.first()
        .filter(|(_, count)| *count > 0)
        .map(|(cat, _)| *cat)
}

/// Run a complete diagnosis from symptoms and root causes.
#[must_use]
pub fn diagnose(symptoms: Vec<Symptom>, root_causes: Vec<RootCause>) -> Diagnosis {
    let dist = category_distribution(&root_causes);
    let dominant = dominant_category(&root_causes);
    let constraint = identify_constraint(&root_causes, &symptoms);

    let void_count = root_causes
        .iter()
        .filter(|rc| rc.category == IshikawaCategory::Measurement)
        .count();
    let process_count = root_causes
        .iter()
        .filter(|rc| rc.category == IshikawaCategory::Process)
        .count();

    let conservation = ConservationAssessment {
        void_status: if void_count > 0 {
            format!("VOID — {void_count} measurement gaps identified")
        } else {
            "OK — all symptoms are measured".to_string()
        },
        state_status: if root_causes.iter().all(|rc| rc.confidence > 0.5) {
            "OK — root causes verified with >50% confidence".to_string()
        } else {
            "THIN — some root causes are hypothetical".to_string()
        },
        boundary_status: if process_count > 0 {
            format!("BROKEN — {process_count} process boundary failures")
        } else {
            "OK — process boundaries intact".to_string()
        },
        existence_status: if constraint.is_some() {
            "IDENTIFIED — constraint exists as actionable target".to_string()
        } else {
            "ABSENT — no constraint identified".to_string()
        },
    };

    Diagnosis {
        symptoms,
        root_causes,
        category_distribution: dist,
        dominant_category: dominant,
        constraint,
        conservation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_symptom(desc: &str, severity: u8) -> Symptom {
        Symptom {
            description: desc.to_string(),
            severity,
            area: None,
        }
    }

    fn make_root_cause(
        symptom_idx: usize,
        category: IshikawaCategory,
        statement: &str,
        confidence: f64,
    ) -> RootCause {
        RootCause {
            symptom_index: symptom_idx,
            why_chain: vec![
                WhyStep {
                    depth: 1,
                    explanation: "immediate cause".to_string(),
                    verified: true,
                },
                WhyStep {
                    depth: 2,
                    explanation: statement.to_string(),
                    verified: true,
                },
            ],
            category,
            statement: statement.to_string(),
            confidence,
            corrective_action: "fix it".to_string(),
            preventive_action: "prevent it".to_string(),
        }
    }

    #[test]
    fn empty_diagnosis() {
        let d = diagnose(vec![], vec![]);
        assert!(d.root_causes.is_empty());
        assert!(d.constraint.is_none());
        assert!(d.dominant_category.is_none());
    }

    #[test]
    fn single_root_cause() {
        let symptoms = vec![make_symptom("Academy agents unwired", 4)];
        let root_causes = vec![make_root_cause(
            0,
            IshikawaCategory::Process,
            "No SOP connects agent creation to tool wiring",
            0.85,
        )];
        let d = diagnose(symptoms, root_causes);

        assert_eq!(d.root_causes.len(), 1);
        assert_eq!(d.dominant_category, Some(IshikawaCategory::Process));
        assert!(d.constraint.is_some());
    }

    #[test]
    fn category_distribution_correct() {
        let rcs = vec![
            make_root_cause(0, IshikawaCategory::Process, "proc1", 0.9),
            make_root_cause(1, IshikawaCategory::Process, "proc2", 0.8),
            make_root_cause(2, IshikawaCategory::Technology, "tech1", 0.7),
        ];
        let dist = category_distribution(&rcs);
        // Process should be first with count 2
        assert_eq!(dist[0].0, IshikawaCategory::Process);
        assert_eq!(dist[0].1, 2);
    }

    #[test]
    fn constraint_prefers_high_leverage() {
        let symptoms = vec![
            make_symptom("symptom A", 5), // high severity
            make_symptom("symptom B", 1), // low severity
        ];
        let root_causes = vec![
            make_root_cause(0, IshikawaCategory::Process, "shared cause", 0.9),
            make_root_cause(1, IshikawaCategory::Technology, "shared cause", 0.5),
        ];
        let constraint = identify_constraint(&root_causes, &symptoms);
        assert!(constraint.is_some());
        let c = constraint.as_ref().unwrap();
        // Root cause 0 has higher leverage (severity 5, confidence 0.9)
        assert_eq!(c.root_cause_index, 0);
    }

    #[test]
    fn ishikawa_diagnostic_questions_exist() {
        for cat in IshikawaCategory::ALL {
            assert!(!cat.diagnostic_question().is_empty());
        }
    }

    #[test]
    fn conservation_detects_measurement_voids() {
        let symptoms = vec![make_symptom("unmeasured thing", 3)];
        let root_causes = vec![make_root_cause(
            0,
            IshikawaCategory::Measurement,
            "Never tracked",
            0.6,
        )];
        let d = diagnose(symptoms, root_causes);
        assert!(d.conservation.void_status.contains("VOID"));
    }

    #[test]
    fn conservation_detects_process_boundary_failures() {
        let symptoms = vec![make_symptom("broken handoff", 4)];
        let root_causes = vec![make_root_cause(
            0,
            IshikawaCategory::Process,
            "No handoff SOP",
            0.9,
        )];
        let d = diagnose(symptoms, root_causes);
        assert!(d.conservation.boundary_status.contains("BROKEN"));
    }
}
