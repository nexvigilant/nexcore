//! Catalog data for Heart Failure.
//!
//! Sources: ACC/AHA/HFSA 2022 HF Guideline, ESC 2021 HF Guidelines,
//! FDA approval documents for HFrEF/HFpEF agents,
//! PARADIGM-HF/DAPA-HF/EMPEROR-Reduced/RALES/CIBIS-II/MERIT-HF trial data.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, Epidemiology,
    EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea, TreatmentLine,
    Trend, UnmetNeed,
};

/// Returns the canonical Heart Failure disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("hf"),
        name: "Heart Failure".to_string(),
        icd10_codes: vec![
            "I50".to_string(),
            "I50.1".to_string(),
            "I50.20".to_string(),
            "I50.30".to_string(),
            "I50.40".to_string(),
            "I50.9".to_string(),
        ],
        therapeutic_area: TherapeuticArea::Cardiovascular,
        epidemiology: Epidemiology {
            global_prevalence: Some(1.20),
            us_prevalence: Some(2.00),
            annual_incidence: Some(960.0),
            demographics: Demographics {
                median_age_onset: Some(72),
                sex_ratio: Some("1.5:1 M:F (HFrEF); 1:1.5 M:F (HFpEF)".to_string()),
                risk_factors: vec![
                    "Coronary artery disease and prior myocardial infarction".to_string(),
                    "Hypertension (most prevalent modifiable risk factor)".to_string(),
                    "Diabetes mellitus type 2".to_string(),
                    "Atrial fibrillation".to_string(),
                    "Obesity and metabolic syndrome".to_string(),
                    "Valvular heart disease".to_string(),
                    "Cardiomyopathy (dilated, hypertrophic, ischemic)".to_string(),
                    "Cardiotoxic chemotherapy exposure (anthracyclines, trastuzumab)".to_string(),
                    "Alcohol abuse and substance use".to_string(),
                    "Chronic kidney disease".to_string(),
                ],
            },
            trend: Trend::Increasing,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec![
                    "ACE Inhibitors / ARBs".to_string(),
                    "Beta-Blockers".to_string(),
                    "Mineralocorticoid Receptor Antagonists".to_string(),
                    "SGLT2 Inhibitors".to_string(),
                ],
                representative_drugs: vec![
                    "enalapril".to_string(),
                    "lisinopril".to_string(),
                    "ramipril".to_string(),
                    "losartan".to_string(),
                    "valsartan".to_string(),
                    "carvedilol".to_string(),
                    "metoprolol succinate".to_string(),
                    "bisoprolol".to_string(),
                    "spironolactone".to_string(),
                    "eplerenone".to_string(),
                    "dapagliflozin".to_string(),
                    "empagliflozin".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "ARNI (Angiotensin Receptor-Neprilysin Inhibitor)".to_string(),
                    "Loop Diuretics".to_string(),
                    "Ivabradine (If-channel inhibitor)".to_string(),
                ],
                representative_drugs: vec![
                    "sacubitril/valsartan".to_string(),
                    "furosemide".to_string(),
                    "bumetanide".to_string(),
                    "torsemide".to_string(),
                    "ivabradine".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec![
                    "Hydralazine/Nitrate Combination".to_string(),
                    "Digoxin".to_string(),
                    "Inotropes (inpatient acute decompensation)".to_string(),
                ],
                representative_drugs: vec![
                    "hydralazine".to_string(),
                    "isosorbide dinitrate".to_string(),
                    "digoxin".to_string(),
                    "dobutamine".to_string(),
                    "milrinone".to_string(),
                ],
                evidence_level: EvidenceLevel::IB,
            },
            TreatmentLine {
                line: LineOfTherapy::Adjunct,
                drug_classes: vec![
                    "Device Therapy".to_string(),
                    "Iron Supplementation (if iron-deficient)".to_string(),
                ],
                representative_drugs: vec![
                    "implantable cardioverter-defibrillator (ICD)".to_string(),
                    "cardiac resynchronization therapy (CRT)".to_string(),
                    "ferric carboxymaltose".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Effective treatment for HFpEF (preserved ejection fraction)"
                    .to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "HFpEF accounts for ~50% of HF cases; most RAAS/beta-blocker trials \
                     showed neutral outcomes; SGLT2i (EMPEROR-Preserved) showed modest \
                     benefit; no approved disease-modifying therapy with mortality reduction"
                    .to_string(),
                potential_approaches: vec![
                    "SGLT2 inhibitors — first class with HFpEF outcomes benefit (empagliflozin)"
                        .to_string(),
                    "GLP-1 receptor agonists for obesity-phenotype HFpEF (STEP-HFpEF trial)"
                        .to_string(),
                    "Cardiac myosin activators for HFrEF with preserved mechanics".to_string(),
                ],
            },
            UnmetNeed {
                description: "Reduction in 30-day hospital readmission rates".to_string(),
                severity: NeedSeverity::High,
                current_gap:
                    "~25% of HF patients are readmitted within 30 days; current telemonitoring \
                     and optimization strategies yield modest reduction; remote hemodynamic \
                     monitoring (CardioMEMS) limited to a subset"
                        .to_string(),
                potential_approaches: vec![
                    "Pulmonary artery pressure monitoring (CardioMEMS) with guided diuresis"
                        .to_string(),
                    "Implantable loop recorders for early arrhythmia detection".to_string(),
                    "AI-driven early warning systems for decompensation prediction".to_string(),
                ],
            },
            UnmetNeed {
                description: "Cardiorenal protection across the HF-CKD continuum".to_string(),
                severity: NeedSeverity::High,
                current_gap:
                    "~50% of HF patients have CKD; worsening renal function limits diuretic \
                     dosing and RAAS intensification; cardiorenal syndrome remains a major \
                     driver of mortality"
                        .to_string(),
                potential_approaches: vec![
                    "SGLT2 inhibitors — demonstrated cardiorenal protection in DAPA-CKD/EMPA-KIDNEY"
                        .to_string(),
                    "Novel vasopressin V2 antagonists for volume overload in CKD".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 14,
            drugs_with_boxed_warnings: 4,
            drugs_with_rems: 0,
            class_effects: vec![
                ClassEffect {
                    drug_class: "ACE Inhibitors".to_string(),
                    event: "Angioedema (especially in Black patients), renal impairment, \
                            hyperkalemia, and teratogenicity"
                        .to_string(),
                    evidence_strength:
                        "Boxed warning for teratogenicity; angioedema rate ~0.1–0.7%".to_string(),
                },
                ClassEffect {
                    drug_class: "Mineralocorticoid Receptor Antagonists".to_string(),
                    event: "Hyperkalemia (risk increased with CKD and concomitant RAAS blockade), \
                            gynecomastia (spironolactone)"
                        .to_string(),
                    evidence_strength:
                        "Well-established; K+ >5.5 mEq/L requires dose reduction or \
                         discontinuation"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "Beta-Blockers".to_string(),
                    event: "Bradycardia, heart block, bronchospasm in reactive airway disease, \
                            masking of hypoglycemia symptoms"
                        .to_string(),
                    evidence_strength:
                        "Class effect; contraindicated in decompensated acute HF; initiate \
                         only in stable euvolemic patients"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "Loop Diuretics".to_string(),
                    event: "Electrolyte imbalances (hypokalemia, hypomagnesemia), \
                            ototoxicity at high IV doses, volume depletion"
                        .to_string(),
                    evidence_strength: "Well-established; electrolyte monitoring mandatory"
                        .to_string(),
                },
            ],
            notable_withdrawals: vec![],
        },
        biomarkers: vec![
            Biomarker {
                name: "NT-proBNP / BNP".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Primary biomarker for HF diagnosis and severity stratification; \
                     NT-proBNP >900 pg/mL (age <75) confirms acute HF; guides diuretic \
                     therapy titration and discharge readiness"
                    .to_string(),
            },
            Biomarker {
                name: "Troponin I / T (high-sensitivity)".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Elevated in acute decompensated HF even without ACS; predicts \
                     30-day mortality and readmission; guides urgency of intervention"
                    .to_string(),
            },
            Biomarker {
                name: "Left Ventricular Ejection Fraction (LVEF)".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use: "Echocardiographic measure classifying HF phenotype: HFrEF (<40%), \
                     HFmrEF (40-49%), HFpEF (≥50%); directs evidence-based therapy selection"
                    .to_string(),
            },
            Biomarker {
                name: "Serum Creatinine / eGFR".to_string(),
                biomarker_type: BiomarkerType::Safety,
                clinical_use: "Mandatory monitoring with RAAS inhibitors and diuretics; eGFR <30 \
                     limits SGLT2i use; creatinine rise >30% from baseline warrants RAAS \
                     dose reduction"
                    .to_string(),
            },
            Biomarker {
                name: "Serum Potassium".to_string(),
                biomarker_type: BiomarkerType::Safety,
                clinical_use: "Critical safety marker with MRA and RAAS combination therapy; \
                     hyperkalemia (K+ >5.5) requires dose reduction; patiromer or \
                     sodium zirconium cyclosilicate enables MRA continuation in CKD"
                    .to_string(),
            },
            Biomarker {
                name: "6-Minute Walk Test (6MWT)".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Functional capacity surrogate; <300 m predicts adverse outcomes; \
                     used in clinical trial endpoints and device therapy eligibility \
                     assessment (CRT, VAD)"
                    .to_string(),
            },
        ],
    }
}
