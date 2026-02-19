//! Seed terms — the 10 foundational PV terms with authoritative definitions.
//!
//! Each term uses real definitions from ICH, WHO, and CIOMS sources.
//! The definitions use only words present in the decomposer's seed vocabulary
//! (or stop words filtered by the codec).

use crate::dictionary::DefinitionSource;

/// A seed term for the built-in dictionary.
pub struct SeedTerm {
    /// Term name.
    pub name: &'static str,
    /// Authoritative definition text.
    pub definition: &'static str,
    /// Source of the definition.
    pub source: DefinitionSource,
}

/// Returns the 10 foundational PV seed terms.
///
/// Each definition is derived from authoritative ICH/WHO sources and uses
/// only words covered by the decomposer seed vocabulary.
#[must_use]
pub fn seed_terms() -> Vec<SeedTerm> {
    vec![
        // 1. Pharmacovigilance (WHO definition)
        SeedTerm {
            name: "Pharmacovigilance",
            definition: "the science and activities relating to the detection assessment and prevention of adverse effects or any other drug related harm",
            source: DefinitionSource::WhoDrug("WHO Essential Medicines".to_string()),
        },
        // 2. Adverse Event (ICH E2A)
        SeedTerm {
            name: "Adverse Event",
            definition: "any undesirable medical occurrence associated with the use of a medicinal product whether or not regarded as related to the product",
            source: DefinitionSource::IchGuideline("ICH E2A".to_string()),
        },
        // 3. Signal (ICH E2E / CIOMS VIII)
        SeedTerm {
            name: "Signal",
            definition: "information arising from one or more sources including data generating a new hypothesis of a causal association between an adverse event and a drug",
            source: DefinitionSource::IchGuideline("ICH E2E".to_string()),
        },
        // 4. Causality Assessment (ICH E2A)
        SeedTerm {
            name: "Causality",
            definition: "the assessment of the relationship between drug treatment and the occurrence of an adverse event determining whether evidence exists for a causal association",
            source: DefinitionSource::IchGuideline("ICH E2A".to_string()),
        },
        // 5. Benefit-Risk (ICH E2C(R2))
        SeedTerm {
            name: "Benefit-Risk",
            definition: "the comparative evaluation of benefit and risk associated with the use of a drug weighing favorable and unfavorable effects",
            source: DefinitionSource::IchGuideline("ICH E2C(R2)".to_string()),
        },
        // 6. ICSR — Individual Case Safety Report (ICH E2B)
        SeedTerm {
            name: "ICSR",
            definition: "a standardized format for reporting individual case safety information describing a suspected adverse reaction associated with the use of a medicinal product",
            source: DefinitionSource::IchGuideline("ICH E2B".to_string()),
        },
        // 7. PSUR — Periodic Safety Update Report (ICH E2C)
        SeedTerm {
            name: "PSUR",
            definition: "a periodic comprehensive review of the worldwide safety data of a medicinal product including analysis of the overall safety profile",
            source: DefinitionSource::IchGuideline("ICH E2C".to_string()),
        },
        // 8. Risk Management Plan (ICH E2E)
        SeedTerm {
            name: "RMP",
            definition: "a document describing the risk management system including safety activities and pharmacovigilance plan outlining proposed measures to characterize and minimize risks",
            source: DefinitionSource::IchGuideline("ICH E2E".to_string()),
        },
        // 9. Serious Adverse Event (ICH E2A)
        SeedTerm {
            name: "SAE",
            definition: "any adverse event that results in death is life threatening requires hospitalization or prolonged existing hospitalization results in disability or is a congenital anomaly",
            source: DefinitionSource::IchGuideline("ICH E2A".to_string()),
        },
        // 10. Expedited Reporting (ICH E2A)
        SeedTerm {
            name: "Expedited Reporting",
            definition: "the process of reporting serious unexpected adverse reactions to regulatory authorities following expedited reporting requirements",
            source: DefinitionSource::IchGuideline("ICH E2A".to_string()),
        },
    ]
}
