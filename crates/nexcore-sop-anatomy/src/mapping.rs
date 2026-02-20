//! # SOP-Anatomy-Code Triple Mapping
//!
//! 18 SOP governance sections mapped through anatomical analogs to code structures.
//! The T1 primitive is the domain-invariant bridge across all three substrates.
//!
//! ## Primitive Foundation
//!
//! | Domain | Root Primitives |
//! |--------|----------------|
//! | SOP | σ (Sequence) + ∂ (Boundary) |
//! | Anatomy | σ (Sequence) + ∂ (Boundary) |
//! | Code | σ (Sequence) + ∂ (Boundary) |
//!
//! Same primitives, different substrates.

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

// ─── Enumerations ──────────────────────────────────────────────────────────

/// The 18 SOP governance sections.
///
/// Tier: T2-P | Dominant: Σ (Sum) — 18-variant alternation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SopSection {
    DocumentControl,
    VersionHistory,
    References,
    Definitions,
    PurposeScope,
    Roles,
    Procedure,
    DataClassification,
    RiskAssessment,
    IncidentEscalation,
    VendorManagement,
    Training,
    ChangeManagement,
    AuditCI,
    RollbackRecovery,
    Compliance,
    Appendices,
    ApprovalSignatures,
}

/// Priority tier for SOP sections.
///
/// Tier: T2-P | Dominant: κ (Comparison) — ordinal ranking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// 2x weight. SOP is non-compliant without these.
    Critical,
    /// 1x weight. Expected in all SOPs.
    Important,
    /// 1x weight. Required for healthcare SOPs.
    Healthcare,
    /// 1x weight. Required when regulatory training is mandated.
    Regulatory,
    /// 1x weight. Required for system/process SOPs.
    Operational,
    /// 1x weight. Always present, may be empty.
    Supporting,
}

/// Biological body system classification.
///
/// Tier: T2-P | Dominant: Σ (Sum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodySystem {
    Musculoskeletal,
    Developmental,
    Nervous,
    Immune,
    Endocrine,
    Digestive,
    Circulatory,
    Integumentary,
    Epigenetic,
    Metabolic,
}

/// The three domains in the triple mapping.
///
/// Tier: T2-P | Dominant: Σ (Sum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Sop,
    Anatomy,
    Code,
}

// ─── Data Structures ───────────────────────────────────────────────────────

/// Anatomical system analog for an SOP section.
///
/// Tier: T3 | Grounding: μ (Mapping) + κ (Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnatomicalAnalog {
    pub name: &'static str,
    pub function: &'static str,
    pub system: BodySystem,
}

/// Code structure equivalent for an SOP section.
///
/// Tier: T3 | Grounding: μ (Mapping) + ∂ (Boundary)
///
/// Serialize-only: constructed from static data, never deserialized.
#[derive(Debug, Clone, Serialize)]
pub struct CodeStructure {
    pub name: &'static str,
    pub pattern: &'static str,
    #[serde(serialize_with = "serialize_static_strs")]
    pub detection: &'static [&'static str],
}

/// Optional wiring to an existing nexcore bio-crate.
///
/// Tier: T3 | Grounding: μ (Mapping) + ∃ (Existence)
///
/// Serialize-only: constructed from static data, never deserialized.
#[derive(Debug, Clone, Serialize)]
pub struct BioCrateWiring {
    pub crate_name: &'static str,
    #[serde(serialize_with = "serialize_static_strs")]
    pub mcp_tools: &'static [&'static str],
}

/// Complete triple mapping for one SOP section.
///
/// Tier: T3 | Grounding: σ (Sequence) + μ (Mapping) — ordered cross-domain map
///
/// Serialize-only: constructed from static data, never deserialized.
#[derive(Debug, Clone, Serialize)]
pub struct SectionMapping {
    pub number: u8,
    pub name: &'static str,
    pub priority: Priority,
    #[serde(serialize_with = "serialize_primitives")]
    pub primitives: &'static [LexPrimitiva],
    pub anatomy: AnatomicalAnalog,
    pub code: CodeStructure,
    pub bio_wiring: Option<BioCrateWiring>,
}

