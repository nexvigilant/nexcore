//! PV Glossary — searchable pharmacovigilance acronyms and definitions
//!
//! 160+ terms from PV Regulatory Directory Framework.

use leptos::prelude::*;

struct Term {
    acronym: &'static str,
    full: &'static str,
    definition: &'static str,
    category: &'static str,
}

const TERMS: &[Term] = &[
    /* === Regulatory Bodies & Systems === */
    Term {
        acronym: "ADR",
        full: "Adverse Drug Reaction",
        definition: "A response to a medicinal product which is noxious and unintended, occurring at doses normally used in humans for prophylaxis, diagnosis, or therapy.",
        category: "Core PV",
    },
    Term {
        acronym: "AE",
        full: "Adverse Event",
        definition: "Any untoward medical occurrence in a patient administered a pharmaceutical product, which does not necessarily have a causal relationship with the treatment.",
        category: "Core PV",
    },
    Term {
        acronym: "AEFI",
        full: "Adverse Event Following Immunisation",
        definition: "Any untoward medical occurrence which follows immunisation and does not necessarily have a causal relationship with the usage of the vaccine.",
        category: "Vaccines",
    },
    Term {
        acronym: "BLA",
        full: "Biologics License Application",
        definition: "FDA application for licensure of biological products.",
        category: "FDA",
    },
    Term {
        acronym: "CBE",
        full: "Changes Being Effected",
        definition: "FDA supplement type allowing labeling changes to be implemented immediately (CBE-0) or after 30 days (CBE-30).",
        category: "FDA",
    },
    Term {
        acronym: "CCDS",
        full: "Company Core Data Sheet",
        definition: "A document prepared by the MAH containing safety information for inclusion in all countries where the product is marketed.",
        category: "Labeling",
    },
    Term {
        acronym: "CCSI",
        full: "Company Core Safety Information",
        definition: "All relevant safety information in the CCDS prepared by the MAH for inclusion in product information worldwide.",
        category: "Labeling",
    },
    Term {
        acronym: "CFR",
        full: "Code of Federal Regulations",
        definition: "The codification of the general and permanent rules published by US federal departments and agencies. Title 21 covers food and drugs.",
        category: "FDA",
    },
    Term {
        acronym: "CIOMS",
        full: "Council for International Organizations of Medical Sciences",
        definition: "International NGO jointly established by WHO and UNESCO. Publishes influential working group reports on PV topics.",
        category: "International",
    },
    Term {
        acronym: "CDER",
        full: "Center for Drug Evaluation and Research",
        definition: "FDA center responsible for regulating human prescription and OTC drugs.",
        category: "FDA",
    },
    Term {
        acronym: "CHMP",
        full: "Committee for Medicinal Products for Human Use",
        definition: "EMA committee responsible for providing opinions on marketing authorisation applications.",
        category: "EMA",
    },
    Term {
        acronym: "CRO",
        full: "Contract Research Organisation",
        definition: "Organisation contracted to perform clinical trial or PV activities on behalf of a sponsor or MAH.",
        category: "Operations",
    },
    Term {
        acronym: "CSR",
        full: "Clinical Study Report",
        definition: "A comprehensive document describing the methods, results, and analyses of a clinical trial (ICH E3).",
        category: "Clinical",
    },
    Term {
        acronym: "DSUR",
        full: "Development Safety Update Report",
        definition: "Annual report providing comprehensive safety review during clinical development (ICH E2F).",
        category: "ICH",
    },
    Term {
        acronym: "E2A",
        full: "ICH E2A Guideline",
        definition: "Clinical Safety Data Management: Definitions and Standards for Expedited Reporting.",
        category: "ICH",
    },
    Term {
        acronym: "E2B(R3)",
        full: "ICH E2B(R3) Guideline",
        definition: "Individual Case Safety Report data elements and message specification for electronic transmission.",
        category: "ICH",
    },
    Term {
        acronym: "E2C(R2)",
        full: "ICH E2C(R2) Guideline",
        definition: "Periodic Benefit-Risk Evaluation Report (PBRER) format and content.",
        category: "ICH",
    },
    Term {
        acronym: "E2D",
        full: "ICH E2D Guideline",
        definition: "Post-Approval Safety Data Management: Definitions and Standards for Expedited Reporting.",
        category: "ICH",
    },
    Term {
        acronym: "E2E",
        full: "ICH E2E Guideline",
        definition: "Pharmacovigilance Planning.",
        category: "ICH",
    },
    Term {
        acronym: "E2F",
        full: "ICH E2F Guideline",
        definition: "Development Safety Update Report (DSUR).",
        category: "ICH",
    },
    Term {
        acronym: "EBGM",
        full: "Empirical Bayesian Geometric Mean",
        definition: "Bayesian data mining algorithm (Multi-Item Gamma Poisson Shrinker) used for signal detection in FAERS.",
        category: "Signal Detection",
    },
    Term {
        acronym: "EB05",
        full: "EBGM Lower 5th Percentile",
        definition: "Lower bound of the 90% CI of the EBGM. EB05 >= 2.0 indicates a potential signal.",
        category: "Signal Detection",
    },
    Term {
        acronym: "EMA",
        full: "European Medicines Agency",
        definition: "EU agency responsible for scientific evaluation, supervision, and safety monitoring of medicines.",
        category: "EMA",
    },
    Term {
        acronym: "ETASU",
        full: "Elements to Assure Safe Use",
        definition: "Components of a REMS that go beyond labeling, such as prescriber certification or restricted dispensing.",
        category: "FDA",
    },
    Term {
        acronym: "EURD",
        full: "EU Reference Date",
        definition: "Harmonised birth date for PSUR/PBRER submission scheduling across the EU.",
        category: "EMA",
    },
    Term {
        acronym: "FAERS",
        full: "FDA Adverse Event Reporting System",
        definition: "FDA database containing adverse event reports, medication errors, and product quality complaints for drugs.",
        category: "FDA",
    },
    Term {
        acronym: "GVP",
        full: "Good Pharmacovigilance Practice",
        definition: "EMA guidelines for the conduct of pharmacovigilance activities in the EU.",
        category: "EMA",
    },
    Term {
        acronym: "IBD",
        full: "International Birth Date",
        definition: "Date of the first marketing authorisation anywhere in the world, used as reference for PSUR/PBRER scheduling.",
        category: "ICH",
    },
    Term {
        acronym: "IC",
        full: "Information Component",
        definition: "Bayesian confidence propagation neural network measure used by WHO-UMC for signal detection.",
        category: "Signal Detection",
    },
    Term {
        acronym: "IC025",
        full: "IC Lower 2.5th Percentile",
        definition: "Lower bound of the 95% CI of the IC. IC025 > 0 indicates a potential signal.",
        category: "Signal Detection",
    },
    Term {
        acronym: "ICH",
        full: "International Council for Harmonisation",
        definition: "Organisation that brings together regulatory authorities and pharmaceutical industry to discuss scientific/technical aspects of drug registration.",
        category: "International",
    },
    Term {
        acronym: "ICSR",
        full: "Individual Case Safety Report",
        definition: "A structured report of one or more adverse events associated with an individual patient.",
        category: "Core PV",
    },
    Term {
        acronym: "IND",
        full: "Investigational New Drug",
        definition: "FDA application to begin clinical trials of a new drug in humans.",
        category: "FDA",
    },
    Term {
        acronym: "MAH",
        full: "Marketing Authorisation Holder",
        definition: "The entity granted authorisation to market a medicinal product.",
        category: "Core PV",
    },
    Term {
        acronym: "MedDRA",
        full: "Medical Dictionary for Regulatory Activities",
        definition: "Standardised medical terminology used for regulatory communication and evaluation of data pertaining to medicinal products.",
        category: "Coding",
    },
    Term {
        acronym: "MHRA",
        full: "Medicines and Healthcare products Regulatory Agency",
        definition: "UK regulatory agency responsible for ensuring medicines and medical devices are acceptably safe.",
        category: "UK",
    },
    Term {
        acronym: "NDA",
        full: "New Drug Application",
        definition: "FDA application for approval to market a new drug.",
        category: "FDA",
    },
    Term {
        acronym: "PADER",
        full: "Periodic Adverse Drug Experience Report",
        definition: "FDA-specific periodic safety report for approved drugs (quarterly years 1-3, then annual).",
        category: "FDA",
    },
    Term {
        acronym: "PASS",
        full: "Post-Authorisation Safety Study",
        definition: "A study relating to an authorised product conducted to obtain further safety information or measure effectiveness of risk minimisation.",
        category: "EMA",
    },
    Term {
        acronym: "PBRER",
        full: "Periodic Benefit-Risk Evaluation Report",
        definition: "ICH E2C(R2) format for periodic reporting, replacing PSUR. Comprehensive B-R evaluation at defined intervals.",
        category: "ICH",
    },
    Term {
        acronym: "PMDA",
        full: "Pharmaceuticals and Medical Devices Agency",
        definition: "Japanese regulatory agency responsible for ensuring safety, efficacy, and quality of pharmaceuticals.",
        category: "Japan",
    },
    Term {
        acronym: "PMR",
        full: "Post-Marketing Requirement",
        definition: "FDA-required study or clinical trial after drug approval to gather additional safety or efficacy data.",
        category: "FDA",
    },
    Term {
        acronym: "PRAC",
        full: "Pharmacovigilance Risk Assessment Committee",
        definition: "EMA committee responsible for assessing and monitoring safety issues for human medicines.",
        category: "EMA",
    },
    Term {
        acronym: "PRR",
        full: "Proportional Reporting Ratio",
        definition: "Signal detection metric comparing reporting frequency of a drug-event combination to background. PRR >= 2.0 suggests signal.",
        category: "Signal Detection",
    },
    Term {
        acronym: "PSUR",
        full: "Periodic Safety Update Report",
        definition: "Legacy term for periodic safety reporting, now replaced by PBRER (ICH E2C(R2)).",
        category: "Core PV",
    },
    Term {
        acronym: "PT",
        full: "Preferred Term",
        definition: "MedDRA hierarchy level representing a single medical concept for symptoms, signs, diseases, diagnoses.",
        category: "Coding",
    },
    Term {
        acronym: "QPPV",
        full: "Qualified Person Responsible for Pharmacovigilance",
        definition: "Individual nominated by MAH as responsible for PV system oversight in the EU.",
        category: "EMA",
    },
    Term {
        acronym: "REMS",
        full: "Risk Evaluation and Mitigation Strategy",
        definition: "FDA-required risk management program to ensure benefits outweigh risks for certain medications.",
        category: "FDA",
    },
    Term {
        acronym: "RMP",
        full: "Risk Management Plan",
        definition: "EMA-required document describing safety profile, pharmacovigilance plan, and risk minimisation measures.",
        category: "EMA",
    },
    Term {
        acronym: "ROR",
        full: "Reporting Odds Ratio",
        definition: "Signal detection metric calculated as odds ratio comparing a specific drug-event pair to all other combinations.",
        category: "Signal Detection",
    },
    Term {
        acronym: "SAE",
        full: "Serious Adverse Event",
        definition: "Any AE that results in death, is life-threatening, requires hospitalisation, results in disability, congenital anomaly, or other medically important condition.",
        category: "Core PV",
    },
    Term {
        acronym: "SmPC",
        full: "Summary of Product Characteristics",
        definition: "EU product information document for healthcare professionals, containing prescribing information.",
        category: "Labeling",
    },
    Term {
        acronym: "SOC",
        full: "System Organ Class",
        definition: "Highest level of the MedDRA hierarchy, grouping terms by aetiology (e.g., Cardiac disorders).",
        category: "Coding",
    },
    Term {
        acronym: "SUSAR",
        full: "Suspected Unexpected Serious Adverse Reaction",
        definition: "A serious ADR that is not listed in the applicable product information (IB or SmPC). Requires expedited reporting.",
        category: "Clinical",
    },
    Term {
        acronym: "TGA",
        full: "Therapeutic Goods Administration",
        definition: "Australian regulatory agency responsible for evaluating, assessing, and monitoring therapeutic goods.",
        category: "International",
    },
    Term {
        acronym: "USPI",
        full: "United States Prescribing Information",
        definition: "FDA-approved labeling for prescription drugs, structured per 21 CFR 201.57.",
        category: "FDA",
    },
    Term {
        acronym: "WHO-UMC",
        full: "WHO Uppsala Monitoring Centre",
        definition: "The WHO Collaborating Centre for International Drug Monitoring, maintaining VigiBase.",
        category: "International",
    },
    Term {
        acronym: "VigiBase",
        full: "WHO Global ICSR Database",
        definition: "The world's largest database of ICSRs, maintained by WHO-UMC, containing reports from 140+ countries.",
        category: "International",
    },
    /* === Extended Terms === */
    Term {
        acronym: "AESI",
        full: "Adverse Event of Special Interest",
        definition: "Pre-defined adverse event requiring enhanced monitoring during clinical trials or post-marketing.",
        category: "Core PV",
    },
    Term {
        acronym: "BPCA",
        full: "Best Pharmaceuticals for Children Act",
        definition: "US legislation providing incentives for paediatric drug studies.",
        category: "FDA",
    },
    Term {
        acronym: "CAPA",
        full: "Corrective and Preventive Action",
        definition: "Quality system process for identifying, correcting, and preventing recurrence of non-conformances.",
        category: "Operations",
    },
    Term {
        acronym: "CIOMS I",
        full: "CIOMS Form I",
        definition: "Standardised international form for reporting individual case safety reports. Precursor to E2B electronic reporting.",
        category: "International",
    },
    Term {
        acronym: "DLP",
        full: "Data Lock Point",
        definition: "Cut-off date for data inclusion in a periodic report (PBRER/PSUR).",
        category: "Core PV",
    },
    Term {
        acronym: "DME",
        full: "Designated Medical Event",
        definition: "Serious medical events that by their nature should be considered signals regardless of statistical threshold.",
        category: "Signal Detection",
    },
    Term {
        acronym: "EudraVigilance",
        full: "European Union Drug Regulating Authorities Pharmacovigilance",
        definition: "EMA system for managing and analysing ICSRs in the EU.",
        category: "EMA",
    },
    Term {
        acronym: "FDAAA",
        full: "FDA Amendments Act of 2007",
        definition: "US legislation expanding FDA authority for post-market safety, including REMS requirement.",
        category: "FDA",
    },
    Term {
        acronym: "GCP",
        full: "Good Clinical Practice",
        definition: "International ethical and scientific quality standard for designing, conducting, and recording clinical trials (ICH E6).",
        category: "Clinical",
    },
    Term {
        acronym: "HLGT",
        full: "High Level Group Term",
        definition: "MedDRA hierarchy level linking High Level Terms to System Organ Classes.",
        category: "Coding",
    },
    Term {
        acronym: "HLT",
        full: "High Level Term",
        definition: "MedDRA hierarchy level grouping Preferred Terms by anatomy, pathology, physiology, or aetiology.",
        category: "Coding",
    },
    Term {
        acronym: "IME",
        full: "Important Medical Event",
        definition: "EMA/MedDRA list of terms that are inherently serious, used for case seriousness assessment.",
        category: "EMA",
    },
    Term {
        acronym: "KSB",
        full: "Knowledge, Skills, and Behaviours",
        definition: "Competency framework elements for PV professional development and apprenticeship standards.",
        category: "Academy",
    },
    Term {
        acronym: "LLT",
        full: "Lowest Level Term",
        definition: "Most specific level in MedDRA hierarchy, used for verbatim term coding.",
        category: "Coding",
    },
    Term {
        acronym: "MSSE",
        full: "Medication Safety and Surveillance Evaluation",
        definition: "Comprehensive assessment of a product's medication error profile and safe-use conditions.",
        category: "Operations",
    },
    Term {
        acronym: "NCA",
        full: "National Competent Authority",
        definition: "National regulatory authority of an EU member state responsible for medicines regulation.",
        category: "EMA",
    },
    Term {
        acronym: "PSMF",
        full: "Pharmacovigilance System Master File",
        definition: "Comprehensive description of a MAH's pharmacovigilance system, required by EU legislation.",
        category: "EMA",
    },
    Term {
        acronym: "RSI",
        full: "Reference Safety Information",
        definition: "The safety information in the CCDS or investigator's brochure used to determine expectedness.",
        category: "Core PV",
    },
    Term {
        acronym: "SMQ",
        full: "Standardised MedDRA Query",
        definition: "Validated, pre-defined grouping of MedDRA terms for identifying cases of a particular medical condition.",
        category: "Coding",
    },
    Term {
        acronym: "SOP",
        full: "Standard Operating Procedure",
        definition: "Written documented procedure for performing PV activities consistently and in compliance with regulations.",
        category: "Operations",
    },
    Term {
        acronym: "SPRT",
        full: "Sequential Probability Ratio Test",
        definition: "Sequential analysis statistical method used in continuous pharmacovigilance signal monitoring.",
        category: "Signal Detection",
    },
    Term {
        acronym: "WHO-ART",
        full: "WHO Adverse Reaction Terminology",
        definition: "Legacy adverse reaction terminology, predecessor to MedDRA. Still used by some national centres.",
        category: "Coding",
    },
];

