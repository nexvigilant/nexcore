//! ICH Guidelines Module
//!
//! Provides O(1) lookup for ICH guideline metadata using PHF
//! (Perfect Hash Functions) for compile-time performance.

use crate::regulatory::ich_glossary::types::{Guideline, GuidelineStatus, IchCategory};
use phf::phf_map;

// ============================================================================
// Guideline Lookup Table (PHF - O(1) compile-time)
// ============================================================================

/// Compile-time guideline lookup table
/// Key: Guideline ID (lowercase, normalized)
pub static GUIDELINES: phf::Map<&'static str, Guideline> = phf_map! {
    // === EFFICACY GUIDELINES (E) ===
    "e2a" => Guideline {
        id: "E2A",
        title: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "1994-10-27",
        url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        term_count: 24,
        description: "Defines adverse event terminology and expedited reporting requirements",
    },
    "e2b(r2)" => Guideline {
        id: "E2B(R2)",
        title: "Maintenance of the ICH Guideline on Clinical Safety Data Management: Data Elements for Transmission of Individual Case Safety Reports",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2001-02-05",
        url: Some("https://database.ich.org/sites/default/files/E2B_R2__Guideline.pdf"),
        term_count: 12,
        description: "Data elements for electronic ICSR transmission",
    },
    "e2b(r3)" => Guideline {
        id: "E2B(R3)",
        title: "Individual Case Safety Reports (ICSRs): Implementation Guide",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2025-07-18",
        url: Some("https://database.ich.org/sites/default/files/E2B_R3_IG_v5.03.pdf"),
        term_count: 45,
        description: "Implementation guide for E2B(R3) ICSR electronic transmission",
    },
    "e2c(r2)" => Guideline {
        id: "E2C(R2)",
        title: "Periodic Benefit-Risk Evaluation Report (PBRER)",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2012-12-17",
        url: Some("https://database.ich.org/sites/default/files/E2C_R2_Guideline.pdf"),
        term_count: 18,
        description: "Format and content for periodic safety update reports",
    },
    "e2d(r1)" => Guideline {
        id: "E2D(R1)",
        title: "Post-Approval Safety Data: Definitions and Standards for Management and Reporting of Individual Case Safety Reports",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2025-09-15",
        url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
        term_count: 32,
        description: "Post-marketing safety reporting requirements and definitions",
    },
    "e2f" => Guideline {
        id: "E2F",
        title: "Development Safety Update Report",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2010-08-17",
        url: Some("https://database.ich.org/sites/default/files/E2F_Guideline.pdf"),
        term_count: 15,
        description: "Annual safety reporting during clinical development",
    },
    "e5(r1)" => Guideline {
        id: "E5(R1)",
        title: "Ethnic Factors in the Acceptability of Foreign Clinical Data",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "1998-02-05",
        url: Some("https://database.ich.org/sites/default/files/E5_R1__Guideline.pdf"),
        term_count: 8,
        description: "Considerations for using foreign clinical data across ethnic populations",
    },
    "e6(r3)" => Guideline {
        id: "E6(R3)",
        title: "Guideline for Good Clinical Practice",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2025-01-06",
        url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        term_count: 89,
        description: "International ethical and scientific quality standard for clinical trials",
    },
    "e8(r1)" => Guideline {
        id: "E8(R1)",
        title: "General Considerations for Clinical Studies",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2021-10-06",
        url: Some("https://database.ich.org/sites/default/files/E8_R1_Guideline.pdf"),
        term_count: 22,
        description: "Overall principles for clinical study design and conduct",
    },
    "e9" => Guideline {
        id: "E9",
        title: "Statistical Principles for Clinical Trials",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "1998-02-05",
        url: Some("https://database.ich.org/sites/default/files/E9_Guideline.pdf"),
        term_count: 28,
        description: "Statistical methodology for clinical trial design and analysis",
    },
    "e9(r1)" => Guideline {
        id: "E9(R1)",
        title: "Addendum on Estimands and Sensitivity Analysis in Clinical Trials",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2019-11-20",
        url: Some("https://database.ich.org/sites/default/files/E9-R1_Step4_Guideline_2019_1203.pdf"),
        term_count: 18,
        description: "Framework for estimands and sensitivity analysis",
    },
    "e11a" => Guideline {
        id: "E11A",
        title: "Pediatric Extrapolation",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2024-08-21",
        url: Some("https://database.ich.org/sites/default/files/E11A_Step4_Guideline.pdf"),
        term_count: 14,
        description: "Framework for extrapolating adult efficacy data to pediatric populations",
    },
    "e15" => Guideline {
        id: "E15",
        title: "Definitions for Genomic Biomarkers, Pharmacogenomics, Pharmacogenetics, Genomic Data and Sample Coding Categories",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2007-11-01",
        url: Some("https://database.ich.org/sites/default/files/E15_Guideline.pdf"),
        term_count: 12,
        description: "Standardized terminology for pharmacogenomics",
    },
    "e19" => Guideline {
        id: "E19",
        title: "A Selective Approach to Safety Data Collection in Specific Late-Stage Pre-approval or Post-Approval Clinical Trials",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step4Final,
        date: "2022-09-27",
        url: Some("https://database.ich.org/sites/default/files/E19_Step4_Guideline_2022_0927.pdf"),
        term_count: 11,
        description: "Guidance on selective safety data collection in late-stage trials",
    },
    "e20" => Guideline {
        id: "E20",
        title: "Adaptive Designs for Clinical Trials",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step2Draft,
        date: "2025-06-25",
        url: None,
        term_count: 8,
        description: "Framework for adaptive clinical trial designs",
    },
    "e22" => Guideline {
        id: "E22",
        title: "Clinical Studies in Rare Diseases",
        category: IchCategory::Efficacy,
        status: GuidelineStatus::Step2Draft,
        date: "2025-11-19",
        url: None,
        term_count: 6,
        description: "Guidance for clinical development in rare diseases",
    },

    // === QUALITY GUIDELINES (Q) ===
    "q1a(r2)" => Guideline {
        id: "Q1A(R2)",
        title: "Stability Testing of New Drug Substances and Products",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2003-02-06",
        url: Some("https://database.ich.org/sites/default/files/Q1A%28R2%29%20Guideline.pdf"),
        term_count: 24,
        description: "Stability testing protocols for drug substances and products",
    },
    "q1 ewg" => Guideline {
        id: "Q1 EWG",
        title: "Stability Testing of Drug Substances and Drug Products",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step2Draft,
        date: "2025-04-11",
        url: None,
        term_count: 18,
        description: "Updated stability testing framework (draft revision)",
    },
    "q2(r2)" => Guideline {
        id: "Q2(R2)",
        title: "Validation of Analytical Procedures",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2023-11-01",
        url: Some("https://database.ich.org/sites/default/files/ICH_Q2%28R2%29_Guideline_Step4.pdf"),
        term_count: 22,
        description: "Validation methodology for analytical procedures",
    },
    "q3a(r2)" => Guideline {
        id: "Q3A(R2)",
        title: "Impurities in New Drug Substances",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2006-10-25",
        url: Some("https://database.ich.org/sites/default/files/Q3A%28R2%29%20Guideline.pdf"),
        term_count: 16,
        description: "Impurity qualification and reporting thresholds",
    },
    "q3d(r2)" => Guideline {
        id: "Q3D(R2)",
        title: "Guideline for Elemental Impurities",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2022-04-26",
        url: Some("https://database.ich.org/sites/default/files/Q3D-R2_Guideline_Step4_2022_0426.pdf"),
        term_count: 28,
        description: "Control of elemental impurities in drug products",
    },
    "q3e ewg" => Guideline {
        id: "Q3E EWG",
        title: "Guideline for Extractables and Leachables",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step2Draft,
        date: "2025-08-01",
        url: None,
        term_count: 12,
        description: "Control of extractables and leachables from packaging",
    },
    "q5a(r2)" => Guideline {
        id: "Q5A(R2)",
        title: "Viral Safety Evaluation of Biotechnology Products Derived from Cell Lines of Human or Animal Origin",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2023-11-01",
        url: Some("https://database.ich.org/sites/default/files/Q5A%28R2%29_Guideline.pdf"),
        term_count: 38,
        description: "Viral safety for biotechnology products",
    },
    "q6a" => Guideline {
        id: "Q6A",
        title: "Specifications: Test Procedures and Acceptance Criteria for New Drug Substances and New Drug Products: Chemical Substances",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "1999-10-06",
        url: Some("https://database.ich.org/sites/default/files/Q6A%20Guideline.pdf"),
        term_count: 28,
        description: "Specification setting for chemical drug substances/products",
    },
    "q6b" => Guideline {
        id: "Q6B",
        title: "Specifications: Test Procedures and Acceptance Criteria for Biotechnological/Biological Products",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "1999-03-10",
        url: Some("https://database.ich.org/sites/default/files/Q6B%20Guideline.pdf"),
        term_count: 19,
        description: "Specification setting for biologics",
    },
    "q7" => Guideline {
        id: "Q7",
        title: "Good Manufacturing Practice Guide for Active Pharmaceutical Ingredients",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2000-11-10",
        url: Some("https://database.ich.org/sites/default/files/Q7%20Guideline.pdf"),
        term_count: 55,
        description: "GMP requirements for API manufacturing",
    },
    "q8(r2)" => Guideline {
        id: "Q8(R2)",
        title: "Pharmaceutical Development",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2009-08-01",
        url: Some("https://database.ich.org/sites/default/files/Q8%28R2%29%20Guideline.pdf"),
        term_count: 17,
        description: "Quality by Design principles for pharmaceutical development",
    },
    "q9(r1)" => Guideline {
        id: "Q9(R1)",
        title: "Quality Risk Management",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2023-01-18",
        url: Some("https://database.ich.org/sites/default/files/ICH_Q9%28R1%29_Guideline.pdf"),
        term_count: 32,
        description: "Risk management principles for pharmaceutical quality",
    },
    "q10" => Guideline {
        id: "Q10",
        title: "Pharmaceutical Quality System",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2008-06-04",
        url: Some("https://database.ich.org/sites/default/files/Q10%20Guideline.pdf"),
        term_count: 15,
        description: "Quality management system model for pharmaceuticals",
    },
    "q11" => Guideline {
        id: "Q11",
        title: "Development and Manufacture of Drug Substances",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2012-05-01",
        url: Some("https://database.ich.org/sites/default/files/Q11%20Guideline.pdf"),
        term_count: 18,
        description: "Drug substance development and manufacturing",
    },
    "q13" => Guideline {
        id: "Q13",
        title: "Continuous Manufacturing of Drug Substances and Drug Products",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2022-11-16",
        url: Some("https://database.ich.org/sites/default/files/ICH_Q13_Step4_Guideline.pdf"),
        term_count: 21,
        description: "Framework for continuous manufacturing",
    },
    "q14" => Guideline {
        id: "Q14",
        title: "Analytical Procedure Development",
        category: IchCategory::Quality,
        status: GuidelineStatus::Step4Final,
        date: "2023-11-01",
        url: Some("https://database.ich.org/sites/default/files/ICH_Q14_Step4_Guideline.pdf"),
        term_count: 24,
        description: "Enhanced approach to analytical procedure development",
    },

    // === SAFETY GUIDELINES (S) ===
    "s2(r1)" => Guideline {
        id: "S2(R1)",
        title: "Guidance on Genotoxicity Testing and Data Interpretation for Pharmaceuticals Intended for Human Use",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "2011-11-09",
        url: Some("https://database.ich.org/sites/default/files/S2%28R1%29%20Guideline.pdf"),
        term_count: 33,
        description: "Genotoxicity testing strategy and interpretation",
    },
    "s3a" => Guideline {
        id: "S3A",
        title: "Note for Guidance on Toxicokinetics: The Assessment of Systemic Exposure in Toxicity Studies",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "1994-10-27",
        url: Some("https://database.ich.org/sites/default/files/S3A_Guideline.pdf"),
        term_count: 12,
        description: "Toxicokinetic assessment in safety studies",
    },
    "s5(r3)" => Guideline {
        id: "S5(R3)",
        title: "Detection of Toxicity to Reproduction for Human Pharmaceuticals",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "2020-02-18",
        url: Some("https://database.ich.org/sites/default/files/S5-R3_Step4_Guideline.pdf"),
        term_count: 12,
        description: "Reproductive toxicology testing requirements",
    },
    "s7a" => Guideline {
        id: "S7A",
        title: "Safety Pharmacology Studies for Human Pharmaceuticals",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "2000-11-08",
        url: Some("https://database.ich.org/sites/default/files/S7A_Guideline.pdf"),
        term_count: 3,
        description: "Core battery safety pharmacology studies",
    },
    "s9" => Guideline {
        id: "S9",
        title: "Nonclinical Evaluation for Anticancer Pharmaceuticals",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "2009-11-18",
        url: Some("https://database.ich.org/sites/default/files/S9_Guideline.pdf"),
        term_count: 2,
        description: "Nonclinical requirements for oncology drugs",
    },
    "s10" => Guideline {
        id: "S10",
        title: "Photosafety Evaluation of Pharmaceuticals",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "2013-11-13",
        url: Some("https://database.ich.org/sites/default/files/S10_Guideline.pdf"),
        term_count: 21,
        description: "Assessment of photosafety risks",
    },
    "s11" => Guideline {
        id: "S11",
        title: "Nonclinical Safety Testing in Support of Development of Paediatric Medicines",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "2020-04-14",
        url: Some("https://database.ich.org/sites/default/files/S11_Step4_FinalGuideline.pdf"),
        term_count: 9,
        description: "Pediatric nonclinical safety requirements",
    },
    "s12" => Guideline {
        id: "S12",
        title: "Nonclinical Biodistribution Considerations for Gene Therapy Products",
        category: IchCategory::Safety,
        status: GuidelineStatus::Step4Final,
        date: "2023-03-14",
        url: Some("https://database.ich.org/sites/default/files/ICH_S12_Step4_Guideline.pdf"),
        term_count: 11,
        description: "Biodistribution studies for gene therapy",
    },

    // === MULTIDISCIPLINARY GUIDELINES (M) ===
    "m2" => Guideline {
        id: "M2",
        title: "Electronic Standards for the Transfer of Regulatory Information",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::NotFormal,
        date: "2015-06-11",
        url: Some("https://database.ich.org/sites/default/files/M2_Glossary.pdf"),
        term_count: 42,
        description: "Electronic data standards and terminology",
    },
    "m4" => Guideline {
        id: "M4",
        title: "The Common Technical Document",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step4Final,
        date: "2004-06-10",
        url: Some("https://database.ich.org/sites/default/files/M4_Q%26As_R3_Q%26As.pdf"),
        term_count: 8,
        description: "Common format for regulatory submissions",
    },
    "m7(r2)" => Guideline {
        id: "M7(R2)",
        title: "Assessment and Control of DNA Reactive (Mutagenic) Impurities in Pharmaceuticals to Limit Potential Carcinogenic Risk",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step4Final,
        date: "2023-04-03",
        url: Some("https://database.ich.org/sites/default/files/M7%28R2%29_Guideline_Step4.pdf"),
        term_count: 18,
        description: "Mutagenic impurity assessment and control",
    },
    "m10" => Guideline {
        id: "M10",
        title: "Bioanalytical Method Validation and Study Sample Analysis",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step4Final,
        date: "2022-05-24",
        url: Some("https://database.ich.org/sites/default/files/M10_Guideline_Step4.pdf"),
        term_count: 35,
        description: "Validation of bioanalytical methods",
    },
    "m11" => Guideline {
        id: "M11",
        title: "Clinical Electronic Structured Harmonised Protocol (CeSHarP)",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step4Final,
        date: "2025-11-19",
        url: Some("https://database.ich.org/sites/default/files/M11_Template_Step4.pdf"),
        term_count: 28,
        description: "Structured clinical protocol template",
    },
    "m12" => Guideline {
        id: "M12",
        title: "Drug Interaction Studies",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step4Final,
        date: "2024-05-21",
        url: Some("https://database.ich.org/sites/default/files/M12_Step4_Guideline.pdf"),
        term_count: 22,
        description: "Drug-drug interaction study design and evaluation",
    },
    "m13a" => Guideline {
        id: "M13A",
        title: "Bioequivalence for Immediate-Release Solid Oral Dosage Forms",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step4Final,
        date: "2024-07-23",
        url: Some("https://database.ich.org/sites/default/files/M13A_Step4_Guideline.pdf"),
        term_count: 16,
        description: "Bioequivalence study design for oral products",
    },
    "m14" => Guideline {
        id: "M14",
        title: "General Principles on Planning, Designing, Analysing, and Reporting of Non-interventional Studies That Utilise Real-World Data for Safety Assessment of Medicines",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step4Final,
        date: "2025-09-04",
        url: Some("https://database.ich.org/sites/default/files/M14_Step4_Guideline.pdf"),
        term_count: 24,
        description: "Non-interventional studies using real-world data",
    },
    "m15 ewg" => Guideline {
        id: "M15 EWG",
        title: "General Principles for Model-Informed Drug Development",
        category: IchCategory::Multidisciplinary,
        status: GuidelineStatus::Step2Draft,
        date: "2024-11-06",
        url: None,
        term_count: 12,
        description: "Framework for model-informed drug development",
    },
};

