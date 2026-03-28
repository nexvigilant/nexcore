//! Catalog data for Rheumatoid Arthritis.
//!
//! Sources: ACR 2021 Guidelines for RA, FDA drug labels, ORAL Surveillance trial,
//! Cochrane reviews on DMARDs, CORRONA registry data.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, DrugWithdrawal,
    Epidemiology, EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea,
    TreatmentLine, Trend, UnmetNeed,
};

/// Returns the canonical Rheumatoid Arthritis disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("ra"),
        name: "Rheumatoid Arthritis".to_string(),
        icd10_codes: vec!["M05".to_string(), "M06".to_string(), "M05.9".to_string()],
        therapeutic_area: TherapeuticArea::Immunology,
        epidemiology: Epidemiology {
            global_prevalence: Some(0.46),
            us_prevalence: Some(0.60),
            annual_incidence: Some(41.0),
            demographics: Demographics {
                median_age_onset: Some(50),
                sex_ratio: Some("3:1 F:M".to_string()),
                risk_factors: vec![
                    "HLA-DRB1 shared epitope alleles".to_string(),
                    "Tobacco smoking".to_string(),
                    "Female sex and hormonal factors".to_string(),
                    "Periodontal disease (Porphyromonas gingivalis)".to_string(),
                    "Obesity".to_string(),
                    "Anti-CCP antibody seropositivity in pre-RA".to_string(),
                ],
            },
            trend: Trend::Stable,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec![
                    "Conventional Synthetic DMARDs".to_string(),
                    "NSAIDs".to_string(),
                    "Corticosteroids (bridge)".to_string(),
                ],
                representative_drugs: vec![
                    "methotrexate".to_string(),
                    "hydroxychloroquine".to_string(),
                    "sulfasalazine".to_string(),
                    "leflunomide".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "Anti-TNF Biologics".to_string(),
                    "IL-6 Inhibitors".to_string(),
                    "CD80/86 Costimulation Blockers".to_string(),
                    "Anti-CD20 Biologics".to_string(),
                ],
                representative_drugs: vec![
                    "adalimumab".to_string(),
                    "etanercept".to_string(),
                    "infliximab".to_string(),
                    "tocilizumab".to_string(),
                    "sarilumab".to_string(),
                    "abatacept".to_string(),
                    "rituximab".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec!["JAK Inhibitors".to_string()],
                representative_drugs: vec![
                    "tofacitinib".to_string(),
                    "baricitinib".to_string(),
                    "upadacitinib".to_string(),
                    "filgotinib".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Drug-free remission after DMARD withdrawal".to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "Remission is rarely sustained after stopping therapy; flare rate >50% within 12 months of withdrawal even in sustained remission".to_string(),
                potential_approaches: vec![
                    "Tolerogenic dendritic cell therapies".to_string(),
                    "Antigen-specific immunotherapy".to_string(),
                    "Biomarker-guided step-down protocols".to_string(),
                ],
            },
            UnmetNeed {
                description: "Safe immunosuppression for patients with high CV risk".to_string(),
                severity: NeedSeverity::High,
                current_gap: "ORAL Surveillance (2021) confirmed elevated MACE/malignancy/thrombosis risk with tofacitinib vs. TNFi in high-CV-risk patients".to_string(),
                potential_approaches: vec![
                    "JAK inhibitor selection based on CV risk stratification".to_string(),
                    "Selective JAK1 inhibitors (upadacitinib) with improved safety profiles".to_string(),
                ],
            },
            UnmetNeed {
                description: "Precision medicine: predicting biologic response before prescribing".to_string(),
                severity: NeedSeverity::High,
                current_gap: "No validated biomarker predicts which biologic class will induce remission; trial-and-error approach standard".to_string(),
                potential_approaches: vec![
                    "Synovial tissue transcriptomic profiling".to_string(),
                    "Multi-omic serum biomarker panels".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 18,
            drugs_with_boxed_warnings: 8,
            drugs_with_rems: 0,
            class_effects: vec![
                ClassEffect {
                    drug_class: "JAK Inhibitors".to_string(),
                    event: "Major adverse cardiovascular events (MACE), malignancy, venous thromboembolism".to_string(),
                    evidence_strength: "Boxed warning; confirmed in ORAL Surveillance and CLASS trials".to_string(),
                },
                ClassEffect {
                    drug_class: "Anti-TNF Biologics".to_string(),
                    event: "Serious infections, reactivation of latent tuberculosis, opportunistic infections".to_string(),
                    evidence_strength: "Established class effect; TB screening mandatory before initiation".to_string(),
                },
                ClassEffect {
                    drug_class: "Conventional Synthetic DMARDs".to_string(),
                    event: "Hepatotoxicity, bone marrow suppression (methotrexate); retinal toxicity (hydroxychloroquine)".to_string(),
                    evidence_strength: "Well-established; requires monitoring protocols".to_string(),
                },
            ],
            notable_withdrawals: vec![DrugWithdrawal {
                drug_name: "rofecoxib".to_string(),
                year: 2004,
                reason: "Twofold increase in serious cardiovascular thrombotic events (APPROVe trial)".to_string(),
            }],
        },
        biomarkers: vec![
            Biomarker {
                name: "Rheumatoid Factor (RF)".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Diagnostic marker; seropositive RA has worse prognosis than seronegative".to_string(),
            },
            Biomarker {
                name: "Anti-CCP (ACPA)".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "High specificity (>95%) for RA; can be positive years before clinical onset; prognostic for erosive disease".to_string(),
            },
            Biomarker {
                name: "ESR / CRP".to_string(),
                biomarker_type: BiomarkerType::Pharmacodynamic,
                clinical_use: "Disease activity monitoring; components of DAS28 score for treat-to-target assessment".to_string(),
            },
            Biomarker {
                name: "DAS28 Score".to_string(),
                biomarker_type: BiomarkerType::Pharmacodynamic,
                clinical_use: "Composite disease activity score; remission defined as DAS28 <2.6; primary T2T endpoint".to_string(),
            },
        ],
    }
}
