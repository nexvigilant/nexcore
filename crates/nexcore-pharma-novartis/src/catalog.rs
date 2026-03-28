use nexcore_pharma::{
    Company, CompanyId, Phase, PipelineCandidate, Product, SafetyProfile, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("novartis"),
        name: "Novartis AG".to_string(),
        ticker: Some("NVS".to_string()),
        headquarters: Some("Basel, Switzerland".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Cardiovascular,
            TherapeuticArea::Immunology,
            TherapeuticArea::Oncology,
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
            generic_name: "sacubitril + valsartan".to_string(),
            brand_names: vec!["Entresto".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Cardiovascular,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "secukinumab".to_string(),
            brand_names: vec!["Cosentyx".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "ribociclib".to_string(),
            brand_names: vec!["Kisqali".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "ofatumumab".to_string(),
            brand_names: vec!["Kesimpta".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2020),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "inclisiran".to_string(),
            brand_names: vec!["Leqvio".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Cardiovascular,
            approval_year: Some(2020),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "lutetium (177Lu) vipivotide tetraxetan".to_string(),
            brand_names: vec!["Pluvicto".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2022),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "asciminib".to_string(),
            brand_names: vec!["Scemblix".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "ruxolitinib".to_string(),
            brand_names: vec!["Jakavi".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2011),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "onasemnogene abeparvovec".to_string(),
            brand_names: vec!["Zolgensma".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2019),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "iptacopan".to_string(),
            brand_names: vec!["Fabhalta".to_string()],
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
            name: "pelacarsen".to_string(),
            mechanism: "Antisense oligonucleotide targeting apolipoprotein(a)".to_string(),
            phase: Phase::Phase3,
            indication: "Elevated lipoprotein(a); cardiovascular risk reduction".to_string(),
            therapeutic_area: TherapeuticArea::Cardiovascular,
        },
        PipelineCandidate {
            name: "abelacimab".to_string(),
            mechanism: "Anti-Factor XI monoclonal antibody".to_string(),
            phase: Phase::Phase3,
            indication: "Stroke prevention in atrial fibrillation".to_string(),
            therapeutic_area: TherapeuticArea::Cardiovascular,
        },
        PipelineCandidate {
            name: "OAV101 intrathecal".to_string(),
            mechanism: "AAV9 gene therapy delivering SMN1 (intrathecal delivery)".to_string(),
            phase: Phase::Phase3,
            indication: "Spinal muscular atrophy in patients >2 years".to_string(),
            therapeutic_area: TherapeuticArea::RareDisease,
        },
    ]
}