/// Serialize `&'static [&'static str]` as a JSON array of strings.
fn serialize_static_strs<S: serde::Serializer>(
    val: &&'static [&'static str],
    ser: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = ser.serialize_seq(Some(val.len()))?;
    for item in *val {
        seq.serialize_element(item)?;
    }
    seq.end()
}

/// Serialize `&'static [LexPrimitiva]` as a JSON array.
fn serialize_primitives<S: serde::Serializer>(
    val: &&'static [LexPrimitiva],
    ser: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut seq = ser.serialize_seq(Some(val.len()))?;
    for item in *val {
        seq.serialize_element(item)?;
    }
    seq.end()
}

// ─── Static Mapping Table ──────────────────────────────────────────────────

impl SopSection {
    /// All 18 sections in order.
    pub const ALL: [SopSection; 18] = [
        Self::DocumentControl,
        Self::VersionHistory,
        Self::References,
        Self::Definitions,
        Self::PurposeScope,
        Self::Roles,
        Self::Procedure,
        Self::DataClassification,
        Self::RiskAssessment,
        Self::IncidentEscalation,
        Self::VendorManagement,
        Self::Training,
        Self::ChangeManagement,
        Self::AuditCI,
        Self::RollbackRecovery,
        Self::Compliance,
        Self::Appendices,
        Self::ApprovalSignatures,
    ];

    /// Section number (1-18).
    pub fn number(self) -> u8 {
        match self {
            Self::DocumentControl => 1,
            Self::VersionHistory => 2,
            Self::References => 3,
            Self::Definitions => 4,
            Self::PurposeScope => 5,
            Self::Roles => 6,
            Self::Procedure => 7,
            Self::DataClassification => 8,
            Self::RiskAssessment => 9,
            Self::IncidentEscalation => 10,
            Self::VendorManagement => 11,
            Self::Training => 12,
            Self::ChangeManagement => 13,
            Self::AuditCI => 14,
            Self::RollbackRecovery => 15,
            Self::Compliance => 16,
            Self::Appendices => 17,
            Self::ApprovalSignatures => 18,
        }
    }

    /// Look up section by number (1-18).
    pub fn from_number(n: u8) -> Option<Self> {
        Self::ALL.get(n.wrapping_sub(1) as usize).copied()
    }

    /// Look up section by name (case-insensitive, partial match).
    pub fn from_name(name: &str) -> Option<Self> {
        let lower = name.to_lowercase();
        Self::ALL
            .iter()
            .find(|s| s.mapping().name.to_lowercase().contains(&lower))
            .copied()
    }

    /// Priority tier.
    pub fn priority(self) -> Priority {
        self.mapping().priority
    }

    /// Scoring weight: Critical = 2, all others = 1.
    pub fn weight(self) -> u8 {
        match self.priority() {
            Priority::Critical => 2,
            _ => 1,
        }
    }

    /// Maximum possible score across all 18 sections.
    /// 7 critical x 2 + 11 standard x 1 = 25.
    pub fn max_score() -> u8 {
        Self::ALL.iter().map(|s| s.weight()).sum()
    }

