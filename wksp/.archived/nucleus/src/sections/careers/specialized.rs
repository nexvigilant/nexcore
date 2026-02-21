//! Specialized career assessments — advisory, board, readiness tools

use leptos::prelude::*;

/* ------------------------------------------------------------------ */
/*  Advisory Readiness Assessment                                      */
/* ------------------------------------------------------------------ */

#[component]
pub fn AdvisoryReadinessPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Advisory Readiness"</h1>
            <p class="mt-2 text-slate-400">"Assess your capability to serve on Scientific Advisory Boards (SAB) or Safety Data Monitoring Committees (IDMC)."</p>

            <div class="mt-8 space-y-6">
                <AssessmentSection title="Subject Matter Expertise">
                    <CheckRow label="Deep therapeutic area knowledge (e.g. Oncology, Rare Disease)"/>
                    <CheckRow label="Understanding of clinical trial methodology"/>
                    <CheckRow label="Ability to interpret complex safety data signals"/>
                </AssessmentSection>

                <AssessmentSection title="Regulatory & Compliance">
                    <CheckRow label="Knowledge of GCP and ICH guidelines"/>
                    <CheckRow label="Understanding of IDMC charter requirements"/>
                    <CheckRow label="Awareness of conflict of interest disclosures"/>
                </AssessmentSection>

                <AssessmentSection title="Soft Skills">
                    <CheckRow label="Ability to challenge senior leadership constructively"/>
                    <CheckRow label="Clear communication of risk-benefit trade-offs"/>
                    <CheckRow label="Objective decision-making under uncertainty"/>
                </AssessmentSection>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Signal Decision Framework                                          */
/* ------------------------------------------------------------------ */

#[component]
pub fn SignalDecisionPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Signal Decision Framework"</h1>
            <p class="mt-2 text-slate-400">"A structured approach to evaluating and documenting safety signal decisions."</p>

            <div class="mt-8 space-y-4">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-bold text-cyan-400 mb-4">"Step 1: Signal Validation"</h2>
                    <p class="text-sm text-slate-400 leading-relaxed">
                        "Verify the source, quality, and clinical relevance of the observations. Is there a plausible biological mechanism?"
                    </p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-bold text-cyan-400 mb-4">"Step 2: Statistical Strength"</h2>
                    <p class="text-sm text-slate-400 leading-relaxed">
                        "Review PRR, ROR, and IC values. Does the signal persist after adjusting for confounding factors?"
                    </p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-bold text-cyan-400 mb-4">"Step 3: Clinical Review"</h2>
                    <p class="text-sm text-slate-400 leading-relaxed">
                        "Evaluate individual case safety reports (ICSRs). Look for temporal relationship, de-challenge/re-challenge, and alternative explanations."
                    </p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-bold text-cyan-400 mb-4">"Step 4: Regulatory Action"</h2>
                    <p class="text-sm text-slate-400 leading-relaxed">
                        "Determine if the signal requires label updates, DHPCs, or enhanced monitoring in RMP."
                    </p>
                </div>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Board Competencies                                                 */
/* ------------------------------------------------------------------ */

#[component]
pub fn BoardCompetenciesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Board Competencies"</h1>
            <p class="mt-2 text-slate-400">"Evaluate your readiness for corporate board seats in the life sciences sector."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2">
                <CompetencyCard title="Governance" level="Expert" desc="Understanding fiduciary duties, committee structures, and board-management relations."/>
                <CompetencyCard title="Strategy" level="Advanced" desc="Long-term value creation, portfolio management, and competitive landscape analysis."/>
                <CompetencyCard title="Risk Oversight" level="Expert" desc="Regulatory compliance, product safety liabilities, and enterprise risk management."/>
                <CompetencyCard title="Financial Literacy" level="Intermediate" desc="Interpreting balance sheets, audit reports, and capital allocation strategies."/>
            </div>
        </div>
    }
}

