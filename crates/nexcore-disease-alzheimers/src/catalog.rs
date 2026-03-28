//! Catalog data for Alzheimer's Disease.
//!
//! Sources: Alzheimer's Association Facts & Figures 2023, FDA drug approvals,
//! NIA epidemiology data, ADUCANUMAB/LEQEMBI approval documentation.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, DrugWithdrawal,
    Epidemiology, EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea,
    TreatmentLine, Trend, UnmetNeed,
};

/// Returns the canonical Alzheimer's Disease disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("alzheimers"),
        name: "Alzheimer's Disease".to_string(),
        icd10_codes: vec!["G30".to_string(), "G30.0".to_string(), "G30.9".to_string()],
        therapeutic_area: TherapeuticArea::Neuroscience,
        epidemiology: Epidemiology {
            global_prevalence: Some(0.84),
            us_prevalence: Some(2.08),
            annual_incidence: Some(153.0),
            demographics: Demographics {
                median_age_onset: Some(72),
                sex_ratio: Some("2:1 F:M".to_string()),
                risk_factors: vec![
                    "Age ≥65 years".to_string(),
                    "ApoE ε4 allele carrier".to_string(),
                    "Family history of Alzheimer's".to_string(),
                    "Down syndrome".to_string(),
                    "Cardiovascular risk factors (hypertension, diabetes)".to_string(),
                    "Traumatic brain injury history".to_string(),
                ],
            },
            trend: Trend::Increasing,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec!["Cholinesterase Inhibitors".to_string()],
                representative_drugs: vec![
                    "donepezil".to_string(),
                    "rivastigmine".to_string(),
                    "galantamine".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec!["NMDA Receptor Antagonists".to_string()],
                representative_drugs: vec!["memantine".to_string()],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec!["Anti-Amyloid Monoclonal Antibodies".to_string()],
                representative_drugs: vec![
                    "lecanemab".to_string(),
                    "donanemab".to_string(),
                ],
                evidence_level: EvidenceLevel::IB,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Disease modification for moderate-to-severe Alzheimer's".to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "Anti-amyloid antibodies restricted to early/mild disease; no approved DMT for advanced stages".to_string(),
                potential_approaches: vec![
                    "Tau-targeting therapies".to_string(),
                    "Neuroinflammation modulators".to_string(),
                    "Combination amyloid + tau".to_string(),
                ],
            },
            UnmetNeed {
                description: "Accessible biomarker testing for early diagnosis".to_string(),
                severity: NeedSeverity::High,
                current_gap: "PET and CSF testing are expensive and invasive; plasma p-tau217 tests emerging".to_string(),
                potential_approaches: vec![
                    "Blood-based biomarker panels (p-tau217, Abeta42/40 ratio)".to_string(),
                    "Retinal imaging for amyloid detection".to_string(),
                ],
            },
            UnmetNeed {
                description: "Safe anti-amyloid therapy without ARIA risk".to_string(),
                severity: NeedSeverity::High,
                current_gap: "Current anti-amyloid antibodies carry 20–35% ARIA-E/ARIA-H rate requiring MRI monitoring".to_string(),
                potential_approaches: vec![
                    "Next-generation lower-dose antibody regimens".to_string(),
                    "ARIA biomarker-guided patient selection".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 6,
            drugs_with_boxed_warnings: 0,
            drugs_with_rems: 1,
            class_effects: vec![
                ClassEffect {
                    drug_class: "Anti-Amyloid Monoclonal Antibodies".to_string(),
                    event: "Amyloid-Related Imaging Abnormalities (ARIA-E edema, ARIA-H microhemorrhage)".to_string(),
                    evidence_strength: "Confirmed in Phase 3 trials; mechanism-based class effect".to_string(),
                },
                ClassEffect {
                    drug_class: "Cholinesterase Inhibitors".to_string(),
                    event: "GI adverse effects (nausea, vomiting, diarrhea)".to_string(),
                    evidence_strength: "Well-established cholinergic mechanism".to_string(),
                },
            ],
            notable_withdrawals: vec![DrugWithdrawal {
                drug_name: "aducanumab".to_string(),
                year: 2024,
                reason: "Voluntary withdrawal by Biogen; lack of demonstrated clinical benefit despite FDA accelerated approval".to_string(),
            }],
        },
        biomarkers: vec![
            Biomarker {
                name: "ApoE ε4 Genotype".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Risk stratification; required for ARIA risk assessment with anti-amyloid therapy".to_string(),
            },
            Biomarker {
                name: "Amyloid PET".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Confirms amyloid pathology; required for anti-amyloid antibody eligibility".to_string(),
            },
            Biomarker {
                name: "CSF Aβ42/40 Ratio".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Indicates amyloid accumulation; alternative to PET for disease confirmation".to_string(),
            },
            Biomarker {
                name: "Plasma p-tau217".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Emerging blood-based biomarker for early Alzheimer's detection (>85% accuracy vs. PET)".to_string(),
            },
        ],
    }
}
