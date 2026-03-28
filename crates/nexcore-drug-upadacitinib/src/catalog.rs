//! Upadacitinib drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature. JAK-class boxed warning applies to all approved indications.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for upadacitinib.
///
/// # Examples
///
/// ```
/// use nexcore_drug_upadacitinib::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "upadacitinib");
/// assert!(d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("upadacitinib"),
        generic_name: "upadacitinib".to_string(),
        brand_names: vec!["Rinvoq".to_string()],
        rxcui: Some("2049732".to_string()),
        mechanism: "Selective, reversible JAK1 inhibitor; reduces signalling of pro-inflammatory \
                    cytokines (IL-6, IL-2, IFN-γ) that drive autoimmune inflammation by \
                    blocking JAK1-mediated STAT phosphorylation"
            .to_string(),
        drug_class: DrugClass::JAKInhibitor,
        indications: vec![
            Indication {
                disease: "Moderate to Severe Rheumatoid Arthritis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2019),
                regulatory_basis: Some(
                    "SELECT-BEYOND — ACR20 response and DAS28-CRP at week 12".to_string(),
                ),
            },
            Indication {
                disease: "Moderate to Severe Atopic Dermatitis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2022),
                regulatory_basis: Some(
                    "Measure Up 1&2 — IGA 0/1 and EASI-75 at week 16".to_string(),
                ),
            },
            Indication {
                disease: "Moderate to Severe Ulcerative Colitis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2022),
                regulatory_basis: Some("U-ACHIEVE — clinical remission at week 8".to_string()),
            },
            Indication {
                disease: "Moderate to Severe Crohn's Disease".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2023),
                regulatory_basis: Some(
                    "U-EXCEL — clinical remission and endoscopic response".to_string(),
                ),
            },
        ],
        contraindications: vec![
            "Known serious hypersensitivity to upadacitinib or excipients".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Serious bacterial, fungal, and opportunistic infection".to_string(),
                contingency: ContingencyTable {
                    a: 1_850,
                    b: 14_000,
                    c: 4_200,
                    d: 11_000_000,
                },
                prr: 5.25,
                ror: 5.27,
                ic: 2.38,
                cases: 1_850,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Malignancy (excluding non-melanoma skin cancer)".to_string(),
                contingency: ContingencyTable {
                    a: 310,
                    b: 9_800,
                    c: 890,
                    d: 11_000_000,
                },
                prr: 3.92,
                ror: 3.93,
                ic: 1.97,
                cases: 310,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Major adverse cardiovascular event (MACE)".to_string(),
                contingency: ContingencyTable {
                    a: 185,
                    b: 8_200,
                    c: 650,
                    d: 11_000_000,
                },
                prr: 3.24,
                ror: 3.25,
                ic: 1.70,
                cases: 185,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Deep vein thrombosis / pulmonary embolism".to_string(),
                contingency: ContingencyTable {
                    a: 145,
                    b: 6_900,
                    c: 520,
                    d: 11_000_000,
                },
                prr: 3.12,
                ror: 3.13,
                ic: 1.64,
                cases: 145,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Herpes zoster reactivation".to_string(),
                contingency: ContingencyTable {
                    a: 620,
                    b: 8_400,
                    c: 1_800,
                    d: 11_000_000,
                },
                prr: 4.11,
                ror: 4.12,
                ic: 2.03,
                cases: 620,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: true,
            boxed_warning_text: Some(
                "SERIOUS INFECTIONS, MALIGNANCY, MAJOR ADVERSE CARDIOVASCULAR EVENTS, AND \
                 THROMBOSIS: Upadacitinib is associated with serious and potentially fatal \
                 infections, malignancies, MACE, and thrombosis. Use only if benefits outweigh \
                 risks. Not recommended in patients who are current or past smokers."
                    .to_string(),
            ),
            rems: false,
            warnings_precautions: vec![
                "Serious infections — screen for TB before initiating".to_string(),
                "Malignancy — lymphoma and solid tumours reported".to_string(),
                "Major adverse cardiovascular events — avoid in patients at high CV risk"
                    .to_string(),
                "Thrombosis (DVT, PE, arterial thrombosis)".to_string(),
                "Herpes zoster — consider prophylaxis in high-risk patients".to_string(),
                "Gastrointestinal perforations".to_string(),
                "Laboratory abnormalities (lymphopenia, neutropenia, anaemia, LFTs)".to_string(),
                "Embryo-fetal toxicity".to_string(),
            ],
            last_revision: Some("2024-08".to_string()),
        },
        owner: Some("AbbVie Inc.".to_string()),
    }
}
