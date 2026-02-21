//! ICSR Case Management — Individual Case Safety Report intake, triage, and lifecycle
//!
//! E2B(R3)-structured case management with MedDRA coding, causality assessment,
//! seriousness classification, narrative generation, and regulatory submission tracking.

use leptos::prelude::*;

/* ── Static demo data ── */

#[derive(Clone)]
struct IcsrCase {
    case_id: &'static str,
    status: &'static str,
    priority: &'static str,
    drug: &'static str,
    event: &'static str,
    pt_code: &'static str,
    soc: &'static str,
    seriousness: &'static str,
    criteria: &'static str,
    reporter_type: &'static str,
    country: &'static str,
    received: &'static str,
    onset: &'static str,
    outcome: &'static str,
    causality: &'static str,
    expectedness: &'static str,
    narrative: &'static str,
}

const CASES: &[IcsrCase] = &[
    IcsrCase {
        case_id: "NV-2026-000147",
        status: "Pending Review",
        priority: "Expedited",
        drug: "Infliximab",
        event: "Anaphylactic reaction",
        pt_code: "10002198",
        soc: "Immune system disorders",
        seriousness: "Serious",
        criteria: "Life-threatening",
        reporter_type: "Physician",
        country: "United States",
        received: "2026-02-14",
        onset: "2026-02-12",
        outcome: "Recovered",
        causality: "Probable (Naranjo: 7)",
        expectedness: "Listed",
        narrative: "A 52-year-old male receiving Infliximab infusion #4 for Crohn's disease developed acute anaphylaxis within 15 minutes of infusion start. Symptoms included dyspnoea, urticaria, hypotension (BP 80/50). Treated with epinephrine, IV fluids, and corticosteroids. Patient recovered within 4 hours. Infliximab permanently discontinued.",
    },
    IcsrCase {
        case_id: "NV-2026-000148",
        status: "Under Assessment",
        priority: "Expedited",
        drug: "Methotrexate",
        event: "Pancytopenia",
        pt_code: "10033661",
        soc: "Blood and lymphatic system disorders",
        seriousness: "Serious",
        criteria: "Hospitalization",
        reporter_type: "Physician",
        country: "United Kingdom",
        received: "2026-02-13",
        onset: "2026-02-08",
        outcome: "Recovering",
        causality: "Certain (Naranjo: 9)",
        expectedness: "Listed",
        narrative: "A 68-year-old female on Methotrexate 15mg weekly for rheumatoid arthritis presented with pancytopenia. WBC 1.2, Hgb 7.8, Plt 45. No concurrent NSAIDs. Renal function mildly impaired (CrCl 52). Methotrexate withheld, folinic acid rescue initiated. Hospital admission for monitoring and supportive care.",
    },
    IcsrCase {
        case_id: "NV-2026-000149",
        status: "Completed",
        priority: "Standard",
        drug: "Atorvastatin",
        event: "Rhabdomyolysis",
        pt_code: "10039020",
        soc: "Musculoskeletal and connective tissue disorders",
        seriousness: "Serious",
        criteria: "Hospitalization",
        reporter_type: "Pharmacist",
        country: "Germany",
        received: "2026-02-10",
        onset: "2026-02-03",
        outcome: "Recovered with sequelae",
        causality: "Probable (Naranjo: 6)",
        expectedness: "Listed",
        narrative: "A 71-year-old male on Atorvastatin 80mg with CKD stage 3 and concurrent clarithromycin developed rhabdomyolysis (CK 15,400 U/L). Dark urine, myalgia, weakness. Admitted for IV hydration. Atorvastatin discontinued. CK normalised over 10 days. Mild residual weakness at follow-up.",
    },
    IcsrCase {
        case_id: "NV-2026-000150",
        status: "Pending Review",
        priority: "Expedited",
        drug: "Nivolumab",
        event: "Autoimmune hepatitis",
        pt_code: "10003827",
        soc: "Hepatobiliary disorders",
        seriousness: "Serious",
        criteria: "Hospitalization",
        reporter_type: "Physician",
        country: "Japan",
        received: "2026-02-14",
        onset: "2026-02-06",
        outcome: "Not recovered",
        causality: "Possible (Naranjo: 5)",
        expectedness: "Listed",
        narrative: "A 63-year-old male receiving Nivolumab for NSCLC (cycle 8) developed immune-related hepatitis. ALT 580, AST 420, ALP 180, Tbili 3.2. No viral hepatitis markers. Grade 3 per CTCAE. Nivolumab held, high-dose prednisolone (1mg/kg) started. Partial response at 1 week, mycophenolate added.",
    },
    IcsrCase {
        case_id: "NV-2026-000151",
        status: "Closed",
        priority: "Standard",
        drug: "Omeprazole",
        event: "Hypomagnesaemia",
        pt_code: "10021027",
        soc: "Metabolism and nutrition disorders",
        seriousness: "Serious",
        criteria: "Hospitalization",
        reporter_type: "Physician",
        country: "France",
        received: "2026-02-07",
        onset: "2026-01-28",
        outcome: "Recovered",
        causality: "Probable (Naranjo: 7)",
        expectedness: "Listed",
        narrative: "A 75-year-old female on long-term Omeprazole (5 years, 40mg daily) presented with symptomatic hypomagnesaemia (Mg 0.4 mmol/L). Muscle cramps, tremor, QTc prolongation (480ms). IV magnesium replacement, Omeprazole switched to famotidine. Magnesium normalised within 72 hours.",
    },
    IcsrCase {
        case_id: "NV-2026-000152",
        status: "Under Assessment",
        priority: "Expedited",
        drug: "Warfarin",
        event: "Intracranial haemorrhage",
        pt_code: "10022763",
        soc: "Nervous system disorders",
        seriousness: "Serious",
        criteria: "Life-threatening",
        reporter_type: "Physician",
        country: "United States",
        received: "2026-02-15",
        onset: "2026-02-14",
        outcome: "Not recovered",
        causality: "Certain (Naranjo: 8)",
        expectedness: "Listed",
        narrative: "An 82-year-old male on Warfarin for AF (target INR 2-3) presented with sudden-onset headache, confusion, left-sided weakness. INR 4.8 (concurrent amiodarone dose increase). CT confirmed right frontoparietal subdural haematoma. Vitamin K and 4-factor PCC administered. Neurosurgery consulted.",
    },
    IcsrCase {
        case_id: "NV-2026-000153",
        status: "Pending Review",
        priority: "Standard",
        drug: "Lisinopril",
        event: "Angioedema",
        pt_code: "10002424",
        soc: "Immune system disorders",
        seriousness: "Serious",
        criteria: "Other medically important",
        reporter_type: "Consumer",
        country: "Canada",
        received: "2026-02-12",
        onset: "2026-02-11",
        outcome: "Recovered",
        causality: "Probable (Naranjo: 6)",
        expectedness: "Listed",
        narrative: "A 58-year-old female developed facial and tongue angioedema 3 weeks after starting Lisinopril 10mg. No prior ACE inhibitor use. Presented to ED, treated with epinephrine and IV corticosteroids. Resolved within 12 hours. Lisinopril discontinued, switched to losartan.",
    },
    IcsrCase {
        case_id: "NV-2026-000154",
        status: "Under Assessment",
        priority: "Expedited",
        drug: "Adalimumab",
        event: "Tuberculosis",
        pt_code: "10044755",
        soc: "Infections and infestations",
        seriousness: "Serious",
        criteria: "Hospitalization",
        reporter_type: "Physician",
        country: "India",
        received: "2026-02-11",
        onset: "2026-01-15",
        outcome: "Not recovered",
        causality: "Possible (Naranjo: 4)",
        expectedness: "Listed",
        narrative: "A 45-year-old male on Adalimumab for AS (18 months) developed pulmonary TB despite negative baseline QuantiFERON. Presented with cough, weight loss, night sweats. Sputum AFB positive. Adalimumab stopped, anti-TB quadruple therapy initiated. Pre-treatment screening reviewed.",
    },
    IcsrCase {
        case_id: "NV-2026-000155",
        status: "Completed",
        priority: "Non-expedited",
        drug: "Metformin",
        event: "Lactic acidosis",
        pt_code: "10023676",
        soc: "Metabolism and nutrition disorders",
        seriousness: "Serious",
        criteria: "Life-threatening",
        reporter_type: "Physician",
        country: "Australia",
        received: "2026-02-05",
        onset: "2026-02-01",
        outcome: "Recovered",
        causality: "Probable (Naranjo: 7)",
        expectedness: "Listed",
        narrative: "A 72-year-old female on Metformin 2g daily with acute kidney injury (dehydration/gastroenteritis) developed lactic acidosis (lactate 9.2, pH 7.18, HCO3 10). ICU admission, Metformin withheld, IV bicarbonate and fluids. Renal function recovered, lactate normalised in 48 hours.",
    },
    IcsrCase {
        case_id: "NV-2026-000156",
        status: "Pending Review",
        priority: "Expedited",
        drug: "Carbamazepine",
        event: "Stevens-Johnson syndrome",
        pt_code: "10042033",
        soc: "Skin and subcutaneous tissue disorders",
        seriousness: "Serious",
        criteria: "Life-threatening, Hospitalization",
        reporter_type: "Physician",
        country: "Thailand",
        received: "2026-02-15",
        onset: "2026-02-09",
        outcome: "Not recovered",
        causality: "Certain (Naranjo: 9)",
        expectedness: "Listed",
        narrative: "A 35-year-old female, HLA-B*1502 positive (not tested prior to prescribing), developed SJS 14 days after starting Carbamazepine 200mg for trigeminal neuralgia. BSA involvement 12%. Burns unit admission, Carbamazepine discontinued. Supportive care with wound management. HLA-B*1502 screening gap identified.",
    },
];

