//! Transformation plan: the central compilation artifact.
//!
//! Compiles segmentation, annotation, and mapping into a complete
//! plan with per-paragraph rewrite instructions.

use crate::annotation::{ConceptAnnotation, annotate};
use crate::mapping::{MappingTable, build_mapping_table};
use crate::profile::{DomainProfile, RhetoricalRole};
use crate::segment::{SourceText, segment};
use serde::{Deserialize, Serialize};

/// Instruction for rewriting a single paragraph.
///
/// Tier: T2-C | Dominant: sigma (Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphInstruction {
    /// Index of the source paragraph.
    pub paragraph_index: usize,
    /// Original text.
    pub original_text: String,
    /// Concept replacements to apply: (source_term, target_term).
    pub replacements: Vec<(String, String)>,
    /// Assigned rhetorical role.
    pub rhetorical_role: RhetoricalRole,
    /// Guidance for the rewriter.
    pub guidance: String,
    /// Word count of original.
    pub original_word_count: usize,
}

/// The complete transformation plan — central artifact.
///
/// Contains everything needed to transform a text from one domain
/// to another: source text, annotations, mapping table, and
/// per-paragraph rewrite instructions.
///
/// Tier: T3 | Dominant: sigma (Sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationPlan {
    /// Unique plan identifier.
    pub plan_id: String,
    /// Source domain name.
    pub source_domain: String,
    /// Target profile name.
    pub target_profile: String,
    /// The segmented source text.
    pub source: SourceText,
    /// Per-paragraph concept annotations.
    pub annotations: Vec<ConceptAnnotation>,
    /// The concept mapping table.
    pub mapping_table: MappingTable,
    /// Per-paragraph rewrite instructions.
    pub instructions: Vec<ParagraphInstruction>,
}

impl TransformationPlan {
    /// Total number of concept replacements across all paragraphs.
    pub fn total_replacements(&self) -> usize {
        self.instructions.iter().map(|i| i.replacements.len()).sum()
    }

    /// Paragraphs that have zero replacements (pass-through).
    pub fn passthrough_count(&self) -> usize {
        self.instructions
            .iter()
            .filter(|i| i.replacements.is_empty())
            .count()
    }
}

/// Compile a full transformation plan.
///
/// Pipeline: segment -> annotate -> map -> generate instructions.
pub fn compile_plan(
    title: &str,
    text: &str,
    source_domain: &str,
    profile: &DomainProfile,
) -> TransformationPlan {
    // Step 1: Segment
    let source = segment(title, text);

    // Convert empty source_domain to None for bridge lookup
    let src_domain = if source_domain.is_empty() {
        None
    } else {
        Some(source_domain)
    };

    // Step 2: Annotate
    let annotations = annotate(&source, profile, src_domain);

    // Step 3: Map
    let mapping_table = build_mapping_table(&annotations, profile, src_domain);

    // Step 4: Generate instructions
    let instructions = generate_instructions(&source, &annotations, &mapping_table);

    // Generate plan ID from title hash
    let plan_id = format!(
        "plan-{:08x}",
        title
            .bytes()
            .chain(source_domain.as_bytes().iter().copied())
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32))
    );

    TransformationPlan {
        plan_id,
        source_domain: source_domain.to_string(),
        target_profile: profile.name.clone(),
        source,
        annotations,
        mapping_table,
        instructions,
    }
}

/// Generate per-paragraph instructions from annotations and mapping table.
fn generate_instructions(
    source: &SourceText,
    annotations: &[ConceptAnnotation],
    mapping_table: &MappingTable,
) -> Vec<ParagraphInstruction> {
    source
        .paragraphs
        .iter()
        .map(|para| {
            let ann = annotations.iter().find(|a| a.paragraph_index == para.index);

            let replacements = match ann {
                Some(a) => a
                    .concepts
                    .iter()
                    .filter_map(|c| {
                        mapping_table.get(&c.term).and_then(|m| {
                            if !m.target.is_empty() {
                                Some((m.source.clone(), m.target.clone()))
                            } else {
                                None
                            }
                        })
                    })
                    .collect(),
                None => Vec::new(),
            };

            let role = infer_rhetorical_role(para.index, source.paragraphs.len());

            ParagraphInstruction {
                paragraph_index: para.index,
                original_text: para.text.clone(),
                replacements,
                rhetorical_role: role,
                guidance: role.rewrite_guidance().to_string(),
                original_word_count: para.word_count,
            }
        })
        .collect()
}

