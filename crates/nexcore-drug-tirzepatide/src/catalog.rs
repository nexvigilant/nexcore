//! Tirzepatide drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for tirzepatide.
///
/// # Examples
///
/// ```
/// use nexcore_drug_tirzepatide::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "tirzepatide");
/// assert!(d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("tirzepatide"),
        generic_name: "tirzepatide".to_string(),
        brand_names: vec!["Mounjaro".to_string(), "Zepbound".to_string()],
        rxcui: Some("2200644".to_string()),
        mechanism: "Dual GLP-1 and GIP receptor agonist; reduces appetite, slows gastric \
                    emptying, and improves insulin sensitivity via two incretin pathways"
            .to_string(),
        drug_class: DrugClass::GLP1GIPDualAgonist,
        indications: vec![
            Indication {
                disease: "Type 2 Diabetes Mellitus".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2022),
                regulatory_basis: Some(
                    "SURPASS phase 3 programme — HbA1c reduction vs comparators".to_string(),
                ),
            },
            Indication {
                disease: "Obesity / Chronic Weight Management".to_string(),
                line_of_therapy: Some(LineOfTherapy::Adjunct),
                approval_year: Some(2023),
                regulatory_basis: Some(
                    "SURMOUNT-1 — 20.9% body-weight reduction vs placebo".to_string(),
                ),
            },
            Indication {
                disease: "Obstructive Sleep Apnoea".to_string(),
                line_of_therapy: Some(LineOfTherapy::Adjunct),
                approval_year: Some(2024),
                regulatory_basis: Some(
                    "SURMOUNT-OSA — AHI reduction in adults with obesity".to_string(),
                ),
            },
        ],
        contraindications: vec![
            "Personal or family history of medullary thyroid carcinoma (MTC)".to_string(),
            "Multiple Endocrine Neoplasia syndrome type 2 (MEN 2)".to_string(),
            "Prior serious hypersensitivity reaction to tirzepatide or excipients".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Pancreatitis".to_string(),
                contingency: ContingencyTable {
                    a: 156,
                    b: 8_200,
                    c: 890,
                    d: 9_800_000,
                },
                prr: 3.02,
                ror: 3.04,
                ic: 1.56,
                cases: 156,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Gastroparesis".to_string(),
                contingency: ContingencyTable {
                    a: 210,
                    b: 4_100,
                    c: 1_200,
                    d: 9_800_000,
                },
                prr: 4.18,
                ror: 4.20,
                ic: 2.03,
                cases: 210,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Alopecia".to_string(),
                contingency: ContingencyTable {
                    a: 88,
                    b: 6_500,
                    c: 500,
                    d: 9_800_000,
                },
                prr: 2.65,
                ror: 2.66,
                ic: 1.38,
                cases: 88,
                on_label: false,
                verdict: SignalVerdict::Moderate,
            },
            SignalEntry {
                event: "Thyroid C-cell tumour".to_string(),
                contingency: ContingencyTable {
                    a: 12,
                    b: 180,
                    c: 90,
                    d: 9_800_000,
                },
                prr: 2.11,
                ror: 2.11,
                ic: 1.06,
                cases: 12,
                on_label: true,
                verdict: SignalVerdict::Moderate,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: true,
            boxed_warning_text: Some(
                "THYROID C-CELL TUMORS: Tirzepatide causes thyroid C-cell tumors in rodents. \
                 Contraindicated in patients with a personal or family history of MTC or MEN 2."
                    .to_string(),
            ),
            rems: false,
            warnings_precautions: vec![
                "Acute pancreatitis — discontinue if suspected".to_string(),
                "Acute gallbladder disease".to_string(),
                "Hypoglycaemia with concomitant insulin secretagogues".to_string(),
                "Acute kidney injury (secondary to nausea/vomiting/diarrhoea)".to_string(),
                "Diabetic retinopathy complications".to_string(),
                "Heart rate increase".to_string(),
                "Suicidal behaviour and ideation (class monitoring requirement)".to_string(),
            ],
            last_revision: Some("2024-06".to_string()),
        },
        owner: Some("Eli Lilly and Company".to_string()),
    }
}
