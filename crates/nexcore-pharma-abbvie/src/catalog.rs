use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("abbvie"),
        name: "AbbVie Inc.".to_string(),
        ticker: Some("ABBV".to_string()),
        headquarters: Some("North Chicago, IL, USA".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Immunology,
            TherapeuticArea::Oncology,
            TherapeuticArea::Neuroscience,
            TherapeuticArea::Ophthalmology,
            TherapeuticArea::Dermatology,
        ],
        products: products(),
        pipeline: pipeline(),
        safety_communications: vec![],
    }
}

pub fn products() -> Vec<Product> {
    vec![
        Product {
            generic_name: "adalimumab".to_string(),
            brand_names: vec!["Humira".to_string()],
            rxcui: Some("327361".to_string()),
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2002),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "risankizumab".to_string(),
            brand_names: vec!["Skyrizi".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2019),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "upadacitinib".to_string(),
            brand_names: vec!["Rinvoq".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2019),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                ..SafetyProfile::default()
            },
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
            generic_name: "venetoclax".to_string(),
            brand_names: vec!["Venclexta".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2016),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "onabotulinumtoxinA".to_string(),
            brand_names: vec!["Botox".to_string()],
            rxcui: Some("1991302".to_string()),
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(1989),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                ..SafetyProfile::default()
            },
        },
        Product {
            generic_name: "cariprazine".to_string(),
            brand_names: vec!["Vraylar".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "epcoritamab".to_string(),
            brand_names: vec!["Epkinly".to_string()],
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
            name: "lutikizumab".to_string(),
            mechanism: "Anti-IL-1α/IL-1β bispecific monoclonal antibody".to_string(),
            phase: Phase::Phase3,
            indication: "Hidradenitis suppurativa".to_string(),
            therapeutic_area: TherapeuticArea::Dermatology,
        },
        PipelineCandidate {
            name: "navitoclax".to_string(),
            mechanism: "BCL-2/BCL-XL inhibitor".to_string(),
            phase: Phase::Phase3,
            indication: "Myelofibrosis in combination with ruxolitinib".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
        PipelineCandidate {
            name: "ABBV-CLS-484".to_string(),
            mechanism: "PTPN11 (SHP2) inhibitor".to_string(),
            phase: Phase::Phase2,
            indication: "Advanced solid tumors".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        },
    ]
}
