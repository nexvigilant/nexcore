//! Regulatory Directory — searchable catalog of PV regulatory documents
//!
//! 60 key documents from FDA, EMA, ICH, WHO, CIOMS, MHRA, PMDA, TGA, Health Canada.
//! Data sourced from PV Regulatory Directory Framework Master Directory.

use leptos::prelude::*;

#[derive(Clone, Copy)]
struct RegDoc {
    id: &'static str,
    title: &'static str,
    jurisdiction: &'static str,
    doc_type: &'static str,
    status: &'static str,
    activity: &'static str,
    risk_level: &'static str,
    summary: &'static str,
    url: &'static str,
}

const DOCS: &[RegDoc] = &[
    /* === FDA — Code of Federal Regulations === */
    RegDoc {
        id: "FDA-CFR-001",
        title: "21 CFR 312.32 — IND Safety Reporting",
        jurisdiction: "FDA",
        doc_type: "Regulation",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Requires sponsors to notify FDA and investigators of serious and unexpected ADRs. 7-day alert for fatal/life-threatening; 15-day written report for all serious unexpected.",
        url: "https://www.ecfr.gov/current/title-21/chapter-I/subchapter-D/part-312/subpart-B/section-312.32",
    },
    RegDoc {
        id: "FDA-CFR-002",
        title: "21 CFR 314.80 — Post-Marketing Reporting of ADEs",
        jurisdiction: "FDA",
        doc_type: "Regulation",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Requires NDA holders to report serious, unexpected ADEs within 15 days. Periodic reporting (quarterly years 1-3, annually thereafter) via PADER.",
        url: "https://www.ecfr.gov/current/title-21/chapter-I/subchapter-D/part-314/subpart-B/section-314.80",
    },
    RegDoc {
        id: "FDA-CFR-003",
        title: "21 CFR 314.81 — Annual Reports for NDAs",
        jurisdiction: "FDA",
        doc_type: "Regulation",
        status: "Active",
        activity: "Periodic Reporting",
        risk_level: "High",
        summary: "Annual progress reports including distribution data, labeling, chemistry changes, non-clinical studies, and clinical data.",
        url: "https://www.ecfr.gov/current/title-21/chapter-I/subchapter-D/part-314/subpart-B/section-314.81",
    },
    RegDoc {
        id: "FDA-CFR-004",
        title: "21 CFR 600.80 — Post-Marketing Reporting for Biologics",
        jurisdiction: "FDA",
        doc_type: "Regulation",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Biologics-specific safety reporting requirements parallel to 314.80. 15-day reports for serious unexpected ADRs.",
        url: "https://www.ecfr.gov/current/title-21/chapter-I/subchapter-F/part-600/subpart-D/section-600.80",
    },
    RegDoc {
        id: "FDA-STAT-001",
        title: "FDCA Section 505(k) — Safety Reporting Requirements",
        jurisdiction: "FDA",
        doc_type: "Statute",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Statutory authority for FDA post-marketing safety reporting. Mandates records and reports for approved drugs.",
        url: "",
    },
    RegDoc {
        id: "FDA-STAT-002",
        title: "FDAAA Section 901 — REMS Authority",
        jurisdiction: "FDA",
        doc_type: "Statute",
        status: "Active",
        activity: "Risk Management",
        risk_level: "Critical",
        summary: "Grants FDA authority to require REMS when necessary to ensure benefits outweigh risks. Includes ETASU provisions.",
        url: "",
    },
    RegDoc {
        id: "FDA-GUID-001",
        title: "Safety Reporting Requirements for INDs and BA/BE Studies",
        jurisdiction: "FDA",
        doc_type: "Guidance",
        status: "Final",
        activity: "Safety Reporting",
        risk_level: "High",
        summary: "Guidance on IND safety reporting under 21 CFR 312.32. Clarifies what constitutes a suspected adverse reaction and reporting obligations.",
        url: "",
    },
    RegDoc {
        id: "FDA-GUID-002",
        title: "Postmarketing Safety Reporting for Combination Products",
        jurisdiction: "FDA",
        doc_type: "Guidance",
        status: "Final",
        activity: "Safety Reporting",
        risk_level: "High",
        summary: "Addresses how combination product applicants should comply with post-marketing safety reporting requirements.",
        url: "",
    },
    RegDoc {
        id: "FDA-GUID-003",
        title: "Good Pharmacovigilance Practices and Pharmacoepidemiologic Assessment",
        jurisdiction: "FDA",
        doc_type: "Guidance",
        status: "Final",
        activity: "Signal Detection",
        risk_level: "High",
        summary: "FDA guidance on pharmacovigilance practices including signal detection, pharmacoepidemiologic studies, and risk management.",
        url: "",
    },
    /* === ICH Guidelines === */
    RegDoc {
        id: "ICH-E2A",
        title: "ICH E2A — Clinical Safety Data Management: Definitions and Standards for Expedited Reporting",
        jurisdiction: "ICH",
        doc_type: "Guideline",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Defines ADR, serious, unexpected. Establishes 7/15-day expedited reporting framework adopted by all ICH regions.",
        url: "https://database.ich.org/sites/default/files/E2A_Guideline.pdf",
    },
    RegDoc {
        id: "ICH-E2B-R3",
        title: "ICH E2B(R3) — Individual Case Safety Reports (ICSRs)",
        jurisdiction: "ICH",
        doc_type: "Guideline",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Defines ICSR data elements and XML message format for electronic transmission between stakeholders.",
        url: "https://database.ich.org/sites/default/files/E2B_R3__Guideline.pdf",
    },
    RegDoc {
        id: "ICH-E2C-R2",
        title: "ICH E2C(R2) — Periodic Benefit-Risk Evaluation Report (PBRER)",
        jurisdiction: "ICH",
        doc_type: "Guideline",
        status: "Active",
        activity: "Periodic Reporting",
        risk_level: "Critical",
        summary: "Standard format for periodic reporting. Replaced PSUR. Comprehensive B-R evaluation at defined intervals.",
        url: "https://database.ich.org/sites/default/files/E2C_R2_Guideline.pdf",
    },
    RegDoc {
        id: "ICH-E2D",
        title: "ICH E2D — Post-Approval Safety Data Management",
        jurisdiction: "ICH",
        doc_type: "Guideline",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "High",
        summary: "Standards for post-approval expedited and periodic reporting, literature review, and special situations.",
        url: "https://database.ich.org/sites/default/files/E2D_Guideline.pdf",
    },
    RegDoc {
        id: "ICH-E2E",
        title: "ICH E2E — Pharmacovigilance Planning",
        jurisdiction: "ICH",
        doc_type: "Guideline",
        status: "Active",
        activity: "Risk Management",
        risk_level: "High",
        summary: "Framework for pharmacovigilance planning including safety specification and PV plan development.",
        url: "https://database.ich.org/sites/default/files/E2E_Guideline.pdf",
    },
    RegDoc {
        id: "ICH-E2F",
        title: "ICH E2F — Development Safety Update Report (DSUR)",
        jurisdiction: "ICH",
        doc_type: "Guideline",
        status: "Active",
        activity: "Periodic Reporting",
        risk_level: "High",
        summary: "Annual safety report during drug development. Comprehensive review of clinical trial safety data.",
        url: "https://database.ich.org/sites/default/files/E2F_Guideline.pdf",
    },
    RegDoc {
        id: "ICH-E6-R2",
        title: "ICH E6(R2) — Good Clinical Practice",
        jurisdiction: "ICH",
        doc_type: "Guideline",
        status: "Active",
        activity: "Clinical Trials",
        risk_level: "Critical",
        summary: "International standard for clinical trial design, conduct, recording, and reporting. Section 4.11 covers safety reporting.",
        url: "https://database.ich.org/sites/default/files/E6_R2__Addendum.pdf",
    },
    RegDoc {
        id: "ICH-M1",
        title: "ICH M1 — MedDRA: Medical Dictionary for Regulatory Activities",
        jurisdiction: "ICH",
        doc_type: "Standard",
        status: "Active",
        activity: "Coding",
        risk_level: "High",
        summary: "Standardised medical terminology for regulatory communication. 5-level hierarchy from LLT to SOC.",
        url: "https://www.meddra.org/",
    },
    /* === EMA / EU Regulations === */
    RegDoc {
        id: "EU-REG-001",
        title: "Regulation (EC) No 726/2004 — Centralised Procedure",
        jurisdiction: "EMA",
        doc_type: "Regulation",
        status: "Active",
        activity: "Authorisation & PV",
        risk_level: "Critical",
        summary: "EU regulation establishing the centralised marketing authorisation procedure and pharmacovigilance obligations.",
        url: "",
    },
    RegDoc {
        id: "EU-DIR-001",
        title: "Directive 2001/83/EC — Community Code for Medicinal Products",
        jurisdiction: "EMA",
        doc_type: "Directive",
        status: "Active",
        activity: "Authorisation & PV",
        risk_level: "Critical",
        summary: "EU directive establishing requirements for marketing authorisation, manufacturing, labelling, and pharmacovigilance.",
        url: "",
    },
    RegDoc {
        id: "EU-IR-001",
        title: "Commission Implementing Regulation (EU) No 520/2012",
        jurisdiction: "EMA",
        doc_type: "Regulation",
        status: "Active",
        activity: "PV Operations",
        risk_level: "High",
        summary: "Detailed rules for pharmacovigilance activities including PSMF, QPPV requirements, and signal management process.",
        url: "",
    },
    RegDoc {
        id: "GVP-I",
        title: "GVP Module I — Pharmacovigilance Systems and Quality Systems",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "PV Operations",
        risk_level: "Critical",
        summary: "Requirements for PSMF, QPPV, quality system, and overall PV system structure.",
        url: "https://www.ema.europa.eu/en/human-regulatory-overview/post-authorisation/pharmacovigilance/good-pharmacovigilance-practices",
    },
    RegDoc {
        id: "GVP-V",
        title: "GVP Module V — Risk Management Systems",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Risk Management",
        risk_level: "Critical",
        summary: "EU Risk Management Plan requirements including safety specification, PV plan, and risk minimisation measures.",
        url: "",
    },
    RegDoc {
        id: "GVP-VI",
        title: "GVP Module VI — Collection, Management and Submission of ICSRs",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Detailed requirements for ICSR handling including collection, quality, coding, and EudraVigilance submission.",
        url: "",
    },
    RegDoc {
        id: "GVP-VII",
        title: "GVP Module VII — Periodic Safety Update Report",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Periodic Reporting",
        risk_level: "Critical",
        summary: "EU-specific requirements for PBRER preparation and submission per EURD list schedule.",
        url: "",
    },
    RegDoc {
        id: "GVP-VIII",
        title: "GVP Module VIII — Post-Authorisation Safety Studies (PASS)",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Post-Marketing Studies",
        risk_level: "High",
        summary: "Framework for conducting PASS including imposed and voluntary studies, EU PAS register requirements.",
        url: "",
    },
    RegDoc {
        id: "GVP-IX",
        title: "GVP Module IX — Signal Management",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Signal Detection",
        risk_level: "Critical",
        summary: "EU signal management process: detection, validation, analysis, prioritisation, and assessment. PRAC signal procedure.",
        url: "",
    },
    RegDoc {
        id: "GVP-X",
        title: "GVP Module X — Additional Monitoring",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Monitoring",
        risk_level: "High",
        summary: "Black triangle scheme for products under additional monitoring. Enhanced ADR reporting requirements.",
        url: "",
    },
    RegDoc {
        id: "GVP-XV",
        title: "GVP Module XV — Safety Communication",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Risk Communication",
        risk_level: "High",
        summary: "Guidance on safety communication including DHPC, press releases, and stakeholder engagement.",
        url: "",
    },
    RegDoc {
        id: "GVP-XVI",
        title: "GVP Module XVI — Risk Minimisation Measures",
        jurisdiction: "EMA",
        doc_type: "GVP Module",
        status: "Active",
        activity: "Risk Management",
        risk_level: "High",
        summary: "Selection, implementation, and effectiveness evaluation of routine and additional risk minimisation measures.",
        url: "",
    },
    /* === WHO === */
    RegDoc {
        id: "WHO-001",
        title: "WHO International Drug Monitoring Programme",
        jurisdiction: "WHO",
        doc_type: "Programme",
        status: "Active",
        activity: "Global PV",
        risk_level: "High",
        summary: "WHO programme coordinating global ADR monitoring through national pharmacovigilance centres and VigiBase.",
        url: "",
    },
    RegDoc {
        id: "WHO-002",
        title: "WHO Guidelines on Safety Monitoring of Herbal Medicines",
        jurisdiction: "WHO",
        doc_type: "Guideline",
        status: "Active",
        activity: "Safety Monitoring",
        risk_level: "Medium",
        summary: "Framework for monitoring safety of herbal medicines within national PV systems.",
        url: "",
    },
    RegDoc {
        id: "WHO-003",
        title: "WHO Pharmacovigilance Indicators: A Practical Manual",
        jurisdiction: "WHO",
        doc_type: "Manual",
        status: "Active",
        activity: "PV Assessment",
        risk_level: "Medium",
        summary: "Structural, process, and outcome indicators for assessing PV system performance.",
        url: "",
    },
    /* === CIOMS === */
    RegDoc {
        id: "CIOMS-V",
        title: "CIOMS V — Current Challenges in Pharmacovigilance",
        jurisdiction: "CIOMS",
        doc_type: "Report",
        status: "Active",
        activity: "PV Best Practices",
        risk_level: "Medium",
        summary: "Pragmatic approaches to pharmacovigilance including signal detection, benefit-risk assessment, and communication.",
        url: "",
    },
    RegDoc {
        id: "CIOMS-VIII",
        title: "CIOMS VIII — Signal Detection",
        jurisdiction: "CIOMS",
        doc_type: "Report",
        status: "Active",
        activity: "Signal Detection",
        risk_level: "High",
        summary: "Comprehensive guide to signal detection methodologies including statistical methods, data sources, and governance.",
        url: "",
    },
    RegDoc {
        id: "CIOMS-IX",
        title: "CIOMS IX — Practical Approaches to Risk Minimisation",
        jurisdiction: "CIOMS",
        doc_type: "Report",
        status: "Active",
        activity: "Risk Management",
        risk_level: "High",
        summary: "Framework for selecting and evaluating risk minimisation measures across jurisdictions.",
        url: "",
    },
    RegDoc {
        id: "CIOMS-X",
        title: "CIOMS X — Meta-Analysis and Safety",
        jurisdiction: "CIOMS",
        doc_type: "Report",
        status: "Active",
        activity: "Safety Analysis",
        risk_level: "Medium",
        summary: "Guidance on using meta-analysis for safety evaluation, including systematic reviews and evidence synthesis.",
        url: "",
    },
    /* === MHRA (UK) === */
    RegDoc {
        id: "MHRA-001",
        title: "UK Human Medicines Regulations 2012",
        jurisdiction: "MHRA",
        doc_type: "Regulation",
        status: "Active",
        activity: "PV Requirements",
        risk_level: "Critical",
        summary: "UK statutory framework for medicines regulation including pharmacovigilance obligations post-Brexit.",
        url: "",
    },
    RegDoc {
        id: "MHRA-002",
        title: "MHRA GVP — UK-Specific Annexes",
        jurisdiction: "MHRA",
        doc_type: "Guidance",
        status: "Active",
        activity: "PV Operations",
        risk_level: "High",
        summary: "UK-specific pharmacovigilance guidance supplementing EU GVP modules, addressing post-Brexit requirements.",
        url: "",
    },
    RegDoc {
        id: "MHRA-003",
        title: "Yellow Card Scheme — ADR Reporting",
        jurisdiction: "MHRA",
        doc_type: "Programme",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "High",
        summary: "UK national ADR reporting system for healthcare professionals and patients. Over 60 years of operation.",
        url: "https://yellowcard.mhra.gov.uk/",
    },
    /* === PMDA (Japan) === */
    RegDoc {
        id: "PMDA-001",
        title: "J-GVP — Japanese Good Vigilance Practice",
        jurisdiction: "PMDA",
        doc_type: "Regulation",
        status: "Active",
        activity: "PV Operations",
        risk_level: "Critical",
        summary: "Japanese GVP regulations for safety management of drugs including collection, evaluation, and reporting of safety information.",
        url: "",
    },
    RegDoc {
        id: "PMDA-002",
        title: "PMDA Safety Measures Consultation",
        jurisdiction: "PMDA",
        doc_type: "Programme",
        status: "Active",
        activity: "Risk Management",
        risk_level: "High",
        summary: "PMDA consultation process for safety measures, including RMP Japan and additional pharmacovigilance activities.",
        url: "",
    },
    /* === TGA (Australia) === */
    RegDoc {
        id: "TGA-001",
        title: "Therapeutic Goods Act 1989 — Safety Provisions",
        jurisdiction: "TGA",
        doc_type: "Legislation",
        status: "Active",
        activity: "PV Requirements",
        risk_level: "Critical",
        summary: "Australian legislative framework for therapeutic goods regulation including adverse event reporting obligations.",
        url: "",
    },
    RegDoc {
        id: "TGA-002",
        title: "TGA Pharmacovigilance Responsibilities of Sponsors",
        jurisdiction: "TGA",
        doc_type: "Guidance",
        status: "Active",
        activity: "PV Operations",
        risk_level: "High",
        summary: "TGA guidance on sponsor obligations for adverse event monitoring, reporting, and risk management in Australia.",
        url: "",
    },
    /* === Health Canada === */
    RegDoc {
        id: "HC-001",
        title: "Food and Drug Regulations C.01.017 — Adverse Reaction Reporting",
        jurisdiction: "Health Canada",
        doc_type: "Regulation",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "Critical",
        summary: "Canadian regulation requiring MAHs to report serious ADRs within 15 days and all ADRs annually.",
        url: "",
    },
    RegDoc {
        id: "HC-002",
        title: "Guidance: Mandatory Reporting of Serious ADRs",
        jurisdiction: "Health Canada",
        doc_type: "Guidance",
        status: "Active",
        activity: "Safety Reporting",
        risk_level: "High",
        summary: "Health Canada guidance on mandatory reporting including definitions, timelines, and Canada Vigilance requirements.",
        url: "",
    },
];

