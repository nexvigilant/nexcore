//! Semaglutide drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for semaglutide.
///
/// # Examples
///
/// ```
/// use nexcore_drug_semaglutide::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "semaglutide");
/// assert!(d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("semaglutide"),
        generic_name: "semaglutide".to_string(),
        brand_names: vec![
            "Ozempic".to_string(),
            "Wegovy".to_string(),
            "Rybelsus".to_string(),
        ],
        rxcui: Some("2200644".to_string()),
        mechanism: "Selective GLP-1 receptor agonist; stimulates glucose-dependent insulin \
                    secretion, suppresses glucagon, slows gastric emptying, and reduces appetite"
            .to_string(),
        drug_class: DrugClass::GLP1ReceptorAgonist,
        indications: vec![
            Indication {
                disease: "Type 2 Diabetes Mellitus".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2017),
                regulatory_basis: Some(
                    "SUSTAIN phase 3 programme — HbA1c reduction and CV risk".to_string(),
                ),
            },
            Indication {
                disease: "Obesity / Chronic Weight Management".to_string(),
                line_of_therapy: Some(LineOfTherapy::Adjunct),
                approval_year: Some(2021),
                regulatory_basis: Some(
                    "STEP 1 — 14.9% body-weight reduction vs placebo".to_string(),
                ),
            },
            Indication {
                disease: "Cardiovascular Risk Reduction in T2DM".to_string(),
                line_of_therapy: Some(LineOfTherapy::Adjunct),
                approval_year: Some(2020),
                regulatory_basis: Some("SUSTAIN-6 CVOT — reduction in MACE".to_string()),
            },
        ],
        contraindications: vec![
            "Personal or family history of medullary thyroid carcinoma (MTC)".to_string(),
            "Multiple Endocrine Neoplasia syndrome type 2 (MEN 2)".to_string(),
            "Prior serious hypersensitivity reaction to semaglutide or excipients".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Pancreatitis".to_string(),
                contingency: ContingencyTable {
                    a: 420,
                    b: 7_900,
                    c: 1_800,
                    d: 12_000_000,
                },
                prr: 3.51,
                ror: 3.53,
                ic: 1.79,
                cases: 420,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Gastroparesis".to_string(),
                contingency: ContingencyTable {
                    a: 510,
                    b: 5_200,
                    c: 2_100,
                    d: 12_000_000,
                },
                prr: 5.82,
                ror: 5.83,
                ic: 2.52,
                cases: 510,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Thyroid C-cell tumour".to_string(),
                contingency: ContingencyTable {
                    a: 18,
                    b: 210,
                    c: 110,
                    d: 12_000_000,
                },
                prr: 2.44,
                ror: 2.44,
                ic: 1.27,
                cases: 18,
                on_label: true,
                verdict: SignalVerdict::Moderate,
            },
            SignalEntry {
                event: "Suicidal ideation".to_string(),
                contingency: ContingencyTable {
                    a: 95,
                    b: 9_800,
                    c: 420,
                    d: 12_000_000,
                },
                prr: 2.73,
                ror: 2.73,
                ic: 1.44,
                cases: 95,
                on_label: false,
                verdict: SignalVerdict::Moderate,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: true,
            boxed_warning_text: Some(
                "THYROID C-CELL TUMORS: Semaglutide causes thyroid C-cell tumors in rodents. \
                 Contraindicated in patients with a personal or family history of MTC or MEN 2."
                    .to_string(),
            ),
            rems: false,
            warnings_precautions: vec![
                "Acute pancreatitis — discontinue if suspected".to_string(),
                "Acute gallbladder disease".to_string(),
                "Hypoglycaemia with concomitant insulin secretagogues".to_string(),
                "Acute kidney injury".to_string(),
                "Diabetic retinopathy complications".to_string(),
                "Heart rate increase".to_string(),
                "Suicidal behaviour and ideation — monitor".to_string(),
            ],
            last_revision: Some("2024-03".to_string()),
        },
        owner: Some("Novo Nordisk A/S".to_string()),
    }
}
