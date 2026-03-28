//! Catalog data for Atrial Fibrillation.
//!
//! Sources: ACC/AHA/HRS 2023 AF Guideline, CDC AF Statistics 2023,
//! FDA DOAC approval documents, RE-LY/ROCKET-AF/ARISTOTLE/ENGAGE trial data.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, DrugWithdrawal,
    Epidemiology, EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea,
    TreatmentLine, Trend, UnmetNeed,
};

/// Returns the canonical Atrial Fibrillation disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("af"),
        name: "Atrial Fibrillation".to_string(),
        icd10_codes: vec!["I48".to_string(), "I48.0".to_string(), "I48.91".to_string()],
        therapeutic_area: TherapeuticArea::Cardiovascular,
        epidemiology: Epidemiology {
            global_prevalence: Some(0.51),
            us_prevalence: Some(1.86),
            annual_incidence: Some(200.0),
            demographics: Demographics {
                median_age_onset: Some(66),
                sex_ratio: Some("1.7:1 M:F".to_string()),
                risk_factors: vec![
                    "Age ≥65 years".to_string(),
                    "Hypertension".to_string(),
                    "Heart failure".to_string(),
                    "Coronary artery disease".to_string(),
                    "Valvular heart disease".to_string(),
                    "Obesity and obstructive sleep apnea".to_string(),
                    "Diabetes mellitus".to_string(),
                    "Alcohol use (holiday heart)".to_string(),
                    "Hyperthyroidism".to_string(),
                ],
            },
            trend: Trend::Increasing,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec![
                    "Rate Control Agents".to_string(),
                    "Direct Oral Anticoagulants (DOACs)".to_string(),
                    "Vitamin K Antagonists".to_string(),
                ],
                representative_drugs: vec![
                    "metoprolol".to_string(),
                    "diltiazem".to_string(),
                    "digoxin".to_string(),
                    "apixaban".to_string(),
                    "rivaroxaban".to_string(),
                    "dabigatran".to_string(),
                    "edoxaban".to_string(),
                    "warfarin".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "Rhythm Control Agents".to_string(),
                    "Catheter Ablation".to_string(),
                ],
                representative_drugs: vec![
                    "flecainide".to_string(),
                    "propafenone".to_string(),
                    "amiodarone".to_string(),
                    "dronedarone".to_string(),
                    "pulmonary vein isolation ablation".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Adjunct,
                drug_classes: vec![
                    "Left Atrial Appendage Occlusion".to_string(),
                ],
                representative_drugs: vec![
                    "Watchman FLX device".to_string(),
                ],
                evidence_level: EvidenceLevel::IB,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Stroke prevention without clinically significant bleeding risk".to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "All DOACs carry major bleeding risk (~2–3%/year); ~30% of eligible patients are not anticoagulated due to bleeding concerns".to_string(),
                potential_approaches: vec![
                    "Factor XI inhibitors (abelacimab, asundexian) — antithrombotic without hemostatic impairment".to_string(),
                    "LAA occlusion device expansion for anticoagulant-intolerant patients".to_string(),
                ],
            },
            UnmetNeed {
                description: "Durable rhythm control without antiarrhythmic drug toxicity".to_string(),
                severity: NeedSeverity::High,
                current_gap: "Amiodarone is most effective but carries thyroid, pulmonary, and hepatic toxicity; ablation recurrence ~30% at 2 years".to_string(),
                potential_approaches: vec![
                    "Pulsed field ablation for improved lesion durability".to_string(),
                    "Novel atrium-selective antiarrhythmics".to_string(),
                ],
            },
            UnmetNeed {
                description: "AF detection in subclinical (silent) patient population".to_string(),
                severity: NeedSeverity::High,
                current_gap: "~25% of strokes are cryptogenic; many attributed to undetected paroxysmal AF; wearable ECG uptake limited by reimbursement".to_string(),
                potential_approaches: vec![
                    "Consumer wearable ECG (Apple Watch, KardiaMobile) integration into clinical workflow".to_string(),
                    "Implantable loop recorder for high-risk populations".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 10,
            drugs_with_boxed_warnings: 3,
            drugs_with_rems: 1,
            class_effects: vec![
                ClassEffect {
                    drug_class: "Direct Oral Anticoagulants".to_string(),
                    event: "Major bleeding events including intracranial and GI hemorrhage".to_string(),
                    evidence_strength: "Established class effect; rate varies by agent and dose".to_string(),
                },
                ClassEffect {
                    drug_class: "Class III Antiarrhythmics (amiodarone)".to_string(),
                    event: "Thyroid dysfunction, pulmonary toxicity, hepatotoxicity, corneal microdeposits".to_string(),
                    evidence_strength: "Boxed warning; mechanism-based iodine accumulation".to_string(),
                },
                ClassEffect {
                    drug_class: "Class IC Antiarrhythmics".to_string(),
                    event: "Proarrhythmia; contraindicated in structural heart disease (CAST trial)".to_string(),
                    evidence_strength: "Boxed warning; confirmed in CAST mortality signal".to_string(),
                },
            ],
            notable_withdrawals: vec![],
        },
        biomarkers: vec![
            Biomarker {
                name: "CHA2DS2-VASc Score".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Stroke risk stratification; score ≥2 (M) or ≥3 (F) mandates anticoagulation".to_string(),
            },
            Biomarker {
                name: "NT-proBNP / BNP".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Atrial stretch marker; elevated in AF and concomitant heart failure; guides diuretic therapy".to_string(),
            },
            Biomarker {
                name: "HAS-BLED Score".to_string(),
                biomarker_type: BiomarkerType::Safety,
                clinical_use: "Bleeding risk assessment before anticoagulation; score ≥3 signals high bleeding risk requiring modifiable factor correction".to_string(),
            },
            Biomarker {
                name: "Thyroid Function (TSH/fT4)".to_string(),
                biomarker_type: BiomarkerType::Safety,
                clinical_use: "Mandatory monitoring during amiodarone therapy; hypothyroidism and hyperthyroidism both occur".to_string(),
            },
        ],
    }
}
