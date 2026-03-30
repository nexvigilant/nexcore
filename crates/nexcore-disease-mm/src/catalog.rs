//! Catalog data for Multiple Myeloma.
//!
//! Sources: NCCN Guidelines Multiple Myeloma v1.2024, IMWG Consensus Criteria,
//! FDA approval documents for IMiDs/PIs/mAbs/BCMA-directed agents,
//! IFM/MAIA/CASSIOPEIA/KRd/POLLUX/CASTOR/KarMMa/CARTITUDE trial data.

use nexcore_disease::{
    Biomarker, BiomarkerType, ClassEffect, Demographics, Disease, DiseaseId, Epidemiology,
    EvidenceLevel, LineOfTherapy, NeedSeverity, SafetyBurden, TherapeuticArea, TreatmentLine,
    Trend, UnmetNeed,
};

/// Returns the canonical Multiple Myeloma disease model.
pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("mm"),
        name: "Multiple Myeloma".to_string(),
        icd10_codes: vec![
            "C90".to_string(),
            "C90.0".to_string(),
            "C90.00".to_string(),
            "C90.01".to_string(),
            "C90.02".to_string(),
        ],
        therapeutic_area: TherapeuticArea::Hematology,
        epidemiology: Epidemiology {
            global_prevalence: Some(0.014),
            us_prevalence: Some(0.024),
            annual_incidence: Some(8.1),
            demographics: Demographics {
                median_age_onset: Some(69),
                sex_ratio: Some("1.4:1 M:F".to_string()),
                risk_factors: vec![
                    "Age ≥65 years (median diagnosis age 69)".to_string(),
                    "Black/African American race (2x higher incidence)".to_string(),
                    "Monoclonal gammopathy of undetermined significance (MGUS)".to_string(),
                    "Smoldering multiple myeloma progression".to_string(),
                    "Obesity".to_string(),
                    "Radiation exposure".to_string(),
                    "Agricultural chemical exposure (pesticides, herbicides)".to_string(),
                    "Family history of plasma cell dyscrasias".to_string(),
                ],
            },
            trend: Trend::Stable,
        },
        standard_of_care: vec![
            TreatmentLine {
                line: LineOfTherapy::First,
                drug_classes: vec![
                    "Immunomodulatory Drugs (IMiDs)".to_string(),
                    "Proteasome Inhibitors (PIs)".to_string(),
                    "Anti-CD38 Monoclonal Antibodies".to_string(),
                    "Corticosteroids".to_string(),
                    "Autologous Stem Cell Transplantation (ASCT)".to_string(),
                ],
                representative_drugs: vec![
                    "lenalidomide".to_string(),
                    "bortezomib".to_string(),
                    "dexamethasone".to_string(),
                    "daratumumab".to_string(),
                    "isatuximab".to_string(),
                    "carfilzomib".to_string(),
                    "thalidomide".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Second,
                drug_classes: vec![
                    "Immunomodulatory Drugs (IMiDs)".to_string(),
                    "Proteasome Inhibitors (PIs)".to_string(),
                    "Anti-CD38 Monoclonal Antibodies".to_string(),
                    "Anti-SLAMF7 Monoclonal Antibodies".to_string(),
                ],
                representative_drugs: vec![
                    "pomalidomide".to_string(),
                    "carfilzomib".to_string(),
                    "ixazomib".to_string(),
                    "daratumumab".to_string(),
                    "elotuzumab".to_string(),
                    "dexamethasone".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
            TreatmentLine {
                line: LineOfTherapy::Third,
                drug_classes: vec![
                    "BCMA-Directed Therapies".to_string(),
                    "CAR-T Cell Therapy".to_string(),
                    "Antibody-Drug Conjugates (ADCs)".to_string(),
                    "Bispecific Antibodies".to_string(),
                    "Nuclear Export Inhibitors".to_string(),
                ],
                representative_drugs: vec![
                    "idecabtagene vicleucel (ide-cel)".to_string(),
                    "ciltacabtagene autoleucel (cilta-cel)".to_string(),
                    "belantamab mafodotin".to_string(),
                    "teclistamab".to_string(),
                    "elranatamab".to_string(),
                    "selinexor".to_string(),
                ],
                evidence_level: EvidenceLevel::IB,
            },
            TreatmentLine {
                line: LineOfTherapy::Supportive,
                drug_classes: vec![
                    "Bisphosphonates / RANK-L Inhibitors".to_string(),
                    "Erythropoiesis-Stimulating Agents".to_string(),
                    "Antithrombotic Prophylaxis".to_string(),
                ],
                representative_drugs: vec![
                    "zoledronic acid".to_string(),
                    "denosumab".to_string(),
                    "darbepoetin alfa".to_string(),
                    "aspirin".to_string(),
                    "enoxaparin".to_string(),
                ],
                evidence_level: EvidenceLevel::IA,
            },
        ],
        unmet_needs: vec![
            UnmetNeed {
                description: "Functional cure — sustained deep remission off therapy".to_string(),
                severity: NeedSeverity::Critical,
                current_gap:
                    "MM remains incurable in the vast majority; median OS has improved to ~10 \
                     years with triplet/quadruplet therapy but relapse is near-universal; \
                     MRD negativity does not reliably translate to long-term cure"
                        .to_string(),
                potential_approaches: vec![
                    "BCMA-directed CAR-T with improved persistence and memory phenotype"
                        .to_string(),
                    "Allogeneic off-the-shelf CAR-T platforms for universal access".to_string(),
                    "Quadruplet induction + consolidation targeting sustained MRD negativity"
                        .to_string(),
                ],
            },
            UnmetNeed {
                description:
                    "Access to novel BCMA-directed therapies for triple-refractory patients"
                        .to_string(),
                severity: NeedSeverity::High,
                current_gap: "CAR-T manufacturing lead time 4-6 weeks limits access for rapidly \
                     progressing patients; leukapheresis failure rate ~10%; REMS complexity \
                     restricts certified center availability"
                    .to_string(),
                potential_approaches: vec![
                    "Bispecific antibodies (teclistamab, elranatamab) as off-the-shelf BCMA \
                     option"
                        .to_string(),
                    "Next-generation CAR-T with shorter manufacturing timelines (<2 weeks)"
                        .to_string(),
                ],
            },
            UnmetNeed {
                description:
                    "Effective treatment for high-risk cytogenetic MM (del17p, t(4;14), t(14;16))"
                        .to_string(),
                severity: NeedSeverity::High,
                current_gap:
                    "High-risk cytogenetics confer median OS <3 years despite modern therapy; \
                     del(17p) TP53 loss drives PI/IMiD resistance; no approved agent \
                     specifically targeting TP53-deleted plasma cells"
                        .to_string(),
                potential_approaches: vec![
                    "Venetoclax for t(11;14) BCL-2-overexpressing MM subgroup".to_string(),
                    "TP53-independent apoptosis induction via MDM2 inhibitors".to_string(),
                    "Consolidation with tandem ASCT for del(17p) patients".to_string(),
                ],
            },
        ],
        safety_burden: SafetyBurden {
            total_drugs_approved: 18,
            drugs_with_boxed_warnings: 8,
            drugs_with_rems: 3,
            class_effects: vec![
                ClassEffect {
                    drug_class: "Immunomodulatory Drugs — Lenalidomide/Thalidomide/Pomalidomide"
                        .to_string(),
                    event: "Teratogenicity (Category X), venous thromboembolism, peripheral \
                            neuropathy (thalidomide), second primary malignancy risk"
                        .to_string(),
                    evidence_strength:
                        "Boxed warning; REMS required (THALOMID REMS, REVLIMID REMS, \
                         POMALYST REMS) — mandatory negative pregnancy test and contraception"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "Proteasome Inhibitors".to_string(),
                    event: "Peripheral neuropathy (bortezomib IV > SC), thrombocytopenia, \
                            herpes zoster reactivation, cardiac events (carfilzomib — \
                            hypertension, cardiomyopathy)"
                        .to_string(),
                    evidence_strength:
                        "Bortezomib: neuropathy rate ~35% (reduced with SC administration); \
                         Carfilzomib: cardiac events ~25% in ASPIRE trial; VZV prophylaxis \
                         mandatory"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "Anti-CD38 Monoclonal Antibodies (Daratumumab/Isatuximab)"
                        .to_string(),
                    event: "Infusion-related reactions (IRR), interference with blood bank \
                            crossmatch testing (CD38 expression on RBCs), increased infection \
                            risk (herpes zoster, opportunistic)"
                        .to_string(),
                    evidence_strength:
                        "IRR rate ~48% (daratumumab first infusion); blood bank must be \
                         notified; type-and-screen assay interference persists for up to 6 months"
                            .to_string(),
                },
                ClassEffect {
                    drug_class: "BCMA-Directed CAR-T Therapies".to_string(),
                    event: "Cytokine release syndrome (CRS), immune effector cell-associated \
                            neurotoxicity syndrome (ICANS), prolonged cytopenias, \
                            hypogammaglobulinemia"
                        .to_string(),
                    evidence_strength:
                        "Boxed warning; REMS required for both ide-cel and cilta-cel; Grade \
                         ≥3 CRS ~5-7%; ICANS ~3-5%; median time to recovery of neutrophils \
                         ~1 month"
                            .to_string(),
                },
            ],
            notable_withdrawals: vec![],
        },
        biomarkers: vec![
            Biomarker {
                name: "Serum Protein Electrophoresis (SPEP) / M-protein".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use:
                    "Primary diagnostic and response monitoring marker; M-protein quantity \
                     defines disease burden; <5 g/dL on SPEP criteria used for IMWG \
                     response categories (PR, VGPR, CR, sCR)"
                        .to_string(),
            },
            Biomarker {
                name: "Free Light Chains (FLC) — Kappa/Lambda ratio".to_string(),
                biomarker_type: BiomarkerType::Diagnostic,
                clinical_use: "Essential for non-secretory MM and light chain disease monitoring; \
                     abnormal ratio (>100 involved FLC) is CRAB-independent myeloma-defining \
                     event; serial monitoring tracks treatment response"
                    .to_string(),
            },
            Biomarker {
                name: "Beta-2 Microglobulin (β2M)".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use: "Core component of R-ISS staging (Stage I: β2M <3.5 + albumin ≥3.5; \
                     Stage III: β2M ≥5.5); elevated levels predict shorter PFS and OS; \
                     also reflects renal function"
                    .to_string(),
            },
            Biomarker {
                name: "Cytogenetic Profile (FISH: del17p, t(4;14), t(14;16), amp1q)".to_string(),
                biomarker_type: BiomarkerType::Predictive,
                clinical_use:
                    "High-risk cytogenetics guide therapy intensity; del(17p)/TP53 loss and \
                     t(4;14) associate with PI resistance; t(11;14)/BCL-2 overexpression \
                     predicts venetoclax sensitivity; guides maintenance duration decisions"
                        .to_string(),
            },
            Biomarker {
                name: "Minimal Residual Disease (MRD) by Next-Generation Sequencing/Flow"
                    .to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use:
                    "MRD negativity (sensitivity 10^-5 to 10^-6) associated with prolonged \
                     PFS and OS; IMWG-validated endpoint for clinical trials; \
                     sustained MRD negativity emerging as surrogate for functional cure"
                        .to_string(),
            },
            Biomarker {
                name: "LDH (Lactate Dehydrogenase)".to_string(),
                biomarker_type: BiomarkerType::Prognostic,
                clinical_use:
                    "Elevated LDH (>ULN) is R-ISS Stage III criterion alongside del(17p) or \
                     t(4;14) or t(14;16); reflects aggressive extramedullary disease; \
                     guides risk stratification and transplant eligibility"
                        .to_string(),
            },
            Biomarker {
                name: "Serum Calcium".to_string(),
                biomarker_type: BiomarkerType::Safety,
                clinical_use: "Hypercalcemia is a CRAB criterion (>11 mg/dL); monitored during \
                     bisphosphonate/denosumab therapy; denosumab cessation can trigger \
                     rebound hypercalcemia requiring clinical vigilance"
                    .to_string(),
            },
        ],
    }
}