    /// Get the full triple mapping for this section.
    pub fn mapping(self) -> SectionMapping {
        match self {
            Self::DocumentControl => SectionMapping {
                number: 1,
                name: "Document Control",
                priority: Priority::Critical,
                primitives: &[LexPrimitiva::Persistence],
                anatomy: AnatomicalAnalog {
                    name: "Skeleton",
                    function: "Rigid framework holding identity and shape across time",
                    system: BodySystem::Musculoskeletal,
                },
                code: CodeStructure {
                    name: "Cargo.toml / package.json",
                    pattern: "Project identity metadata — the structural frame of the codebase",
                    detection: &["Cargo.toml", "package.json", "pyproject.toml"],
                },
                bio_wiring: None,
            },
            Self::VersionHistory => SectionMapping {
                number: 2,
                name: "Version History",
                priority: Priority::Critical,
                primitives: &[LexPrimitiva::Sequence],
                anatomy: AnatomicalAnalog {
                    name: "Growth Rings",
                    function: "Sequential immutable record of every growth phase",
                    system: BodySystem::Developmental,
                },
                code: CodeStructure {
                    name: "git log / CHANGELOG.md",
                    pattern: "Ordered append-only history of state changes",
                    detection: &[".git", "CHANGELOG.md", "CHANGES.md"],
                },
                bio_wiring: None,
            },
            Self::References => SectionMapping {
                number: 3,
                name: "References & Linked Documents",
                priority: Priority::Important,
                primitives: &[LexPrimitiva::Mapping],
                anatomy: AnatomicalAnalog {
                    name: "Synaptic Connections",
                    function: "Axons linking one neuron to another across the nervous system",
                    system: BodySystem::Nervous,
                },
                code: CodeStructure {
                    name: "use / import / [dependencies]",
                    pattern: "External linkage declarations — cross-module references",
                    detection: &["Cargo.toml", "package.json"],
                },
                bio_wiring: None,
            },
            Self::Definitions => SectionMapping {
                number: 4,
                name: "Definitions & Acronyms",
                priority: Priority::Important,
                primitives: &[LexPrimitiva::Comparison],
                anatomy: AnatomicalAnalog {
                    name: "Broca's Area",
                    function: "Brain region standardizing language production and comprehension",
                    system: BodySystem::Nervous,
                },
                code: CodeStructure {
                    name: "type aliases / enum definitions / types.rs",
                    pattern: "Shared vocabulary ensuring equivalence across the codebase",
                    detection: &["types.rs", "types.ts", "types/"],
                },
                bio_wiring: None,
            },
            Self::PurposeScope => SectionMapping {
                number: 5,
                name: "Purpose & Scope",
                priority: Priority::Critical,
                primitives: &[LexPrimitiva::Boundary],
                anatomy: AnatomicalAnalog {
                    name: "Cell Membrane",
                    function: "Lipid bilayer defining inside vs outside of the cell",
                    system: BodySystem::Integumentary,
                },
                code: CodeStructure {
                    name: "pub / pub(crate) / mod visibility",
                    pattern: "Visibility modifiers defining the API surface boundary",
                    detection: &["lib.rs", "mod.rs", "index.ts"],
                },
                bio_wiring: None,
            },
            Self::Roles => SectionMapping {
                number: 6,
                name: "Roles & Responsibilities",
                priority: Priority::Critical,
                primitives: &[LexPrimitiva::Mapping],
                anatomy: AnatomicalAnalog {
                    name: "Organ Systems",
                    function: "Each organ has one job: heart pumps, lungs breathe, liver filters",
                    system: BodySystem::Circulatory,
                },
                code: CodeStructure {
                    name: "trait / interface definitions",
                    pattern: "Each type has defined capabilities and a single responsibility",
                    detection: &["traits.rs", "interfaces/", "*.trait.ts"],
                },
                bio_wiring: Some(BioCrateWiring {
                    crate_name: "nexcore-hormones",
                    mcp_tools: &["hormone_stimulus", "hormone_get", "hormone_status"],
                }),
            },
            Self::Procedure => SectionMapping {
                number: 7,
                name: "Procedure",
                priority: Priority::Critical,
                primitives: &[LexPrimitiva::Sequence, LexPrimitiva::Causality],
                anatomy: AnatomicalAnalog {
                    name: "Motor Cortex -> Spinal Cord -> Muscle",
                    function: "Ordered commands producing physical action via causal chain",
                    system: BodySystem::Nervous,
                },
                code: CodeStructure {
                    name: "fn main() / business logic",
                    pattern: "The actual algorithm steps — imperative execution path",
                    detection: &["main.rs", "lib.rs", "index.ts", "app.ts"],
                },
                bio_wiring: None,
            },
            Self::DataClassification => SectionMapping {
                number: 8,
                name: "Data Classification & Handling",
                priority: Priority::Healthcare,
                primitives: &[LexPrimitiva::Boundary, LexPrimitiva::State],
                anatomy: AnatomicalAnalog {
                    name: "Blood-Brain Barrier",
                    function: "Selective permeability based on molecular sensitivity level",
                    system: BodySystem::Nervous,
                },
                code: CodeStructure {
                    name: "Access modifiers + encryption layers",
                    pattern: "pub/private visibility + data sensitivity tiers (PHI/PII/Internal)",
                    detection: &["encryption", "crypto", "classify", "sensitivity"],
                },
                bio_wiring: None,
            },
            Self::RiskAssessment => SectionMapping {
                number: 9,
                name: "Risk Assessment",
                priority: Priority::Important,
                primitives: &[LexPrimitiva::Quantity, LexPrimitiva::Boundary],
                anatomy: AnatomicalAnalog {
                    name: "Immune Surveillance",
                    function: "T-cells scoring threat level with threshold-based response",
                    system: BodySystem::Immune,
                },
                code: CodeStructure {
                    name: "Result<T, E> + error taxonomy",
                    pattern: "Typed error variants with severity scoring and mitigation handlers",
                    detection: &["error.rs", "errors.rs", "errors/", "error.ts"],
                },
                bio_wiring: Some(BioCrateWiring {
                    crate_name: "nexcore-immunity",
                    mcp_tools: &["immunity_scan", "immunity_propose"],
                }),
            },
            Self::IncidentEscalation => SectionMapping {
                number: 10,
                name: "Incident Escalation",
                priority: Priority::Critical,
                primitives: &[LexPrimitiva::Causality, LexPrimitiva::Sequence],
                anatomy: AnatomicalAnalog {
                    name: "Nociception (Pain Reflex)",
                    function: "Rapid stimulus-to-response with escalating severity signals",
                    system: BodySystem::Nervous,
                },
                code: CodeStructure {
                    name: "? propagation + tracing levels",
                    pattern: "Error bubbling with severity-gated notification and logging",
                    detection: &["tracing", "log", "sentry", "alerting"],
                },
                bio_wiring: Some(BioCrateWiring {
                    crate_name: "nexcore-cytokine",
                    mcp_tools: &["cytokine_emit", "cytokine_status"],
                }),
            },
            Self::VendorManagement => SectionMapping {
                number: 11,
                name: "Vendor & Third-Party Management",
                priority: Priority::Important,
                primitives: &[LexPrimitiva::Existence, LexPrimitiva::Boundary],
                anatomy: AnatomicalAnalog {
                    name: "Gut Microbiome",
                    function: "External organisms managed within mucosal boundaries",
                    system: BodySystem::Digestive,
                },
                code: CodeStructure {
                    name: "[dependencies] with version constraints",
                    pattern: "Third-party crates pinned, audited, and SLA-bounded",
                    detection: &["Cargo.toml", "package.json", "Cargo.lock", "yarn.lock"],
                },
                bio_wiring: None,
            },
            Self::Training => SectionMapping {
                number: 12,
                name: "Training & Competency",
                priority: Priority::Regulatory,
                primitives: &[LexPrimitiva::Mapping, LexPrimitiva::Comparison],
                anatomy: AnatomicalAnalog {
                    name: "Cerebellum",
                    function: "Motor learning through repetition and competency assessment",
                    system: BodySystem::Nervous,
                },
                code: CodeStructure {
                    name: "/// doc comments + #[test] + /examples/",
                    pattern: "Documentation that teaches, tests that verify competency",
                    detection: &["examples/", "docs/", "README.md"],
                },
                bio_wiring: Some(BioCrateWiring {
                    crate_name: "nexcore-synapse",
                    mcp_tools: &["synapse_observe", "synapse_get"],
                }),
            },
            Self::ChangeManagement => SectionMapping {
                number: 13,
                name: "Change Management",
                priority: Priority::Important,
                primitives: &[LexPrimitiva::State, LexPrimitiva::Irreversibility],
                anatomy: AnatomicalAnalog {
                    name: "Metamorphosis",
                    function: "Controlled irreversible state change: caterpillar to butterfly",
                    system: BodySystem::Developmental,
                },
                code: CodeStructure {
                    name: "PR workflow + git merge",
                    pattern: "Branch, review, approve, merge — irreversible on main",
                    detection: &[".github/", "CONTRIBUTING.md", ".gitlab-ci.yml"],
                },
                bio_wiring: Some(BioCrateWiring {
                    crate_name: "nexcore-cytokine",
                    mcp_tools: &["cytokine_emit"],
                }),
            },
            Self::AuditCI => SectionMapping {
                number: 14,
                name: "Audit & Continuous Improvement",
                priority: Priority::Important,
                primitives: &[LexPrimitiva::Frequency, LexPrimitiva::Recursion],
                anatomy: AnatomicalAnalog {
                    name: "Circadian Rhythm / Heartbeat",
                    function: "Recurring cycles monitoring and maintaining homeostasis",
                    system: BodySystem::Circulatory,
                },
                code: CodeStructure {
                    name: "CI/CD pipeline",
                    pattern: "cargo clippy && cargo test on every push, scheduled security scans",
                    detection: &[
                        ".github/workflows/",
                        "Justfile",
                        ".gitlab-ci.yml",
                        "Makefile",
                    ],
                },
                bio_wiring: Some(BioCrateWiring {
                    crate_name: "nexcore-guardian",
                    mcp_tools: &["guardian_homeostasis_tick", "guardian_status"],
                }),
            },
            Self::RollbackRecovery => SectionMapping {
                number: 15,
                name: "Rollback & Recovery",
                priority: Priority::Operational,
                primitives: &[LexPrimitiva::Irreversibility, LexPrimitiva::State],
                anatomy: AnatomicalAnalog {
                    name: "Wound Healing / Liver Regeneration",
                    function: "Restoring tissue to prior functional state after damage",
                    system: BodySystem::Integumentary,
                },
                code: CodeStructure {
                    name: "git revert / migration down()",
                    pattern: "Backup restoration, feature flag kill-switches, rollback scripts",
                    detection: &["migrations/", "rollback", "backup"],
                },
                bio_wiring: None,
            },
            Self::Compliance => SectionMapping {
                number: 16,
                name: "Compliance & Regulatory",
                priority: Priority::Healthcare,
                primitives: &[LexPrimitiva::Boundary, LexPrimitiva::Sequence],
                anatomy: AnatomicalAnalog {
                    name: "Homeostasis",
                    function: "Hypothalamus maintaining temperature and pH within set-points",
                    system: BodySystem::Endocrine,
                },
                code: CodeStructure {
                    name: "#![deny(...)] lint rules",
                    pattern: "Enforced invariants at compile time — the regulatory gates",
                    detection: &["clippy.toml", "rustfmt.toml", ".eslintrc", "deny.toml"],
                },
                bio_wiring: None,
            },
            Self::Appendices => SectionMapping {
                number: 17,
                name: "Appendices",
                priority: Priority::Supporting,
                primitives: &[LexPrimitiva::Sum],
                anatomy: AnatomicalAnalog {
                    name: "Adipose Tissue",
                    function: "Supplementary energy reserves available when needed",
                    system: BodySystem::Metabolic,
                },
                code: CodeStructure {
                    name: "/docs/ /templates/ /examples/",
                    pattern: "Supplementary materials aggregated outside the core source",
                    detection: &["docs/", "templates/", "assets/"],
                },
                bio_wiring: None,
            },
            Self::ApprovalSignatures => SectionMapping {
                number: 18,
                name: "Approval Signatures",
                priority: Priority::Critical,
                primitives: &[LexPrimitiva::Irreversibility, LexPrimitiva::Existence],
                anatomy: AnatomicalAnalog {
                    name: "Epigenetic Methylation",
                    function: "Permanent marks attesting to cellular identity and lineage",
                    system: BodySystem::Epigenetic,
                },
                code: CodeStructure {
                    name: "git tag -s / GPG signatures",
                    pattern: "Cryptographic proof of authorship — immutable release attestation",
                    detection: &[".gnupg", "cosign", "CODEOWNERS"],
                },
                bio_wiring: None,
            },
        }
    }
}

