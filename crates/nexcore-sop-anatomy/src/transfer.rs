//! # Capability Transfer Protocol
//!
//! Cross-domain translation between SOP, Anatomy, and Code using a 4-stage
//! pipeline: FISSION -> CHIRALITY -> FUSION -> TITRATION.
//!
//! Given a concept in one domain, decompose it to T1 primitives, verify the
//! mapping holds in the target domain, recombine, and calibrate.

use crate::mapping::{Domain, SopSection};
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

// ─── Transfer Result ───────────────────────────────────────────────────────

/// Result of a cross-domain capability transfer.
///
/// Tier: T3 | Grounding: μ (Mapping) + κ (Comparison)
#[derive(Debug, Clone, Serialize)]
pub struct TransferResult {
    pub source_domain: Domain,
    pub target_domain: Domain,
    pub source_concept: String,
    pub target_concept: String,
    pub primitives_used: Vec<LexPrimitiva>,
    pub confidence: f64,
    pub chirality_warnings: Vec<String>,
    pub stages: Vec<TransferStage>,
}

/// One stage of the transfer pipeline.
#[derive(Debug, Clone, Serialize)]
pub struct TransferStage {
    pub name: &'static str,
    pub description: String,
}

// ─── Transfer Pipeline ─────────────────────────────────────────────────────

/// Execute the Capability Transfer Protocol.
///
/// 4-stage pipeline:
/// 1. FISSION — decompose concept to T1 primitives
/// 2. CHIRALITY — verify each primitive maps correctly to target domain
/// 3. FUSION — recombine adapted primitives for target context
/// 4. TITRATION — calibrate for target domain scale
pub fn transfer(source: Domain, concept: &str, target: Domain) -> TransferResult {
    let mut stages = Vec::with_capacity(4);
    let mut warnings = Vec::new();

    // Stage 1: FISSION — find the SOP section(s) matching the concept
    let matched_section = find_section_for_concept(source, concept);
    let primitives = match &matched_section {
        Some(section) => section.mapping().primitives.to_vec(),
        None => {
            // No direct match — use heuristic primitive extraction
            vec![LexPrimitiva::Mapping]
        }
    };

    stages.push(TransferStage {
        name: "FISSION",
        description: format!(
            "Decomposed '{}' ({}) into {} primitive(s): [{}]",
            concept,
            source,
            primitives.len(),
            primitives
                .iter()
                .map(|p| format!("{p}"))
                .collect::<Vec<_>>()
                .join(", "),
        ),
    });

    // Stage 2: CHIRALITY — check each primitive maps correctly
    let mut chirality_ok = true;
    for prim in &primitives {
        if is_chiral_mismatch(prim, source, target) {
            warnings.push(format!(
                "Chirality warning: {} behaves differently in {} vs {}",
                prim, source, target,
            ));
            chirality_ok = false;
        }
    }

    stages.push(TransferStage {
        name: "CHIRALITY",
        description: if chirality_ok {
            format!("All primitives map cleanly from {} to {}", source, target)
        } else {
            format!(
                "{} chirality warning(s) detected — mirror-image behavior",
                warnings.len()
            )
        },
    });

    // Stage 3: FUSION — recombine for target domain
    let target_concept = match &matched_section {
        Some(section) => {
            let m = section.mapping();
            match target {
                Domain::Sop => format!("S{}: {}", m.number, m.name),
                Domain::Anatomy => m.anatomy.name.to_string(),
                Domain::Code => m.code.name.to_string(),
            }
        }
        None => format!("[unmapped: {} in {} domain]", concept, target),
    };

    stages.push(TransferStage {
        name: "FUSION",
        description: format!(
            "Recombined primitives into {} domain: '{}'",
            target, target_concept,
        ),
    });

    // Stage 4: TITRATION — calibrate confidence
    let base_confidence = if matched_section.is_some() {
        0.90
    } else {
        0.40
    };
    let chirality_penalty = warnings.len() as f64 * 0.05;
    let same_domain_bonus = if source == target { 0.05 } else { 0.0 };
    let confidence = (base_confidence - chirality_penalty + same_domain_bonus).clamp(0.0, 1.0);

    stages.push(TransferStage {
        name: "TITRATION",
        description: format!(
            "Calibrated confidence: {:.2} (base={:.2}, chirality_penalty={:.2}, same_domain={:.2})",
            confidence, base_confidence, chirality_penalty, same_domain_bonus,
        ),
    });

    TransferResult {
        source_domain: source,
        target_domain: target,
        source_concept: concept.to_string(),
        target_concept,
        primitives_used: primitives,
        confidence,
        chirality_warnings: warnings,
        stages,
    }
}