#[component]
pub fn IcsrPage() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new(String::from("All"));
    let priority_filter = RwSignal::new(String::from("All"));
    let selected_case = RwSignal::new(Option::<usize>::None);

    let statuses = vec![
        "All",
        "Pending Review",
        "Under Assessment",
        "Completed",
        "Closed",
    ];
    let priorities = vec!["All", "Expedited", "Standard", "Non-expedited"];

    let filtered = Signal::derive(move || {
        let q = search.get().to_lowercase();
        let st = status_filter.get();
        let pr = priority_filter.get();
        CASES
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                let st_match = st == "All" || c.status == st;
                let pr_match = pr == "All" || c.priority == pr;
                let q_match = q.is_empty()
                    || c.case_id.to_lowercase().contains(&q)
                    || c.drug.to_lowercase().contains(&q)
                    || c.event.to_lowercase().contains(&q)
                    || c.country.to_lowercase().contains(&q)
                    || c.narrative.to_lowercase().contains(&q);
                st_match && pr_match && q_match
            })
            .collect::<Vec<_>>()
    });

    /* Stats */
    let pending = CASES
        .iter()
        .filter(|c| c.status == "Pending Review")
        .count();
    let expedited = CASES.iter().filter(|c| c.priority == "Expedited").count();
    let serious = CASES.iter().filter(|c| c.seriousness == "Serious").count();

    view! {
        <div class="mx-auto max-w-7xl px-4 py-8">
            <header class="mb-6">
                <h1 class="text-3xl font-black text-white font-mono uppercase tracking-tight">"ICSR Case Management"</h1>
                <p class="mt-2 text-slate-400 max-w-3xl">
                    "Individual Case Safety Report intake, triage, and lifecycle management. E2B(R3) structured."
                </p>
            </header>

            /* Stats */
            <div class="grid grid-cols-2 md:grid-cols-5 gap-3 mb-6">
                <StatBox label="Total Cases" value=CASES.len().to_string() color="text-white" />
                <StatBox label="Pending Review" value=pending.to_string() color="text-amber-400" />
                <StatBox label="Expedited" value=expedited.to_string() color="text-red-400" />
                <StatBox label="Serious" value=serious.to_string() color="text-red-400" />
                <StatBox label="Countries" value="8".to_string() color="text-cyan-400" />
            </div>

            /* Filters */
            <div class="flex flex-col md:flex-row gap-4 mb-4">
                <input
                    type="text"
                    placeholder="Search by case ID, drug, event, country, or narrative..."
                    class="flex-1 rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
            </div>
            <div class="flex flex-wrap gap-4 mb-6">
                <FilterGroup label="Status" options=statuses signal=status_filter />
                <FilterGroup label="Priority" options=priorities signal=priority_filter />
            </div>

            /* Split view: list + detail */
            <div class="grid lg:grid-cols-5 gap-4">
                /* Case list */
                <div class="lg:col-span-2 space-y-2 max-h-[800px] overflow-y-auto pr-1">
                    {move || {
                        let sel = selected_case.get();
                        filtered.get().into_iter().map(|(idx, c)| {
                            let is_selected = sel == Some(idx);
                            let case_id = c.case_id.to_string();
                            let drug = c.drug.to_string();
                            let event = c.event.to_string();
                            let status = c.status.to_string();
                            let priority = c.priority.to_string();
                            let received = c.received.to_string();
                            let country = c.country.to_string();

                            let border = if is_selected {
                                "rounded-lg border-2 border-cyan-500/50 bg-cyan-500/5 p-3 cursor-pointer transition-all"
                            } else {
                                "rounded-lg border border-slate-800 bg-slate-900/50 p-3 cursor-pointer hover:border-slate-700 transition-all"
                            };

                            let priority_color = match c.priority {
                                "Expedited" => "text-red-400 bg-red-500/10 border-red-500/20",
                                "Standard" => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                                _ => "text-slate-500 bg-slate-500/10 border-slate-500/20",
                            };
                            let priority_badge = format!("px-1.5 py-0.5 rounded border text-[8px] font-bold font-mono uppercase {priority_color}");

                            let status_color = match c.status {
                                "Pending Review" => "text-amber-400",
                                "Under Assessment" => "text-cyan-400",
                                "Completed" => "text-emerald-400",
                                "Closed" => "text-slate-500",
                                _ => "text-slate-400",
                            };

                            view! {
                                <div class=border on:click=move |_| selected_case.set(Some(idx))>
                                    <div class="flex items-center justify-between mb-1.5">
                                        <span class="text-[10px] font-black text-cyan-400 font-mono">{case_id}</span>
                                        <span class=priority_badge>{priority}</span>
                                    </div>
                                    <p class="text-xs font-bold text-white mb-0.5">{drug} " \u{2192} " {event}</p>
                                    <div class="flex items-center justify-between">
                                        <span class=format!("text-[9px] font-bold font-mono {status_color}")>{status}</span>
                                        <span class="text-[9px] text-slate-600 font-mono">{country} " \u{2022} " {received}</span>
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>

                /* Case detail */
                <div class="lg:col-span-3">
                    {move || {
                        match selected_case.get() {
                            Some(idx) if idx < CASES.len() => {
                                let c = &CASES[idx];
                                view! { <CaseDetail case=c /> }.into_any()
                            }
                            _ => view! {
                                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-12 text-center">
                                    <p class="text-slate-500 font-mono text-xs">"Select a case from the list to view details"</p>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn CaseDetail(case: &'static IcsrCase) -> impl IntoView {
    let case_id = case.case_id.to_string();
    let status = case.status.to_string();
    let priority = case.priority.to_string();
    let drug = case.drug.to_string();
    let event = case.event.to_string();
    let pt_code = case.pt_code.to_string();
    let soc = case.soc.to_string();
    let seriousness = case.seriousness.to_string();
    let criteria = case.criteria.to_string();
    let reporter_type = case.reporter_type.to_string();
    let country = case.country.to_string();
    let received = case.received.to_string();
    let onset = case.onset.to_string();
    let outcome = case.outcome.to_string();
    let causality = case.causality.to_string();
    let expectedness = case.expectedness.to_string();
    let narrative = case.narrative.to_string();

    let status_color = match case.status {
        "Pending Review" => "text-amber-400 bg-amber-500/10 border-amber-500/30",
        "Under Assessment" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/30",
        "Completed" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/30",
        "Closed" => "text-slate-400 bg-slate-500/10 border-slate-500/30",
        _ => "text-slate-400 bg-slate-500/10 border-slate-500/30",
    };
    let status_badge = format!(
        "px-3 py-1 rounded-lg border text-[10px] font-bold font-mono uppercase {status_color}"
    );

    let priority_color = match case.priority {
        "Expedited" => "text-red-400 bg-red-500/10 border-red-500/30",
        _ => "text-slate-400 bg-slate-500/10 border-slate-500/30",
    };
    let priority_badge = format!(
        "px-3 py-1 rounded-lg border text-[10px] font-bold font-mono uppercase {priority_color}"
    );

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden">
            /* Header */
            <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
                <div>
                    <p class="text-lg font-black text-white font-mono">{case_id}</p>
                    <p class="text-xs text-slate-400 mt-0.5">{drug.clone()} " \u{2192} " {event.clone()}</p>
                </div>
                <div class="flex gap-2">
                    <span class=status_badge>{status}</span>
                    <span class=priority_badge>{priority}</span>
                </div>
            </div>

            <div class="p-6 space-y-5">
                /* E2B(R3) Sections */

                /* A. Administrative */
                <Section title="A. Administrative & Identification">
                    <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
                        <Field label="Received Date" value=received />
                        <Field label="Reporter Type" value=reporter_type />
                        <Field label="Country of Reporter" value=country />
                    </div>
                </Section>

                /* B. Patient */
                <Section title="B. Patient Information">
                    <p class="text-[10px] text-slate-500 italic">"Demographics extracted from narrative (age, sex, medical history)"</p>
                </Section>

                /* C. Reaction/Event */
                <Section title="C. Reaction / Event">
                    <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
                        <Field label="Preferred Term" value=event />
                        <Field label="MedDRA PT Code" value=pt_code />
                        <Field label="System Organ Class" value=soc />
                        <Field label="Onset Date" value=onset />
                        <Field label="Outcome" value=outcome />
                    </div>
                </Section>

                /* D. Drug Information */
                <Section title="D. Drug Information">
                    <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
                        <Field label="Suspect Drug" value=drug />
                        <Field label="Role" value="Suspect".to_string() />
                    </div>
                </Section>

                /* E. Seriousness */
                <Section title="E. Seriousness Assessment (ICH E2A)">
                    <div class="grid grid-cols-2 gap-3">
                        <div class="rounded-lg bg-red-500/5 border border-red-500/20 p-3">
                            <p class="text-[9px] font-bold text-red-400 uppercase tracking-widest mb-1">"Classification"</p>
                            <p class="text-sm font-bold text-red-400 font-mono">{seriousness}</p>
                        </div>
                        <div class="rounded-lg bg-slate-950 border border-slate-800 p-3">
                            <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest mb-1">"Criteria Met"</p>
                            <p class="text-sm font-bold text-slate-300 font-mono">{criteria}</p>
                        </div>
                    </div>
                </Section>

                /* F. Causality & Expectedness */
                <Section title="F. Causality & Expectedness">
                    <div class="grid grid-cols-2 gap-3">
                        <div class="rounded-lg bg-slate-950 border border-slate-800 p-3">
                            <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest mb-1">"Causality Assessment"</p>
                            <p class="text-sm font-bold text-amber-400 font-mono">{causality}</p>
                        </div>
                        <div class="rounded-lg bg-slate-950 border border-slate-800 p-3">
                            <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest mb-1">"Expectedness (vs RSI)"</p>
                            <p class="text-sm font-bold text-slate-300 font-mono">{expectedness}</p>
                        </div>
                    </div>
                </Section>

                /* G. Narrative */
                <Section title="G. Case Narrative">
                    <div class="rounded-lg bg-slate-950 border border-slate-800 p-4">
                        <p class="text-xs text-slate-300 leading-relaxed font-mono">{narrative}</p>
                    </div>
                </Section>

                /* H. Regulatory Status */
                <Section title="H. Submission Tracking">
                    <div class="grid grid-cols-3 gap-3">
                        <SubmissionStatus authority="FDA" status="Pending" deadline="15 days" />
                        <SubmissionStatus authority="EMA" status="Pending" deadline="15 days" />
                        <SubmissionStatus authority="MHRA" status="Not required" deadline="-" />
                    </div>
                </Section>
            </div>
        </div>
    }
}

#[component]
fn Section(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div>
            <h3 class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-3">{title}</h3>
            {children()}
        </div>
    }
}

#[component]
fn Field(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div>
            <p class="text-[8px] font-bold text-slate-600 uppercase tracking-widest mb-0.5">{label}</p>
            <p class="text-xs text-slate-300 font-mono">{value}</p>
        </div>
    }
}

#[component]
fn SubmissionStatus(
    authority: &'static str,
    status: &'static str,
    deadline: &'static str,
) -> impl IntoView {
    let color = match status {
        "Submitted" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
        "Pending" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
    };
    view! {
        <div class=format!("rounded-lg border p-2.5 {color}")>
            <p class="text-[9px] font-black uppercase tracking-widest mb-1">{authority}</p>
            <p class="text-[10px] font-bold font-mono">{status}</p>
            <p class="text-[8px] text-slate-500 font-mono mt-0.5">"Deadline: " {deadline}</p>
        </div>
    }
}

#[component]
fn StatBox(label: &'static str, value: String, color: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-3 text-center">
            <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">{label}</p>
            <p class=format!("text-lg font-black font-mono mt-0.5 {color}")>{value}</p>
        </div>
    }
}

#[component]
fn FilterGroup(
    label: &'static str,
    options: Vec<&'static str>,
    signal: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            <p class="text-[8px] font-bold text-slate-600 uppercase tracking-widest mb-1.5">{label}</p>
            <div class="flex flex-wrap gap-1">
                {options.into_iter().map(|o| {
                    let ov = o.to_string();
                    view! {
                        <button
                            on:click={let ov = ov.clone(); move |_| signal.set(ov.clone())}
                            class=move || {
                                if signal.get() == o {
                                    "px-2 py-1 rounded bg-cyan-500/20 border border-cyan-500/40 text-[9px] font-bold text-cyan-400 font-mono"
                                } else {
                                    "px-2 py-1 rounded border border-slate-700 text-[9px] font-bold text-slate-500 font-mono hover:border-slate-500 transition-all"
                                }
                            }
                        >{o}</button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
