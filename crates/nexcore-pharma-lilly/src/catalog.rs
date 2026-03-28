use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("eli-lilly"),
        name: "Eli Lilly and Company".to_string(),
        ticker: Some("LLY".to_string()),
        headquarters: Some("Indianapolis, Indiana, USA".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Metabolic,
            TherapeuticArea::Oncology,
            TherapeuticArea::Immunology,
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
            generic_name: "tirzepatide".to_string(),
            brand_names: vec!["Mounjaro".to_string(), "Zepbound".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2022),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "dulaglutide".to_string(),
            brand_names: vec!["Trulicity".to_string()],
            rxcui: Some("1860484".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2014),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "insulin lispro".to_string(),
            brand_names: vec!["Humalog".to_string()],
            rxcui: Some("86009".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(1996),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "empagliflozin".to_string(),
            brand_names: vec!["Jardiance".to_string()],
            rxcui: Some("1545653".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2014),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "abemaciclib".to_string(),
            brand_names: vec!["Verzenio".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "donanemab".to_string(),
            brand_names: vec!["Kisunla".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2024),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "ixekizumab".to_string(),
            brand_names: vec!["Taltz".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2016),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "galcanezumab".to_string(),
            brand_names: vec!["Emgality".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2018),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "ramucirumab".to_string(),
            brand_names: vec!["Cyramza".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2014),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "selpercatinib".to_string(),
            brand_names: vec!["Retevmo".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2020),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "pirtobrutinib".to_string(),
            brand_names: vec!["Jaypirca".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2023),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "orforglipron".to_string(),
            mechanism: "Oral GLP-1 receptor agonist (small molecule)".to_string(),
            phase: Phase::Phase3,
            indication: "Type 2 diabetes; obesity".to_string(),
            therapeutic_area: TherapeuticArea::Metabolic,
        },
        PipelineCandidate {
            name: "retatrutide".to_string(),
            mechanism: "GIP/GLP-1/glucagon triple receptor agonist".to_string(),
            phase: Phase::Phase3,
            indication: "Obesity; metabolic dysfunction-associated steatohepatitis".to_string(),
            therapeutic_area: TherapeuticArea::Metabolic,
        },
        PipelineCandidate {
            name: "lebrikizumab".to_string(),
            mechanism: "Anti-IL-13 monoclonal antibody".to_string(),
            phase: Phase::Approved,
            indication: "Atopic dermatitis".to_string(),
            therapeutic_area: TherapeuticArea::Dermatology,
        },
        PipelineCandidate {
            name: "mirikizumab".to_string(),
            mechanism: "Anti-IL-23p19 monoclonal antibody".to_string(),
            phase: Phase::Approved,
            indication: "Ulcerative colitis; Crohn's disease".to_string(),
            therapeutic_area: TherapeuticArea::Immunology,
        },
    ]
}