#[component]
fn CompetencyCard(title: &'static str, level: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div class="flex items-center justify-between">
                <h3 class="font-bold text-white">{title}</h3>
                <span class="rounded bg-violet-500/10 px-2 py-0.5 text-[10px] font-bold font-mono text-violet-400 uppercase">{level}</span>
            </div>
            <p class="mt-2 text-xs text-slate-400 leading-relaxed">{desc}</p>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Change Readiness                                                   */
/* ------------------------------------------------------------------ */

#[component]
pub fn ChangeReadinessPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Change Readiness"</h1>
            <p class="mt-2 text-slate-400">"Assess your ability to lead safety teams through organizational transformations."</p>

            <div class="mt-8 space-y-4">
                <MaturityLevel level=1 name="Awareness" desc="Identifying the need for change and communicating the 'why' to stakeholders." current=1 progress_pct=25.0/>
                <MaturityLevel level=2 name="Desire" desc="Building buy-in and addressing resistance within the PV department." current=1 progress_pct=0.0/>
                <MaturityLevel level=3 name="Knowledge" desc="Training teams on new technologies (AI/ML) or processes." current=1 progress_pct=0.0/>
                <MaturityLevel level=4 name="Ability" desc="Implementing change and sustaining new behaviors through metrics." current=1 progress_pct=0.0/>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Hidden Job Market                                                  */
/* ------------------------------------------------------------------ */

#[component]
pub fn HiddenJobMarketPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Hidden Job Market"</h1>
            <p class="mt-2 text-slate-400">"Unlock PV opportunities that aren't advertised on public job boards."</p>

            <div class="mt-8 rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                <h2 class="text-lg font-bold text-amber-400 mb-6">"Networking Strategy"</h2>
                <div class="space-y-4">
                    <StrategyStep number="01" title="Target Company List" desc="Identify 20 companies in your therapeutic domain."/>
                    <StrategyStep number="02" title="Contact Identification" desc="Find the Safety Head or QPPV at each target company."/>
                    <StrategyStep number="03" title="Informational Interviews" desc="Request 15-minute 'coffee chats' focused on their challenges."/>
                    <StrategyStep number="04" title="Value Proposition" desc="Position yourself as a solution to their specific safety needs."/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StrategyStep(number: &'static str, title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="flex gap-4">
            <span class="text-2xl font-bold text-slate-700 font-mono">{number}</span>
            <div>
                <h4 class="font-bold text-white">{title}</h4>
                <p class="text-sm text-slate-400">{desc}</p>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Board Effectiveness                                                */
/* ------------------------------------------------------------------ */

#[component]
pub fn BoardEffectivenessPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Board Effectiveness"</h1>
            <p class="mt-2 text-slate-400">"Measure and improve the performance of your safety governance boards."</p>

            <div class="mt-8 space-y-3">
                <MetricRow label="Meeting Attendance" target=">90%" actual="95%"/>
                <MetricRow label="Decision Cycle Time" target="<14 days" actual="18 days"/>
                <MetricRow label="Action Item Completion" target="100%" actual="82%"/>
                <MetricRow label="Stakeholder Satisfaction" target=">4.0/5.0" actual="4.2"/>
            </div>
        </div>
    }
}

#[component]
fn MetricRow(label: &'static str, target: &'static str, actual: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800 bg-slate-900/50 p-4">
            <span class="text-sm text-white font-medium">{label}</span>
            <div class="flex gap-6 text-[10px] font-mono uppercase tracking-wider">
                <div class="flex flex-col items-end">
                    <span class="text-slate-500">"Target"</span>
                    <span class="text-cyan-400">{target}</span>
                </div>
                <div class="flex flex-col items-end">
                    <span class="text-slate-500">"Actual"</span>
                    <span class="text-white">{actual}</span>
                </div>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Fellowship Evaluator                                               */
/* ------------------------------------------------------------------ */

#[component]
pub fn FellowshipEvaluatorPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Fellowship Evaluator"</h1>
            <p class="mt-2 text-slate-400">"Evaluate PV fellowship programs or your own progress as a fellow."</p>

            <div class="mt-8 grid gap-6 md:grid-cols-3">
                <FrameworkCard authority="Clinical" region="ICSR" key_regs="Medical review, Coding (MedDRA), Narrative writing"/>
                <FrameworkCard authority="Aggregate" region="PBRER" key_regs="Data synthesis, Risk-benefit evaluation, Global submission"/>
                <FrameworkCard authority="Signal" region="Analytics" key_regs="Statistical methods, EudraVigilance, FAERS mining"/>
            </div>
        </div>
    }
}

