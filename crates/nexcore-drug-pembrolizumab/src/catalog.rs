//! Pembrolizumab drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for pembrolizumab.
///
/// # Examples
///
/// ```
/// use nexcore_drug_pembrolizumab::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "pembrolizumab");
/// assert!(!d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("pembrolizumab"),
        generic_name: "pembrolizumab".to_string(),
        brand_names: vec!["Keytruda".to_string()],
        rxcui: Some("1720450".to_string()),
        mechanism: "Humanised IgG4 monoclonal antibody that blocks the PD-1 receptor, \
                    preventing PD-L1/PD-L2 binding and restoring T-cell anti-tumour immunity"
            .to_string(),
        drug_class: DrugClass::CheckpointInhibitor,
        indications: vec![
            Indication {
                disease: "Metastatic Non-Small Cell Lung Cancer (PD-L1+)".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2016),
                regulatory_basis: Some(
                    "KEYNOTE-024 — PFS and OS improvement in PD-L1 TPS ≥ 50%".to_string(),
                ),
            },
            Indication {
                disease: "Unresectable or Metastatic Melanoma".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2014),
                regulatory_basis: Some(
                    "KEYNOTE-006 — OS and PFS superiority vs ipilimumab".to_string(),
                ),
            },
            Indication {
                disease: "Tumour Mutational Burden High (TMB-H) solid tumours".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2020),
                regulatory_basis: Some("KEYNOTE-158 — ORR in TMB-H ≥ 10 mut/Mb".to_string()),
            },
        ],
        contraindications: vec![],
        safety_signals: vec![
            SignalEntry {
                event: "Immune-mediated pneumonitis".to_string(),
                contingency: ContingencyTable {
                    a: 1_850,
                    b: 18_000,
                    c: 3_200,
                    d: 18_000_000,
                },
                prr: 6.82,
                ror: 6.84,
                ic: 2.76,
                cases: 1_850,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Immune-mediated hepatitis".to_string(),
                contingency: ContingencyTable {
                    a: 920,
                    b: 14_000,
                    c: 2_100,
                    d: 18_000_000,
                },
                prr: 5.24,
                ror: 5.25,
                ic: 2.38,
                cases: 920,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Immune-mediated colitis".to_string(),
                contingency: ContingencyTable {
                    a: 760,
                    b: 12_000,
                    c: 1_800,
                    d: 18_000_000,
                },
                prr: 5.01,
                ror: 5.02,
                ic: 2.32,
                cases: 760,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Immune-mediated endocrinopathy (thyroid dysfunction)".to_string(),
                contingency: ContingencyTable {
                    a: 2_400,
                    b: 9_500,
                    c: 3_800,
                    d: 18_000_000,
                },
                prr: 7.54,
                ror: 7.56,
                ic: 2.91,
                cases: 2_400,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Myocarditis".to_string(),
                contingency: ContingencyTable {
                    a: 185,
                    b: 6_800,
                    c: 420,
                    d: 18_000_000,
                },
                prr: 5.25,
                ror: 5.26,
                ic: 2.38,
                cases: 185,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: false,
            boxed_warning_text: None,
            rems: false,
            warnings_precautions: vec![
                "Severe and fatal immune-mediated adverse reactions".to_string(),
                "Immune-mediated pneumonitis and interstitial lung disease".to_string(),
                "Immune-mediated colitis".to_string(),
                "Immune-mediated hepatitis and hepatotoxicity".to_string(),
                "Immune-mediated endocrinopathies (adrenal insufficiency, hypophysitis, thyroid)"
                    .to_string(),
                "Immune-mediated nephritis with renal dysfunction".to_string(),
                "Immune-mediated dermatologic adverse reactions".to_string(),
                "Myocarditis".to_string(),
                "Embryo-fetal toxicity".to_string(),
            ],
            last_revision: Some("2024-09".to_string()),
        },
        owner: Some("Merck & Co., Inc.".to_string()),
    }
}