/// Infer rhetorical role from position (heuristic baseline).
///
/// The LLM pipeline can override these with semantic analysis.
fn infer_rhetorical_role(index: usize, total: usize) -> RhetoricalRole {
    if total == 0 {
        return RhetoricalRole::Expository;
    }
    if total == 1 {
        return RhetoricalRole::Introduction;
    }

    let position = index as f64 / (total - 1) as f64;

    if index == 0 {
        RhetoricalRole::Introduction
    } else if index == 1 && total > 3 {
        RhetoricalRole::Thesis
    } else if index == total - 1 {
        RhetoricalRole::Conclusion
    } else if position < 0.3 {
        RhetoricalRole::Argument
    } else if position < 0.7 {
        RhetoricalRole::Argument
    } else {
        RhetoricalRole::Argument
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::builtin_pharmacovigilance;

    const SAMPLE_TEXT: &str = "\
To the People of the Safety Community, on the subject of vigilance:

After an unequivocal experience of the inefficacy of the subsisting system, \
you are called upon to deliberate on the importance of proper danger detection.

It has been observed that good citizen participation in reporting leads to \
better outcomes for liberty and justice.

In conclusion, the duty of every person in the union is to support the \
governance of our republic with vigilance and dedication.";

    #[test]
    fn test_compile_plan_basic() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Federalist No. 1", SAMPLE_TEXT, "political-philosophy", &pv);

        assert!(!plan.plan_id.is_empty());
        assert_eq!(plan.source_domain, "political-philosophy");
        assert_eq!(plan.target_profile, "pharmacovigilance");
        assert!(!plan.source.paragraphs.is_empty());
        assert_eq!(plan.instructions.len(), plan.source.paragraphs.len());
    }

    #[test]
    fn test_plan_has_replacements() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE_TEXT, "politics", &pv);
        assert!(plan.total_replacements() > 0);
    }

    #[test]
    fn test_plan_instructions_match_paragraphs() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE_TEXT, "politics", &pv);
        for (i, inst) in plan.instructions.iter().enumerate() {
            assert_eq!(inst.paragraph_index, i);
            assert!(!inst.original_text.is_empty());
            assert!(!inst.guidance.is_empty());
        }
    }

    #[test]
    fn test_rhetorical_roles_assigned() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Test", SAMPLE_TEXT, "politics", &pv);
        let first = &plan.instructions[0];
        assert_eq!(first.rhetorical_role, RhetoricalRole::Introduction);
        let last = plan
            .instructions
            .last()
            .unwrap_or_else(|| panic!("no instructions"));
        assert_eq!(last.rhetorical_role, RhetoricalRole::Conclusion);
    }

    #[test]
    fn test_plan_id_deterministic() {
        let pv = builtin_pharmacovigilance();
        let plan1 = compile_plan("Test", "Content.", "domain", &pv);
        let plan2 = compile_plan("Test", "Content.", "domain", &pv);
        assert_eq!(plan1.plan_id, plan2.plan_id);
    }

    #[test]
    fn test_plan_id_varies_with_input() {
        let pv = builtin_pharmacovigilance();
        let plan1 = compile_plan("Title A", "Content.", "domain", &pv);
        let plan2 = compile_plan("Title B", "Content.", "domain", &pv);
        assert_ne!(plan1.plan_id, plan2.plan_id);
    }

    #[test]
    fn test_empty_text_plan() {
        let pv = builtin_pharmacovigilance();
        let plan = compile_plan("Empty", "", "none", &pv);
        assert!(plan.source.paragraphs.is_empty());
        assert!(plan.instructions.is_empty());
        assert_eq!(plan.total_replacements(), 0);
    }

    #[test]
    fn test_passthrough_count() {
        let pv = builtin_pharmacovigilance();
        // This text has no political concepts — all paragraphs pass through
        let plan = compile_plan(
            "No Match",
            "The quick brown fox.\n\nJumped over the lazy dog.",
            "none",
            &pv,
        );
        assert_eq!(plan.passthrough_count(), 2);
    }

    #[test]
    fn test_infer_role_single_paragraph() {
        assert_eq!(infer_rhetorical_role(0, 1), RhetoricalRole::Introduction);
    }

    #[test]
    fn test_infer_role_empty() {
        assert_eq!(infer_rhetorical_role(0, 0), RhetoricalRole::Expository);
    }

    // ═══════════════════════════════════════════════════════════════
    // Source-aware bridge end-to-end tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_biochemistry_to_military_bridges_fire() {
        use crate::profile::builtin_military_defense;
        let mil = builtin_military_defense();
        let biochem_text = "\
The enzyme catalyzes the reaction through a specific pathway. \
The substrate binds to the active site while the inhibitor blocks it.

The cascade of events triggers feedback regulation. \
Kinetics determine the equilibrium of the metabolite concentration.

A cofactor enables activation of the downstream process.";

        let plan = compile_plan("Biochem Test", biochem_text, "biochemistry", &mil);

        // Should have >10 bridge hits from biochemistry-specific bridges
        assert!(
            plan.total_replacements() >= 10,
            "expected >=10 replacements, got {}",
            plan.total_replacements()
        );
        assert!(
            plan.mapping_table.aggregate_confidence > 0.75,
            "expected aggregate_confidence > 0.75, got {}",
            plan.mapping_table.aggregate_confidence
        );
    }

    #[test]
    fn test_plan_id_varies_with_source_domain() {
        let pv = builtin_pharmacovigilance();
        let plan1 = compile_plan("Same Title", "Content.", "biochemistry", &pv);
        let plan2 = compile_plan("Same Title", "Content.", "political-philosophy", &pv);
        assert_ne!(
            plan1.plan_id, plan2.plan_id,
            "plan_id should differ when source_domain differs"
        );
    }
}
