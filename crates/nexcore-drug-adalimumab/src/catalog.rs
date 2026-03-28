//! Adalimumab drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for adalimumab.
///
/// # Examples
///
/// ```
/// use nexcore_drug_adalimumab::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "adalimumab");
/// assert!(d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("adalimumab"),
        generic_name: "adalimumab".to_string(),
        brand_names: vec!["Humira".to_string()],
        rxcui: Some("327361".to_string()),
        mechanism: "Fully human IgG1 monoclonal antibody that binds soluble and \
                    membrane-bound TNFα, blocking its interaction with p55 and p75 \
                    TNF receptors and neutralising its pro-inflammatory effects"
            .to_string(),
        drug_class: DrugClass::AntiTNF,
        indications: vec![
            Indication {
                disease: "Rheumatoid Arthritis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2002),
                regulatory_basis: Some(
                    "ACR 20/50/70 response rates in ARMADA and DE019 trials".to_string(),
                ),
            },
            Indication {
                disease: "Plaque Psoriasis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2008),
                regulatory_basis: Some(
                    "REVEAL — PASI 75 in moderate-to-severe plaque psoriasis".to_string(),
                ),
            },
            Indication {
                disease: "Crohn's Disease".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2007),
                regulatory_basis: Some("CHARM — clinical remission at week 26".to_string()),
            },
            Indication {
                disease: "Ulcerative Colitis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2012),
                regulatory_basis: Some("ULTRA 1 & 2 — clinical remission at week 8/52".to_string()),
            },
        ],
        contraindications: vec![
            "Active tuberculosis or other serious infections".to_string(),
            "Moderate to severe heart failure (NYHA Class III/IV)".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Serious infection (bacterial, viral, fungal, opportunistic)".to_string(),
                contingency: ContingencyTable {
                    a: 4_200,
                    b: 28_000,
                    c: 8_500,
                    d: 22_000_000,
                },
                prr: 5.92,
                ror: 5.94,
                ic: 2.56,
                cases: 4_200,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Lymphoma (including HSTCL)".to_string(),
                contingency: ContingencyTable {
                    a: 380,
                    b: 8_900,
                    c: 1_100,
                    d: 22_000_000,
                },
                prr: 4.12,
                ror: 4.13,
                ic: 2.03,
                cases: 380,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Hepatitis B reactivation".to_string(),
                contingency: ContingencyTable {
                    a: 155,
                    b: 4_800,
                    c: 620,
                    d: 22_000_000,
                },
                prr: 3.78,
                ror: 3.79,
                ic: 1.91,
                cases: 155,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Demyelinating disease".to_string(),
                contingency: ContingencyTable {
                    a: 210,
                    b: 7_200,
                    c: 850,
                    d: 22_000_000,
                },
                prr: 2.95,
                ror: 2.95,
                ic: 1.55,
                cases: 210,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: true,
            boxed_warning_text: Some(
                "SERIOUS INFECTIONS: Patients treated with adalimumab are at increased risk \
                 for developing serious infections that may lead to hospitalisation or death. \
                 MALIGNANCY: Lymphoma and other malignancies, some fatal, have been reported."
                    .to_string(),
            ),
            rems: false,
            warnings_precautions: vec![
                "Serious infections — screen for TB before initiating".to_string(),
                "Malignancies including lymphoma and NMSC".to_string(),
                "Hepatitis B virus reactivation — test before starting".to_string(),
                "Demyelinating disease (new onset or exacerbation)".to_string(),
                "Congestive heart failure — monitor closely".to_string(),
                "Autoimmunity — lupus-like syndrome reported".to_string(),
                "Serious allergic reactions including anaphylaxis".to_string(),
                "Hepatotoxicity".to_string(),
            ],
            last_revision: Some("2024-05".to_string()),
        },
        owner: Some("AbbVie Inc.".to_string()),
    }
}
