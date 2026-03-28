use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("gsk"),
        name: "GSK plc".to_string(),
        ticker: Some("GSK".to_string()),
        headquarters: Some("Brentford, United Kingdom".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Infectious,
            TherapeuticArea::Respiratory,
            TherapeuticArea::Oncology,
            TherapeuticArea::Immunology,
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
            generic_name: "zoster vaccine recombinant, adjuvanted".to_string(),
            brand_names: vec!["Shingrix".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Vaccines,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "mepolizumab".to_string(),
            brand_names: vec!["Nucala".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Respiratory,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "dolutegravir + lamivudine".to_string(),
            brand_names: vec!["Dovato".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Infectious,
            approval_year: Some(2019),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "fluticasone furoate + umeclidinium + vilanterol".to_string(),
            brand_names: vec!["Trelegy Ellipta".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Respiratory,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "RSV vaccine recombinant, adjuvanted".to_string(),
            brand_names: vec!["Arexvy".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Vaccines,
            approval_year: Some(2023),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "dostarlimab".to_string(),
            brand_names: vec!["Jemperli".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "belantamab mafodotin".to_string(),
            brand_names: vec!["Blenrep".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2020),
            safety_profile: SafetyProfile {
                rems: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "momelotinib".to_string(),
            brand_names: vec!["Ojjaara".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2023),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "depemokimab".to_string(),
            mechanism: "Anti-IL-5 monoclonal antibody (ultra-long-acting, 6-monthly dosing)"
                .to_string(),
            phase: Phase::Phase3,
            indication: "Severe eosinophilic asthma; EGPA".to_string(),
            therapeutic_area: TherapeuticArea::Respiratory,
        },
        PipelineCandidate {
            name: "gepotidacin".to_string(),
            mechanism: "Bacterial topoisomerase II and IV inhibitor (triazaacenaphthylene class)"
                .to_string(),
            phase: Phase::Approved,
            indication: "Uncomplicated urinary tract infections; gonorrhoea".to_string(),
            therapeutic_area: TherapeuticArea::Infectious,
        },
        PipelineCandidate {
            name: "linerixibat".to_string(),
            mechanism: "Ileal bile acid transporter (IBAT) inhibitor".to_string(),
            phase: Phase::Phase3,
            indication: "Cholestatic pruritus in primary biliary cholangitis".to_string(),
            therapeutic_area: TherapeuticArea::RareDisease,
        },
    ]
}