impl std::fmt::Display for SopSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = self.mapping();
        write!(f, "S{}: {}", m.number, m.name)
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sop => write!(f, "SOP"),
            Self::Anatomy => write!(f, "Anatomy"),
            Self::Code => write!(f, "Code"),
        }
    }
}

impl Domain {
    /// Parse domain from string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sop" | "governance" | "procedure" => Some(Self::Sop),
            "anatomy" | "bio" | "biological" | "body" => Some(Self::Anatomy),
            "code" | "software" | "rust" | "structure" => Some(Self::Code),
            _ => None,
        }
    }
}

// ─── Coverage Report ───────────────────────────────────────────────────────

/// Coverage summary across all 18 sections.
///
/// Tier: T3 | Grounding: Σ (Sum) + N (Quantity)
#[derive(Debug, Serialize)]
pub struct CoverageReport {
    pub total_sections: u8,
    pub wired_sections: u8,
    pub unwired_sections: u8,
    pub max_score: u8,
    pub sections: Vec<SectionCoverage>,
}

/// Coverage status for one section.
#[derive(Debug, Serialize)]
pub struct SectionCoverage {
    pub number: u8,
    pub name: &'static str,
    pub priority: Priority,
    pub weight: u8,
    pub has_bio_wiring: bool,
    pub bio_crate: Option<&'static str>,
}

