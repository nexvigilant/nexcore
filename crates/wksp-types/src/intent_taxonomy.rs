//! # Intent Taxonomy and Governance
//!
//! Formal definition of all NLI user intents for pharmacovigilance.
//! This file acts as the source-of-truth for intent classification and
//! threshold governance (0.85 confidence gate).

use serde::{Deserialize, Serialize};

/// Confidence gate for intent dispatch.
pub const INTENT_CONFIDENCE_THRESHOLD: f32 = 0.85;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentType {
    /// Querying a safety signal (e.g., "Any new cases for drug X?")
    SignalQuery,
    /// Searching for case records (e.g., "Find case 123-ABC")
    CaseSearch,
    /// Triggering regulatory report generation (e.g., "Start PSUR for drug Y")
    ReportGenerate,
    /// Lookup in regulatory/compliance guidelines
    RegulatoryLookup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDefinition {
    pub intent: IntentType,
    pub min_confidence: f32,
    pub requires_context: bool,
}

impl Default for IntentDefinition {
    fn default() -> Self {
        Self {
            intent: IntentType::SignalQuery,
            min_confidence: INTENT_CONFIDENCE_THRESHOLD,
            requires_context: true,
        }
    }
}