#[component]
pub fn DirectoryPage() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let jurisdiction_filter = RwSignal::new(String::from("All"));
    let activity_filter = RwSignal::new(String::from("All"));
    let risk_filter = RwSignal::new(String::from("All"));

    let jurisdictions = vec![
        "All",
        "FDA",
        "ICH",
        "EMA",
        "WHO",
        "CIOMS",
        "MHRA",
        "PMDA",
        "TGA",
        "Health Canada",
    ];
    let risk_levels = vec!["All", "Critical", "High", "Medium"];

    let filtered = Signal::derive(move || {
        let q = search.get().to_lowercase();
        let jur = jurisdiction_filter.get();
        let act = activity_filter.get();
        let risk = risk_filter.get();
        DOCS.iter()
            .filter(|d| {
                let jur_match = jur == "All" || d.jurisdiction == jur;
                let act_match = act == "All" || d.activity == act;
                let risk_match = risk == "All" || d.risk_level == risk;
                let search_match = q.is_empty()
                    || d.title.to_lowercase().contains(&q)
                    || d.id.to_lowercase().contains(&q)
                    || d.summary.to_lowercase().contains(&q)
                    || d.activity.to_lowercase().contains(&q);
                jur_match && act_match && risk_match && search_match
            })
            .collect::<Vec<_>>()
    });

    let count = Signal::derive(move || filtered.get().len());

    /* Stats */
    let total = DOCS.len();
    let critical_count = DOCS.iter().filter(|d| d.risk_level == "Critical").count();

    view! {
        <div class="mx-auto max-w-7xl px-4 py-8">
            <header class="mb-8">
                <h1 class="text-3xl font-black text-white font-mono uppercase tracking-tight">"Regulatory Directory"</h1>
                <p class="mt-2 text-slate-400 max-w-3xl">
                    "Comprehensive catalog of global PV regulatory documents. Filterable by jurisdiction, activity, and compliance risk level."
                </p>
            </header>

            /* Stats bar */
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-3 text-center">
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">"Total Documents"</p>
                    <p class="text-lg font-black text-white font-mono">{total.to_string()}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-3 text-center">
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">"Jurisdictions"</p>
                    <p class="text-lg font-black text-cyan-400 font-mono">"9"</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-3 text-center">
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">"Critical Risk"</p>
                    <p class="text-lg font-black text-red-400 font-mono">{critical_count.to_string()}</p>
                </div>
                <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-3 text-center">
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">"Active"</p>
                    <p class="text-lg font-black text-emerald-400 font-mono">{total.to_string()}</p>
                </div>
            </div>

            /* Search + Filters */
            <div class="space-y-3 mb-6">
                <input
                    type="text"
                    placeholder="Search documents by title, ID, or description..."
                    class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />

                <div class="flex flex-wrap gap-4">
                    /* Jurisdiction */
                    <div>
                        <p class="text-[8px] font-bold text-slate-600 uppercase tracking-widest mb-1.5">"Jurisdiction"</p>
                        <div class="flex flex-wrap gap-1">
                            {jurisdictions.into_iter().map(|j| {
                                let jv = j.to_string();
                                view! {
                                    <button
                                        on:click={let jv = jv.clone(); move |_| jurisdiction_filter.set(jv.clone())}
                                        class=move || {
                                            if jurisdiction_filter.get() == j {
                                                "px-2 py-1 rounded bg-cyan-500/20 border border-cyan-500/40 text-[9px] font-bold text-cyan-400 font-mono"
                                            } else {
                                                "px-2 py-1 rounded border border-slate-700 text-[9px] font-bold text-slate-500 font-mono hover:border-slate-500 transition-all"
                                            }
                                        }
                                    >{j}</button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    /* Risk Level */
                    <div>
                        <p class="text-[8px] font-bold text-slate-600 uppercase tracking-widest mb-1.5">"Risk Level"</p>
                        <div class="flex gap-1">
                            {risk_levels.into_iter().map(|r| {
                                let rv = r.to_string();
                                view! {
                                    <button
                                        on:click={let rv = rv.clone(); move |_| risk_filter.set(rv.clone())}
                                        class=move || {
                                            if risk_filter.get() == r {
                                                "px-2 py-1 rounded bg-cyan-500/20 border border-cyan-500/40 text-[9px] font-bold text-cyan-400 font-mono"
                                            } else {
                                                "px-2 py-1 rounded border border-slate-700 text-[9px] font-bold text-slate-500 font-mono hover:border-slate-500 transition-all"
                                            }
                                        }
                                    >{r}</button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </div>

            <p class="text-[10px] text-slate-600 font-mono uppercase tracking-widest mb-4">
                {move || format!("{} DOCUMENTS", count.get())}
            </p>

            /* Document list */
            <div class="space-y-2">
                {move || filtered.get().into_iter().map(|d| {
                    let doc_id = d.id.to_string();
                    let title = d.title.to_string();
                    let jurisdiction = d.jurisdiction.to_string();
                    let doc_type = d.doc_type.to_string();
                    let activity = d.activity.to_string();
                    let risk = d.risk_level.to_string();
                    let summary = d.summary.to_string();
                    let url = d.url.to_string();
                    let has_url = !d.url.is_empty();

                    let risk_class = match d.risk_level {
                        "Critical" => "text-red-400 bg-red-500/10 border-red-500/20",
                        "High" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        "Medium" => "text-yellow-400 bg-yellow-500/10 border-yellow-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    let risk_badge = format!("px-2 py-0.5 rounded-full border text-[8px] font-bold font-mono uppercase {risk_class}");

                    let jur_class = match d.jurisdiction {
                        "FDA" => "text-blue-400",
                        "EMA" => "text-cyan-400",
                        "ICH" => "text-emerald-400",
                        "WHO" => "text-amber-400",
                        "CIOMS" => "text-purple-400",
                        "MHRA" => "text-pink-400",
                        "PMDA" => "text-rose-400",
                        "TGA" => "text-teal-400",
                        "Health Canada" => "text-red-300",
                        _ => "text-slate-400",
                    };

                    view! {
                        <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-4 hover:border-slate-700 transition-all">
                            <div class="flex items-start justify-between gap-3 mb-2">
                                <div class="flex-1">
                                    <div class="flex items-center gap-2 mb-1">
                                        <span class=format!("text-[10px] font-black font-mono {jur_class}")>{jurisdiction.clone()}</span>
                                        <span class="text-[9px] text-slate-600 font-mono">{doc_id}</span>
                                    </div>
                                    {if has_url {
                                        let u = url.clone();
                                        view! {
                                            <a href=u target="_blank" rel="noopener" class="text-sm font-bold text-white hover:text-cyan-400 transition-colors">{title}</a>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="text-sm font-bold text-white">{title}</span>
                                        }.into_any()
                                    }}
                                </div>
                                <div class="flex items-center gap-2 flex-shrink-0">
                                    <span class="px-2 py-0.5 rounded bg-slate-800 text-[8px] font-bold text-slate-400 font-mono uppercase">{doc_type}</span>
                                    <span class=risk_badge>{risk}</span>
                                </div>
                            </div>
                            <p class="text-[11px] text-slate-400 leading-relaxed mb-2">{summary}</p>
                            <span class="inline-block px-2 py-0.5 rounded bg-slate-950 border border-slate-800 text-[8px] font-bold text-slate-500 font-mono uppercase">{activity}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
