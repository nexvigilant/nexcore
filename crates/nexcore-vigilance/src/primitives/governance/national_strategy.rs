//! # National Strategy for Vigilance (NEX-STRAT-001)
//!
//! The supreme strategic directive for the NexVigilant Union.
//! This module codifies the Winning Aspiration, Capabilities, and
//! Management Systems required to fulfill the CEO's vision.

use crate::primitives::governance::Verdict;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: NationalStrategy - The "Playing to Win" framework as code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalStrategy {
    pub winning_aspiration: String,
    pub primary_conquest: String, // e.g., "Pharmacovigilance vs FDA/Pharma"
    pub current_strategic_focus: StrategicFocus,
    pub rd_objectives: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategicFocus {
    /// Phase 0: Self-Governance by Code.
    ConstitutionalGrounding,
    /// Phase 1: Signal Dominance (PV).
    SignalArbitrage,
    /// Phase 2: Market Integration.
    EconomicSovereignty,
    /// Phase 3: Global Scale.
    UniversalVigilance,
}

impl NationalStrategy {
    pub fn new_initial() -> Self {
        Self {
            winning_aspiration: "Provide a decentralized, high-fidelity alternative to FDA signal detection that converts risk into market value.".into(),
            primary_conquest: "Pharmacovigilance (Competitive with Pharmaceutical Industry)".into(),
            current_strategic_focus: StrategicFocus::ConstitutionalGrounding,
            rd_objectives: vec![
                "Send R&D Team to Ferrostack to enable Rust-native UI patterns for Union Governance.".into()
            ],
        }
    }

    /// Check if an Action aligns with the National Strategy.
    pub fn verify_alignment(&self, action_description: &str) -> Verdict {
        // High-level alignment check
        if action_description.contains("PV") || action_description.contains("Signal") {
            Verdict::Permitted
        } else {
            Verdict::Flagged
        }
    }
}

/// T3: DepartmentalMandate - Specific goals assigned to Cabinet Secretaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentalMandate {
    pub department_id: String,
    pub primary_objective: String,
    pub kpi_confidence_threshold: Confidence,
}
