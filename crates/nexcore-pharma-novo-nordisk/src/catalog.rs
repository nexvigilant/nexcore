use nexcore_pharma::{
    CommType, Company, CompanyId, Phase, PipelineCandidate, Product, SafetyCommunication,
    SafetyProfile, SignalSummary, SignalVerdict, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("novo-nordisk"),
        name: "Novo Nordisk A/S".to_string(),
        ticker: Some("NVO".to_string()),
        headquarters: Some("Bagsværd, Denmark".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Metabolic,
            TherapeuticArea::Hematology,
            TherapeuticArea::Other,
        ],
        products: products(),
        pipeline: pipeline(),
        safety_communications: safety_communications(),
    }
}

pub fn products() -> Vec<Product> {
    vec![
        Product {
            generic_name: "semaglutide".to_string(),
            brand_names: vec!["Ozempic".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2017),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: false,
                signals: vec![
                    SignalSummary {
                        event: "pancreatitis".to_string(),
                        prr: 2.1,
                        ror: 2.4,
                        cases: 312,
                        on_label: true,
                        verdict: SignalVerdict::Moderate,
                    },
                    SignalSummary {
                        event: "gastroparesis".to_string(),
                        prr: 3.8,
                        ror: 4.2,
                        cases: 198,
                        on_label: true,
                        verdict: SignalVerdict::Strong,
                    },
                    SignalSummary {
                        event: "thyroid C-cell tumor".to_string(),
                        prr: 1.9,
                        ror: 2.0,
                        cases: 47,
                        on_label: true,
                        verdict: SignalVerdict::Moderate,
                    },
                    SignalSummary {
                        event: "suicidal ideation".to_string(),
                        prr: 1.6,
                        ror: 1.7,
                        cases: 89,
                        on_label: false,
                        verdict: SignalVerdict::Weak,
                    },
                ],
                label_warnings: vec![
                    "Thyroid C-Cell Tumors (boxed)".to_string(),
                    "Pancreatitis".to_string(),
                    "Gastroparesis".to_string(),
                    "Intestinal obstruction".to_string(),
                ],
            },
        },
        Product {
            generic_name: "semaglutide 2.4mg".to_string(),
            brand_names: vec!["Wegovy".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2021),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: false,
                signals: vec![
                    SignalSummary {
                        event: "pancreatitis".to_string(),
                        prr: 2.3,
                        ror: 2.6,
                        cases: 145,
                        on_label: true,
                        verdict: SignalVerdict::Moderate,
                    },
                    SignalSummary {
                        event: "gastroparesis".to_string(),
                        prr: 4.1,
                        ror: 4.5,
                        cases: 201,
                        on_label: true,
                        verdict: SignalVerdict::Strong,
                    },
                ],
                label_warnings: vec![
                    "Thyroid C-Cell Tumors (boxed)".to_string(),
                    "Pancreatitis".to_string(),
                    "Cardiovascular benefit labeling (2024)".to_string(),
                ],
            },
        },
        Product {
            generic_name: "oral semaglutide".to_string(),
            brand_names: vec!["Rybelsus".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2019),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: false,
                signals: vec![SignalSummary {
                    event: "pancreatitis".to_string(),
                    prr: 1.9,
                    ror: 2.1,
                    cases: 78,
                    on_label: true,
                    verdict: SignalVerdict::Moderate,
                }],
                label_warnings: vec!["Thyroid C-Cell Tumors (boxed)".to_string()],
            },
        },
        Product {
            generic_name: "liraglutide".to_string(),
            brand_names: vec!["Victoza".to_string()],
            rxcui: Some("1991302".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2010),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: false,
                signals: vec![SignalSummary {
                    event: "pancreatitis".to_string(),
                    prr: 2.0,
                    ror: 2.2,
                    cases: 420,
                    on_label: true,
                    verdict: SignalVerdict::Moderate,
                }],
                label_warnings: vec![
                    "Thyroid C-Cell Tumors (boxed)".to_string(),
                    "Pancreatitis".to_string(),
                ],
            },
        },
        Product {
            generic_name: "liraglutide 3mg".to_string(),
            brand_names: vec!["Saxenda".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2014),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: false,
                signals: vec![],
                label_warnings: vec!["Thyroid C-Cell Tumors (boxed)".to_string()],
            },
        },
        Product {
            generic_name: "insulin degludec".to_string(),
            brand_names: vec!["Tresiba".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "insulin detemir".to_string(),
            brand_names: vec!["Levemir".to_string()],
            rxcui: Some("274783".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2005),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "insulin aspart".to_string(),
            brand_names: vec!["NovoRapid".to_string(), "NovoLog".to_string()],
            rxcui: Some("1151131".to_string()),
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2000),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "insulin degludec/liraglutide".to_string(),
            brand_names: vec!["Xultophy".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Metabolic,
            approval_year: Some(2016),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "somatropin".to_string(),
            brand_names: vec!["Norditropin".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Other,
            approval_year: Some(1987),
            safety_profile: SafetyProfile::default(),
        },
        Product {
            generic_name: "eptacog alfa / nonacog beta pegol / turoctocog alfa pegol".to_string(),
            brand_names: vec![
                "NovoSeven".to_string(),
                "NovoEight".to_string(),
                "Esperoct".to_string(),
            ],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Hematology,
            approval_year: Some(1999),
            safety_profile: SafetyProfile::default(),
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "CagriSema".to_string(),
            mechanism: "Dual amylin/GLP-1 receptor agonist (cagrilintide + semaglutide)"
                .to_string(),
            phase: Phase::Phase3,
            indication: "Obesity".to_string(),
            therapeutic_area: TherapeuticArea::Metabolic,
        },
        PipelineCandidate {
            name: "Amycretin".to_string(),
            mechanism: "Oral amylin/GLP-1 dual agonist".to_string(),
            phase: Phase::Phase2,
            indication: "Obesity".to_string(),
            therapeutic_area: TherapeuticArea::Metabolic,
        },
        PipelineCandidate {
            name: "Alhemo (concizumab)".to_string(),
            mechanism: "Anti-TFPI monoclonal antibody".to_string(),
            phase: Phase::Phase3,
            indication: "Hemophilia with inhibitors".to_string(),
            therapeutic_area: TherapeuticArea::Hematology,
        },
        PipelineCandidate {
            name: "Mim8".to_string(),
            mechanism: "Bispecific antibody mimicking factor VIIIa".to_string(),
            phase: Phase::Phase3,
            indication: "Hemophilia A".to_string(),
            therapeutic_area: TherapeuticArea::Hematology,
        },
        PipelineCandidate {
            name: "Icosema".to_string(),
            mechanism: "Once-weekly insulin icodec + semaglutide combination".to_string(),
            phase: Phase::Phase3,
            indication: "Type 2 diabetes".to_string(),
            therapeutic_area: TherapeuticArea::Metabolic,
        },
    ]
}

pub fn safety_communications() -> Vec<SafetyCommunication> {
    vec![
        SafetyCommunication {
            title: "Ozempic/Wegovy: Gastroparesis and Intestinal Obstruction Label Update".to_string(),
            date: "2023-09".to_string(),
            comm_type: CommType::SafetyUpdate,
            product: "semaglutide".to_string(),
            summary: "Label updated to add gastroparesis and intestinal obstruction warnings for Ozempic and Wegovy.".to_string(),
        },
        SafetyCommunication {
            title: "Wegovy: Cardiovascular Risk Reduction Indication Added".to_string(),
            date: "2024-03".to_string(),
            comm_type: CommType::SafetyUpdate,
            product: "semaglutide 2.4mg".to_string(),
            summary: "Wegovy label updated with approved cardiovascular risk reduction indication based on SELECT trial data.".to_string(),
        },
    ]
}
