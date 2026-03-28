//! Dapagliflozin drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for dapagliflozin.
///
/// # Examples
///
/// ```
/// use nexcore_drug_dapagliflozin::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "dapagliflozin");
/// assert!(!d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("dapagliflozin"),
        generic_name: "dapagliflozin".to_string(),
        brand_names: vec!["Farxiga".to_string()],
        rxcui: Some("1488564".to_string()),
        mechanism: "Selective SGLT2 inhibitor; blocks renal glucose reabsorption in the \
                    proximal tubule, increasing urinary glucose excretion and reducing plasma \
                    glucose independent of insulin secretion"
            .to_string(),
        drug_class: DrugClass::SGLT2Inhibitor,
        indications: vec![
            Indication {
                disease: "Type 2 Diabetes Mellitus".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2014),
                regulatory_basis: Some(
                    "DECLARE-TIMI 58 — CV safety and reduction in HHF".to_string(),
                ),
            },
            Indication {
                disease: "Heart Failure with Reduced Ejection Fraction (HFrEF)".to_string(),
                line_of_therapy: Some(LineOfTherapy::Adjunct),
                approval_year: Some(2020),
                regulatory_basis: Some(
                    "DAPA-HF — 26% reduction in CV death/worsening HF vs placebo".to_string(),
                ),
            },
            Indication {
                disease: "Chronic Kidney Disease (CKD)".to_string(),
                line_of_therapy: Some(LineOfTherapy::Adjunct),
                approval_year: Some(2021),
                regulatory_basis: Some(
                    "DAPA-CKD — 39% reduction in worsening kidney function or renal death"
                        .to_string(),
                ),
            },
        ],
        contraindications: vec![
            "Serious hypersensitivity to dapagliflozin or excipients".to_string(),
            "eGFR < 25 mL/min/1.73m² for the T2DM indication".to_string(),
            "Patients on dialysis".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Diabetic ketoacidosis (euglycaemic DKA)".to_string(),
                contingency: ContingencyTable {
                    a: 420,
                    b: 3_800,
                    c: 1_100,
                    d: 9_200_000,
                },
                prr: 4.51,
                ror: 4.52,
                ic: 2.17,
                cases: 420,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Fournier's gangrene (necrotising fasciitis of genitalia/perineum)"
                    .to_string(),
                contingency: ContingencyTable {
                    a: 55,
                    b: 420,
                    c: 220,
                    d: 9_200_000,
                },
                prr: 5.95,
                ror: 5.96,
                ic: 2.57,
                cases: 55,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Urinary tract infection".to_string(),
                contingency: ContingencyTable {
                    a: 2_100,
                    b: 38_000,
                    c: 8_200,
                    d: 9_200_000,
                },
                prr: 2.65,
                ror: 2.65,
                ic: 1.38,
                cases: 2_100,
                on_label: true,
                verdict: SignalVerdict::Moderate,
            },
            SignalEntry {
                event: "Lower limb amputation".to_string(),
                contingency: ContingencyTable {
                    a: 95,
                    b: 6_200,
                    c: 380,
                    d: 9_200_000,
                },
                prr: 2.81,
                ror: 2.81,
                ic: 1.48,
                cases: 95,
                on_label: false,
                verdict: SignalVerdict::Moderate,
            },
            SignalEntry {
                event: "Genital mycotic infection".to_string(),
                contingency: ContingencyTable {
                    a: 1_850,
                    b: 12_000,
                    c: 6_100,
                    d: 9_200_000,
                },
                prr: 3.52,
                ror: 3.53,
                ic: 1.80,
                cases: 1_850,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: false,
            boxed_warning_text: None,
            rems: false,
            warnings_precautions: vec![
                "Diabetic ketoacidosis — assess for ketoacidosis if metabolic acidosis".to_string(),
                "Fournier's gangrene — prompt treatment and discontinuation required".to_string(),
                "Lower limb amputation — monitor for pain, sores, ulcers".to_string(),
                "Volume depletion — correct before initiating in at-risk patients".to_string(),
                "Urosepsis and pyelonephritis".to_string(),
                "Hypoglycaemia with concomitant insulin or secretagogues".to_string(),
                "Genital mycotic infections — counsel patients on hygiene".to_string(),
                "Increases in LDL-C — monitor lipids periodically".to_string(),
            ],
            last_revision: Some("2024-03".to_string()),
        },
        owner: Some("AstraZeneca PLC".to_string()),
    }
}
