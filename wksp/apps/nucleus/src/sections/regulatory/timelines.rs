//! Reporting Timelines — cross-jurisdictional deadline comparison tool
//!
//! Data sourced from PV Regulatory Directory Framework.
//! Compares FDA, EMA, ICH, and WHO reporting timeframes for every safety report type.

use leptos::prelude::*;

struct Timeline {
    report_type: &'static str,
    category: &'static str,
    fda: &'static str,
    ema: &'static str,
    ich: &'static str,
    who: &'static str,
    day_0: &'static str,
    differences: &'static str,
}

const TIMELINES: &[Timeline] = &[
    /* === Individual Case Safety Reports (ICSRs) === */
    Timeline {
        report_type: "Fatal / Life-Threatening ICSR (Expedited)",
        category: "ICSR",
        fda: "7 calendar days (initial), 15 days (follow-up)",
        ema: "15 calendar days",
        ich: "15 calendar days (E2A)",
        who: "As soon as possible, no later than 15 days",
        day_0: "Date MAH first becomes aware of minimum criteria",
        differences: "FDA requires 7-day alert for fatal/life-threatening; EMA and ICH use 15-day standard",
    },
    Timeline {
        report_type: "Serious, Unexpected ICSR (Expedited)",
        category: "ICSR",
        fda: "15 calendar days",
        ema: "15 calendar days",
        ich: "15 calendar days (E2A)",
        who: "15 calendar days",
        day_0: "Date MAH first becomes aware of minimum criteria",
        differences: "Harmonized across all jurisdictions for serious unexpected events",
    },
    Timeline {
        report_type: "Serious, Expected ICSR",
        category: "ICSR",
        fda: "Periodic (PSUR/PBRER)",
        ema: "90 days (EudraVigilance)",
        ich: "Periodic reporting (E2C(R2))",
        who: "Periodic reporting",
        day_0: "Date MAH becomes aware",
        differences: "FDA does not require individual expedited reporting for expected events",
    },
    Timeline {
        report_type: "Non-Serious ICSR",
        category: "ICSR",
        fda: "Not individually reported to FDA",
        ema: "90 days (EudraVigilance)",
        ich: "Not individually expedited",
        who: "Not individually required",
        day_0: "N/A",
        differences: "EMA requires all ICSRs in EudraVigilance; FDA only collects through periodic reports",
    },
    Timeline {
        report_type: "Consumer/Patient Direct Report",
        category: "ICSR",
        fda: "15 days if serious/unexpected",
        ema: "15 days (serious), 90 days (non-serious)",
        ich: "Same as HCP reports (E2D)",
        who: "Same standards as HCP reports",
        day_0: "Date MAH receives report with minimum criteria",
        differences: "All jurisdictions treat consumer reports equally to HCP reports for expedited criteria",
    },
    Timeline {
        report_type: "Literature Case Report",
        category: "ICSR",
        fda: "15 days if serious/unexpected (from awareness)",
        ema: "15 days (serious), 90 days (non-serious)",
        ich: "15 days (E2D) from awareness",
        who: "15 days from awareness",
        day_0: "Date MAH identifies the case through systematic literature review",
        differences: "All require systematic literature screening; Day 0 is date of awareness, not publication",
    },
    /* === Clinical Trial Reports === */
    Timeline {
        report_type: "SUSAR (Clinical Trial)",
        category: "Clinical Trial",
        fda: "7 days (fatal/life-threatening), 15 days (other serious)",
        ema: "7 days (fatal/life-threatening), 15 days (other serious)",
        ich: "7 days (fatal/life-threatening), 15 days (other serious) (E2A)",
        who: "Aligned with ICH E2A",
        day_0: "Date sponsor first becomes aware",
        differences: "Harmonized 7/15 day framework across all jurisdictions for SUSARs",
    },
    Timeline {
        report_type: "IND Safety Report",
        category: "Clinical Trial",
        fda: "7 days (fatal/life-threatening), 15 days (other serious unexpected)",
        ema: "N/A (uses SUSAR framework)",
        ich: "Covered under SUSAR (E2A/E6)",
        who: "N/A",
        day_0: "Date sponsor becomes aware",
        differences: "FDA-specific terminology; aligns with SUSAR timelines",
    },
    Timeline {
        report_type: "Annual IND Safety Report",
        category: "Clinical Trial",
        fda: "Within 60 days of IND anniversary",
        ema: "Annual Safety Report (ASR) per CT Regulation",
        ich: "DSUR annually (E2F)",
        who: "N/A",
        day_0: "IND anniversary date / DSUR data lock point",
        differences: "FDA uses IND-specific report; EMA uses ASR; ICH recommends DSUR format for all",
    },
    Timeline {
        report_type: "DSUR (Development Safety Update Report)",
        category: "Clinical Trial",
        fda: "Accepted as annual IND safety report",
        ema: "Required annually",
        ich: "Annually, within 60 days of DIBD (E2F)",
        who: "Recommended annually",
        day_0: "Development International Birth Date (DIBD)",
        differences: "ICH E2F format accepted globally; covers entire development program",
    },
    /* === Periodic Reports === */
    Timeline {
        report_type: "PSUR/PBRER",
        category: "Periodic",
        fda: "Not required (replaced by PADER for NDA)",
        ema: "Per EURD list schedule (6mo/1yr/3yr)",
        ich: "Per IBD schedule (E2C(R2))",
        who: "Recommended per ICH schedule",
        day_0: "International Birth Date (IBD) or EURD list date",
        differences: "FDA does not accept PSUR/PBRER; uses PADER. EMA follows EURD list. ICH uses IBD.",
    },
    Timeline {
        report_type: "PADER (Periodic Adverse Drug Experience Report)",
        category: "Periodic",
        fda: "Quarterly (Yr 1-3), then annually",
        ema: "N/A (uses PBRER)",
        ich: "N/A (PBRER recommended)",
        who: "N/A",
        day_0: "US approval date",
        differences: "FDA-specific format. Not used outside the US.",
    },
    Timeline {
        report_type: "Addendum to Clinical Overview (ACO)",
        category: "Periodic",
        fda: "N/A",
        ema: "Required with PBRER for some products",
        ich: "N/A",
        who: "N/A",
        day_0: "PBRER data lock point",
        differences: "EMA-specific requirement for certain product types",
    },
    /* === Risk Management === */
    Timeline {
        report_type: "RMP (Risk Management Plan)",
        category: "Risk Management",
        fda: "REMS (not RMP format)",
        ema: "At time of MAA + updates per triggers",
        ich: "E2E provides framework",
        who: "Recommended for essential medicines",
        day_0: "MAA submission / significant safety trigger",
        differences: "EMA requires formal RMP; FDA uses REMS; ICH E2E is non-binding guidance",
    },
    Timeline {
        report_type: "REMS (Risk Evaluation & Mitigation Strategy)",
        category: "Risk Management",
        fda: "Required if benefits need additional measures; REMS assessments at 18mo, 3yr, 7yr",
        ema: "N/A (uses RMP additional risk minimization)",
        ich: "N/A",
        who: "N/A",
        day_0: "FDA approval with REMS requirement",
        differences: "US-specific. REMS can include ETASU, medication guides, communication plans.",
    },
    Timeline {
        report_type: "Signal Detection & Evaluation",
        category: "Risk Management",
        fda: "Ongoing (no fixed timeline); FDA Sentinel active surveillance",
        ema: "Monthly screening (EU signal management process)",
        ich: "No fixed timeline (E2E guidance)",
        who: "VigiBase screening cycles",
        day_0: "Continuous process",
        differences: "EMA has formalized monthly signal detection. FDA uses Sentinel + FAERS mining.",
    },
    /* === Post-Marketing Commitments === */
    Timeline {
        report_type: "Post-Marketing Study Report (PASS/PMR)",
        category: "Post-Marketing",
        fda: "Annual status reports; final report per agreed timeline",
        ema: "Per PASS protocol; interim + final reports to PRAC",
        ich: "N/A (jurisdiction-specific)",
        who: "N/A",
        day_0: "Study protocol approval date",
        differences: "FDA PMR and EMA PASS have different governance structures",
    },
    Timeline {
        report_type: "Safety Variation / Labeling Update",
        category: "Post-Marketing",
        fda: "CBE-0 (immediate), CBE-30, PAS",
        ema: "Type IA, IAIN, IB, Type II (per urgency)",
        ich: "N/A (jurisdiction-specific)",
        who: "N/A",
        day_0: "Date safety issue identified requiring labeling change",
        differences: "FDA CBE-0 allows immediate implementation; EMA urgent safety restriction is equivalent",
    },
    Timeline {
        report_type: "Urgent Safety Restriction",
        category: "Post-Marketing",
        fda: "CBE-0 supplement (immediate implementation)",
        ema: "Immediate + notify within 24 hours",
        ich: "N/A",
        who: "Immediate action recommended",
        day_0: "Date risk identified requiring urgent action",
        differences: "EMA requires 24h notification to national competent authorities",
    },
    /* === Regulatory Submissions === */
    Timeline {
        report_type: "MedWatch 3500A (Mandatory)",
        category: "FDA-Specific",
        fda: "15 days (serious), periodic (non-serious)",
        ema: "N/A",
        ich: "N/A",
        who: "N/A",
        day_0: "Date manufacturer becomes aware",
        differences: "FDA-specific mandatory reporting form for manufacturers",
    },
    Timeline {
        report_type: "EudraVigilance Submission",
        category: "EMA-Specific",
        fda: "N/A",
        ema: "15 days (serious), 90 days (non-serious) via E2B(R3)",
        ich: "E2B(R3) format standard",
        who: "VigiFlow for member states",
        day_0: "Date MAH becomes aware of case",
        differences: "EMA mandates electronic submission via EudraVigilance; WHO uses VigiFlow",
    },
    Timeline {
        report_type: "CIOMS I Form",
        category: "International",
        fda: "Accepted as attachment to MedWatch",
        ema: "Superseded by E2B(R3) electronic submission",
        ich: "Legacy format; E2B(R3) preferred",
        who: "Accepted by some member states",
        day_0: "Per individual case awareness date",
        differences: "Paper-based legacy format. E2B(R3) XML is the current international standard.",
    },
    /* === Aggregate Safety === */
    Timeline {
        report_type: "Benefit-Risk Assessment Update",
        category: "Aggregate",
        fda: "Part of NDA annual report / PADER",
        ema: "Part of PBRER (Module SVIII)",
        ich: "PBRER Section 16 (E2C(R2))",
        who: "Recommended in periodic reviews",
        day_0: "Per periodic reporting schedule",
        differences: "All frameworks require B-R assessment; format varies by jurisdiction",
    },
    Timeline {
        report_type: "Signal Summary Report",
        category: "Aggregate",
        fda: "Ad hoc to FDA; part of annual report",
        ema: "Within 60 days of PRAC signal assessment",
        ich: "Part of PBRER signal section",
        who: "Part of WHO signal review process",
        day_0: "Date signal validated",
        differences: "EMA has most structured signal governance via PRAC",
    },
];

