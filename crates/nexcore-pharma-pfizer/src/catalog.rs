use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("pfizer"),
        name: "Pfizer Inc.".to_string(),
        ticker: Some("PFE".to_string()),
        headquarters: Some("New York, NY, USA".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Oncology,
            TherapeuticArea::Immunology,
            TherapeuticArea::Metabolic,
            TherapeuticArea::Infectious,
            TherapeuticArea::Neuroscience,
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
            generic_name: "apixaban".to_string(),
            brand_names: vec!["Eliquis".to_string()],
            rxcui: Some("1364430".to_string()),
            therapeutic_area: TherapeuticArea::Cardiovascular,
            approval_year: Some(2012),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "palbociclib".to_string(),
            brand_names: vec!["Ibrance".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "tofacitinib".to_string(),
            brand_names: vec!["Xeljanz".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2012),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "nirmatrelvir + ritonavir".to_string(),
            brand_names: vec!["Paxlovid".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Infectious,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "COVID-19 mRNA vaccine (BNT162b2)".to_string(),
            brand_names: vec!["Comirnaty".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Vaccines,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "pneumococcal 20-valent conjugate vaccine".to_string(),
            brand_names: vec!["Prevnar 20".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Vaccines,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "tafamidis meglumine".to_string(),
            brand_names: vec!["Vyndaqel".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2019),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "enzalutamide".to_string(),
            brand_names: vec!["Xtandi".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2012),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "rimegepant".to_string(),
            brand_names: vec!["Nurtec ODT".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2020),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "lorlatinib".to_string(),
            brand_names: vec!["Lorbrena".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2018),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "danuglipron".to_string(),
            mechanism: "Oral GLP-1 receptor agonist (small molecule)".to_string(),
            phase: Phase::Phase3,
            indication: "Type 2 diabetes; obesity".to_string(),
            therapeutic_area: TherapeuticArea::Metabolic,
        },
        PipelineCandidate {
            name: "sasanlimab".to_string(),
            mechanism: "Anti-PD-1 monoclonal antibody".to_string(),
            phase: Phase::Phase3,
            indication: "Non-muscle invasive bladder cancer".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
        PipelineCandidate {
            name: "elranatamab".to_string(),
            mechanism: "BCMA x CD3 bispecific antibody".to_string(),
            phase: Phase::Approved,
            indication: "Relapsed/refractory multiple myeloma".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
    ]
}
