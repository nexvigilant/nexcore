//! Canonical vocabulary registry — one word, one definition.
//!
//! Resolves the F2 equivocation fallacy: the same bare name used for
//! fundamentally different semantics across crates.  Each `VocabEntry`
//! pins a symbol to its calculated definition, grounding primitives,
//! and the overloaded names it replaces.
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_lex_primitiva::vocabulary::{lookup, THREAT_LEVEL};
//!
//! let entry = lookup("ThreatLevel");
//! assert!(entry.is_some());
//! assert_eq!(THREAT_LEVEL.symbol, "ThreatLevel");
//! ```

#![forbid(unsafe_code)]

// ═══════════════════════════════════════════════════════════════════════════
// VOCABULARY ENTRY
// ═══════════════════════════════════════════════════════════════════════════

/// Canonical vocabulary entry — one word, one definition.
///
/// Tier: T2-C (κ + π — comparison of meanings, persisted definition)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct VocabEntry {
    /// Canonical symbol name (e.g., "ThreatLevel").
    pub symbol: &'static str,
    /// Calculated definition — precise, unique semantics.
    pub definition: &'static str,
    /// Lex Primitiva grounding symbols (e.g., `["κ", "∂"]`).
    pub primitives: &'static [&'static str],
    /// Overloaded names this symbol replaces (e.g., `["Severity@guardian-engine"]`).
    pub replaces: &'static [&'static str],
}

// ═══════════════════════════════════════════════════════════════════════════
// SEVERITY FAMILY (12 defs → 5 families)
// ═══════════════════════════════════════════════════════════════════════════

/// Ordinal escalation scale for real-time operational threat detection.
/// 5-level: Info→Critical.
pub const THREAT_LEVEL: VocabEntry = VocabEntry {
    symbol: "ThreatLevel",
    definition: "Ordinal escalation scale for real-time operational threat detection. 5-level: Info, Low, Medium, High, Critical.",
    primitives: &["κ", "∂"],
    replaces: &[
        "Severity@guardian-engine",
        "Severity@immunity",
        "Severity@cytokine",
        "Severity@vigilance/security",
    ],
};

/// Classification of validation/schema issue severity.
/// 3-level: Info→Error.
pub const DIAGNOSTIC_LEVEL: VocabEntry = VocabEntry {
    symbol: "DiagnosticLevel",
    definition: "Classification of validation or schema issue severity. 3-level: Info, Warning, Error.",
    primitives: &["κ", "∂"],
    replaces: &[
        "Severity@lex-primitiva",
        "Severity@transcriptase",
        "Severity@skill-hunter",
        "Severity@ctvp",
    ],
};

/// PV signal detection confidence from multi-metric concordance.
/// 5-level: None→Critical.
pub const SIGNAL_CONFIDENCE: VocabEntry = VocabEntry {
    symbol: "SignalConfidence",
    definition: "PV signal detection confidence from multi-metric concordance. 5-level: None, Low, Moderate, High, Critical.",
    primitives: &["N", "κ"],
    replaces: &["Severity@vigilance/control/pv"],
};

/// Quantitative 1–10 problem severity with blocking threshold at ≥7.
pub const IMPACT_MAGNITUDE: VocabEntry = VocabEntry {
    symbol: "ImpactMagnitude",
    definition: "Quantitative 1-10 problem severity with blocking threshold at >=7.",
    primitives: &["N"],
    replaces: &["Severity@hooks/state/problems"],
};

// ClinicalSeverity — already uniquely named `SeverityLevel` in pv-core. No entry needed.

// ═══════════════════════════════════════════════════════════════════════════
// PRIORITY FAMILY (5 defs → 2 families + 1 bug fix)
// ═══════════════════════════════════════════════════════════════════════════

/// Task/event processing urgency. Ascending: Low=0 < Critical=3.
pub const URGENCY: VocabEntry = VocabEntry {
    symbol: "Urgency",
    definition: "Task or event processing urgency. Ascending ordinal: Low=0, Normal=1, High=2, Critical=3.",
    primitives: &["κ", "N"],
    replaces: &[
        "Priority@hook-lib",
        "Priority@orchestration",
        "Priority@pvos",
        "Priority@vigil",
    ],
};

/// Strategic 6-tier risk classification (P0 Patient Safety → P5 Enhancement).
pub const RISK_TIER: VocabEntry = VocabEntry {
    symbol: "RiskTier",
    definition: "Strategic 6-tier risk classification. P0: Patient Safety, P1: Signal Integrity, P2: Data Quality, P3: Compliance, P4: Efficiency, P5: Enhancement.",
    primitives: &["κ", "∝"],
    replaces: &["Priority@value-mining"],
};

// ═══════════════════════════════════════════════════════════════════════════
// CONFIDENCE FAMILY (5 defs → 1 canonical)
// ═══════════════════════════════════════════════════════════════════════════

/// Bayesian probability in [0,1]. Clamped construction, multiplicative composition.
pub const CONFIDENCE: VocabEntry = VocabEntry {
    symbol: "Confidence",
    definition: "Bayesian probability in [0,1]. Clamped construction, multiplicative composition via product rule.",
    primitives: &["N", "∂"],
    replaces: &[
        "Confidence@dtree (duplicate)",
        "Confidence@jeopardy (duplicate)",
        "Confidence@pvos (duplicate)",
        "Confidence@hook-lib (hooks-workspace isolation)",
    ],
};