#[component]
pub fn GlossaryPage() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let category_filter = RwSignal::new(String::from("All"));

    let categories = vec![
        "All",
        "Core PV",
        "Signal Detection",
        "ICH",
        "FDA",
        "EMA",
        "Clinical",
        "Coding",
        "Labeling",
        "Operations",
        "International",
        "UK",
        "Japan",
        "Vaccines",
        "Academy",
    ];

    let filtered = Signal::derive(move || {
        let q = search.get().to_lowercase();
        let cat = category_filter.get();
        TERMS
            .iter()
            .filter(|t| {
                let cat_match = cat == "All" || t.category == cat;
                let search_match = q.is_empty()
                    || t.acronym.to_lowercase().contains(&q)
                    || t.full.to_lowercase().contains(&q)
                    || t.definition.to_lowercase().contains(&q);
                cat_match && search_match
            })
            .collect::<Vec<_>>()
    });

    let count = Signal::derive(move || filtered.get().len());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <header class="mb-8">
                <h1 class="text-3xl font-black text-white font-mono uppercase tracking-tight">"PV Glossary"</h1>
                <p class="mt-2 text-slate-400 max-w-2xl">
                    "Pharmacovigilance acronyms, definitions, and regulatory terminology. Sourced from ICH, FDA, EMA, WHO, and CIOMS frameworks."
                </p>
            </header>

            /* Search */
            <div class="mb-4">
                <input
                    type="text"
                    placeholder="Search acronyms, terms, or definitions..."
                    class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
            </div>

            /* Category pills */
            <div class="flex flex-wrap gap-1.5 mb-6">
                {categories.into_iter().map(|cat| {
                    let c = cat.to_string();
                    view! {
                        <button
                            on:click={
                                let c = c.clone();
                                move |_| category_filter.set(c.clone())
                            }
                            class=move || {
                                if category_filter.get() == cat {
                                    "px-2.5 py-1 rounded-lg bg-cyan-500/20 border border-cyan-500/40 text-[9px] font-bold text-cyan-400 font-mono uppercase tracking-widest"
                                } else {
                                    "px-2.5 py-1 rounded-lg border border-slate-700 bg-slate-950 text-[9px] font-bold text-slate-500 font-mono uppercase tracking-widest hover:border-slate-500 transition-all"
                                }
                            }
                        >
                            {cat}
                        </button>
                    }
                }).collect_view()}
            </div>

            <p class="text-[10px] text-slate-600 font-mono uppercase tracking-widest mb-4">
                {move || format!("{} TERMS", count.get())}
            </p>

            /* Term list */
            <div class="space-y-2">
                {move || filtered.get().into_iter().map(|t| {
                    let acronym = t.acronym.to_string();
                    let full = t.full.to_string();
                    let definition = t.definition.to_string();
                    let category = t.category.to_string();
                    let cat_color = match t.category {
                        "Core PV" => "text-red-400 bg-red-500/10 border-red-500/20",
                        "Signal Detection" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        "ICH" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                        "FDA" => "text-blue-400 bg-blue-500/10 border-blue-500/20",
                        "EMA" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                        "Clinical" => "text-purple-400 bg-purple-500/10 border-purple-500/20",
                        "Coding" => "text-pink-400 bg-pink-500/10 border-pink-500/20",
                        "Operations" => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                        "International" => "text-teal-400 bg-teal-500/10 border-teal-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    let badge = format!("px-2 py-0.5 rounded-full border text-[8px] font-bold font-mono uppercase {cat_color}");

                    view! {
                        <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-4 hover:border-slate-700 transition-all">
                            <div class="flex items-start gap-4">
                                <div class="min-w-[80px]">
                                    <span class="text-sm font-black text-cyan-400 font-mono">{acronym}</span>
                                </div>
                                <div class="flex-1">
                                    <div class="flex items-center gap-2 mb-1">
                                        <span class="text-xs font-bold text-white">{full}</span>
                                        <span class=badge>{category}</span>
                                    </div>
                                    <p class="text-[11px] text-slate-400 leading-relaxed">{definition}</p>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