#[component]
fn FrameworkCard(
    authority: &'static str,
    region: &'static str,
    key_regs: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h3 class="text-lg font-bold text-cyan-400">{authority}</h3>
            <p class="text-sm text-slate-500">{region}</p>
            <p class="mt-3 text-sm text-slate-300 leading-relaxed">{key_regs}</p>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Performance Conditions                                             */
/* ------------------------------------------------------------------ */

#[component]
pub fn PerformanceConditionsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Performance Conditions"</h1>
            <p class="mt-2 text-slate-400">"Optimize your environment for peak pharmacovigilance performance."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-2">
                <ToolCategory title="Cognitive Load" icon="🧠" desc="Minimize distractions during critical signal analysis sessions."/>
                <ToolCategory title="Data Ergonomics" icon="🖥\u{fe0f}" desc="Streamline multi-monitor setups for aggregate report writing."/>
                <ToolCategory title="Social Connectivity" icon="🤝" desc="Foster real-time collaboration between case processors and analysts."/>
                <ToolCategory title="Rest & Recovery" icon="🔋" desc="Prevent burnout in high-volume safety departments."/>
            </div>
        </div>
    }
}

#[component]
fn ToolCategory(title: &'static str, icon: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-colors">
            <div class="text-2xl mb-2">{icon}</div>
            <h3 class="font-bold text-white">{title}</h3>
            <p class="mt-1 text-xs text-slate-400 leading-relaxed">{desc}</p>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Startup Health                                                     */
/* ------------------------------------------------------------------ */

#[component]
pub fn StartupHealthPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Startup Health"</h1>
            <p class="mt-2 text-slate-400">"Assess the pharmacovigilance readiness of early-stage biotech startups."</p>

            <div class="mt-8 space-y-4">
                <CheckRow label="Safety Management Plan (SMP) in place"/>
                <CheckRow label="Validated safety database (or outsourced solution)"/>
                <CheckRow label="Designated person responsible for PV"/>
                <CheckRow label="SOPs for ICSR processing and expedited reporting"/>
                <CheckRow label="Signal management process defined"/>
            </div>
        </div>
    }
}

/* ------------------------------------------------------------------ */
/*  Shared UI Components                                               */
/* ------------------------------------------------------------------ */

#[component]
fn MaturityLevel(
    level: u32,
    name: &'static str,
    desc: &'static str,
    current: u32,
    progress_pct: f64,
) -> impl IntoView {
    let _ = progress_pct; // placeholder
    let is_current = level == current;
    let completed = level < current;
    let future = level > current;

    let border = if is_current {
        "border-cyan-500/60 ring-1 ring-cyan-500/30"
    } else if completed {
        "border-cyan-900/50"
    } else {
        "border-slate-800"
    };

    let opacity = if future { "opacity-40" } else { "" };

    view! {
        <div class=format!("flex gap-4 rounded-xl border {border} bg-slate-900/50 p-5 transition-all {opacity}")>
            <div class="flex flex-col items-center">
                <div class="flex h-8 w-8 items-center justify-center rounded-full bg-slate-800 text-sm font-bold text-slate-500">
                    {level}
                </div>
            </div>
            <div class="flex-1">
                <div class="flex items-center gap-2">
                    <h3 class=format!("font-semibold {}", if is_current { "text-cyan-400" } else { "text-white" })>
                        {name}
                    </h3>
                </div>
                <p class="mt-1 text-sm text-slate-400">{desc}</p>
            </div>
        </div>
    }
}

#[component]
fn AssessmentSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-4">{title}</h2>
            <div class="space-y-3">
                {children()}
            </div>
        </div>
    }
}

#[component]
fn CheckRow(label: &'static str) -> impl IntoView {
    let checked = RwSignal::new(false);
    view! {
        <div
            class="flex items-center gap-3 p-2 rounded-lg hover:bg-slate-800/50 transition-colors cursor-pointer"
            on:click=move |_| checked.update(|v| *v = !*v)
        >
            <div class=move || {
                let base = "flex h-5 w-5 items-center justify-center rounded border transition-all";
                if checked.get() {
                    format!("{base} bg-cyan-500 border-cyan-500 text-slate-950")
                } else {
                    format!("{base} border-slate-700 bg-slate-950")
                }
            }>
                {move || if checked.get() {
                    view! {
                        <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="4" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"/>
                        </svg>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>
            <span class=move || if checked.get() { "text-sm text-white transition-colors" } else { "text-sm text-slate-400 transition-colors" }>
                {label}
            </span>
        </div>
    }
}
