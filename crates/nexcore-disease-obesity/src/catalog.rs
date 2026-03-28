//! Catalog data for Obesity.
//!
//! Sources: CDC BRFSS 2022, NIDDK obesity statistics, SURMOUNT trial data,
//! AHA/ACC/TOS obesity guideline 2023, FDA drug labels.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, DrugWithdrawal,
    Epidemiology, EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea,
    TreatmentLine, Trend, UnmetNeed,
};

/// Returns the canonical Obesity disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("obesity"),
        name: "Obesity".to_string(),
        icd10_codes: vec!["E66".to_string(), "E66.0".to_string(), "E66.9".to_string()],
        therapeutic_area: TherapeuticArea::Metabolic,
        epidemiology: Epidemiology {
            global_prevalence: Some(16.0),
            us_prevalence: Some(42.4),
            annual_incidence: None,
            demographics: Demographics {
                median_age_onset: Some(40),
                sex_ratio: Some("1.0:1 F:M".to_string()),
                risk_factors: vec![
                    "High-calorie dietary patterns".to_string(),
                    "Physical inactivity".to_string(),
                    "Genetic susceptibility (FTO, MC4R variants)".to_string(),
                    "Sleep deprivation".to_string(),
                    "Certain medications (antipsychotics, corticosteroids)".to_string(),
                    "Socioeconomic deprivation".to_string(),
                    "Gut microbiome dysbiosis".to_string(),
                ],
            },
            trend: Trend::Increasing,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec!["Lifestyle Intervention".to_string()],
                representative_drugs: vec![
                    "dietary modification".to_string(),
                    "structured exercise program".to_string(),
                    "behavioral counseling".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "GLP-1 Receptor Agonists".to_string(),
                    "GLP-1/GIP Dual Agonists".to_string(),
                    "Sympathomimetic Agents".to_string(),
                    "Lipase Inhibitors".to_string(),
                ],
                representative_drugs: vec![
                    "semaglutide 2.4mg".to_string(),
                    "liraglutide 3mg".to_string(),
                    "tirzepatide".to_string(),
                    "phentermine-topiramate".to_string(),
                    "naltrexone-bupropion".to_string(),
                    "orlistat".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec!["Bariatric Surgery".to_string()],
                representative_drugs: vec![
                    "Roux-en-Y gastric bypass".to_string(),
                    "sleeve gastrectomy".to_string(),
                    "adjustable gastric band".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Lean mass preservation during weight loss".to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "GLP-1 agonists cause ~25–40% of weight loss as lean muscle mass, increasing sarcopenic obesity risk".to_string(),
                potential_approaches: vec![
                    "Combination therapy with anabolic agents".to_string(),
                    "Bimagrumab (anti-ActRII antibody)".to_string(),
                    "Exercise co-prescription protocols".to_string(),
                ],
            },
            UnmetNeed {
                description: "Weight maintenance after pharmacotherapy discontinuation".to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "67% of weight regained within 1 year of stopping semaglutide (STEP 4 extension)".to_string(),
                potential_approaches: vec![
                    "Long-term maintenance dosing strategies".to_string(),
                    "Transition to lower-burden maintenance agents".to_string(),
                    "Combination lifestyle + pharmacotherapy".to_string(),
                ],
            },
            UnmetNeed {
                description: "Treatment of obesity-associated HFpEF".to_string(),
                severity: NeedSeverity::High,
                current_gap: "Heart failure with preserved ejection fraction is the dominant CV complication; semaglutide shows signal but not yet approved".to_string(),
                potential_approaches: vec![
                    "GLP-1 agonist obesity-HFpEF label expansion".to_string(),
                    "SGLT2i combination for cardiometabolic phenotype".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 7,
            drugs_with_boxed_warnings: 2,
            drugs_with_rems: 1,
            class_effects: vec![
                ClassEffect {
                    drug_class: "GLP-1 Receptor Agonists".to_string(),
                    event: "Nausea, vomiting, gastroparesis; pancreatitis risk".to_string(),
                    evidence_strength: "Established class effect; mechanism-based".to_string(),
                },
                ClassEffect {
                    drug_class: "Sympathomimetic Agents".to_string(),
                    event: "Cardiovascular stimulation: tachycardia, hypertension".to_string(),
                    evidence_strength: "Well-established sympathomimetic mechanism".to_string(),
                },
            ],
            notable_withdrawals: vec![
                DrugWithdrawal {
                    drug_name: "sibutramine".to_string(),
                    year: 2010,
                    reason: "Increased risk of major adverse cardiovascular events (SCOUT trial)".to_string(),
                },
                DrugWithdrawal {
                    drug_name: "rimonabant".to_string(),
                    year: 2008,
                    reason: "Increased risk of serious psychiatric adverse events including suicidality".to_string(),
                },
            ],
        },
        biomarkers: vec![
            Biomarker {
                name: "BMI".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Primary diagnostic criterion; ≥30 kg/m² defines obesity; guides drug and surgical thresholds".to_string(),
            },
            Biomarker {
                name: "Waist Circumference".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Central adiposity marker; >102 cm (M) or >88 cm (F) indicates elevated cardiometabolic risk".to_string(),
            },
            Biomarker {
                name: "Body Composition (DEXA)".to_string(),
                biomarker_type: BiomarkerType::Pharmacodynamic,
                clinical_use: "Quantifies lean vs. fat mass change during pharmacotherapy to detect sarcopenic obesity".to_string(),
            },
        ],
    }
}
