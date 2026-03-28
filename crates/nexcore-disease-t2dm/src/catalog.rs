//! Catalog data for Type 2 Diabetes Mellitus.
//!
//! Sources: ADA Standards of Care 2024, CDC Diabetes Statistics 2022,
//! IDF Diabetes Atlas 10th Edition, FDA drug safety communications.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, DrugWithdrawal,
    Epidemiology, EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea,
    TreatmentLine, Trend, UnmetNeed,
};

/// Returns the canonical Type 2 Diabetes Mellitus disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("t2dm"),
        name: "Type 2 Diabetes Mellitus".to_string(),
        icd10_codes: vec!["E11".to_string()],
        therapeutic_area: TherapeuticArea::Metabolic,
        epidemiology: Epidemiology {
            global_prevalence: Some(10.5),
            us_prevalence: Some(11.3),
            annual_incidence: Some(400.0),
            demographics: Demographics {
                median_age_onset: Some(55),
                sex_ratio: Some("1.1:1 M:F".to_string()),
                risk_factors: vec![
                    "Obesity (BMI ≥30)".to_string(),
                    "Physical inactivity".to_string(),
                    "Family history".to_string(),
                    "Gestational diabetes history".to_string(),
                    "Pre-diabetes (HbA1c 5.7–6.4%)".to_string(),
                    "Ethnicity (higher risk: Black, Hispanic, South Asian)".to_string(),
                ],
            },
            trend: Trend::Increasing,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec!["Biguanides".to_string()],
                representative_drugs: vec!["metformin".to_string()],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "GLP-1 Receptor Agonists".to_string(),
                    "SGLT-2 Inhibitors".to_string(),
                    "DPP-4 Inhibitors".to_string(),
                    "Sulfonylureas".to_string(),
                ],
                representative_drugs: vec![
                    "semaglutide".to_string(),
                    "liraglutide".to_string(),
                    "empagliflozin".to_string(),
                    "dapagliflozin".to_string(),
                    "sitagliptin".to_string(),
                    "glipizide".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec![
                    "Basal Insulin".to_string(),
                    "Combination GLP-1/GIP".to_string(),
                ],
                representative_drugs: vec![
                    "insulin glargine".to_string(),
                    "insulin degludec".to_string(),
                    "tirzepatide".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "CV mortality reduction in patients with CKD".to_string(),
                severity: NeedSeverity::Critical,
                current_gap:
                    "SGLT2i have renal restriction at low eGFR; GLP-1 limited in advanced CKD"
                        .to_string(),
                potential_approaches: vec![
                    "Non-steroidal MRA (finerenone)".to_string(),
                    "Novel SGLT2i with broader CKD approval".to_string(),
                ],
            },
            UnmetNeed {
                description: "Beta cell preservation and disease modification".to_string(),
                severity: NeedSeverity::High,
                current_gap: "All approved agents are symptomatic; none halt beta cell decline"
                    .to_string(),
                potential_approaches: vec![
                    "Teplizumab-class immune modulation".to_string(),
                    "GLP-1/GIP combinations".to_string(),
                    "Stem cell therapy".to_string(),
                ],
            },
            UnmetNeed {
                description: "Weight maintenance after GLP-1 discontinuation".to_string(),
                severity: NeedSeverity::High,
                current_gap: "Most patients regain weight within 1 year of stopping GLP-1 therapy"
                    .to_string(),
                potential_approaches: vec![
                    "Combination maintenance therapy".to_string(),
                    "Durable beta cell restoration".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 12,
            drugs_with_boxed_warnings: 2,
            drugs_with_rems: 0,
            class_effects: vec![
                ClassEffect {
                    drug_class: "GLP-1 Receptor Agonists".to_string(),
                    event: "Pancreatitis (acute and chronic)".to_string(),
                    evidence_strength: "Established class effect per labeling".to_string(),
                },
                ClassEffect {
                    drug_class: "SGLT-2 Inhibitors".to_string(),
                    event: "Diabetic ketoacidosis (euglycemic)".to_string(),
                    evidence_strength: "Established class effect per FDA safety communication"
                        .to_string(),
                },
                ClassEffect {
                    drug_class: "Sulfonylureas".to_string(),
                    event: "Hypoglycemia".to_string(),
                    evidence_strength: "Well-established, mechanism-based".to_string(),
                },
            ],
            notable_withdrawals: vec![DrugWithdrawal {
                drug_name: "troglitazone".to_string(),
                year: 2000,
                reason: "Severe idiosyncratic hepatotoxicity with fatal cases".to_string(),
            }],
        },
        biomarkers: vec![
            Biomarker {
                name: "HbA1c".to_string(),
                biomarker_type: BiomarkerType::Pharmacodynamic,
                clinical_use: "Primary glycemic target; goal <7% for most adults".to_string(),
            },
            Biomarker {
                name: "Fasting Plasma Glucose".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Diagnostic threshold ≥126 mg/dL; treatment monitoring".to_string(),
            },
            Biomarker {
                name: "eGFR".to_string(),
                biomarker_type: BiomarkerType::Safety,
                clinical_use: "Guides SGLT2i and metformin dosing adjustments".to_string(),
            },
            Biomarker {
                name: "C-peptide".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use:
                    "Distinguishes T2DM from T1DM by measuring residual beta cell function"
                        .to_string(),
            },
        ],
    }
}
