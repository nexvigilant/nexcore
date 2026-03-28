use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("astrazeneca"),
        name: "AstraZeneca plc".to_string(),
        ticker: Some("AZN".to_string()),
        headquarters: Some("Cambridge, United Kingdom".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Oncology,
            TherapeuticArea::Cardiovascular,
            TherapeuticArea::Respiratory,
            TherapeuticArea::RareDisease,
            TherapeuticArea::Immunology,
        ],
        products: products(),
        pipeline: pipeline(),
        safety_communications: vec![],
    }
}

pub fn products() -> Vec<Product> {
    vec![
        Product {
            generic_name: "osimertinib".to_string(),
            brand_names: vec!["Tagrisso".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "durvalumab".to_string(),
            brand_names: vec!["Imfinzi".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "dapagliflozin".to_string(),
            brand_names: vec!["Farxiga".to_string()],
            rxcui: Some("1486977".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2012),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "olaparib".to_string(),
            brand_names: vec!["Lynparza".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2014),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "trastuzumab deruxtecan".to_string(),
            brand_names: vec!["Enhertu".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2019),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "acalabrutinib".to_string(),
            brand_names: vec!["Calquence".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "anifrolumab".to_string(),
            brand_names: vec!["Saphnelo".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "budesonide + glycopyrronium + formoterol fumarate".to_string(),
            brand_names: vec!["Breztri Aerosphere".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Respiratory,
            approval_year: Some(2020),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "datopotamab deruxtecan".to_string(),
            mechanism:
                "TROP2-directed antibody-drug conjugate (DXd topoisomerase I inhibitor payload)"
                    .to_string(),
            phase: Phase::Approved,
            indication: "HR+/HER2- breast cancer; EGFR-mutated NSCLC".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
        PipelineCandidate {
            name: "camizestrant".to_string(),
            mechanism: "Oral selective estrogen receptor degrader (SERD)".to_string(),
            phase: Phase::Phase3,
            indication: "ER+/HER2- advanced breast cancer".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
        PipelineCandidate {
            name: "volrustomig".to_string(),
            mechanism: "PD-1 x CTLA-4 bispecific antibody".to_string(),
            phase: Phase::Phase3,
            indication: "NSCLC; head and neck squamous cell carcinoma".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
    ]
}
