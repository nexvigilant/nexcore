use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("merck"),
        name: "Merck & Co., Inc.".to_string(),
        ticker: Some("MRK".to_string()),
        headquarters: Some("Rahway, NJ, USA".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Oncology,
            TherapeuticArea::Infectious,
            TherapeuticArea::Metabolic,
            TherapeuticArea::Cardiovascular,
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
            generic_name: "pembrolizumab".to_string(),
            brand_names: vec!["Keytruda".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2014),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "human papillomavirus 9-valent vaccine".to_string(),
            brand_names: vec!["Gardasil 9".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Vaccines,
            approval_year: Some(2014),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "sitagliptin".to_string(),
            brand_names: vec!["Januvia".to_string()],
            rxcui: Some("593411".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2006),
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
            generic_name: "lenvatinib".to_string(),
            brand_names: vec!["Lenvima".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "molnupiravir".to_string(),
            brand_names: vec!["Lagevrio".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Infectious,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "belzutifan".to_string(),
            brand_names: vec!["Welireg".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "sotatercept".to_string(),
            brand_names: vec!["Winrevair".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2024),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "clesrovimab".to_string(),
            mechanism: "Anti-RSV prefusion F protein monoclonal antibody".to_string(),
            phase: Phase::Approved,
            indication: "Prevention of RSV lower respiratory tract disease in infants".to_string(),
            therapeutic_area: TherapeuticArea::Infectious,
        },
        PipelineCandidate {
            name: "gefapixant".to_string(),
            mechanism: "P2X3 receptor antagonist".to_string(),
            phase: Phase::Approved,
            indication: "Refractory or unexplained chronic cough".to_string(),
            therapeutic_area: TherapeuticArea::Respiratory,
        },
        PipelineCandidate {
            name: "MK-0616".to_string(),
            mechanism: "Oral macrocyclic peptide PCSK9 inhibitor".to_string(),
            phase: Phase::Phase3,
            indication: "Hypercholesterolaemia".to_string(),
            therapeutic_area: TherapeuticArea::Cardiovascular,
        },
    ]
}