// ═══════════════════════════════════════════════════════════════════════════
// SIGNAL FAMILY (5 defs → 4 families)
// ═══════════════════════════════════════════════════════════════════════════

/// Detected threat pattern (PAMP/DAMP) with severity and confidence.
pub const THREAT_SIGNAL: VocabEntry = VocabEntry {
    symbol: "ThreatSignal",
    definition: "Detected threat pattern (PAMP/DAMP) with severity classification and confidence score.",
    primitives: &["∃", "∂", "κ"],
    replaces: &["Signal@guardian-engine", "Signal@hooks/homeostasis"],
};

/// Drug-event pair causality detection with strength and resonance scores.
pub const DRUG_EVENT_SIGNAL: VocabEntry = VocabEntry {
    symbol: "DrugEventSignal",
    definition: "Drug-event pair causality detection with disproportionality strength and resonance scores.",
    primitives: &["→", "N"],
    replaces: &["Signal@dna"],
};

/// Fire-and-forget telemetry emission with priority routing.
pub const TELEMETRY_SIGNAL: VocabEntry = VocabEntry {
    symbol: "TelemetrySignal",
    definition: "Fire-and-forget telemetry emission with priority-based routing.",
    primitives: &["∃", "σ"],
    replaces: &["Signal@hook-lib"],
};

/// Economic/social value detection with PV algorithm analogs.
pub const VALUE_SIGNAL: VocabEntry = VocabEntry {
    symbol: "ValueSignal",
    definition: "Economic or social value detection using PV algorithm analogs.",
    primitives: &["N", "ν"],
    replaces: &["Signal@value-mining"],
};

// ═══════════════════════════════════════════════════════════════════════════
// ACTION FAMILY (5 defs → 5 symbols)
// ═══════════════════════════════════════════════════════════════════════════

/// FRIDAY decision engine response (InvokeClaude, Escalate, etc).
pub const DECISION_ACTION: VocabEntry = VocabEntry {
    symbol: "DecisionAction",
    definition: "FRIDAY decision engine response action: InvokeClaude, QuickResponse, SilentLog, AutonomousAct, Escalate.",
    primitives: &["ς", "∂"],
    replaces: &["Action@vigil"],
};

/// Autonomous PV response (AutoAlert, signal escalation).
pub const PV_ACTION: VocabEntry = VocabEntry {
    symbol: "PvAction",
    definition: "Autonomous pharmacovigilance response action: auto-alert, signal escalation.",
    primitives: &["ς", "→"],
    replaces: &["Action@vigilance/avc"],
};

/// Network packet disposition (Allow/Drop).
pub const FIREWALL_RULE: VocabEntry = VocabEntry {
    symbol: "FirewallRule",
    definition: "Network packet disposition: Allow or Drop.",
    primitives: &["∂"],
    replaces: &["Action@network"],
};

/// File organization operation (Move/Archive/Delete).
pub const FILE_OP: VocabEntry = VocabEntry {
    symbol: "FileOp",
    definition: "File organization operation: Move, Archive, Delete.",
    primitives: &["σ"],
    replaces: &["Action@organize"],
};

/// Value signal response (Observe/Monitor/Act).
pub const RESPONSE_ACTION: VocabEntry = VocabEntry {
    symbol: "ResponseAction",
    definition: "Value signal response strategy: Observe, Monitor, Act.",
    primitives: &["ς", "κ"],
    replaces: &["Action@value-mining"],
};

// ═══════════════════════════════════════════════════════════════════════════
// VIOLATION FAMILY (6 defs → 5 symbols)
// ═══════════════════════════════════════════════════════════════════════════

/// Guideline requirement / conservation law violation.
pub const REGULATORY_VIOLATION: VocabEntry = VocabEntry {
    symbol: "RegulatoryViolation",
    definition: "Regulatory guideline requirement or conservation law violation.",
    primitives: &["∂", "→"],
    replaces: &["Violation@pv-core"],
};

/// Safety boundary crossed with actual vs threshold values.
pub const BOUNDARY_BREACH: VocabEntry = VocabEntry {
    symbol: "BoundaryBreach",
    definition: "Safety boundary crossed, with actual value vs threshold comparison.",
    primitives: &["∂", "N"],
    replaces: &["Violation@vigilance/safety"],
};

/// MCP tool execution violation with code and evidence.
pub const TOOL_VIOLATION: VocabEntry = VocabEntry {
    symbol: "ToolViolation",
    definition: "MCP tool execution violation with error code and evidence payload.",
    primitives: &["∂", "∃"],
    replaces: &["Violation@mcp", "Violation@docs-mcp"],
};

/// Code quality violation detected by hook with confidence.
pub const CODE_VIOLATION: VocabEntry = VocabEntry {
    symbol: "CodeViolation",
    definition: "Code quality violation detected by cognitive hook with confidence score.",
    primitives: &["∂", "κ"],
    replaces: &["Violation@hook-lib"],
};