#[component]
pub fn TimelinesPage() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let category_filter = RwSignal::new(String::from("All"));

    let categories = vec![
        "All",
        "ICSR",
        "Clinical Trial",
        "Periodic",
        "Risk Management",
        "Post-Marketing",
        "FDA-Specific",
        "EMA-Specific",
        "International",
        "Aggregate",
    ];

    let filtered = Signal::derive(move || {
        let q = search.get().to_lowercase();
        let cat = category_filter.get();
        TIMELINES
            .iter()
            .filter(|t| {
                let cat_match = cat == "All" || t.category == cat;
                let search_match = q.is_empty()
                    || t.report_type.to_lowercase().contains(&q)
                    || t.fda.to_lowercase().contains(&q)
                    || t.ema.to_lowercase().contains(&q)
                    || t.differences.to_lowercase().contains(&q)
                    || t.day_0.to_lowercase().contains(&q);
                cat_match && search_match
            })
            .collect::<Vec<_>>()
    });

    let count = Signal::derive(move || filtered.get().len());

    view! {
        <div class="mx-auto max-w-7xl px-4 py-8">
            <header class="mb-8">
                <h1 class="text-3xl font-black text-white font-mono uppercase tracking-tight">"Reporting Timelines"</h1>
                <p class="mt-2 text-slate-400 max-w-3xl">
                    "Cross-jurisdictional comparison of PV reporting deadlines. FDA, EMA, ICH, and WHO requirements side-by-side."
                </p>
            </header>

            /* Filters */
            <div class="flex flex-col md:flex-row gap-4 mb-6">
                <div class="flex-1">
                    <input
                        type="text"
                        placeholder="Search timelines..."
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                        prop:value=move || search.get()
                        on:input=move |ev| search.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex flex-wrap gap-1.5">
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
                                        "px-3 py-1.5 rounded-lg bg-cyan-500/20 border border-cyan-500/40 text-[10px] font-bold text-cyan-400 font-mono uppercase tracking-widest"
                                    } else {
                                        "px-3 py-1.5 rounded-lg border border-slate-700 bg-slate-950 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest hover:border-slate-500 transition-all"
                                    }
                                }
                            >
                                {cat}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            /* Count */
            <p class="text-[10px] text-slate-600 font-mono uppercase tracking-widest mb-4">
                {move || format!("{} TIMELINES", count.get())}
            </p>

            /* Timeline Cards */
            <div class="space-y-3">
                {move || filtered.get().into_iter().map(|t| {
                    let report_type = t.report_type.to_string();
                    let category = t.category.to_string();
                    let fda = t.fda.to_string();
                    let ema = t.ema.to_string();
                    let ich = t.ich.to_string();
                    let who = t.who.to_string();
                    let day_0 = t.day_0.to_string();
                    let differences = t.differences.to_string();
                    let cat_color = match t.category {
                        "ICSR" => "text-red-400 bg-red-500/10 border-red-500/20",
                        "Clinical Trial" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        "Periodic" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                        "Risk Management" => "text-purple-400 bg-purple-500/10 border-purple-500/20",
                        "Post-Marketing" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    let badge_class = format!("px-2 py-0.5 rounded-full border text-[9px] font-bold font-mono uppercase {cat_color}");

                    view! {
                        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-all">
                            <div class="flex items-start justify-between gap-4 mb-4">
                                <h3 class="text-sm font-bold text-white font-mono">{report_type}</h3>
                                <span class=badge_class>{category}</span>
                            </div>

                            <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 mb-4">
                                <AuthorityCell authority="FDA" value=fda />
                                <AuthorityCell authority="EMA" value=ema />
                                <AuthorityCell authority="ICH" value=ich />
                                <AuthorityCell authority="WHO" value=who />
                            </div>

                            <div class="flex flex-col md:flex-row gap-4 text-[10px]">
                                <div class="flex-1">
                                    <span class="font-bold text-slate-600 uppercase tracking-widest">"Day 0: "</span>
                                    <span class="text-slate-400 font-mono">{day_0}</span>
                                </div>
                                <div class="flex-1">
                                    <span class="font-bold text-amber-500/80 uppercase tracking-widest">"Key Differences: "</span>
                                    <span class="text-slate-400 font-mono">{differences}</span>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn AuthorityCell(authority: &'static str, value: String) -> impl IntoView {
    let color = match authority {
        "FDA" => "text-blue-400",
        "EMA" => "text-cyan-400",
        "ICH" => "text-emerald-400",
        "WHO" => "text-amber-400",
        _ => "text-slate-400",
    };
    view! {
        <div class="rounded-lg bg-slate-950 border border-slate-800 p-2.5">
            <p class=format!("text-[9px] font-black uppercase tracking-widest mb-1 {color}")>{authority}</p>
            <p class="text-[10px] text-slate-300 font-mono leading-relaxed">{value}</p>
        </div>
    }
}