// ============================================================================
// Lookup Functions
// ============================================================================

/// O(1) lookup of guideline by ID (case-insensitive)
pub fn lookup_guideline(id: &str) -> Option<&'static Guideline> {
    // Single lowercase allocation, reused for both normalized and plain lookups
    let lower = id.to_ascii_lowercase();

    // Try direct lowercase lookup first (cheapest)
    if let Some(g) = GUIDELINES.get(&lower) {
        return Some(g);
    }

    // Try normalized (strip parens and hyphens)
    let normalized = lower.replace(['(', ')'], "").replace('-', "");
    if let Some(g) = GUIDELINES.get(&normalized) {
        return Some(g);
    }

    // Partial match fallback (O(n) but GUIDELINES is ~50 entries)
    GUIDELINES.entries().find_map(|(key, guideline)| {
        if key.starts_with(&normalized) || normalized.starts_with(key) {
            Some(guideline)
        } else {
            None
        }
    })
}

/// Get all guidelines
pub fn all_guidelines() -> Vec<&'static Guideline> {
    GUIDELINES.values().collect()
}

/// Get guidelines by category
pub fn guidelines_by_category(category: IchCategory) -> Vec<&'static Guideline> {
    GUIDELINES
        .values()
        .filter(|g| g.category == category)
        .collect()
}