/// Data schema assertion failure with field reference.
pub const SCHEMA_VIOLATION: VocabEntry = VocabEntry {
    symbol: "SchemaViolation",
    definition: "Data schema assertion failure with field path reference.",
    primitives: &["∂", "κ"],
    replaces: &["Violation@transcriptase"],
};

// ═══════════════════════════════════════════════════════════════════════════
// TIER (9 defs → 1 canonical)
// ═══════════════════════════════════════════════════════════════════════════

/// T1/T2P/T2C/T3 classification. Canonical source: nexcore-lex-primitiva::Tier.
pub const TIER: VocabEntry = VocabEntry {
    symbol: "Tier",
    definition: "Type complexity classification: T1 Universal, T2-P Cross-domain Primitive, T2-C Cross-domain Composite, T3 Domain-specific.",
    primitives: &["κ"],
    replaces: &["Tier (all duplicates consolidated to nexcore-lex-primitiva::Tier)"],
};

// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY
// ═══════════════════════════════════════════════════════════════════════════

/// All canonical vocabulary entries.
pub const ALL_ENTRIES: &[&VocabEntry] = &[
    // Severity family
    &THREAT_LEVEL,
    &DIAGNOSTIC_LEVEL,
    &SIGNAL_CONFIDENCE,
    &IMPACT_MAGNITUDE,
    // Priority family
    &URGENCY,
    &RISK_TIER,
    // Confidence
    &CONFIDENCE,
    // Signal family
    &THREAT_SIGNAL,
    &DRUG_EVENT_SIGNAL,
    &TELEMETRY_SIGNAL,
    &VALUE_SIGNAL,
    // Action family
    &DECISION_ACTION,
    &PV_ACTION,
    &FIREWALL_RULE,
    &FILE_OP,
    &RESPONSE_ACTION,
    // Violation family
    &REGULATORY_VIOLATION,
    &BOUNDARY_BREACH,
    &TOOL_VIOLATION,
    &CODE_VIOLATION,
    &SCHEMA_VIOLATION,
    // Tier
    &TIER,
];

/// Look up a vocabulary entry by symbol name.
///
/// Case-sensitive exact match.
#[must_use]
pub fn lookup(symbol: &str) -> Option<&'static VocabEntry> {
    ALL_ENTRIES.iter().find(|e| e.symbol == symbol).copied()
}

/// Find all entries that replace a given overloaded name.
///
/// Searches the `replaces` field. Partial match on the name portion
/// (before the `@` crate locator).
#[must_use]
pub fn find_replacements(old_name: &str) -> Vec<&'static VocabEntry> {
    ALL_ENTRIES
        .iter()
        .filter(|e| {
            e.replaces
                .iter()
                .any(|r| r.split('@').next().is_some_and(|n| n == old_name))
        })
        .copied()
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_lookup_existing() {
        assert_eq!(lookup("ThreatLevel"), Some(&THREAT_LEVEL));
        assert_eq!(lookup("Urgency"), Some(&URGENCY));
        assert_eq!(lookup("Confidence"), Some(&CONFIDENCE));
    }

    #[test]
    fn test_lookup_missing() {
        assert_eq!(lookup("NonExistent"), None);
        assert_eq!(lookup(""), None);
    }

    #[test]
    fn test_all_symbols_unique() {
        let symbols: BTreeSet<_> = ALL_ENTRIES.iter().map(|e| e.symbol).collect();
        assert_eq!(
            symbols.len(),
            ALL_ENTRIES.len(),
            "Duplicate symbols in vocabulary"
        );
    }

    #[test]
    fn test_all_entries_have_primitives() {
        for entry in ALL_ENTRIES {
            assert!(
                !entry.primitives.is_empty(),
                "{} has no primitives",
                entry.symbol
            );
        }
    }

    #[test]
    fn test_all_entries_have_definition() {
        for entry in ALL_ENTRIES {
            assert!(
                !entry.definition.is_empty(),
                "{} has empty definition",
                entry.symbol
            );
        }
    }

    #[test]
    fn test_all_entries_have_replaces() {
        for entry in ALL_ENTRIES {
            assert!(
                !entry.replaces.is_empty(),
                "{} replaces nothing",
                entry.symbol
            );
        }
    }

    #[test]
    fn test_find_replacements_severity() {
        let replacements = find_replacements("Severity");
        assert!(
            replacements.len() >= 4,
            "Expected at least 4 Severity families"
        );
    }

    #[test]
    fn test_find_replacements_priority() {
        let replacements = find_replacements("Priority");
        // Urgency replaces Priority@hook-lib, orchestration, pvos, vigil
        assert!(replacements.iter().any(|e| e.symbol == "Urgency"));
    }

    #[test]
    fn test_find_replacements_none() {
        let replacements = find_replacements("NoSuchType");
        assert!(replacements.is_empty());
    }

    #[test]
    fn test_entry_count() {
        // 4 severity + 2 priority + 1 confidence + 4 signal + 5 action + 5 violation + 1 tier = 22
        assert_eq!(ALL_ENTRIES.len(), 22);
    }
}
