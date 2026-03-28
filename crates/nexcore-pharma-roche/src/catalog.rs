use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("roche"),
        name: "F. Hoffmann-La Roche Ltd".to_string(),
        ticker: Some("ROG".to_string()),
        headquarters: Some("Basel, Switzerland".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Oncology,
            TherapeuticArea::Neuroscience,
            TherapeuticArea::Immunology,
            TherapeuticArea::Ophthalmology,
            TherapeuticArea::RareDisease,
        ],
        products: products(),
        pipeline: pipeline(),
        safety_communications: vec![],
    }
}

pub fn products() -> Vec<Product> {
    vec![
        Product {
            generic_name: "ocrelizumab".to_string(),
            brand_names: vec!["Ocrevus".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "emicizumab".to_string(),
            brand_names: vec!["Hemlibra".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "atezolizumab".to_string(),
            brand_names: vec!["Tecentriq".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2016),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "pertuzumab".to_string(),
            brand_names: vec!["Perjeta".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2012),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "ado-trastuzumab emtansine".to_string(),
            brand_names: vec!["Kadcyla".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2013),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "bevacizumab".to_string(),
            brand_names: vec!["Avastin".to_string()],
            rxcui: Some("354891".to_string()),
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2004),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "trastuzumab".to_string(),
            brand_names: vec!["Herceptin".to_string()],
            rxcui: Some("224905".to_string()),
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(1998),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "faricimab".to_string(),
            brand_names: vec!["Vabysmo".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Ophthalmology,
            approval_year: Some(2022),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "risdiplam".to_string(),
            brand_names: vec!["Evrysdi".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2020),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "fenebrutinib".to_string(),
            mechanism: "Non-covalent BTK inhibitor".to_string(),
            phase: Phase::Phase3,
            indication: "Relapsing MS; primary progressive MS".to_string(),
            therapeutic_area: TherapeuticArea::Neuroscience,
        },
        PipelineCandidate {
            name: "crovalimab".to_string(),
            mechanism: "Anti-complement C5 monoclonal antibody (recycling antibody)".to_string(),
            phase: Phase::Approved,
            indication: "Paroxysmal nocturnal haemoglobinuria".to_string(),
            therapeutic_area: TherapeuticArea::RareDisease,
        },
        PipelineCandidate {
            name: "tiragolumab".to_string(),
            mechanism: "Anti-TIGIT monoclonal antibody".to_string(),
            phase: Phase::Phase3,
            indication: "NSCLC in combination with atezolizumab".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
    ]
}