/// Find the SOP section best matching a concept in the given domain.
fn find_section_for_concept(domain: Domain, concept: &str) -> Option<SopSection> {
    let lower = concept.to_lowercase();

    SopSection::ALL
        .iter()
        .find(|section| {
            let m = section.mapping();
            match domain {
                Domain::Sop => {
                    m.name.to_lowercase().contains(&lower) || format!("s{}", m.number) == lower
                }
                Domain::Anatomy => {
                    m.anatomy.name.to_lowercase().contains(&lower)
                        || m.anatomy.function.to_lowercase().contains(&lower)
                }
                Domain::Code => {
                    m.code.name.to_lowercase().contains(&lower)
                        || m.code.pattern.to_lowercase().contains(&lower)
                }
            }
        })
        .copied()
}

/// Check if a primitive has chiral (mirror-image) behavior between two domains.
///
/// Some primitives behave subtly differently across domains:
/// - Persistence (π): SOP = document lifetime (months), Code = deployment lifetime (hours)
/// - Frequency (ν): SOP = review cadence (quarterly), Code = CI cadence (per-commit)
/// - Irreversibility (∝): SOP = approval is permanent, Code = git revert exists
fn is_chiral_mismatch(prim: &LexPrimitiva, source: Domain, target: Domain) -> bool {
    if source == target {
        return false;
    }

    // These primitives behave differently between SOP and Code domains
    // (regardless of transfer direction).
    let is_chiral_primitive = matches!(
        prim,
        LexPrimitiva::Persistence | LexPrimitiva::Frequency | LexPrimitiva::Irreversibility
    );

    let involves_sop_code = matches!(
        (source, target),
        (Domain::Sop, Domain::Code) | (Domain::Code, Domain::Sop)
    );

    is_chiral_primitive && involves_sop_code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_sop_to_code() {
        let result = transfer(Domain::Sop, "Document Control", Domain::Code);
        assert_eq!(result.source_domain, Domain::Sop);
        assert_eq!(result.target_domain, Domain::Code);
        assert!(!result.target_concept.is_empty());
        assert!(result.confidence > 0.5);
        assert_eq!(result.stages.len(), 4);
    }

    #[test]
    fn transfer_anatomy_to_sop() {
        let result = transfer(Domain::Anatomy, "Skeleton", Domain::Sop);
        assert_eq!(result.target_concept, "S1: Document Control");
        assert!(result.confidence > 0.8);
    }

    #[test]
    fn transfer_code_to_anatomy() {
        let result = transfer(Domain::Code, "Cargo.toml", Domain::Anatomy);
        assert_eq!(result.target_concept, "Skeleton");
    }

    #[test]
    fn chirality_warning_on_persistence() {
        let result = transfer(Domain::Sop, "Version History", Domain::Code);
        // Version History uses Sequence (σ), not Persistence — no chirality warning
        // But Approval Signatures uses Irreversibility — would warn
        let result2 = transfer(Domain::Sop, "Approval Signatures", Domain::Code);
        assert!(!result2.chirality_warnings.is_empty());
    }

    #[test]
    fn same_domain_transfer_no_warnings() {
        let result = transfer(Domain::Sop, "Procedure", Domain::Sop);
        assert!(result.chirality_warnings.is_empty());
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn unknown_concept_low_confidence() {
        let result = transfer(Domain::Sop, "nonexistent thing", Domain::Code);
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn four_stages_always_present() {
        let result = transfer(Domain::Code, "trait", Domain::Anatomy);
        assert_eq!(result.stages.len(), 4);
        assert_eq!(result.stages[0].name, "FISSION");
        assert_eq!(result.stages[1].name, "CHIRALITY");
        assert_eq!(result.stages[2].name, "FUSION");
        assert_eq!(result.stages[3].name, "TITRATION");
    }
}
