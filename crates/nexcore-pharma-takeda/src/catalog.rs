use nexcore_pharma::{
    CommType, Company, CompanyId, Phase, PipelineCandidate, Product, SafetyCommunication,
    SafetyProfile, SignalSummary, SignalVerdict, TherapeuticArea,
};

pub fn company() -> Company {
    Company {
        id: CompanyId::new("takeda"),
        name: "Takeda Pharmaceutical Company".to_string(),
        ticker: Some("TAK".to_string()),
        headquarters: Some("Tokyo, Japan / Cambridge, Massachusetts, USA".to_string()),
        therapeutic_areas: vec![
            TherapeuticArea::Immunology,
            TherapeuticArea::RareDisease,
            TherapeuticArea::Neuroscience,
            TherapeuticArea::Oncology,
        ],
        products: products(),
        pipeline: pipeline(),
        safety_communications: safety_communications(),
    }
}

pub fn products() -> Vec<Product> {
    vec![
        // 1. Entyvio — anti-integrin, UC and Crohn's
        Product {
            generic_name: "vedolizumab".to_string(),
            brand_names: vec!["Entyvio".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2014),
            safety_profile: SafetyProfile {
                boxed_warning: false,
                rems: false,
                signals: vec![
                    SignalSummary {
                        event: "serious infections".to_string(),
                        prr: 1.4,
                        ror: 1.5,
                        cases: 340,
                        on_label: true,
                        verdict: SignalVerdict::Moderate,
                    },
                    SignalSummary {
                        event: "progressive multifocal leukoencephalopathy".to_string(),
                        prr: 1.1,
                        ror: 1.2,
                        cases: 2,
                        on_label: false,
                        verdict: SignalVerdict::Weak,
                    },
                ],
                label_warnings: vec![
                    "Serious infections including opportunistic infections reported".to_string(),
                    "PML risk as class concern for anti-integrin therapies".to_string(),
                ],
            },
        },
        // 2. Vyvanse — CNS stimulant, ADHD and binge eating disorder
        Product {
            generic_name: "lisdexamfetamine".to_string(),
            brand_names: vec!["Vyvanse".to_string()],
            rxcui: Some("854795".to_string()),
            therapeutic_area: TherapeuticArea::Neuroscience,
            approval_year: Some(2007),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: false,
                signals: vec![
                    SignalSummary {
                        event: "cardiovascular events".to_string(),
                        prr: 2.1,
                        ror: 2.3,
                        cases: 512,
                        on_label: true,
                        verdict: SignalVerdict::Strong,
                    },
                    SignalSummary {
                        event: "psychiatric adverse events".to_string(),
                        prr: 3.8,
                        ror: 4.1,
                        cases: 1024,
                        on_label: true,
                        verdict: SignalVerdict::Strong,
                    },
                ],
                label_warnings: vec![
                    "High potential for abuse and dependence. Schedule II controlled substance."
                        .to_string(),
                    "Serious cardiovascular events reported in adults and children".to_string(),
                ],
            },
        },
        // 3. Takhzyro — anti-kallikrein, hereditary angioedema prophylaxis
        Product {
            generic_name: "lanadelumab".to_string(),
            brand_names: vec!["Takhzyro".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2018),
            safety_profile: SafetyProfile::default(),
        },
        // 4. Adynovate — PEGylated recombinant Factor VIII, hemophilia A
        Product {
            generic_name: "rurioctocog alfa pegol".to_string(),
            brand_names: vec!["Adynovate".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        // 5. Eohilia — budesonide oral suspension, eosinophilic esophagitis
        Product {
            generic_name: "budesonide".to_string(),
            brand_names: vec!["Eohilia".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Immunology,
            approval_year: Some(2024),
            safety_profile: SafetyProfile::default(),
        },
        // 6. Fruzaqla — VEGFR inhibitor, metastatic colorectal cancer
        Product {
            generic_name: "fruquintinib".to_string(),
            brand_names: vec!["Fruzaqla".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2023),
            safety_profile: SafetyProfile::default(),
        },
        // 7. Exkivity — EGFR exon 20 inhibitor, NSCLC (voluntarily withdrawn 2023)
        Product {
            generic_name: "mobocertinib".to_string(),
            brand_names: vec!["Exkivity".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2021),
            safety_profile: SafetyProfile {
                boxed_warning: false,
                rems: false,
                signals: vec![SignalSummary {
                    event: "lack of confirmatory clinical benefit in PAPILLON trial".to_string(),
                    prr: 0.9,
                    ror: 0.9,
                    cases: 0,
                    on_label: false,
                    verdict: SignalVerdict::Noise,
                }],
                label_warnings: vec![
                    "Voluntarily withdrawn October 2023 following failure of confirmatory trial"
                        .to_string(),
                ],
            },
        },
        // 8. Ninlaro — proteasome inhibitor, multiple myeloma
        Product {
            generic_name: "ixazomib".to_string(),
            brand_names: vec!["Ninlaro".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        // 9. Alunbrig — ALK inhibitor, ALK-positive NSCLC
        Product {
            generic_name: "brigatinib".to_string(),
            brand_names: vec!["Alunbrig".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2017),
            safety_profile: SafetyProfile::default(),
        },
        // 10. Natpara — PTH analog, hypoparathyroidism
        Product {
            generic_name: "nateriparatide".to_string(),
            brand_names: vec!["Natpara".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::RareDisease,
            approval_year: Some(2015),
            safety_profile: SafetyProfile::default(),
        },
        // 11. Livtencity — UL97 kinase inhibitor, CMV infection post-transplant
        Product {
            generic_name: "maribavir".to_string(),
            brand_names: vec!["Livtencity".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Infectious,
            approval_year: Some(2021),
            safety_profile: SafetyProfile::default(),
        },
        // 12. ICLUSIG — BCR-ABL TKI, CML and Ph+ ALL
        Product {
            generic_name: "ponatinib".to_string(),
            brand_names: vec!["ICLUSIG".to_string()],
            rxcui: None,
            therapeutic_area: TherapeuticArea::Oncology,
            approval_year: Some(2012),
            safety_profile: SafetyProfile {
                boxed_warning: true,
                rems: true,
                signals: vec![
                    SignalSummary {
                        event: "arterial occlusive events".to_string(),
                        prr: 4.2,
                        ror: 4.8,
                        cases: 287,
                        on_label: true,
                        verdict: SignalVerdict::Strong,
                    },
                    SignalSummary {
                        event: "hepatotoxicity".to_string(),
                        prr: 2.9,
                        ror: 3.1,
                        cases: 143,
                        on_label: true,
                        verdict: SignalVerdict::Strong,
                    },
                ],
                label_warnings: vec![
                    "Arterial occlusions, venous thromboembolism, heart failure, and hepatotoxicity. REMS program required.".to_string(),
                ],
            },
        },
    ]
}

pub fn pipeline() -> Vec<PipelineCandidate> {
    vec![
        PipelineCandidate {
            name: "TAK-279".to_string(),
            mechanism: "TYK2 (tyrosine kinase 2) inhibitor".to_string(),
            phase: Phase::Phase3,
            indication: "Plaque psoriasis; inflammatory bowel disease".to_string(),
            therapeutic_area: TherapeuticArea::Immunology,
        },
        PipelineCandidate {
            name: "TAK-861".to_string(),
            mechanism: "Orexin 2 receptor agonist".to_string(),
            phase: Phase::Phase3,
            indication: "Narcolepsy type 1".to_string(),
            therapeutic_area: TherapeuticArea::Neuroscience,
        },
        PipelineCandidate {
            name: "TAK-999".to_string(),
            mechanism: "AAV-based gene therapy (alpha-galactosidase A replacement)".to_string(),
            phase: Phase::Phase2,
            indication: "Fabry disease".to_string(),
            therapeutic_area: TherapeuticArea::RareDisease,
        },
        PipelineCandidate {
            name: "soticlestat".to_string(),
            mechanism: "Cholesterol 24-hydroxylase (CH24H) inhibitor".to_string(),
            phase: Phase::Phase3,
            indication: "Dravet syndrome; Lennox-Gastaut syndrome".to_string(),
            therapeutic_area: TherapeuticArea::Neuroscience,
        },
    ]
}

pub fn safety_communications() -> Vec<SafetyCommunication> {
    vec![
        SafetyCommunication {
            title: "Voluntary Withdrawal of Exkivity (mobocertinib) from US Market".to_string(),
            product: "mobocertinib".to_string(),
            comm_type: CommType::SafetyUpdate,
            date: "2023-10-18".to_string(),
            summary: "Voluntary withdrawal of accelerated approval indication for EGFR exon 20 insertion-positive NSCLC following failure to demonstrate clinical benefit in the confirmatory PAPILLON trial.".to_string(),
        },
        SafetyCommunication {
            title: "ICLUSIG (ponatinib): Updated Boxed Warning for Vascular Occlusion and Hepatotoxicity".to_string(),
            product: "ponatinib".to_string(),
            comm_type: CommType::DearHcpLetter,
            date: "2022-06-01".to_string(),
            summary: "Updated boxed warning to include vascular occlusion, heart failure, and hepatotoxicity. Risk Evaluation and Mitigation Strategy (REMS) program required for prescribers and dispensing pharmacies.".to_string(),
        },
    ]
}
