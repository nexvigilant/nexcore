//! Regulatory Intelligence Dashboard
//!
//! AI-powered FDA/EMA/ICH monitoring with search, tabbed feeds,
//! stat cards, and document impact analysis.

use leptos::prelude::*;

/// Active tab for the regulatory dashboard
#[derive(Clone, Copy, PartialEq, Eq)]
enum RegTab {
    Feed,
    Guidances,
    Enforcement,
    Safety,
}

/// Impact level for regulatory documents
#[derive(Clone, Copy)]
enum Impact {
    High,
    Medium,
    Low,
}

impl Impact {
    fn class(self) -> &'static str {
        match self {
            Self::High => "bg-red-500/20 text-red-400",
            Self::Medium => "bg-amber-500/20 text-amber-400",
            Self::Low => "bg-emerald-500/20 text-emerald-400",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::High => "High Impact",
            Self::Medium => "Medium Impact",
            Self::Low => "Low Impact",
        }
    }
}

/// Source type icon mapping
#[derive(Clone, Copy)]
enum SourceType {
    Guidance,
    WarningLetter,
    SafetyCommunication,
    Recall,
    FederalRegister,
}

impl SourceType {
    fn icon(self) -> &'static str {
        match self {
            Self::Guidance => "📖",
            Self::WarningLetter => "⚠️",
            Self::SafetyCommunication => "🔔",
            Self::Recall => "🔄",
            Self::FederalRegister => "📄",
        }
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(RegTab::Feed);
    let (search_query, set_search_query) = signal(String::new());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-6">
            /* Header */
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white">"Regulatory Intelligence"</h1>
                    <p class="text-slate-400">"AI-powered FDA monitoring with personalized alerts"</p>
                </div>
                <div class="flex items-center gap-2">
                    <button class="flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-slate-300 hover:border-slate-600 transition-colors">
                        "🔔 Alerts"
                        <span class="rounded-full bg-red-500/20 px-1.5 py-0.5 text-xs text-red-400">"3"</span>
                    </button>
                    <button class="flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-slate-300 hover:border-slate-600 transition-colors">
                        "📅 Deadlines"
                    </button>
                </div>
            </div>

            /* Search & Filter Bar */
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4">
                <div class="flex flex-col gap-4 sm:flex-row">
                    <div class="relative flex-1">
                        <span class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500">"🔍"</span>
                        <input
                            type="text"
                            placeholder="Search guidances, warning letters, safety communications..."
                            class="w-full rounded-lg border border-slate-700 bg-slate-800 py-2 pl-10 pr-4 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="flex gap-2">
                        <button class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-slate-400 hover:text-white transition-colors">
                            "⚙️"
                        </button>
                        <button class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-slate-400 hover:text-white transition-colors">
                            "🔄"
                        </button>
                    </div>
                </div>
            </div>

            /* Tabs */
            <div class="flex gap-2">
                <RegTabButton label="📈 Feed" active=Signal::derive(move || active_tab.get() == RegTab::Feed)
                    on_click=move |_| set_active_tab.set(RegTab::Feed)/>
                <RegTabButton label="📖 Guidances" active=Signal::derive(move || active_tab.get() == RegTab::Guidances)
                    on_click=move |_| set_active_tab.set(RegTab::Guidances)/>
                <RegTabButton label="⚠️ Enforcement" active=Signal::derive(move || active_tab.get() == RegTab::Enforcement)
                    on_click=move |_| set_active_tab.set(RegTab::Enforcement)/>
                <RegTabButton label="🛡️ Safety" active=Signal::derive(move || active_tab.get() == RegTab::Safety)
                    on_click=move |_| set_active_tab.set(RegTab::Safety)/>
            </div>

            /* Tab content */
            {move || match active_tab.get() {
                RegTab::Feed => view! { <FeedContent search=search_query/> }.into_any(),
                RegTab::Guidances => view! { <GuidancesContent/> }.into_any(),
                RegTab::Enforcement => view! { <EnforcementContent/> }.into_any(),
                RegTab::Safety => view! { <SafetyContent/> }.into_any(),
            }}
        </div>
    }
}

