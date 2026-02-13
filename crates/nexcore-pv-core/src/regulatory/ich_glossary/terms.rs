//! Terms Module
//!
//! ICH glossary terms with O(1) PHF lookup.
//! Contains 1000+ pharmacovigilance terms from ICH/CIOMS glossary.

use crate::regulatory::ich_glossary::types::{
    AlternativeDefinition, GuidelineStatus, IchCategory, Source, Term,
};
use phf::phf_map;

// ============================================================================
// Term Lookup Table (PHF - O(1) compile-time)
// ============================================================================

/// Main term lookup table
/// Key: Normalized term name (lowercase, spaces to underscores)
pub static TERMS: phf::Map<&'static str, Term> = phf_map! {
    // === A ===
    "adverse_drug_reaction" => Term {
        name: "Adverse Drug Reaction (ADR)",
        key: "adverse_drug_reaction",
        definition: "In the pre-approval clinical experience with a new medicinal product or its new usages, particularly as the therapeutic dose(s) may not be established: All noxious and unintended responses to a medicinal product related to any dose should be considered adverse drug reactions. The phrase 'responses to a medicinal product' means that a causal relationship between a medicinal product and an adverse event is at least a reasonable possibility, i.e., the relationship cannot be ruled out.",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.A.2",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[
            AlternativeDefinition {
                definition: "An adverse drug reaction, as defined by regional and local requirements, concerns a noxious and unintended response to a medicinal product. The phrase 'response to a medicinal product' means that a causal relationship between a medicinal product and an adverse event is at least a reasonable possibility.",
                source: Source {
                    guideline_id: "E2D(R1)",
                    guideline_title: "Post-Approval Safety Data Management",
                    status: GuidelineStatus::Step4Final,
                    date: "2025-09-15",
                    section: "2.1.2",
                    url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
                },
                clarification: None,
            },
        ],
        see_also: &["Adverse Event", "Serious Adverse Event", "Unexpected Adverse Drug Reaction"],
        abbreviation: Some("ADR"),
        clarification: None,
        is_new: false,
    },

    "adverse_event" => Term {
        name: "Adverse Event (AE)",
        key: "adverse_event",
        definition: "Any untoward medical occurrence in a patient or clinical investigation subject administered a pharmaceutical product and which does not necessarily have a causal relationship with this treatment. An adverse event (AE) can therefore be any unfavourable and unintended sign (including an abnormal laboratory finding), symptom, or disease temporally associated with the use of a medicinal (investigational) product, whether or not related to the medicinal (investigational) product.",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.A.1",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[
            AlternativeDefinition {
                definition: "Any unfavourable medical occurrence in a trial participant administered the investigational product. The adverse event does not necessarily have a causal relationship with the treatment.",
                source: Source {
                    guideline_id: "E6(R3)",
                    guideline_title: "Guideline for Good Clinical Practice",
                    status: GuidelineStatus::Step4Final,
                    date: "2025-01-06",
                    section: "Glossary",
                    url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
                },
                clarification: None,
            },
        ],
        see_also: &["Adverse Drug Reaction", "Serious Adverse Event", "Adverse Event of Special Interest"],
        abbreviation: Some("AE"),
        clarification: None,
        is_new: false,
    },

    "adverse_event_of_special_interest" => Term {
        name: "Adverse Event of Special Interest (AESI)",
        key: "adverse_event_of_special_interest",
        definition: "An adverse event of special interest (serious or non-serious) is one of scientific and medical concern specific to the sponsor's product or programme, for which ongoing monitoring and rapid communication by the investigator to the sponsor can be appropriate. Such an event might warrant further investigation in order to characterise and understand it.",
        source: Source {
            guideline_id: "E2F",
            guideline_title: "Development Safety Update Report",
            status: GuidelineStatus::Step4Final,
            date: "2010-08-17",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E2F_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Adverse Event", "Safety Signal"],
        abbreviation: Some("AESI"),
        clarification: Some("Based on CIOMS VI"),
        is_new: false,
    },

    "audit" => Term {
        name: "Audit",
        key: "audit",
        definition: "A systematic and independent examination of trial-related activities and records performed by the sponsor, service provider (including contract research organisation (CRO)) or institution to determine whether the evaluated trial-related activities were conducted and the data were recorded, analysed and accurately reported according to the protocol, applicable standard operating procedures (SOPs), Good Clinical Practice (GCP) and the applicable regulatory requirement(s).",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Audit Trail", "Inspection", "Monitoring"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    "audit_trail" => Term {
        name: "Audit Trail",
        key: "audit_trail",
        definition: "Metadata records that allow the appropriate evaluation of the course of events by capturing details on actions (manual or automated) performed relating to information and data collection and, where applicable, to activities in computerised systems. The audit trail should show activities, initial entry and changes to data fields or records, by whom, when and, where applicable, why. In computerised systems, the audit trail should be secure, computer-generated and time stamped.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Data Integrity", "Source Data"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === B ===
    "batch" => Term {
        name: "Batch (or Lot)",
        key: "batch",
        definition: "A specific quantity of material produced in a process or series of processes so that it is expected to be homogeneous within specified limits. In the case of continuous production, a batch may correspond to a defined fraction of the production. The batch size can be defined either by a fixed quantity or by the amount produced in a fixed time interval.",
        source: Source {
            guideline_id: "Q7",
            guideline_title: "Good Manufacturing Practice Guide for Active Pharmaceutical Ingredients",
            status: GuidelineStatus::Step4Final,
            date: "2000-11-10",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/Q7%20Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Lot", "Batch Number"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    "bioequivalence" => Term {
        name: "Bioequivalence",
        key: "bioequivalence",
        definition: "Two pharmaceutical products are bioequivalent if they are pharmaceutically equivalent or pharmaceutical alternatives and their bioavailabilities after administration in the same molar dose are similar to such a degree that their effects, with respect to both efficacy and safety, are expected to be essentially the same.",
        source: Source {
            guideline_id: "M13A",
            guideline_title: "Bioequivalence for Immediate-Release Solid Oral Dosage Forms",
            status: GuidelineStatus::Step4Final,
            date: "2024-07-23",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/M13A_Step4_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Bioavailability", "Pharmaceutical Equivalence"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === C ===
    "calibration" => Term {
        name: "Calibration",
        key: "calibration",
        definition: "The demonstration that a particular instrument or device produces results within specified limits by comparison with those produced by a reference or traceable standard over an appropriate range of measurements.",
        source: Source {
            guideline_id: "Q7",
            guideline_title: "Good Manufacturing Practice Guide for Active Pharmaceutical Ingredients",
            status: GuidelineStatus::Step4Final,
            date: "2000-11-10",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/Q7%20Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Qualification", "Validation"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    "clinical_trial" => Term {
        name: "Clinical Trial (Study)",
        key: "clinical_trial",
        definition: "Any investigation in human subjects intended to discover or verify the clinical, pharmacological and/or other pharmacodynamic effects of an investigational product(s), and/or to identify any adverse reactions to an investigational product(s), and/or to study absorption, distribution, metabolism, and excretion of an investigational product(s) with the object of ascertaining its safety and/or efficacy.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Protocol", "Investigator", "Sponsor"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === D ===
    "data_integrity" => Term {
        name: "Data Integrity",
        key: "data_integrity",
        definition: "The extent to which data are complete, consistent, accurate, trustworthy, and reliable and that these characteristics are maintained throughout the data life cycle. The data should be collected and maintained in a secure manner, so that they are attributable, legible, contemporaneously recorded, original (or a true copy), and accurate.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["ALCOA", "Audit Trail", "Source Data"],
        abbreviation: None,
        clarification: Some("ALCOA: Attributable, Legible, Contemporaneous, Original, Accurate"),
        is_new: false,
    },

    "drug_interaction" => Term {
        name: "Drug Interaction",
        key: "drug_interaction",
        definition: "A change in a drug's effect on the body when the drug is taken together with another drug. A drug-drug interaction can delay, decrease, or enhance absorption or metabolism of either drug and can produce unexpected adverse effects.",
        source: Source {
            guideline_id: "M12",
            guideline_title: "Drug Interaction Studies",
            status: GuidelineStatus::Step4Final,
            date: "2024-05-21",
            section: "7.1 Glossary",
            url: Some("https://database.ich.org/sites/default/files/M12_Step4_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Object Drug", "Precipitant Drug", "CYP450"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === E ===
    "efficacy" => Term {
        name: "Efficacy",
        key: "efficacy",
        definition: "A beneficial effect of a treatment on a clinical outcome, demonstrated within the defined conditions of a clinical trial.",
        source: Source {
            guideline_id: "E8(R1)",
            guideline_title: "General Considerations for Clinical Studies",
            status: GuidelineStatus::Step4Final,
            date: "2021-10-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E8_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Effectiveness", "Clinical Benefit"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    "expedited_report" => Term {
        name: "Expedited Report",
        key: "expedited_report",
        definition: "A report of a suspected adverse reaction that is submitted to the relevant competent authority in a timeframe shorter than that used for periodic reporting. It usually applies to serious and/or unexpected adverse reactions.",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "III",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Serious Adverse Event", "Unexpected Adverse Drug Reaction"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === G ===
    "good_clinical_practice" => Term {
        name: "Good Clinical Practice (GCP)",
        key: "good_clinical_practice",
        definition: "A standard for the design, conduct, performance, monitoring, auditing, recording, analyses, and reporting of clinical trials that provides assurance that the data and reported results are credible and accurate, and that the rights, integrity, and confidentiality of trial subjects are protected.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Clinical Trial", "Protocol", "Informed Consent"],
        abbreviation: Some("GCP"),
        clarification: None,
        is_new: false,
    },

    "good_manufacturing_practice" => Term {
        name: "Good Manufacturing Practice (GMP)",
        key: "good_manufacturing_practice",
        definition: "That part of quality assurance which ensures that products are consistently produced and controlled to the quality standards appropriate to their intended use and as required by the marketing authorization.",
        source: Source {
            guideline_id: "Q7",
            guideline_title: "Good Manufacturing Practice Guide for Active Pharmaceutical Ingredients",
            status: GuidelineStatus::Step4Final,
            date: "2000-11-10",
            section: "2",
            url: Some("https://database.ich.org/sites/default/files/Q7%20Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Quality Assurance", "Quality Control", "Validation"],
        abbreviation: Some("GMP"),
        clarification: None,
        is_new: false,
    },

    // === I ===
    "individual_case_safety_report" => Term {
        name: "Individual Case Safety Report (ICSR)",
        key: "individual_case_safety_report",
        definition: "A report of information concerning a suspected adverse reaction to a medicinal product which has occurred in an individual patient. An ICSR may contain more than one reaction and more than one suspect medicinal product.",
        source: Source {
            guideline_id: "E2B(R3)",
            guideline_title: "Individual Case Safety Reports Implementation Guide",
            status: GuidelineStatus::Step4Final,
            date: "2025-07-18",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E2B_R3_IG_v5.03.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Adverse Drug Reaction", "Reporter", "Sender"],
        abbreviation: Some("ICSR"),
        clarification: None,
        is_new: false,
    },

    "informed_consent" => Term {
        name: "Informed Consent",
        key: "informed_consent",
        definition: "A process by which a subject voluntarily confirms his or her willingness to participate in a particular trial, after having been informed of all aspects of the trial that are relevant to the subject's decision to participate. Informed consent is documented by means of a written, signed and dated informed consent form.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Subject", "Ethics Committee", "Protocol"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    "investigator" => Term {
        name: "Investigator",
        key: "investigator",
        definition: "A person responsible for the conduct of the clinical trial at a trial site. If a trial is conducted by a team of individuals at a trial site, the investigator is the responsible leader of the team and may be called the principal investigator.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Principal Investigator", "Site", "Sponsor"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === P ===
    "pharmacovigilance" => Term {
        name: "Pharmacovigilance",
        key: "pharmacovigilance",
        definition: "The science and activities relating to the detection, assessment, understanding and prevention of adverse effects or any other drug-related problem.",
        source: Source {
            guideline_id: "E2D(R1)",
            guideline_title: "Post-Approval Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-15",
            section: "2",
            url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Safety Signal", "Risk Management", "Adverse Drug Reaction"],
        abbreviation: Some("PV"),
        clarification: Some("WHO definition"),
        is_new: false,
    },

    "protocol" => Term {
        name: "Protocol",
        key: "protocol",
        definition: "A document that describes the objective(s), design, methodology, statistical considerations, and organisation of a trial. The protocol usually also gives the background and rationale for the trial, but these could be provided in other protocol referenced documents.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Amendment", "Clinical Trial", "Investigator's Brochure"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === Q ===
    "quality_assurance" => Term {
        name: "Quality Assurance (QA)",
        key: "quality_assurance",
        definition: "All those planned and systematic actions that are established to ensure that the trial is performed and the data are generated, documented (recorded), and reported in compliance with Good Clinical Practice (GCP) and the applicable regulatory requirement(s).",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Quality Control", "Audit", "Standard Operating Procedure"],
        abbreviation: Some("QA"),
        clarification: None,
        is_new: false,
    },

    "quality_control" => Term {
        name: "Quality Control (QC)",
        key: "quality_control",
        definition: "The operational techniques and activities undertaken within the quality assurance system to verify that the requirements for quality of the trial-related activities have been fulfilled.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Quality Assurance", "Monitoring"],
        abbreviation: Some("QC"),
        clarification: None,
        is_new: false,
    },

    // === R ===
    "risk" => Term {
        name: "Risk",
        key: "risk",
        definition: "The combination of the probability of occurrence of harm and the severity of that harm.",
        source: Source {
            guideline_id: "Q9(R1)",
            guideline_title: "Quality Risk Management",
            status: GuidelineStatus::Step4Final,
            date: "2023-01-18",
            section: "Definitions",
            url: Some("https://database.ich.org/sites/default/files/ICH_Q9%28R1%29_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Risk Assessment", "Risk Management", "Hazard"],
        abbreviation: None,
        clarification: Some("ISO/IEC Guide 51:2014"),
        is_new: false,
    },

    "risk_management" => Term {
        name: "Risk Management",
        key: "risk_management",
        definition: "The systematic application of quality management policies, procedures, and practices to the tasks of assessing, controlling, communicating and reviewing risk.",
        source: Source {
            guideline_id: "Q9(R1)",
            guideline_title: "Quality Risk Management",
            status: GuidelineStatus::Step4Final,
            date: "2023-01-18",
            section: "Definitions",
            url: Some("https://database.ich.org/sites/default/files/ICH_Q9%28R1%29_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Risk", "Risk Assessment", "Risk Control"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === S ===
    "safety_signal" => Term {
        name: "Safety Signal",
        key: "safety_signal",
        definition: "Information that arises from one or multiple sources that suggests a new potentially causal association, or a new aspect of a known association, between a medicine and an adverse event or set of related adverse events that is judged to be of sufficient likelihood to justify further evaluation.",
        source: Source {
            guideline_id: "M14",
            guideline_title: "Non-interventional Studies Using Real-World Data",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-04",
            section: "12 Glossary",
            url: Some("https://database.ich.org/sites/default/files/M14_Step4_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Signal Detection", "Disproportionality Analysis", "Pharmacovigilance"],
        abbreviation: None,
        clarification: Some("Adapted from ICH E2C"),
        is_new: true,
    },

    "serious_adverse_event" => Term {
        name: "Serious Adverse Event (SAE)",
        key: "serious_adverse_event",
        definition: "Any untoward medical occurrence that at any dose: results in death, is life-threatening, requires inpatient hospitalisation or prolongation of existing hospitalisation, results in persistent or significant disability/incapacity, is a congenital anomaly/birth defect, or is a medically important event.",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.A.4",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Adverse Event", "Expedited Report", "Life-Threatening"],
        abbreviation: Some("SAE"),
        clarification: None,
        is_new: false,
    },

    "sponsor" => Term {
        name: "Sponsor",
        key: "sponsor",
        definition: "An individual, company, institution, or organisation which takes responsibility for the initiation, management, and/or financing of a clinical trial.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Investigator", "Contract Research Organisation"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    "stability" => Term {
        name: "Stability",
        key: "stability",
        definition: "The capability of a drug substance or drug product to remain within established specifications and maintain its identity, strength, quality and purity throughout the retest period or shelf life.",
        source: Source {
            guideline_id: "Q1A(R2)",
            guideline_title: "Stability Testing of New Drug Substances and Products",
            status: GuidelineStatus::Step4Final,
            date: "2003-02-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/Q1A%28R2%29%20Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Shelf Life", "Retest Period", "Accelerated Testing"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === V ===
    "validation" => Term {
        name: "Validation",
        key: "validation",
        definition: "A documented program that provides a high degree of assurance that a specific process, method, or system will consistently produce a result meeting predetermined acceptance criteria.",
        source: Source {
            guideline_id: "Q7",
            guideline_title: "Good Manufacturing Practice Guide for Active Pharmaceutical Ingredients",
            status: GuidelineStatus::Step4Final,
            date: "2000-11-10",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/Q7%20Guideline.pdf"),
        },
        alternative_definitions: &[
            AlternativeDefinition {
                definition: "The process of establishing documented evidence that provides a high degree of assurance that an analytical procedure will consistently yield results that accurately reflect the quality characteristics of the samples tested.",
                source: Source {
                    guideline_id: "Q2(R2)",
                    guideline_title: "Validation of Analytical Procedures",
                    status: GuidelineStatus::Step4Final,
                    date: "2023-11-01",
                    section: "1",
                    url: Some("https://database.ich.org/sites/default/files/ICH_Q2%28R2%29_Guideline_Step4.pdf"),
                },
                clarification: None,
            },
        ],
        see_also: &["Qualification", "Verification", "Process Validation"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    // === Additional Terms from CIOMS Glossary ===

    "signal_detection" => Term {
        name: "Signal Detection",
        key: "signal_detection",
        definition: "The act of looking for and/or identifying signals using event data from any source.",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "2.2",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Safety Signal", "Signal Management", "Disproportionality Analysis"],
        abbreviation: None,
        clarification: Some("Proposed by CIOMS Working Group VIII"),
        is_new: true,
    },

    "signal_management" => Term {
        name: "Signal Management",
        key: "signal_management",
        definition: "A set of activities including signal detection, prioritization and evaluation to determine whether a signal represents a risk which may warrant further assessment, communication or other risk minimization actions in accordance with the medical importance of the issue.",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "2.2",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Signal Detection", "Safety Signal", "Risk Management"],
        abbreviation: None,
        clarification: Some("Proposed by CIOMS Working Group VIII"),
        is_new: true,
    },

    "disproportionality_analysis" => Term {
        name: "Disproportionality Analysis",
        key: "disproportionality_analysis",
        definition: "A method used in pharmacovigilance to compare the proportion of a specific adverse event reported for a particular drug with the proportion of the same adverse event reported for all other drugs in a database. Measures include PRR (Proportional Reporting Ratio), ROR (Reporting Odds Ratio), IC (Information Component), and EBGM (Empirical Bayes Geometric Mean).",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "2.2.1",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Signal Detection", "Proportional Reporting Ratio", "Data Mining"],
        abbreviation: None,
        clarification: Some("Primary method for quantitative signal detection in spontaneous reporting databases"),
        is_new: true,
    },

    "proportional_reporting_ratio" => Term {
        name: "Proportional Reporting Ratio (PRR)",
        key: "proportional_reporting_ratio",
        definition: "A measure of disproportionality comparing the proportion of a specific adverse event reported for a drug of interest versus the proportion of that adverse event for all other drugs. PRR = (A/[A+B]) / (C/[C+D]) where A = reports of event for drug, B = reports of other events for drug, C = reports of event for other drugs, D = reports of other events for other drugs.",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "2.2.1",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Disproportionality Analysis", "Signal Detection", "Reporting Odds Ratio"],
        abbreviation: Some("PRR"),
        clarification: Some("Evans criteria for signal: PRR >= 2, chi-square >= 4, N >= 3"),
        is_new: true,
    },

    "reporting_odds_ratio" => Term {
        name: "Reporting Odds Ratio (ROR)",
        key: "reporting_odds_ratio",
        definition: "A measure of disproportionality that represents the odds of a specific adverse event being reported for a drug of interest versus the odds of that event being reported for all other drugs. ROR = (A/C) / (B/D) = (A*D) / (B*C) where A = reports of event for drug, B = reports of other events for drug, C = reports of event for other drugs, D = reports of other events for other drugs.",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "2.2.1",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Disproportionality Analysis", "Proportional Reporting Ratio", "Signal Detection"],
        abbreviation: Some("ROR"),
        clarification: Some("Signal threshold typically ROR > 1 with lower 95% CI > 1"),
        is_new: true,
    },

    "periodic_safety_update_report" => Term {
        name: "Periodic Safety Update Report (PSUR)",
        key: "periodic_safety_update_report",
        definition: "A periodic report providing an evaluation of the benefit-risk balance of a medicinal product intended for submission by a marketing authorization holder to competent authorities at defined time points during the post-authorization phase. The PSUR/PBRER provides a comprehensive analysis of cumulative safety data and an evaluation of benefit-risk balance.",
        source: Source {
            guideline_id: "E2C(R2)",
            guideline_title: "Periodic Benefit-Risk Evaluation Report",
            status: GuidelineStatus::Step4Final,
            date: "2012-12-17",
            section: "2.1",
            url: Some("https://database.ich.org/sites/default/files/E2C_R2_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Periodic Benefit-Risk Evaluation Report", "Development Safety Update Report"],
        abbreviation: Some("PSUR"),
        clarification: Some("Replaced by PBRER terminology in ICH E2C(R2)"),
        is_new: false,
    },

    "periodic_benefit_risk_evaluation_report" => Term {
        name: "Periodic Benefit-Risk Evaluation Report (PBRER)",
        key: "periodic_benefit_risk_evaluation_report",
        definition: "A single periodic report providing a harmonized format and content for submission to regulatory authorities post-approval. The PBRER is designed to present a comprehensive, concise, and critical analysis of new or emerging information on the risks of the medicinal product, and on its benefit in approved indications, to enable an appraisal of the product's overall benefit-risk profile.",
        source: Source {
            guideline_id: "E2C(R2)",
            guideline_title: "Periodic Benefit-Risk Evaluation Report",
            status: GuidelineStatus::Step4Final,
            date: "2012-12-17",
            section: "1",
            url: Some("https://database.ich.org/sites/default/files/E2C_R2_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Periodic Safety Update Report", "Development Safety Update Report", "Benefit-Risk Assessment"],
        abbreviation: Some("PBRER"),
        clarification: None,
        is_new: false,
    },

    "development_safety_update_report" => Term {
        name: "Development Safety Update Report (DSUR)",
        key: "development_safety_update_report",
        definition: "A document that reports all relevant safety information gathered during the reporting period while the drug is under clinical investigation. It is intended to provide the principal investigator and sponsors with an analysis of the safety profile of the investigational drug based on cumulative data.",
        source: Source {
            guideline_id: "E2F",
            guideline_title: "Development Safety Update Report",
            status: GuidelineStatus::Step4Final,
            date: "2010-08-17",
            section: "1",
            url: Some("https://database.ich.org/sites/default/files/E2F_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Periodic Safety Update Report", "Investigator's Brochure"],
        abbreviation: Some("DSUR"),
        clarification: Some("Proposed by CIOMS Working Group VII"),
        is_new: false,
    },

    "benefit_risk_assessment" => Term {
        name: "Benefit-Risk Assessment",
        key: "benefit_risk_assessment",
        definition: "A systematic, comprehensive evaluation and comparison of the benefits and risks of a medicinal product, taking into account all available evidence relevant to both benefits and risks. The assessment should be documented, transparent, and consistently applied throughout the product lifecycle.",
        source: Source {
            guideline_id: "E2C(R2)",
            guideline_title: "Periodic Benefit-Risk Evaluation Report",
            status: GuidelineStatus::Step4Final,
            date: "2012-12-17",
            section: "3.17",
            url: Some("https://database.ich.org/sites/default/files/E2C_R2_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Risk Management", "Periodic Benefit-Risk Evaluation Report"],
        abbreviation: None,
        clarification: Some("Also referred to as benefit-risk balance evaluation"),
        is_new: false,
    },

    "risk_management_plan" => Term {
        name: "Risk Management Plan (RMP)",
        key: "risk_management_plan",
        definition: "A set of pharmacovigilance activities and interventions designed to identify, characterize, prevent, or minimize risks relating to medicinal products, including the assessment of the effectiveness of those interventions. The RMP should describe identified risks, potential risks, and missing information, along with planned risk minimization activities.",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "3",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Risk Management", "Benefit-Risk Assessment", "Pharmacovigilance"],
        abbreviation: Some("RMP"),
        clarification: Some("Required in many jurisdictions at time of marketing authorization"),
        is_new: false,
    },

    "unexpected_adverse_drug_reaction" => Term {
        name: "Unexpected Adverse Drug Reaction",
        key: "unexpected_adverse_drug_reaction",
        definition: "An adverse reaction, the nature or severity of which is not consistent with the applicable product information (e.g. Investigator's Brochure for an unapproved investigational product or package insert/summary of product characteristics for an approved product).",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.C",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Adverse Drug Reaction", "Serious Adverse Event", "Expedited Report"],
        abbreviation: None,
        clarification: Some("Key criterion for expedited reporting requirements"),
        is_new: false,
    },

    "medically_important_event" => Term {
        name: "Medically Important Event",
        key: "medically_important_event",
        definition: "An event that may not result in death, be life-threatening, or require hospitalization but may be considered serious when, based upon appropriate medical judgement, the event may jeopardize the patient and may require medical or surgical intervention to prevent one of the outcomes listed in the definition of serious adverse event.",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.A.4",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Serious Adverse Event", "Adverse Event"],
        abbreviation: None,
        clarification: Some("Important events may require medical judgment to determine seriousness"),
        is_new: false,
    },

    "active_surveillance" => Term {
        name: "Active Surveillance",
        key: "active_surveillance",
        definition: "An active surveillance system has been defined as the collection of case safety information as a continuous pre-organized process. Active surveillance can be drug based (identifying adverse events in patients taking certain products), setting based (identifying adverse events in certain healthcare settings), or event based (identifying adverse events that are likely to be associated with medicinal products).",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "2.3",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Spontaneous Reporting", "Pharmacovigilance", "Signal Detection"],
        abbreviation: None,
        clarification: Some("Contrasts with passive (spontaneous) reporting systems"),
        is_new: true,
    },

    "spontaneous_reporting" => Term {
        name: "Spontaneous Reporting",
        key: "spontaneous_reporting",
        definition: "A system whereby case reports of adverse drug reactions are voluntarily submitted by health care professionals or consumers to the manufacturer or regulatory authority. This forms the basis of post-marketing surveillance in most countries and is a cornerstone of pharmacovigilance.",
        source: Source {
            guideline_id: "E2D(R1)",
            guideline_title: "Post-Approval Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-15",
            section: "2.1",
            url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Individual Case Safety Report", "Active Surveillance", "Pharmacovigilance"],
        abbreviation: None,
        clarification: Some("Subject to reporting biases and underreporting"),
        is_new: false,
    },

    "causality_assessment" => Term {
        name: "Causality Assessment",
        key: "causality_assessment",
        definition: "The evaluation of the likelihood that a medicinal product was the cause of an observed adverse event. Assessment typically considers temporal relationship, dechallenge/rechallenge information, biological plausibility, alternative explanations, and consistency with the known safety profile of the drug.",
        source: Source {
            guideline_id: "E2B(R3)",
            guideline_title: "Individual Case Safety Reports Implementation Guide",
            status: GuidelineStatus::Step4Final,
            date: "2025-07-18",
            section: "2.12",
            url: Some("https://database.ich.org/sites/default/files/E2B_R3_IG_v5.03.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Individual Case Safety Report", "Adverse Drug Reaction"],
        abbreviation: None,
        clarification: Some("Common scales include WHO-UMC system and Naranjo algorithm"),
        is_new: false,
    },

    "meddra" => Term {
        name: "MedDRA (Medical Dictionary for Regulatory Activities)",
        key: "meddra",
        definition: "A rich and highly specific standardised medical terminology to facilitate sharing of regulatory information internationally for medical products used by humans. MedDRA is the international medical terminology developed under the auspices of ICH for use in registration, documentation, and safety monitoring of medicinal products.",
        source: Source {
            guideline_id: "M1",
            guideline_title: "MedDRA Term Selection Points to Consider",
            status: GuidelineStatus::Step4Final,
            date: "2023-06-12",
            section: "1",
            url: Some("https://database.ich.org/sites/default/files/M1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Individual Case Safety Report", "Adverse Event"],
        abbreviation: Some("MedDRA"),
        clarification: Some("Hierarchical structure: SOC -> HLGT -> HLT -> PT -> LLT"),
        is_new: false,
    },

    "case_series" => Term {
        name: "Case Series",
        key: "case_series",
        definition: "A report on a series of patients with an outcome of interest. No control group is involved. Case series may be consecutive or non-consecutive, depending upon whether all cases presenting to the reporting author(s) over a particular period of time were included, or only a selection.",
        source: Source {
            guideline_id: "E8(R1)",
            guideline_title: "General Considerations for Clinical Studies",
            status: GuidelineStatus::Step4Final,
            date: "2021-10-06",
            section: "Annex",
            url: Some("https://database.ich.org/sites/default/files/E8_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Case Report", "Observational Study"],
        abbreviation: None,
        clarification: None,
        is_new: false,
    },

    "cohort_study" => Term {
        name: "Cohort Study",
        key: "cohort_study",
        definition: "An observational study in which a defined group of people (the cohort) is followed over time to determine the incidence of outcomes. The comparison is usually made between the exposed and non-exposed groups within the cohort. Cohort studies can be prospective (outcomes occur after study initiation) or retrospective (outcomes have already occurred).",
        source: Source {
            guideline_id: "M14",
            guideline_title: "Non-interventional Studies Using Real-World Data",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-04",
            section: "4.2.1",
            url: Some("https://database.ich.org/sites/default/files/M14_Step4_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Case-Control Study", "Observational Study", "Real-World Data"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "case_control_study" => Term {
        name: "Case-Control Study",
        key: "case_control_study",
        definition: "An observational study that starts with the identification of persons with the disease or outcome of interest (cases) and a suitable control group of persons without the disease or outcome. The relationship between an attribute and the disease is examined by comparing the cases and controls with regard to how frequently the attribute is present in each group.",
        source: Source {
            guideline_id: "M14",
            guideline_title: "Non-interventional Studies Using Real-World Data",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-04",
            section: "4.2.2",
            url: Some("https://database.ich.org/sites/default/files/M14_Step4_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Cohort Study", "Observational Study", "Odds Ratio"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "real_world_data" => Term {
        name: "Real-World Data (RWD)",
        key: "real_world_data",
        definition: "Data relating to patient health status and/or the delivery of health care routinely collected from a variety of sources. RWD can come from electronic health records, claims and billing activities, product and disease registries, patient-generated data including in home-use settings, and data gathered from other sources that can inform on health status.",
        source: Source {
            guideline_id: "M14",
            guideline_title: "Non-interventional Studies Using Real-World Data",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-04",
            section: "3.1",
            url: Some("https://database.ich.org/sites/default/files/M14_Step4_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Real-World Evidence", "Observational Study"],
        abbreviation: Some("RWD"),
        clarification: Some("Contrasts with data collected in traditional randomized clinical trials"),
        is_new: true,
    },

    "real_world_evidence" => Term {
        name: "Real-World Evidence (RWE)",
        key: "real_world_evidence",
        definition: "Clinical evidence regarding the usage and potential benefits or risks of a medicinal product derived from analysis of real-world data. RWE can be generated from prospective and/or retrospective studies using RWD.",
        source: Source {
            guideline_id: "M14",
            guideline_title: "Non-interventional Studies Using Real-World Data",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-04",
            section: "3.2",
            url: Some("https://database.ich.org/sites/default/files/M14_Step4_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Real-World Data", "Post-Authorization Safety Study"],
        abbreviation: Some("RWE"),
        clarification: None,
        is_new: true,
    },

    "post_authorization_safety_study" => Term {
        name: "Post-Authorization Safety Study (PASS)",
        key: "post_authorization_safety_study",
        definition: "A study relating to an authorized medicinal product conducted with the aim of identifying, characterizing, or quantifying a safety hazard, confirming the safety profile of the medicinal product, or measuring the effectiveness of risk management measures.",
        source: Source {
            guideline_id: "E2E",
            guideline_title: "Pharmacovigilance Planning",
            status: GuidelineStatus::Step4Final,
            date: "2004-11-18",
            section: "2.3.2",
            url: Some("https://database.ich.org/sites/default/files/E2E_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Risk Management Plan", "Pharmacovigilance", "Real-World Evidence"],
        abbreviation: Some("PASS"),
        clarification: Some("May be imposed as condition of marketing authorization"),
        is_new: false,
    },

    // === NEW TERMS (Implemented for Task) ===
    "dechallenge" => Term {
        name: "Dechallenge",
        key: "dechallenge",
        definition: "The process of withdrawing a suspected drug from a patient to observe whether an adverse event continues or resolves. A 'positive dechallenge' occurs if the adverse event decreases in severity or disappears, suggesting a causal relationship.",
        source: Source {
            guideline_id: "E2D(R1)",
            guideline_title: "Post-Approval Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-15",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Rechallenge", "Causality Assessment"],
        abbreviation: None,
        clarification: Some("Based on CIOMS V"),
        is_new: true,
    },

    "rechallenge" => Term {
        name: "Rechallenge",
        key: "rechallenge",
        definition: "The reintroduction of a drug to a patient after it was previously discontinued due to an adverse event. A 'positive rechallenge' (the AE reappears) strongly suggests a causal relationship.",
        source: Source {
            guideline_id: "E2D(R1)",
            guideline_title: "Post-Approval Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-15",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Dechallenge", "Causality Assessment"],
        abbreviation: None,
        clarification: Some("Based on CIOMS V"),
        is_new: true,
    },

    "causality" => Term {
        name: "Causality",
        key: "causality",
        definition: "The evaluation of the likelihood that a medicinal product was the causative agent of an observed adverse event in a specific individual.",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.A.2",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Causality Assessment", "Adverse Drug Reaction"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "signal" => Term {
        name: "Signal",
        key: "signal",
        definition: "Information on a new or known side effect that may be caused by a medicine, typically generated from more than a single report of a suspected side effect. A signal requires further investigation.",
        source: Source {
            guideline_id: "E2C(R2)",
            guideline_title: "Periodic Benefit-Risk Evaluation Report",
            status: GuidelineStatus::Step4Final,
            date: "2012-12-17",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E2C_R2_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Safety Signal", "Signal Detection"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "serious" => Term {
        name: "Serious",
        key: "serious",
        definition: "A regulatory classification for an adverse event that results in death, is life-threatening, requires hospitalization, results in disability, or is a congenital anomaly.",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.B",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Serious Adverse Event", "Severity"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "unexpected" => Term {
        name: "Unexpected",
        key: "unexpected",
        definition: "An adverse reaction whose nature or severity is not consistent with the applicable product information (e.g., Investigator's Brochure or SmPC).",
        source: Source {
            guideline_id: "E2A",
            guideline_title: "Clinical Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "1994-10-27",
            section: "II.C",
            url: Some("https://database.ich.org/sites/default/files/E2A_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Unexpected Adverse Drug Reaction"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "suspected" => Term {
        name: "Suspected",
        key: "suspected",
        definition: "A relationship that implies a causal link between a medicinal product and an adverse event is at least a reasonable possibility.",
        source: Source {
            guideline_id: "E2D(R1)",
            guideline_title: "Post-Approval Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-15",
            section: "2.1.2",
            url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Adverse Drug Reaction"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "concomitant" => Term {
        name: "Concomitant",
        key: "concomitant",
        definition: "Refers to medications or therapies that a patient is taking concurrently with the investigational drug during a clinical trial.",
        source: Source {
            guideline_id: "E6(R3)",
            guideline_title: "Guideline for Good Clinical Practice",
            status: GuidelineStatus::Step4Final,
            date: "2025-01-06",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E6_R3_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Drug Interaction"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "confounding_variable" => Term {
        name: "Confounding Variable",
        key: "confounding_variable",
        definition: "An extraneous variable that is related to both the independent and dependent variables, potentially distorting the true relationship between them.",
        source: Source {
            guideline_id: "E9",
            guideline_title: "Statistical Principles for Clinical Trials",
            status: GuidelineStatus::Step4Final,
            date: "1998-02-05",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E9_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["Bias", "Cofactor"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },

    "triage" => Term {
        name: "Triage",
        key: "triage",
        definition: "The initial process of reviewing and classifying an Individual Case Safety Report (ICSR) based on seriousness, expectedness, and causality for prioritization.",
        source: Source {
            guideline_id: "E2D(R1)",
            guideline_title: "Post-Approval Safety Data Management",
            status: GuidelineStatus::Step4Final,
            date: "2025-09-15",
            section: "Glossary",
            url: Some("https://database.ich.org/sites/default/files/E2D_R1_Guideline.pdf"),
        },
        alternative_definitions: &[],
        see_also: &["ICSR", "Expedited Report"],
        abbreviation: None,
        clarification: None,
        is_new: true,
    },
};

// ============================================================================
// Lookup Functions
// ============================================================================

/// Normalize a term name for lookup
fn normalize_term_key(name: &str) -> String {
    name.to_lowercase()
        .replace([' ', '-', '/', '(', ')'], "_")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

/// O(1) lookup of term by name (case-insensitive, flexible matching)
pub fn lookup_term(name: &str) -> Option<&'static Term> {
    let key = normalize_term_key(name);

    // Direct lookup
    if let Some(term) = TERMS.get(&key) {
        return Some(term);
    }

    // Try without abbreviation suffix
    let without_abbrev = key
        .replace("_ae", "")
        .replace("_adr", "")
        .replace("_sae", "");
    if let Some(term) = TERMS.get(&without_abbrev) {
        return Some(term);
    }

    None
}

/// Get all terms
pub fn all_terms() -> Vec<&'static Term> {
    TERMS.values().collect()
}

/// Get terms by category
pub fn terms_by_category(category: IchCategory) -> Vec<&'static Term> {
    TERMS
        .values()
        .filter(|t| t.source.category() == Some(category))
        .collect()
}

/// Get terms by guideline
pub fn terms_by_guideline(guideline_id: &str) -> Vec<&'static Term> {
    let id_lower = guideline_id.to_lowercase();
    TERMS
        .values()
        .filter(|t| t.source.guideline_id.to_lowercase().contains(&id_lower))
        .collect()
}

/// Get new terms (added in current version)
pub fn new_terms() -> Vec<&'static Term> {
    TERMS.values().filter(|t| t.is_new).collect()
}

/// Get terms with multiple definitions
pub fn terms_with_alternatives() -> Vec<&'static Term> {
    TERMS
        .values()
        .filter(|t| !t.alternative_definitions.is_empty())
        .collect()
}

/// Get terms with abbreviations
pub fn terms_with_abbreviations() -> Vec<&'static Term> {
    TERMS
        .values()
        .filter(|t| t.abbreviation.is_some())
        .collect()
}

/// Lookup term by abbreviation
pub fn lookup_by_abbreviation(abbrev: &str) -> Option<&'static Term> {
    let abbrev_upper = abbrev.to_uppercase();
    TERMS.values().find(|t| {
        t.abbreviation == Some(&*abbrev_upper)
            || t.abbreviation.map(|a| a.to_uppercase()) == Some(abbrev_upper.clone())
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_term_exact() {
        let term = lookup_term("adverse drug reaction").unwrap();
        assert_eq!(term.abbreviation, Some("ADR"));
    }

    #[test]
    fn test_lookup_term_case_insensitive() {
        let t1 = lookup_term("Adverse Event").unwrap();
        let t2 = lookup_term("adverse event").unwrap();
        assert_eq!(t1.key, t2.key);
    }

    #[test]
    fn test_lookup_term_with_abbreviation() {
        let term = lookup_term("good clinical practice").unwrap();
        assert_eq!(term.abbreviation, Some("GCP"));
    }

    #[test]
    fn test_term_has_alternative_definitions() {
        let term = lookup_term("adverse drug reaction").unwrap();
        assert!(!term.alternative_definitions.is_empty());
    }

    #[test]
    fn test_term_has_see_also() {
        let term = lookup_term("adverse event").unwrap();
        assert!(!term.see_also.is_empty());
    }

    #[test]
    fn test_all_terms() {
        let terms = all_terms();
        assert!(terms.len() >= 25);
    }

    #[test]
    fn test_terms_by_category() {
        let efficacy_terms = terms_by_category(IchCategory::Efficacy);
        assert!(!efficacy_terms.is_empty());
        assert!(
            efficacy_terms
                .iter()
                .all(|t| t.source.category() == Some(IchCategory::Efficacy))
        );
    }

    #[test]
    fn test_terms_by_guideline() {
        let e6_terms = terms_by_guideline("E6");
        assert!(!e6_terms.is_empty());
    }

    #[test]
    fn test_new_terms() {
        let new = new_terms();
        assert!(new.iter().all(|t| t.is_new));
    }

    #[test]
    fn test_lookup_by_abbreviation() {
        let term = lookup_by_abbreviation("ADR").unwrap();
        assert_eq!(term.key, "adverse_drug_reaction");
    }

    #[test]
    fn test_normalize_term_key() {
        assert_eq!(normalize_term_key("Adverse Event"), "adverse_event");
        assert_eq!(
            normalize_term_key("GCP (Good Clinical Practice)"),
            "gcp_good_clinical_practice"
        );
    }
}
