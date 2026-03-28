use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("bristol-myers-squibb"),
        name: "Bristol-Myers Squibb Company".to_string(),
        ticker: Some("BMY".to_string()),
        headquarters: Some("Princeton, NJ, USA".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Oncology,
            TherapeuticArea::Cardiovascular,
            TherapeuticArea::Immunology,
            TherapeuticArea::RareDisease,
            TherapeuticArea::Neuroscience,
        ],
        products: products(),
        pipeline: pipeline(),
        safety_communications: vec![],
    }
}

pub fn products() -> Vec<Product> {
    vec![
        Product {
            generic_name: "nivolumab".to_string(),
            brand_names: vec!["Opdivo".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2014),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "apixaban".to_string(),
            brand_names: vec!["Eliquis".to_string()],
            rxcui: Some("1364430".to_string()),
            therapeutic_area: TherapeuticArea::Cardiovascular,
            approval_year: Some(2012),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "lenalidomide".to_string(),
            brand_names: vec!["Revlimid".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2005),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "pomalidomide".to_string(),
            brand_names: vec!["Pomalyst".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2013),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "luspatercept".to_string(),
            brand_names: vec!["Reblozyl".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2019),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "idecabtagene vicleucel".to_string(),
            brand_names: vec!["Abecma".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2021),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "mavacamten".to_string(),
            brand_names: vec!["Camzyos".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Cardiovascular,
            approval_year: Some(2022),
            safety_profile: SafetyProfile {
                rems: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "deucravacitinib".to_string(),
            brand_names: vec!["Sotyktu".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2022),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "cobenfy (KarXT)".to_string(),
            mechanism: "M1/M4 muscarinic agonist + trospium peripheral blocker".to_string(),
            phase: Phase::Approved,
            indication: "Schizophrenia".to_string(),
            therapeutic_area: TherapeuticArea::Neuroscience,
        },
        PipelineCandidate {
            name: "iberdomide".to_string(),
            mechanism: "Cereblon E3 ligase modulator (CELMoD)".to_string(),
            phase: Phase::Phase3,
            indication: "Relapsed/refractory multiple myeloma".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
        PipelineCandidate {
            name: "BMS-986325".to_string(),
            mechanism: "Oral IL-13 small molecule inhibitor".to_string(),
            phase: Phase::Phase2,
            indication: "Atopic dermatitis".to_string(),
            therapeutic_area: TherapeuticArea::Dermatology,
        },
    ]
}
