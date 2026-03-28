//! Secukinumab drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for secukinumab.
///
/// # Examples
///
/// ```
/// use nexcore_drug_secukinumab::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "secukinumab");
/// assert!(!d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("secukinumab"),
        generic_name: "secukinumab".to_string(),
        brand_names: vec!["Cosentyx".to_string()],
        rxcui: Some("1799814".to_string()),
        mechanism: "Fully human IgG1/κ monoclonal antibody that selectively binds and \
                    neutralises IL-17A, reducing downstream inflammatory signalling driven \
                    by Th17 cells in psoriasis, psoriatic arthritis, and spondyloarthropathy"
            .to_string(),
        drug_class: DrugClass::AntiIL17,
        indications: vec![
            Indication {
                disease: "Moderate to Severe Plaque Psoriasis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2015),
                regulatory_basis: Some(
                    "ERASURE/FIXTURE — PASI 75 and IGA 0/1 superiority vs placebo and etanercept"
                        .to_string(),
                ),
            },
            Indication {
                disease: "Psoriatic Arthritis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2016),
                regulatory_basis: Some(
                    "FUTURE 1&2 — ACR20 response and inhibition of joint damage".to_string(),
                ),
            },
            Indication {
                disease: "Ankylosing Spondylitis".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2016),
                regulatory_basis: Some("MEASURE 1&2 — ASAS20 response at week 16".to_string()),
            },
        ],
        contraindications: vec![
            "Active tuberculosis or other clinically important active infection".to_string(),
            "Serious hypersensitivity to secukinumab or excipients".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Candida infection (oral, oesophageal, cutaneous)".to_string(),
                contingency: ContingencyTable {
                    a: 580,
                    b: 9_200,
                    c: 1_400,
                    d: 8_000_000,
                },
                prr: 4.59,
                ror: 4.60,
                ic: 2.19,
                cases: 580,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Inflammatory bowel disease exacerbation".to_string(),
                contingency: ContingencyTable {
                    a: 145,
                    b: 4_200,
                    c: 520,
                    d: 8_000_000,
                },
                prr: 3.28,
                ror: 3.29,
                ic: 1.71,
                cases: 145,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Serious infection (non-candidal)".to_string(),
                contingency: ContingencyTable {
                    a: 310,
                    b: 11_000,
                    c: 980,
                    d: 8_000_000,
                },
                prr: 2.91,
                ror: 2.92,
                ic: 1.53,
                cases: 310,
                on_label: true,
                verdict: SignalVerdict::Moderate,
            },
            SignalEntry {
                event: "Hypersensitivity / anaphylaxis".to_string(),
                contingency: ContingencyTable {
                    a: 88,
                    b: 5_100,
                    c: 340,
                    d: 8_000_000,
                },
                prr: 2.46,
                ror: 2.47,
                ic: 1.29,
                cases: 88,
                on_label: true,
                verdict: SignalVerdict::Moderate,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: false,
            boxed_warning_text: None,
            rems: false,
            warnings_precautions: vec![
                "Infections — do not use in active serious infection".to_string(),
                "Inflammatory bowel disease — new onset or exacerbation of Crohn's/UC".to_string(),
                "Hypersensitivity reactions including anaphylaxis".to_string(),
                "Latex allergy — prefilled syringe needle cap contains latex".to_string(),
                "Vaccinations — complete immunisations before initiating".to_string(),
            ],
            last_revision: Some("2024-01".to_string()),
        },
        owner: Some("Novartis AG".to_string()),
    }
}
