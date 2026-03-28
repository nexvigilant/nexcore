//! Catalog data for Psoriasis.
//!
//! Sources: AAD-NPF 2019 Guidelines, WHO Psoriasis Report 2016,
//! FDA biologic approvals, VOYAGE/UNCOVER/CLEAR/IMMhance trial data.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, DrugWithdrawal,
    Epidemiology, EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea,
    TreatmentLine, Trend, UnmetNeed,
};

/// Returns the canonical Psoriasis disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("psoriasis"),
        name: "Plaque Psoriasis".to_string(),
        icd10_codes: vec!["L40".to_string(), "L40.0".to_string()],
        therapeutic_area: TherapeuticArea::Dermatology,
        epidemiology: Epidemiology {
            global_prevalence: Some(3.0),
            us_prevalence: Some(3.2),
            annual_incidence: Some(78.0),
            demographics: Demographics {
                median_age_onset: Some(28),
                sex_ratio: Some("1.1:1 M:F".to_string()),
                risk_factors: vec![
                    "Family history (HLA-Cw6 allele)".to_string(),
                    "Obesity".to_string(),
                    "Tobacco smoking".to_string(),
                    "Heavy alcohol consumption".to_string(),
                    "Psychological stress".to_string(),
                    "Certain medications (beta-blockers, lithium, antimalarials)".to_string(),
                    "Streptococcal throat infections (guttate trigger)".to_string(),
                ],
            },
            trend: Trend::Stable,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec![
                    "Topical Corticosteroids".to_string(),
                    "Topical Vitamin D Analogues".to_string(),
                    "Topical Retinoids".to_string(),
                    "Keratolytics".to_string(),
                ],
                representative_drugs: vec![
                    "betamethasone dipropionate".to_string(),
                    "clobetasol propionate".to_string(),
                    "calcipotriol".to_string(),
                    "tazarotene".to_string(),
                    "salicylic acid".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "Phototherapy (NB-UVB)".to_string(),
                    "Systemic Non-Biologic DMARDs".to_string(),
                ],
                representative_drugs: vec![
                    "narrowband UVB phototherapy".to_string(),
                    "methotrexate".to_string(),
                    "cyclosporine".to_string(),
                    "acitretin".to_string(),
                    "apremilast".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec![
                    "IL-17 Inhibitors".to_string(),
                    "IL-23 Inhibitors".to_string(),
                    "IL-12/23 Inhibitors".to_string(),
                    "Anti-TNF Biologics".to_string(),
                ],
                representative_drugs: vec![
                    "secukinumab".to_string(),
                    "ixekizumab".to_string(),
                    "bimekizumab".to_string(),
                    "risankizumab".to_string(),
                    "guselkumab".to_string(),
                    "tildrakizumab".to_string(),
                    "ustekinumab".to_string(),
                    "adalimumab".to_string(),
                    "etanercept".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Sustained PASI 100 (complete skin clearance) without chronic immunosuppression".to_string(),
                severity: NeedSeverity::Critical,
                current_gap: "IL-17 and IL-23 inhibitors achieve PASI 100 in ~40–60% at week 16, but durability requires continuous dosing; no cure available".to_string(),
                potential_approaches: vec![
                    "Targeted induction followed by intermittent dosing strategies".to_string(),
                    "T-cell tolerance induction approaches".to_string(),
                    "TYK2 inhibitors (deucravacitinib) for oral option with selectivity".to_string(),
                ],
            },
            UnmetNeed {
                description: "Effective therapy for psoriatic arthritis without PsA-specific trial requirement".to_string(),
                severity: NeedSeverity::High,
                current_gap: "~30% of plaque psoriasis patients develop psoriatic arthritis; skin and joint responses to biologics are partially dissociated".to_string(),
                potential_approaches: vec![
                    "IL-17 inhibitors with demonstrated PsA joint efficacy (secukinumab)".to_string(),
                    "JAK inhibitors for combined skin + joint disease".to_string(),
                ],
            },
            UnmetNeed {
                description: "Long-term safety data for IL-17/IL-23 inhibitors beyond 5 years".to_string(),
                severity: NeedSeverity::Moderate,
                current_gap: "Most biologics approved 2015–2020; long-term malignancy and infection registries still maturing".to_string(),
                potential_approaches: vec![
                    "PSOLAR and comparable registries reaching 10-year endpoints".to_string(),
                    "Comparative effectiveness studies across biologic classes".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 14,
            drugs_with_boxed_warnings: 2,
            drugs_with_rems: 0,
            class_effects: vec![
                ClassEffect {
                    drug_class: "IL-17 Inhibitors".to_string(),
                    event: "Candida infections (oral/esophageal); inflammatory bowel disease exacerbation".to_string(),
                    evidence_strength: "Established class effect; IL-17 disrupts mucosal immunity".to_string(),
                },
                ClassEffect {
                    drug_class: "Anti-TNF Biologics".to_string(),
                    event: "Serious infections, latent TB reactivation, demyelinating disease risk".to_string(),
                    evidence_strength: "Well-established; TB screening mandatory".to_string(),
                },
                ClassEffect {
                    drug_class: "Systemic Retinoids".to_string(),
                    event: "Teratogenicity (Category X); hypertriglyceridemia; mucocutaneous dryness".to_string(),
                    evidence_strength: "Boxed warning; mandatory iPLEDGE REMS program".to_string(),
                },
            ],
            notable_withdrawals: vec![],
        },
        biomarkers: vec![
            Biomarker {
                name: "PASI Score (Psoriasis Area and Severity Index)".to_string(),
                biomarker_type: BiomarkerType::Pharmacodynamic,
                clinical_use: "Primary efficacy endpoint in clinical trials; PASI 75/90/100 = 75%/90%/100% improvement from baseline".to_string(),
            },
            Biomarker {
                name: "HLA-Cw6".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Genetic risk allele; associated with early-onset psoriasis and ustekinumab response prediction".to_string(),
            },
            Biomarker {
                name: "IL-17A / IL-22 Serum Levels".to_string(),
                biomarker_type: BiomarkerType::Pharmacodynamic,
                clinical_use: "Correlate with disease activity; used in mechanistic studies to confirm IL-17 pathway engagement".to_string(),
            },
        ],
    }
}
