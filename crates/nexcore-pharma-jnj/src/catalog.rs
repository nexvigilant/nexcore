use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("johnson-johnson"),
        name: "Johnson & Johnson (Janssen Pharmaceuticals)".to_string(),
        ticker: Some("JNJ".to_string()),
        headquarters: Some("New Brunswick, NJ, USA".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Immunology,
            TherapeuticArea::Oncology,
            TherapeuticArea::Neuroscience,
            TherapeuticArea::Infectious,
            TherapeuticArea::Cardiovascular,
        ],
        products: products(),
        pipeline: pipeline(),
        safety_communications: vec![],
    }
}

pub fn products() -> Vec<Product> {
    vec![
        Product {
            generic_name: "ustekinumab".to_string(),
            brand_names: vec!["Stelara".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2009),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "daratumumab".to_string(),
            brand_names: vec!["Darzalex".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "guselkumab".to_string(),
            brand_names: vec!["Tremfya".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "ibrutinib".to_string(),
            brand_names: vec!["Imbruvica".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2013),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "apalutamide".to_string(),
            brand_names: vec!["Erleada".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2018),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "esketamine".to_string(),
            brand_names: vec!["Spravato".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2019),
            safety_profile: SafetyProfile {
                rems: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "ciltacabtagene autoleucel".to_string(),
            brand_names: vec!["Carvykti".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2022),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "amivantamab".to_string(),
            brand_names: vec!["Rybrevant".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "nipocalimab".to_string(),
            mechanism: "Anti-FcRn monoclonal antibody".to_string(),
            phase: Phase::Phase3,
            indication:
                "Generalised myasthenia gravis; haemolytic disease of the fetus and newborn"
                    .to_string(),
            therapeutic_area: TherapeuticArea::Immunology,
        },
        PipelineCandidate {
            name: "milvexian".to_string(),
            mechanism: "Oral Factor XIa inhibitor".to_string(),
            phase: Phase::Phase3,
            indication: "Stroke prevention; acute coronary syndrome".to_string(),
            therapeutic_area: TherapeuticArea::Cardiovascular,
        },
        PipelineCandidate {
            name: "icotrokinra (JNJ-2113)".to_string(),
            mechanism: "Oral IL-23 receptor peptide antagonist".to_string(),
            phase: Phase::Phase3,
            indication: "Plaque psoriasis".to_string(),
            therapeutic_area: TherapeuticArea::Immunology,
        },
    ]
}
