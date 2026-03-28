//! Catalog data for Non-Small Cell Lung Cancer (NSCLC).
//!
//! Sources: NCCN Guidelines Lung Cancer v2.2024, SEER Cancer Statistics 2023,
//! FDA oncology approvals, KEYNOTE/IMpower/FLAURA/ALEX trial data.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, DrugWithdrawal,
    Epidemiology, EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea,
    TreatmentLine, Trend, UnmetNeed,
};

/// Returns the canonical Non-Small Cell Lung Cancer disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("nsclc"),
        name: "Non-Small Cell Lung Cancer".to_string(),
        icd10_codes: vec!["C34".to_string(), "C34.1".to_string(), "C34.9".to_string()],
        therapeutic_area: TherapeuticArea::Oncology,
        epidemiology: Epidemiology {
            global_prevalence: None,
            us_prevalence: None,
            annual_incidence: Some(72.0),
            demographics: Demographics {
                median_age_onset: Some(70),
                sex_ratio: Some("1.2:1 M:F".to_string()),
                risk_factors: vec![
                    "Tobacco smoking (accounts for ~85% of cases)".to_string(),
                    "Secondhand smoke exposure".to_string(),
                    "Radon gas exposure".to_string(),
                    "Occupational carcinogens (asbestos, arsenic, chromium)".to_string(),
                    "Air pollution (PM2.5)".to_string(),
                    "EGFR/KRAS/ALK germline susceptibility variants".to_string(),
                ],
            },
            trend: Trend::Decreasing,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec![
                    "Checkpoint Inhibitors (anti-PD-1/PD-L1)".to_string(),
                    "EGFR Tyrosine Kinase Inhibitors".to_string(),
                    "ALK Inhibitors".to_string(),
                    "Platinum-Based Chemotherapy".to_string(),
                ],
                representative_drugs: vec![
                    "pembrolizumab".to_string(),
                    "atezolizumab".to_string(),
                    "osimertinib".to_string(),
                    "alectinib".to_string(),
                    "lorlatinib".to_string(),
                    "carboplatin + paclitaxel".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "KRAS G12C Inhibitors".to_string(),
                    "ROS1 Inhibitors".to_string(),
                    "MET Inhibitors".to_string(),
                    "Antibody-Drug Conjugates".to_string(),
                ],
                representative_drugs: vec![
                    "sotorasib".to_string(),
                    "adagrasib".to_string(),
                    "crizotinib".to_string(),
                    "tepotinib".to_string(),
                    "trastuzumab deruxtecan".to_string(),
                ],
                evidence_level: EvidenceLevel::IB,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec![
                    "Docetaxel + Ramucirumab".to_string(),
                ],
                representative_drugs: vec![
                    "docetaxel".to_string(),
                    "ramucirumab".to_string(),
                    "pemetrexed".to_string(),
                ],
                evidence_level: EvidenceLevel::IB,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Overcoming acquired resistance to targeted therapies".to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "Nearly all patients on EGFR TKIs and ALK inhibitors develop resistance within 1–3 years; C797S, MET amplification are dominant mechanisms".to_string(),
                potential_approaches: vec![
                    "4th-generation EGFR inhibitors targeting C797S".to_string(),
                    "BiTE/bispecific antibodies for heterogeneous resistance".to_string(),
                    "ctDNA-guided early resistance detection and switch".to_string(),
                ],
            },
            UnmetNeed {
                description: "Effective therapy for KRAS G12C in PD-L1 high population".to_string(),
                severity: NeedSeverity::High,
                current_gap: "KRAS G12C inhibitor monotherapy ORR ~36%; combination with checkpoint inhibitors is being explored but not approved".to_string(),
                potential_approaches: vec![
                    "Sotorasib + pembrolizumab combinations".to_string(),
                    "KRAS G12C + SHP2 inhibitor combinations".to_string(),
                ],
            },
            UnmetNeed {
                description: "CNS efficacy for brain metastases".to_string(),
                severity: NeedSeverity::High,
                current_gap: "40% of advanced NSCLC patients develop brain metastases; CNS penetration varies significantly across approved agents".to_string(),
                potential_approaches: vec![
                    "Lorlatinib (already strong CNS penetration for ALK+)".to_string(),
                    "Next-generation BBB-penetrant TKIs".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 22,
            drugs_with_boxed_warnings: 4,
            drugs_with_rems: 0,
            class_effects: vec![
                ClassEffect {
                    drug_class: "Checkpoint Inhibitors (anti-PD-1/PD-L1)".to_string(),
                    event: "Immune-mediated adverse reactions (pneumonitis, colitis, hepatitis, endocrinopathies)".to_string(),
                    evidence_strength: "Established class effect across all approved agents".to_string(),
                },
                ClassEffect {
                    drug_class: "EGFR Tyrosine Kinase Inhibitors".to_string(),
                    event: "Dermatologic toxicity (rash, paronychia), QTc prolongation (3rd gen)".to_string(),
                    evidence_strength: "Mechanism-based; observed across 1st, 2nd, 3rd generation".to_string(),
                },
                ClassEffect {
                    drug_class: "ALK Inhibitors".to_string(),
                    event: "Interstitial lung disease, visual disturbances, bradycardia".to_string(),
                    evidence_strength: "Confirmed across crizotinib, alectinib, lorlatinib".to_string(),
                },
            ],
            notable_withdrawals: vec![],
        },
        biomarkers: vec![
            Biomarker {
                name: "PD-L1 Expression (TPS/CPS)".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use: "Guides checkpoint inhibitor selection; TPS ≥50% predicts pembrolizumab monotherapy benefit".to_string(),
            },
            Biomarker {
                name: "EGFR Mutation (exon 19 del, L858R)".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use: "Mandates EGFR TKI (osimertinib preferred); present in ~15% of Western, ~50% of Asian NSCLC".to_string(),
            },
            Biomarker {
                name: "ALK Rearrangement".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use: "Mandates ALK inhibitor; present in ~5% of NSCLC; detected by FISH or IHC".to_string(),
            },
            Biomarker {
                name: "ROS1 Rearrangement".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use: "Mandates ROS1 inhibitor (crizotinib, entrectinib); present in ~1–2% of NSCLC".to_string(),
            },
            Biomarker {
                name: "KRAS G12C".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use: "Targetable with sotorasib or adagrasib; present in ~13% of NSCLC adenocarcinoma".to_string(),
            },
            Biomarker {
                name: "TMB (Tumor Mutational Burden)".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use: "High TMB (≥10 mut/Mb) associated with checkpoint inhibitor benefit; FDA-approved companion diagnostic".to_string(),
            },
        ],
    }
}