#[component]
fn RegTabButton(
    label: &'static str,
    #[prop(into)] active: Signal<bool>,
    on_click: impl Fn(leptos::ev::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <button
            class=move || if active.get() {
                "rounded-lg bg-cyan-500/20 px-4 py-2 text-sm font-medium text-cyan-400 border border-cyan-500/30"
            } else {
                "rounded-lg bg-slate-800 px-4 py-2 text-sm font-medium text-slate-400 hover:text-white transition-colors border border-transparent"
            }
            on:click=on_click
        >
            {label}
        </button>
    }
}

/* ── Feed Tab ──────────────────────────────────────── */

#[component]
fn FeedContent(search: ReadSignal<String>) -> impl IntoView {
    view! {
        <div class="space-y-6">
            /* Stats Overview */
            <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
                <FeedStat label="New Today" value="12" color="text-white"/>
                <FeedStat label="Pending Deadlines" value="5" color="text-amber-400"/>
                <FeedStat label="High Impact" value="3" color="text-red-400"/>
                <FeedStat label="This Week" value="47" color="text-white"/>
            </div>

            /* Document Feed */
            <div class="space-y-4">
                <DocCard
                    title="Draft Guidance: Pharmacovigilance Considerations for Cell and Gene Therapy Products"
                    source=SourceType::Guidance
                    center="CBER"
                    date="Jan 15, 2026"
                    impact=Impact::High
                    summary="New guidance addressing unique safety monitoring requirements for advanced therapy products including CAR-T, gene therapy, and tissue-engineered products."
                    areas=vec!["biologics", "oncology", "rare diseases"]
                    comment_deadline="Apr 15, 2026"
                />
                <DocCard
                    title="Warning Letter: ABC Pharma — CGMP Violations"
                    source=SourceType::WarningLetter
                    center="CDER"
                    date="Jan 12, 2026"
                    impact=Impact::Medium
                    summary="Data integrity and laboratory control violations identified during inspection. Includes failure to investigate OOS results and backdating of records."
                    areas=vec!["drugs", "CGMP", "data integrity"]
                    comment_deadline=""
                />
                <DocCard
                    title="Safety Communication: Risk of Serious Allergic Reactions with Drug X"
                    source=SourceType::SafetyCommunication
                    center="CDER"
                    date="Jan 10, 2026"
                    impact=Impact::High
                    summary="New safety information regarding anaphylaxis risk requiring label update. Post-marketing data reveals higher-than-expected incidence in pediatric populations."
                    areas=vec!["drugs", "immunology", "pediatrics"]
                    comment_deadline=""
                />
                <DocCard
                    title="Final Guidance: Real-World Evidence for Regulatory Decision-Making"
                    source=SourceType::Guidance
                    center="CDER"
                    date="Jan 8, 2026"
                    impact=Impact::Medium
                    summary="Framework for using real-world data (RWD) to generate real-world evidence (RWE) for regulatory submissions, including electronic health records and claims data."
                    areas=vec!["drugs", "RWE", "data science"]
                    comment_deadline=""
                />
                <DocCard
                    title="Class I Recall: Medical Device Y — Software Defect"
                    source=SourceType::Recall
                    center="CDRH"
                    date="Jan 5, 2026"
                    impact=Impact::High
                    summary="Software defect in infusion pump may deliver incorrect dose. Firm-initiated recall affecting 12,000 units distributed in the US."
                    areas=vec!["devices", "software", "Class I"]
                    comment_deadline=""
                />
                <DocCard
                    title="Federal Register: Proposed Rule for Electronic Submission of Adverse Event Reports"
                    source=SourceType::FederalRegister
                    center="FDA"
                    date="Jan 3, 2026"
                    impact=Impact::Low
                    summary="Proposed amendment to 21 CFR 314 and 600 to require electronic submission of all post-marketing adverse event reports via FAERS."
                    areas=vec!["regulation", "FAERS", "electronic submission"]
                    comment_deadline="Mar 15, 2026"
                />
            </div>

            /* Data source info + Live feed link */
            <div class="rounded-xl border border-dashed border-slate-700 bg-slate-900/30 p-4">
                <div class="flex items-center justify-between text-sm text-slate-500">
                    <span>"Showing 6 documents from openFDA"</span>
                    <a href="/regulatory/live" class="flex items-center gap-2 text-cyan-400 hover:text-cyan-300 transition-colors font-mono text-xs">
                        <span class="h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></span>
                        "Open Live Feed \u{2192}"
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn FeedStat(label: &'static str, value: &'static str, color: &'static str) -> impl IntoView {
    let value_class = format!("text-2xl font-bold {color}");
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4">
            <p class="pb-1 text-sm text-slate-500">{label}</p>
            <p class=value_class>{value}</p>
        </div>
    }
}

#[component]
fn DocCard(
    title: &'static str,
    source: SourceType,
    center: &'static str,
    date: &'static str,
    impact: Impact,
    summary: &'static str,
    areas: Vec<&'static str>,
    comment_deadline: &'static str,
) -> impl IntoView {
    view! {
        <div class="cursor-pointer rounded-xl border border-slate-800 bg-slate-900/50 p-4 hover:border-slate-700 transition-colors">
            <div class="flex items-start justify-between gap-4">
                <div class="flex items-start gap-3">
                    <div class="rounded-lg bg-slate-800 p-2 text-lg">
                        {source.icon()}
                    </div>
                    <div class="space-y-1">
                        <h3 class="text-sm font-semibold leading-tight text-white">{title}</h3>
                        <div class="flex flex-wrap items-center gap-2 text-xs text-slate-500">
                            <span>{center}</span>
                            <span>"·"</span>
                            <span>{date}</span>
                            {if !comment_deadline.is_empty() {
                                view! {
                                    <span>"·"</span>
                                    <span class="flex items-center text-amber-400">
                                        "⏰ Comments due "{comment_deadline}
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>
                    </div>
                </div>
                <span class=format!("shrink-0 rounded-full px-2.5 py-0.5 text-xs font-medium {}", impact.class())>
                    {impact.label()}
                </span>
            </div>
            <p class="mt-3 text-sm text-slate-400">{summary}</p>
            <div class="mt-3 flex flex-wrap gap-2">
                {areas.into_iter().map(|area| view! {
                    <span class="rounded-full border border-slate-700 px-2 py-0.5 text-xs text-slate-400">{area}</span>
                }).collect_view()}
            </div>
        </div>
    }
}

/* ── Guidances Tab ─────────────────────────────────── */

#[component]
fn GuidancesContent() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="mb-4 text-lg font-semibold text-white">"Active FDA Guidances"</h2>
                <p class="mb-6 text-sm text-slate-400">"Draft and final guidances with AI-generated impact summaries."</p>
                <div class="space-y-3">
                    <GuidanceRow
                        code="FDA-2025-D-4521"
                        title="PV for Cell and Gene Therapy"
                        status="Draft"
                        impact=Impact::High
                        deadline="Apr 15, 2026"
                    />
                    <GuidanceRow
                        code="FDA-2025-D-3891"
                        title="Real-World Evidence for Regulatory Decisions"
                        status="Final"
                        impact=Impact::Medium
                        deadline=""
                    />
                    <GuidanceRow
                        code="FDA-2025-D-2234"
                        title="AI/ML in Drug Development"
                        status="Draft"
                        impact=Impact::High
                        deadline="Mar 30, 2026"
                    />
                    <GuidanceRow
                        code="FDA-2025-D-1876"
                        title="Diversity in Clinical Trials"
                        status="Final"
                        impact=Impact::Medium
                        deadline=""
                    />
                    <GuidanceRow
                        code="FDA-2024-D-9923"
                        title="Digital Health Technologies for Remote Data Acquisition"
                        status="Final"
                        impact=Impact::Low
                        deadline=""
                    />
                </div>
            </div>

            /* ICH Guidelines Reference */
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="mb-4 text-lg font-semibold text-white">"ICH PV Guidelines"</h2>
                <div class="space-y-3">
                    <IchRow code="ICH E2A" title="Clinical Safety Data Management" scope="ICSR definitions, expedited reporting"/>
                    <IchRow code="ICH E2B(R3)" title="Electronic Transmission of ICSRs" scope="E2B format and data elements"/>
                    <IchRow code="ICH E2C(R2)" title="Periodic Benefit-Risk Evaluation" scope="PBRER format and content"/>
                    <IchRow code="ICH E2D" title="Post-Approval Safety Data" scope="Expedited and periodic reporting"/>
                    <IchRow code="ICH E2E" title="Pharmacovigilance Planning" scope="Risk-based PV planning"/>
                    <IchRow code="ICH E2F" title="DSUR" scope="Development Safety Update Report"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn GuidanceRow(
    code: &'static str,
    title: &'static str,
    status: &'static str,
    impact: Impact,
    deadline: &'static str,
) -> impl IntoView {
    let status_class = match status {
        "Draft" => "text-amber-400 bg-amber-500/10",
        "Final" => "text-emerald-400 bg-emerald-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-3 hover:border-slate-700 transition-colors cursor-pointer">
            <div class="flex items-center gap-3">
                <span class="shrink-0 rounded bg-slate-800 px-2 py-1 text-xs font-mono text-cyan-400">{code}</span>
                <div>
                    <h3 class="text-sm font-medium text-white">{title}</h3>
                    {if !deadline.is_empty() {
                        view! { <p class="text-xs text-amber-400">"Comment deadline: "{deadline}</p> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
            <div class="flex items-center gap-2">
                <span class=format!("rounded-full px-2 py-0.5 text-xs font-medium {}", impact.class())>{impact.label()}</span>
                <span class=format!("rounded-full px-2 py-0.5 text-xs font-medium {status_class}")>{status}</span>
            </div>
        </div>
    }
}

#[component]
fn IchRow(code: &'static str, title: &'static str, scope: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-start gap-4 rounded-lg border border-slate-800/50 bg-slate-900/30 p-3 hover:border-slate-700 transition-colors">
            <span class="shrink-0 rounded bg-slate-800 px-2 py-1 text-xs font-mono text-cyan-400">{code}</span>
            <div>
                <h3 class="text-sm font-medium text-white">{title}</h3>
                <p class="text-xs text-slate-500">{scope}</p>
            </div>
        </div>
    }
}

/* ── Enforcement Tab ───────────────────────────────── */

#[component]
fn EnforcementContent() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="mb-4 text-lg font-semibold text-white">"Enforcement Actions"</h2>
                <p class="mb-6 text-sm text-slate-400">"Warning letters, Form 483 observations, and recall activity."</p>

                <div class="space-y-4">
                    <EnforcementCard
                        title="Warning Letter: ABC Pharma — CGMP Violations"
                        action_type="Warning Letter"
                        center="CDER"
                        date="Jan 12, 2026"
                        severity="Serious"
                        findings=vec![
                            "Failure to investigate OOS results (21 CFR 211.192)",
                            "Inadequate laboratory controls (21 CFR 211.160)",
                            "Data integrity violations — backdated analytical records",
                        ]
                    />
                    <EnforcementCard
                        title="Form 483: XYZ Biologics — Sterility Assurance"
                        action_type="Form 483"
                        center="CBER"
                        date="Jan 8, 2026"
                        severity="Major"
                        findings=vec![
                            "Environmental monitoring excursions not investigated",
                            "Media fill failures not adequately documented",
                            "Training records incomplete for aseptic processing staff",
                        ]
                    />
                    <EnforcementCard
                        title="Class I Recall: Infusion Pump Software Defect"
                        action_type="Recall"
                        center="CDRH"
                        date="Jan 5, 2026"
                        severity="Critical"
                        findings=vec![
                            "Software defect may deliver incorrect dose (±15% error)",
                            "12,000 units affected in US distribution",
                            "Firmware update v4.2.1 issued as corrective action",
                        ]
                    />
                    <EnforcementCard
                        title="Consent Decree: DEF Labs — Repeated CGMP Failures"
                        action_type="Consent Decree"
                        center="CDER"
                        date="Dec 20, 2025"
                        severity="Critical"
                        findings=vec![
                            "Third Warning Letter in 5 years with unresolved observations",
                            "Court-ordered suspension of manufacturing operations",
                            "Independent third-party audit required before resumption",
                        ]
                    />
                </div>
            </div>

            /* Enforcement stats */
            <div class="grid gap-4 md:grid-cols-3">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4 text-center">
                    <p class="text-xs text-slate-500">"Warning Letters (2026)"</p>
                    <p class="text-2xl font-bold text-red-400">"23"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4 text-center">
                    <p class="text-xs text-slate-500">"Form 483s Issued"</p>
                    <p class="text-2xl font-bold text-amber-400">"156"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4 text-center">
                    <p class="text-xs text-slate-500">"Active Recalls"</p>
                    <p class="text-2xl font-bold text-white">"42"</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn EnforcementCard(
    title: &'static str,
    action_type: &'static str,
    center: &'static str,
    date: &'static str,
    severity: &'static str,
    findings: Vec<&'static str>,
) -> impl IntoView {
    let type_color = match action_type {
        "Warning Letter" => "text-red-400 bg-red-500/10",
        "Form 483" => "text-amber-400 bg-amber-500/10",
        "Recall" => "text-orange-400 bg-orange-500/10",
        "Consent Decree" => "text-red-400 bg-red-500/20",
        _ => "text-slate-400 bg-slate-800",
    };
    let severity_color = match severity {
        "Critical" => "text-red-400",
        "Serious" => "text-orange-400",
        "Major" => "text-amber-400",
        _ => "text-slate-400",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/30 p-4 hover:border-slate-700 transition-colors">
            <div class="flex items-start justify-between">
                <div class="flex items-center gap-2">
                    <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {type_color}")>{action_type}</span>
                    <span class="text-xs text-slate-500">{center}" · "{date}</span>
                </div>
                <span class=format!("text-xs font-medium {severity_color}")>{severity}</span>
            </div>
            <h3 class="mt-2 text-sm font-semibold text-white">{title}</h3>
            <ul class="mt-3 space-y-1.5">
                {findings.into_iter().map(|f| view! {
                    <li class="flex items-start gap-2 text-xs text-slate-400">
                        <span class="mt-0.5 text-red-400">"•"</span>
                        {f}
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

/* ── Safety Tab ────────────────────────────────────── */

#[component]
fn SafetyContent() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="mb-4 text-lg font-semibold text-white">"Safety Communications"</h2>
                <p class="mb-6 text-sm text-slate-400">"MedWatch alerts, drug safety communications, and risk evaluations."</p>

                <div class="space-y-4">
                    <SafetyAlert
                        title="Risk of Serious Allergic Reactions with Drug X"
                        alert_type="Drug Safety Communication"
                        date="Jan 10, 2026"
                        severity="Serious"
                        action="Label update required — new boxed warning for anaphylaxis risk in pediatric patients"
                        affected="All marketed formulations of Drug X (oral and injectable)"
                    />
                    <SafetyAlert
                        title="Updated REMS for Opioid Analgesics"
                        alert_type="REMS Update"
                        date="Jan 7, 2026"
                        severity="Important"
                        action="Revised patient counseling document and updated prescriber training requirements"
                        affected="All extended-release/long-acting opioid analgesics"
                    />
                    <SafetyAlert
                        title="Signal of Hepatotoxicity with Biologic Z"
                        alert_type="Safety Signal"
                        date="Jan 4, 2026"
                        severity="Potential"
                        action="Enhanced monitoring and hepatic function testing recommended during first 6 months"
                        affected="Biologic Z — all indications"
                    />
                    <SafetyAlert
                        title="PRAC Recommendation: Suspension of Marketing Authorization"
                        alert_type="EMA Action"
                        date="Dec 28, 2025"
                        severity="Critical"
                        action="European suspension pending Article 31 referral — benefit-risk unfavorable in approved indications"
                        affected="Product W — cardiovascular indications"
                    />
                </div>
            </div>

            /* Signal detection summary */
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="mb-4 text-lg font-semibold text-white">"Signal Detection Summary"</h2>
                <table class="w-full">
                    <thead>
                        <tr class="border-b border-slate-800 text-left text-xs text-slate-500">
                            <th class="pb-2">"Drug"</th>
                            <th class="pb-2">"Event"</th>
                            <th class="pb-2">"PRR"</th>
                            <th class="pb-2">"ROR"</th>
                            <th class="pb-2 text-right">"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <SignalRow drug="Drug X" event="Anaphylaxis" prr="4.2" ror="5.1" status="Confirmed"/>
                        <SignalRow drug="Biologic Z" event="Hepatotoxicity" prr="2.8" ror="3.4" status="Under Review"/>
                        <SignalRow drug="Product W" event="MI" prr="3.1" ror="3.8" status="Validated"/>
                        <SignalRow drug="Drug Y" event="Stevens-Johnson" prr="1.9" ror="2.1" status="Monitoring"/>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn SafetyAlert(
    title: &'static str,
    alert_type: &'static str,
    date: &'static str,
    severity: &'static str,
    action: &'static str,
    affected: &'static str,
) -> impl IntoView {
    let severity_color = match severity {
        "Critical" => "bg-red-500/20 text-red-400",
        "Serious" => "bg-orange-500/20 text-orange-400",
        "Important" => "bg-amber-500/20 text-amber-400",
        "Potential" => "bg-cyan-500/20 text-cyan-400",
        _ => "bg-slate-800 text-slate-400",
    };
    let type_color = match alert_type {
        "Drug Safety Communication" => "text-red-400 bg-red-500/10",
        "REMS Update" => "text-amber-400 bg-amber-500/10",
        "Safety Signal" => "text-cyan-400 bg-cyan-500/10",
        "EMA Action" => "text-violet-400 bg-violet-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/30 p-4 hover:border-slate-700 transition-colors">
            <div class="flex items-start justify-between">
                <div class="flex items-center gap-2">
                    <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {type_color}")>{alert_type}</span>
                    <span class="text-xs text-slate-500">{date}</span>
                </div>
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {severity_color}")>{severity}</span>
            </div>
            <h3 class="mt-2 text-sm font-semibold text-white">{title}</h3>
            <div class="mt-3 space-y-2 text-xs">
                <p class="text-slate-400"><span class="text-slate-500">"Action: "</span>{action}</p>
                <p class="text-slate-400"><span class="text-slate-500">"Affected: "</span>{affected}</p>
            </div>
        </div>
    }
}

#[component]
fn SignalRow(
    drug: &'static str,
    event: &'static str,
    prr: &'static str,
    ror: &'static str,
    status: &'static str,
) -> impl IntoView {
    let status_class = match status {
        "Confirmed" => "text-red-400",
        "Under Review" => "text-amber-400",
        "Validated" => "text-orange-400",
        "Monitoring" => "text-cyan-400",
        _ => "text-slate-400",
    };
    let prr_val: f32 = prr.parse().unwrap_or(0.0);
    let prr_color = if prr_val >= 2.0 {
        "text-red-400 font-medium"
    } else {
        "text-white"
    };
    let ror_val: f32 = ror.parse().unwrap_or(0.0);
    let ror_color = if ror_val >= 2.0 {
        "text-red-400 font-medium"
    } else {
        "text-white"
    };

    view! {
        <tr class="border-b border-slate-800/50 text-sm">
            <td class="py-2.5 font-medium text-white">{drug}</td>
            <td class="py-2.5 text-slate-300">{event}</td>
            <td class=format!("py-2.5 {prr_color}")>{prr}</td>
            <td class=format!("py-2.5 {ror_color}")>{ror}</td>
            <td class=format!("py-2.5 text-right {status_class}")>{status}</td>
        </tr>
    }
}