/// Get guidelines by status
pub fn guidelines_by_status(status: GuidelineStatus) -> Vec<&'static Guideline> {
    GUIDELINES.values().filter(|g| g.status == status).collect()
}

/// Get active (non-withdrawn) guidelines
pub fn active_guidelines() -> Vec<&'static Guideline> {
    GUIDELINES
        .values()
        .filter(|g| g.status.is_active())
        .collect()
}

/// Count guidelines by category (single pass, no intermediate Vecs)
pub fn guideline_count_by_category() -> Vec<(IchCategory, usize)> {
    let mut counts = [0usize; 4];
    for g in GUIDELINES.values() {
        match g.category {
            IchCategory::Quality => counts[0] += 1,
            IchCategory::Safety => counts[1] += 1,
            IchCategory::Efficacy => counts[2] += 1,
            IchCategory::Multidisciplinary => counts[3] += 1,
        }
    }
    vec![
        (IchCategory::Quality, counts[0]),
        (IchCategory::Safety, counts[1]),
        (IchCategory::Efficacy, counts[2]),
        (IchCategory::Multidisciplinary, counts[3]),
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_guideline_exact() {
        let g = lookup_guideline("E2A").unwrap();
        assert_eq!(g.id, "E2A");
        assert_eq!(g.category, IchCategory::Efficacy);
    }

    #[test]
    fn test_lookup_guideline_case_insensitive() {
        let g1 = lookup_guideline("e2a").unwrap();
        let g2 = lookup_guideline("E2A").unwrap();
        assert_eq!(g1.id, g2.id);
    }

    #[test]
    fn test_lookup_guideline_with_revision() {
        let g = lookup_guideline("Q9(R1)").unwrap();
        assert_eq!(g.id, "Q9(R1)");
    }

    #[test]
    fn test_guidelines_by_category() {
        let quality = guidelines_by_category(IchCategory::Quality);
        assert!(!quality.is_empty());
        assert!(quality.iter().all(|g| g.category == IchCategory::Quality));
    }

    #[test]
    fn test_all_guidelines() {
        let all = all_guidelines();
        assert!(all.len() >= 30);
    }

    #[test]
    fn test_active_guidelines() {
        let active = active_guidelines();
        assert!(active.iter().all(|g| g.status.is_active()));
    }

    #[test]
    fn test_guideline_count_by_category() {
        let counts = guideline_count_by_category();
        assert_eq!(counts.len(), 4);
        let total: usize = counts.iter().map(|(_, c)| c).sum();
        assert!(total >= 30);
    }

    #[test]
    fn test_guideline_has_url() {
        let g = lookup_guideline("E6(R3)").unwrap();
        assert!(g.url.is_some());
    }
}
