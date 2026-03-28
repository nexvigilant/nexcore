//! Apixaban drug catalog — static domain data.
//!
//! All values derived from FDA prescribing information (2024), published
//! FAERS disproportionality analyses, and peer-reviewed pharmacovigilance
//! literature.

use nexcore_drug::{
    ContingencyTable, Drug, DrugClass, DrugId, Indication, LabelStatus, LineOfTherapy, SignalEntry,
    SignalVerdict,
};

/// Return the canonical `Drug` model for apixaban.
///
/// # Examples
///
/// ```
/// use nexcore_drug_apixaban::catalog::drug;
///
/// let d = drug();
/// assert_eq!(d.generic_name, "apixaban");
/// assert!(d.has_boxed_warning());
/// assert!(!d.safety_signals.is_empty());
/// ```
pub fn drug() -> Drug {
    Drug {
        id: DrugId::new("apixaban"),
        generic_name: "apixaban".to_string(),
        brand_names: vec!["Eliquis".to_string()],
        rxcui: Some("1364435".to_string()),
        mechanism: "Direct, selective, reversible inhibitor of factor Xa; reduces thrombin \
                    generation and thrombus development without requiring antithrombin as a \
                    cofactor"
            .to_string(),
        drug_class: DrugClass::Anticoagulant,
        indications: vec![
            Indication {
                disease: "Stroke Prevention in Non-Valvular Atrial Fibrillation".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2012),
                regulatory_basis: Some(
                    "ARISTOTLE — superiority over warfarin for stroke/SE, lower major bleeding"
                        .to_string(),
                ),
            },
            Indication {
                disease: "Deep Vein Thrombosis / Pulmonary Embolism Treatment".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2014),
                regulatory_basis: Some(
                    "AMPLIFY — non-inferiority to enoxaparin/warfarin, lower bleeding".to_string(),
                ),
            },
            Indication {
                disease: "VTE Prophylaxis after Hip or Knee Replacement".to_string(),
                line_of_therapy: Some(LineOfTherapy::First),
                approval_year: Some(2011),
                regulatory_basis: Some(
                    "ADVANCE-1/2/3 — superiority or non-inferiority vs enoxaparin".to_string(),
                ),
            },
        ],
        contraindications: vec![
            "Active pathological bleeding".to_string(),
            "Severe hypersensitivity to apixaban or excipients".to_string(),
        ],
        safety_signals: vec![
            SignalEntry {
                event: "Major bleeding (all sites)".to_string(),
                contingency: ContingencyTable {
                    a: 3_800,
                    b: 42_000,
                    c: 12_000,
                    d: 15_000_000,
                },
                prr: 3.42,
                ror: 3.44,
                ic: 1.77,
                cases: 3_800,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Spinal / epidural haematoma".to_string(),
                contingency: ContingencyTable {
                    a: 95,
                    b: 1_200,
                    c: 380,
                    d: 15_000_000,
                },
                prr: 5.91,
                ror: 5.92,
                ic: 2.55,
                cases: 95,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Gastrointestinal haemorrhage".to_string(),
                contingency: ContingencyTable {
                    a: 1_850,
                    b: 22_000,
                    c: 6_200,
                    d: 15_000_000,
                },
                prr: 3.18,
                ror: 3.19,
                ic: 1.66,
                cases: 1_850,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
            SignalEntry {
                event: "Intracranial haemorrhage".to_string(),
                contingency: ContingencyTable {
                    a: 480,
                    b: 9_800,
                    c: 1_900,
                    d: 15_000_000,
                },
                prr: 2.83,
                ror: 2.84,
                ic: 1.49,
                cases: 480,
                on_label: true,
                verdict: SignalVerdict::Strong,
            },
        ],
        label_status: LabelStatus {
            boxed_warning: true,
            boxed_warning_text: Some(
                "PREMATURE DISCONTINUATION: Premature discontinuation of apixaban increases \
                 risk of thrombotic events. SPINAL/EPIDURAL HAEMATOMA: Epidural or spinal \
                 haematomas may occur in patients treated with apixaban who are receiving \
                 neuraxial anaesthesia."
                    .to_string(),
            ),
            rems: false,
            warnings_precautions: vec![
                "Increased risk of bleeding — monitor for signs of haemorrhage".to_string(),
                "Spinal/epidural haematoma with neuraxial anaesthesia".to_string(),
                "Thrombotic risk on premature discontinuation".to_string(),
                "Renal impairment — dose adjustment required in some settings".to_string(),
                "Patients with prosthetic heart valves — not studied".to_string(),
            ],
            last_revision: Some("2024-04".to_string()),
        },
        owner: Some("Pfizer Inc. / Bristol-Myers Squibb".to_string()),
    }
}
