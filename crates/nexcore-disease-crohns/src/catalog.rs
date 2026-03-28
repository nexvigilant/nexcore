use nexcore_disease::*;

pub fn disease() -> Disease {
    Disease {
        id: DiseaseId::new("crohns"),
        name: "crohns".to_string(),
        icd10_codes: vec![],
        therapeutic_area: TherapeuticArea::Other("crohns".to_string()),
        epidemiology: Epidemiology {
            global_prevalence: None,
            us_prevalence: None,
            annual_incidence: None,
            demographics: Demographics {
                median_age_onset: None,
                sex_ratio: None,
                risk_factors: vec![],
            },
            trend: Trend::Stable,
        },
        standard_of_care: vec![],
        unmet_needs: vec![],
        safety_burden: SafetyBurden {
            total_drugs_approved: 0,
            drugs_with_boxed_warnings: 0,
            drugs_with_rems: 0,
            class_effects: vec![],
            notable_withdrawals: vec![],
        },
        biomarkers: vec![],
    }
}
