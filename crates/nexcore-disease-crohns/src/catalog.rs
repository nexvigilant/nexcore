//! Catalog data for Crohn's Disease.
//!
//! Sources: AGA/ACG 2021 IBD Guidelines, CDC IBD Statistics 2022,
//! FDA approvals for anti-TNF/anti-integrin/anti-IL-12/23 agents,
//! SONIC/GEMINI/UNIFI/SEAVUE trial data.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, Epidemiology,
    EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea, TreatmentLine,
    Trend, UnmetNeed,
};

/// Returns the canonical Crohn's Disease disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("crohns"),
        name: "Crohn's Disease".to_string(),
        icd10_codes: vec![
            "K50".to_string(),
            "K50.0".to_string(),
            "K50.1".to_string(),
            "K50.8".to_string(),
            "K50.9".to_string(),
        ],
        therapeutic_area: TherapeuticArea::Immunology,
        epidemiology: Epidemiology {
            global_prevalence: Some(0.15),
            us_prevalence: Some(0.24),
            annual_incidence: Some(6.3),
            demographics: Demographics {
                median_age_onset: Some(29),
                sex_ratio: Some("1.1:1 F:M".to_string()),
                risk_factors: vec![
                    "Family history of IBD".to_string(),
                    "Smoking (active smoker doubles risk)".to_string(),
                    "Ashkenazi Jewish ancestry".to_string(),
                    "NOD2/CARD15 gene variants".to_string(),
                    "Appendectomy history".to_string(),
                    "High-fat, low-fiber Western diet".to_string(),
                    "NSAID use".to_string(),
                    "Altered gut microbiome (reduced Firmicutes)".to_string(),
                ],
            },
            trend: Trend::Increasing,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec![
                    "Corticosteroids".to_string(),
                    "5-Aminosalicylates".to_string(),
                    "Immunomodulators".to_string(),
                ],
                representative_drugs: vec![
                    "prednisone".to_string(),
                    "budesonide".to_string(),
                    "mesalamine".to_string(),
                    "azathioprine".to_string(),
                    "6-mercaptopurine".to_string(),
                    "methotrexate".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "Anti-TNF Biologic Agents".to_string(),
                    "Anti-Integrin Biologic Agents".to_string(),
                ],
                representative_drugs: vec![
                    "adalimumab".to_string(),
                    "infliximab".to_string(),
                    "certolizumab pegol".to_string(),
                    "vedolizumab".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec![
                    "Anti-IL-12/23 Biologic Agents".to_string(),
                    "Anti-IL-23 Biologic Agents".to_string(),
                    "JAK Inhibitors".to_string(),
                ],
                representative_drugs: vec![
                    "ustekinumab".to_string(),
                    "risankizumab".to_string(),
                    "upadacitinib".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Adjunct,
                drug_classes: vec!["Antibiotics".to_string(), "Nutritional Therapy".to_string()],
                representative_drugs: vec![
                    "metronidazole".to_string(),
                    "ciprofloxacin".to_string(),
                    "exclusive enteral nutrition (EEN)".to_string(),
                ],
                evidence_level: EvidenceLevel::IIA,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Durable remission without long-term immunosuppression toxicity"
                    .to_string(),
                severity: NeedSeverity::Critical,
                current_gap:
                    "Most biologics lose response over time (~40% at 1 year); sequential therapy \
                     escalation exhausts approved mechanisms; no cure outside surgical resection"
                        .to_string(),
                potential_approaches: vec![
                    "Combination biologic therapy (e.g., anti-TNF + anti-IL-23)".to_string(),
                    "Selective JAK1 inhibitors with improved gut selectivity".to_string(),
                    "Microbiome-based restoration therapies (FMT protocols)".to_string(),
                ],
            },
            UnmetNeed {
                description: "Reliable treat-to-target biomarker for mucosal healing".to_string(),
                severity: NeedSeverity::High,
                current_gap:
                    "CRP and fecal calprotectin imperfectly correlate with endoscopic healing; \
                     colonoscopy required for confirmation adds patient burden and cost"
                        .to_string(),
                potential_approaches: vec![
                    "Point-of-care fecal calprotectin assays with validated mucosal healing cutoffs"
                        .to_string(),
                    "Serum proteomics panels (IBDX) for non-invasive disease activity monitoring"
                        .to_string(),
                ],
            },
            UnmetNeed {
                description: "Prevention of disease-related complications (strictures, fistulas)"
                    .to_string(),
                severity: NeedSeverity::High,
                current_gap:
                    "~50% of patients require surgery within 10 years; anti-fibrotic therapies \
                     absent; fistula closure rates remain low even with biologic therapy"
                        .to_string(),
                potential_approaches: vec![
                    "Anti-TGF-β pathways to prevent fibrosis and stricture formation".to_string(),
                    "Mesenchymal stem cell therapy for complex perianal fistulas (darvadstrocel)"
                        .to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 12,
            drugs_with_boxed_warnings: 7,
            drugs_with_rems: 0,
            class_effects: vec![
                ClassEffect {
                    drug_class: "Anti-TNF Agents".to_string(),
                    event: "Serious infections including tuberculosis reactivation, opportunistic \
                            infections, and bacterial sepsis"
                        .to_string(),
                    evidence_strength:
                        "Boxed warning class effect; TB screening mandatory before initiation"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "Anti-TNF Agents".to_string(),
                    event: "Malignancy risk including lymphoma and non-melanoma skin cancers"
                        .to_string(),
                    evidence_strength:
                        "Boxed warning; risk elevated with concomitant thiopurine use (HSTCL)"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "JAK Inhibitors".to_string(),
                    event: "Major adverse cardiovascular events (MACE), malignancy, thrombosis, \
                            and serious infections"
                        .to_string(),
                    evidence_strength:
                        "Boxed warning added 2021 based on ORAL Surveillance trial in RA"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "Corticosteroids (long-term)".to_string(),
                    event: "Adrenal suppression, osteoporosis, hyperglycemia, Cushing's syndrome"
                        .to_string(),
                    evidence_strength: "Well-established; not for maintenance therapy".to_string(),
                },
            ],
            notable_withdrawals: vec![],
        },
        biomarkers: vec![
            Biomarker {
                name: "Fecal Calprotectin".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use:
                    "Non-invasive marker of intestinal inflammation; >250 µg/g correlates \
                     with active Crohn's; used for treat-to-target monitoring and relapse \
                     prediction"
                        .to_string(),
            },
            Biomarker {
                name: "C-Reactive Protein (CRP)".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Elevated in active inflammation (>5 mg/L); used alongside fecal \
                     calprotectin for disease activity assessment and treatment response \
                     monitoring"
                    .to_string(),
            },
            Biomarker {
                name: "ASCA / pANCA".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Anti-Saccharomyces cerevisiae antibodies (ASCA+/pANCA-) pattern \
                     supports Crohn's diagnosis; used in IBD serology panels when diagnosis \
                     is ambiguous"
                    .to_string(),
            },
            Biomarker {
                name: "Drug Trough Levels (Infliximab/Adalimumab)".to_string(),
                biomarker_type: BiomarkerType::Pharmacodynamic,
                clinical_use: "Therapeutic drug monitoring guides dose optimization; infliximab \
                     trough >5 µg/mL associated with mucosal healing; anti-drug antibodies \
                     signal loss of response"
                    .to_string(),
            },
            Biomarker {
                name: "Thiopurine Methyltransferase (TPMT) / NUDT15".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use:
                    "Pharmacogenomic testing before azathioprine/6-MP initiation; TPMT or \
                     NUDT15 deficiency predicts life-threatening myelosuppression; guides \
                     dose reduction or alternative selection"
                        .to_string(),
            },
        ],
    }
}