impl CoverageReport {
    /// Generate full coverage report.
    pub fn generate() -> Self {
        let sections: Vec<SectionCoverage> = SopSection::ALL
            .iter()
            .map(|s| {
                let m = s.mapping();
                SectionCoverage {
                    number: m.number,
                    name: m.name,
                    priority: m.priority,
                    weight: s.weight(),
                    has_bio_wiring: m.bio_wiring.is_some(),
                    bio_crate: m.bio_wiring.as_ref().map(|w| w.crate_name),
                }
            })
            .collect();

        let wired = sections.iter().filter(|s| s.has_bio_wiring).count() as u8;

        Self {
            total_sections: 18,
            wired_sections: wired,
            unwired_sections: 18 - wired,
            max_score: SopSection::max_score(),
            sections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_sections_numbered_1_to_18() {
        for (i, section) in SopSection::ALL.iter().enumerate() {
            assert_eq!(section.number(), (i + 1) as u8);
        }
    }

    #[test]
    fn from_number_roundtrips() {
        for section in &SopSection::ALL {
            let n = section.number();
            assert_eq!(SopSection::from_number(n), Some(*section));
        }
    }

    #[test]
    fn from_number_zero_returns_none() {
        assert_eq!(SopSection::from_number(0), None);
    }

    #[test]
    fn from_number_19_returns_none() {
        assert_eq!(SopSection::from_number(19), None);
    }

    #[test]
    fn max_score_is_25() {
        assert_eq!(SopSection::max_score(), 25);
    }

    #[test]
    fn critical_sections_count() {
        let critical_count = SopSection::ALL
            .iter()
            .filter(|s| s.priority() == Priority::Critical)
            .count();
        assert_eq!(critical_count, 7);
    }

    #[test]
    fn every_section_has_at_least_one_primitive() {
        for section in &SopSection::ALL {
            assert!(
                !section.mapping().primitives.is_empty(),
                "S{} has no primitives",
                section.number()
            );
        }
    }

    #[test]
    fn bio_wiring_count() {
        let wired = SopSection::ALL
            .iter()
            .filter(|s| s.mapping().bio_wiring.is_some())
            .count();
        assert_eq!(wired, 6);
    }

    #[test]
    fn from_name_case_insensitive() {
        assert_eq!(
            SopSection::from_name("document"),
            Some(SopSection::DocumentControl)
        );
        assert_eq!(
            SopSection::from_name("PROCEDURE"),
            Some(SopSection::Procedure)
        );
    }

    #[test]
    fn domain_from_str_loose() {
        assert_eq!(Domain::from_str_loose("sop"), Some(Domain::Sop));
        assert_eq!(Domain::from_str_loose("Bio"), Some(Domain::Anatomy));
        assert_eq!(Domain::from_str_loose("Rust"), Some(Domain::Code));
        assert_eq!(Domain::from_str_loose("unknown"), None);
    }

    #[test]
    fn coverage_report_generates() {
        let report = CoverageReport::generate();
        assert_eq!(report.total_sections, 18);
        assert_eq!(report.wired_sections, 6);
        assert_eq!(report.unwired_sections, 12);
        assert_eq!(report.max_score, 25);
    }
}
