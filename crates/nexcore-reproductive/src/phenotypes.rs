//! # Somatic Tissue Phenotypes
//!
//! Differentiates generic subagents into specialized "Tissue" roles.

use serde::{Deserialize, Serialize};

/// Specialized agent roles based on biological tissues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TissuePhenotype {
    /// Nervous Tissue: Specialized in Hook infrastructure and signal conduction.
    Nervous,
    /// Immune Tissue: Specialized in safety audits and threat detection.
    Immune,
    /// Muscular Tissue: Specialized in high-compute refactoring and execution.
    Muscular,
    /// Germ Cell: Specialized in architectural preservation.
    Germ,
}

/// Parameters for a specialized subagent "Organ".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaticSpecialization {
    pub phenotype: TissuePhenotype,
    /// Specific tools this phenotype is allowed to use.
    pub tool_allowlist: Vec<String>,
    /// Target saturation capacity (Vsat).
    pub target_vsat: f64,
}

impl SomaticSpecialization {
    pub fn for_phenotype(phenotype: TissuePhenotype) -> Self {
        match phenotype {
            TissuePhenotype::Nervous => Self {
                phenotype,
                tool_allowlist: vec!["hooks_stats".to_string(), "hooks_for_event".to_string()],
                target_vsat: 0.9,
            },
            TissuePhenotype::Immune => Self {
                phenotype,
                tool_allowlist: vec!["immunity_scan".to_string(), "guardian_status".to_string()],
                target_vsat: 0.85,
            },
            TissuePhenotype::Muscular => Self {
                phenotype,
                tool_allowlist: vec!["run_shell_command".to_string(), "replace".to_string()],
                target_vsat: 0.95,
            },
            TissuePhenotype::Germ => Self {
                phenotype,
                tool_allowlist: vec!["lex_primitiva_list".to_string(), "validation_run".to_string()],
                target_vsat: 1.0,
            },
        }
    }
}
