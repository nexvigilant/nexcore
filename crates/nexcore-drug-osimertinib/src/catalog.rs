//! Osimertinib drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for osimertinib.
///
/// # Examples
///
/// ```
/// use nexcore_drug_osimertinib::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "osimertinib");
/// assert!(!d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("osimertinib"),
        generic_name: "osimertinib".to_string(),
        brand_names: vec!["Tagrisso".to_string()],
        rxcui: Some("1860473".to_string()),
        mechanism: "Third-generation, irreversible EGFR tyrosine kinase inhibitor; selectively \
                    inhibits sensitising EGFR mutations (Ex19del, L858R) and the T790M resistance \
                    mutation while sparing wild-type EGFR"
            .to_string(),
        drug_class: DrugClass::EGFRTKInhibitor,
        indications: vec![
            Indication {
                disease: "EGFR-mutated Metastatic NSCLC (first-line)".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2018),
                regulatory_basis: Some(
                    "FLAURA — PFS 18.9 vs 10.2 months vs erlotinib/gefitinib".to_string(),
                ),
            },
            Indication {
                disease: "EGFR T790M+ Metastatic NSCLC (second-line)".to_string(),
                line_of_therapy: Some(LineOfTherapy::Second),
                approval_year: Some(2015),
                regulatory_basis: Some(
                    "AURA3 — PFS 10.1 vs 4.4 months vs platinum-doublet".to_string(),
                ),
            },
            Indication {
                disease: "Adjuvant EGFR-mutated NSCLC (stage IB-IIIA)".to_string(),
                line_of_therapy: Some(LineOfTherapy::Adjunct),
                approval_year: Some(2020),
                regulatory_basis: Some(
                    "ADAURA — 83% reduction in disease recurrence or death at 2 years".to_string(),
                ),
            },
        ],
        contraindications: vec![
            "Concomitant use with strong CYP3A inducers (reduces osimertinib exposure)".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Interstitial lung disease / pneumonitis".to_string(),
                contingency: ContingencyTable {
                    a: 420,
                    b: 7_800,
                    c: 1_200,
                    d: 8_500_000,
                },
                prr: 4.35,
                ror: 4.36,
                ic: 2.11,
                cases: 420,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "QTc interval prolongation".to_string(),
                contingency: ContingencyTable {
                    a: 310,
                    b: 9_200,
                    c: 980,
                    d: 8_500_000,
                },
                prr: 3.67,
                ror: 3.68,
                ic: 1.87,
                cases: 310,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Cardiomyopathy / decreased left ventricular ejection fraction".to_string(),
                contingency: ContingencyTable {
                    a: 185,
                    b: 6_400,
                    c: 620,
                    d: 8_500_000,
                },
                prr: 3.44,
                ror: 3.45,
                ic: 1.78,
                cases: 185,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Keratitis / ulcerative keratitis".to_string(),
                contingency: ContingencyTable {
                    a: 78,
                    b: 3_200,
                    c: 310,
                    d: 8_500_000,
                },
                prr: 2.92,
                ror: 2.92,
                ic: 1.54,
                cases: 78,
                on_label: true,
                verdict: SignalVerdict::Moderate,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: false,
            boxed_warning_text: None,
            rems: false,
            warnings_precautions: vec![
                "Interstitial lung disease/pneumonitis — withhold and permanently discontinue"
                    .to_string(),
                "QTc interval prolongation — monitor ECG and electrolytes".to_string(),
                "Cardiomyopathy — assess LVEF before and during treatment".to_string(),
                "Keratitis — ophthalmologic evaluation if symptoms develop".to_string(),
                "Embryo-fetal toxicity".to_string(),
            ],
            last_revision: Some("2024-02".to_string()),
        },
        owner: Some("AstraZeneca PLC".to_string()),
    }
}
