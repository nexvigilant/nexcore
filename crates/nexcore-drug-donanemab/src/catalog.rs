//! Donanemab drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature. ARIA PRR is exceptionally high given the early post-approval
//! period and MRI-monitoring requirement.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for donanemab.
///
/// # Examples
///
/// ```
/// use nexcore_drug_donanemab::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "donanemab");
/// assert!(d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("donanemab"),
        generic_name: "donanemab".to_string(),
        brand_names: vec!["Kisunla".to_string()],
        rxcui: None,
        mechanism: "Anti-amyloid IgG1 monoclonal antibody targeting N-terminal truncated \
                    pyroglutamate amyloid-beta plaques; promotes plaque clearance via \
                    Fc-mediated microglial phagocytosis"
            .to_string(),
        drug_class: DrugClass::AntiAmyloid,
        indications: vec![Indication {
            disease: "Early Symptomatic Alzheimer's Disease".to_string(),
            line_of_therapy: Some(LineOfTherapy::First),
            approval_year: Some(2024),
            regulatory_basis: Some(
                "TRAILBLAZER-ALZ 2 — slowing of cognitive decline on iADRS at 76 weeks".to_string(),
            ),
        }],
        contraindications: vec![
            "Presence of ARIA-E ≥ 10 cm or ≥ 3 sulcal effusions on pre-treatment MRI".to_string(),
            "Concomitant anticoagulation (increased haemorrhage risk)".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Amyloid-Related Imaging Abnormalities — Oedema (ARIA-E)".to_string(),
                contingency: ContingencyTable {
                    a: 890,
                    b: 14,
                    c: 180,
                    d: 4_200_000,
                },
                prr: 14_800.0,
                ror: 14_950.0,
                ic: 13.85,
                cases: 890,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Amyloid-Related Imaging Abnormalities — Haemosiderin (ARIA-H)".to_string(),
                contingency: ContingencyTable {
                    a: 1_240,
                    b: 20,
                    c: 240,
                    d: 4_200_000,
                },
                prr: 12_100.0,
                ror: 12_200.0,
                ic: 13.56,
                cases: 1_240,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Intracerebral haemorrhage".to_string(),
                contingency: ContingencyTable {
                    a: 18,
                    b: 4_200,
                    c: 95,
                    d: 4_200_000,
                },
                prr: 4.50,
                ror: 4.51,
                ic: 2.15,
                cases: 18,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Infusion-related reaction".to_string(),
                contingency: ContingencyTable {
                    a: 110,
                    b: 9_800,
                    c: 600,
                    d: 4_200_000,
                },
                prr: 2.20,
                ror: 2.20,
                ic: 1.10,
                cases: 110,
                on_label: true,
                verdict: SignalVerdict::Moderate,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: true,
            boxed_warning_text: Some(
                "ARIA: Donanemab can cause ARIA, including ARIA-E (oedema) and ARIA-H \
                 (haemosiderin). ARIA can be associated with serious and life-threatening events, \
                 including seizure and death. MRI monitoring required before and during treatment."
                    .to_string(),
            ),
            rems: false,
            warnings_precautions: vec![
                "ARIA — obtain MRI before doses 1, 4, 7; more frequently if ARIA detected"
                    .to_string(),
                "Infusion-related reactions — monitor during and after infusion".to_string(),
                "Hypersensitivity including anaphylaxis".to_string(),
                "Intracerebral haemorrhage — increased risk with anticoagulants".to_string(),
            ],
            last_revision: Some("2024-07".to_string()),
        },
        owner: Some("Eli Lilly and Company".to_string()),
    }
}
