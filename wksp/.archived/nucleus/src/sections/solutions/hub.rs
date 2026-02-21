//! Solutions hub — consulting services, workflow templates, and B2B offerings

use leptos::prelude::*;

/// Active tab for the solutions dashboard
#[derive(Clone, Copy, PartialEq, Eq)]
enum SolutionsTab {
    Consulting,
    Templates,
    Programs,
}

#[component]
pub fn HubPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(SolutionsTab::Consulting);

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <header class="mb-8">
                <div class="mb-3 flex items-center gap-3">
                    <div class="rounded-lg bg-cyan-500/10 p-2">
                        <span class="text-2xl">"🤝"</span>
                    </div>
                    <h1 class="text-4xl font-extrabold text-amber-400 md:text-5xl">"Solutions"</h1>
                </div>
                <p class="text-base font-medium text-slate-400 md:text-lg">
                    "Professional pharmaceutical consulting, workflow templates, and safety programs"
                </p>
            </header>

            /* KPI stat cards */
            <div class="mb-8 grid gap-4 md:grid-cols-4">
                <StatCard label="Active Engagements" value="12" color="text-cyan-400"/>
                <StatCard label="Templates Available" value="24" color="text-violet-400"/>
                <StatCard label="Expert Consultants" value="8" color="text-amber-400"/>
                <StatCard label="Programs Delivered" value="47" color="text-emerald-400"/>
            </div>

            /* Tab navigation */
            <div class="mb-6 flex gap-2">
                <TabButton
                    label="Consulting Services"
                    active=Signal::derive(move || active_tab.get() == SolutionsTab::Consulting)
                    on_click=move |_| set_active_tab.set(SolutionsTab::Consulting)
                />
                <TabButton
                    label="Workflow Templates"
                    active=Signal::derive(move || active_tab.get() == SolutionsTab::Templates)
                    on_click=move |_| set_active_tab.set(SolutionsTab::Templates)
                />
                <TabButton
                    label="Safety Programs"
                    active=Signal::derive(move || active_tab.get() == SolutionsTab::Programs)
                    on_click=move |_| set_active_tab.set(SolutionsTab::Programs)
                />
            </div>

            /* Tab content */
            <div>
                {move || match active_tab.get() {
                    SolutionsTab::Consulting => view! { <ConsultingContent/> }.into_any(),
                    SolutionsTab::Templates => view! { <TemplatesContent/> }.into_any(),
                    SolutionsTab::Programs => view! { <ProgramsContent/> }.into_any(),
                }}
            </div>
        </div>
    }
}

#[component]
fn TabButton(
    label: &'static str,
    #[prop(into)] active: Signal<bool>,
    on_click: impl Fn(leptos::ev::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <button
            class=move || if active.get() {
                "rounded-full bg-cyan-500/20 px-4 py-1.5 text-sm font-medium text-cyan-400 border border-cyan-500/30"
            } else {
                "rounded-full bg-slate-800 px-4 py-1.5 text-sm font-medium text-slate-400 hover:text-white transition-colors border border-transparent"
            }
            on:click=on_click
        >
            {label}
        </button>
    }
}

#[component]
fn StatCard(label: &'static str, value: &'static str, color: &'static str) -> impl IntoView {
    let value_class = format!("text-2xl font-bold {color}");
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-4">
            <p class="text-sm text-slate-500">{label}</p>
            <p class=value_class>{value}</p>
        </div>
    }
}

/* ── Consulting Tab ────────────────────────────────── */

#[component]
fn ConsultingContent() -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-2">
            <ServiceCard
                title="Regulatory Consulting"
                icon="📋"
                tier="Enterprise"
                desc="Expert guidance on FDA/EMA/ICH regulatory strategy, submissions, and compliance. Covers 21 CFR 312/314, GVP modules, and CIOMS frameworks."
                features=vec!["Regulatory strategy development", "Submission support (IND/NDA/BLA)", "Compliance gap analysis", "Inspection readiness"]
            />
            <ServiceCard
                title="Drug Safety Program Design"
                icon="🛡️"
                tier="Professional"
                desc="End-to-end pharmacovigilance system design from signal detection to periodic reporting."
                features=vec!["QPPV/PV system setup", "Signal management SOPs", "ICSR processing workflows", "Aggregate report strategy"]
            />
            <ServiceCard
                title="Clinical Trial Safety"
                icon="🔬"
                tier="Professional"
                desc="Safety monitoring and reporting support for clinical development programs."
                features=vec!["DSUR preparation", "SUSAR reporting", "DSMB support", "Safety database design"]
            />
            <ServiceCard
                title="Quality Management"
                icon="✅"
                tier="Standard"
                desc="Quality system implementation aligned with ICH Q10 and FDA expectations."
                features=vec!["QMS implementation", "CAPA management", "Training programs", "Audit preparation"]
            />
            <ServiceCard
                title="Technology Advisory"
                icon="⚙️"
                tier="Enterprise"
                desc="Safety database selection, AI/ML signal detection, and automation strategy."
                features=vec!["Database evaluation", "AI signal detection", "Automation roadmap", "Data migration planning"]
            />
            <ServiceCard
                title="Risk Management"
                icon="📊"
                tier="Professional"
                desc="Benefit-risk assessment, RMP development, and REMS program support."
                features=vec!["Benefit-risk frameworks", "EU RMP authoring", "REMS design", "Risk minimization measures"]
            />
        </div>
    }
}

#[component]
fn ServiceCard(
    title: &'static str,
    icon: &'static str,
    tier: &'static str,
    desc: &'static str,
    features: Vec<&'static str>,
) -> impl IntoView {
    let tier_color = match tier {
        "Enterprise" => "text-amber-400 bg-amber-500/10",
        "Professional" => "text-cyan-400 bg-cyan-500/10",
        "Standard" => "text-emerald-400 bg-emerald-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-colors">
            <div class="flex items-start justify-between">
                <div class="flex items-center gap-3">
                    <span class="text-2xl">{icon}</span>
                    <h3 class="text-lg font-semibold text-white">{title}</h3>
                </div>
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {tier_color}")>{tier}</span>
            </div>
            <p class="mt-3 text-sm text-slate-400">{desc}</p>
            <ul class="mt-4 space-y-1.5">
                {features.into_iter().map(|f| view! {
                    <li class="flex items-center gap-2 text-xs text-slate-300">
                        <span class="text-emerald-400">"✓"</span>
                        {f}
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

/* ── Templates Tab ─────────────────────────────────── */

#[component]
fn TemplatesContent() -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <TemplateCard title="Signal Detection Workflow" category="Detection"
                desc="Complete PRR/ROR/IC/EBGM analysis pipeline with configurable thresholds and automated report generation."/>
            <TemplateCard title="ICSR Processing SOP" category="Compliance"
                desc="Standard operating procedure for individual case safety report intake, triage, coding, assessment, and submission."/>
            <TemplateCard title="PBRER Template" category="Reporting"
                desc="Periodic benefit-risk evaluation report structure per ICH E2C(R2) with all required sections."/>
            <TemplateCard title="Risk Management Plan" category="Risk"
                desc="EU RMP template with safety specification, PV plan, and risk minimization measures."/>
            <TemplateCard title="DSUR Template" category="Reporting"
                desc="Development Safety Update Report per ICH E2F for clinical-stage products."/>
            <TemplateCard title="Aggregate Report Scheduler" category="Compliance"
                desc="Automated tracking of PSUR/PBRER/DSUR submission deadlines across multiple products."/>
            <TemplateCard title="MedDRA Coding Guide" category="Detection"
                desc="Best practices for MedDRA term selection, SMQ use, and coding consistency."/>
            <TemplateCard title="Causality Assessment" category="Detection"
                desc="Structured Naranjo and WHO-UMC causality assessment templates with decision trees."/>
            <TemplateCard title="CAPA Management" category="Compliance"
                desc="Corrective and Preventive Action tracking template with root cause analysis framework."/>
        </div>
    }
}

#[component]
fn TemplateCard(title: &'static str, category: &'static str, desc: &'static str) -> impl IntoView {
    let cat_color = match category {
        "Detection" => "text-cyan-400 bg-cyan-500/10",
        "Compliance" => "text-amber-400 bg-amber-500/10",
        "Reporting" => "text-violet-400 bg-violet-500/10",
        "Risk" => "text-rose-400 bg-rose-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="cursor-pointer rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
            <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {cat_color}")>{category}</span>
            <h3 class="mt-2 font-semibold text-white">{title}</h3>
            <p class="mt-1 text-sm text-slate-400">{desc}</p>
        </div>
    }
}

/* ── Programs Tab ──────────────────────────────────── */

#[component]
fn ProgramsContent() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <ProgramCard
                title="PV System Startup Package"
                duration="8-12 weeks"
                status="Available"
                desc="Complete pharmacovigilance system implementation for startups and emerging biotech companies."
                deliverables=vec![
                    "QPPV designation and support",
                    "PV System Master File (PSMF)",
                    "Core SOPs (12 procedures)",
                    "Safety database configuration",
                    "Staff training program",
                    "Regulatory authority notification"
                ]
            />
            <ProgramCard
                title="Signal Management Excellence"
                duration="6 weeks"
                status="Available"
                desc="Advanced signal detection and evaluation program using quantitative methods and AI-assisted analysis."
                deliverables=vec![
                    "Signal detection SOP and work instructions",
                    "PRR/ROR/IC/EBGM threshold configuration",
                    "Signal evaluation committee charter",
                    "Automated FAERS data mining setup",
                    "Signal tracking database",
                    "Regulatory communication templates"
                ]
            />
            <ProgramCard
                title="Inspection Readiness Program"
                duration="4-6 weeks"
                status="Available"
                desc="Comprehensive audit preparation covering FDA, EMA, and MHRA inspection requirements."
                deliverables=vec![
                    "Gap analysis against regulatory expectations",
                    "Document inventory and remediation plan",
                    "Mock inspection with findings report",
                    "Back room preparation guide",
                    "Staff interview coaching",
                    "CAPA tracking for identified gaps"
                ]
            />
        </div>
    }
}

#[component]
fn ProgramCard(
    title: &'static str,
    duration: &'static str,
    status: &'static str,
    desc: &'static str,
    deliverables: Vec<&'static str>,
) -> impl IntoView {
    let status_color = match status {
        "Available" => "text-emerald-400 bg-emerald-500/10",
        "Waitlist" => "text-amber-400 bg-amber-500/10",
        _ => "text-slate-400 bg-slate-800",
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-colors">
            <div class="flex items-start justify-between">
                <div>
                    <h3 class="text-lg font-semibold text-white">{title}</h3>
                    <p class="mt-1 text-sm text-slate-500">"Duration: "{duration}</p>
                </div>
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {status_color}")>{status}</span>
            </div>
            <p class="mt-3 text-sm text-slate-400">{desc}</p>
            <div class="mt-4">
                <p class="text-xs font-medium text-slate-300 mb-2">"Deliverables:"</p>
                <div class="grid gap-1.5 md:grid-cols-2">
                    {deliverables.into_iter().map(|d| view! {
                        <div class="flex items-center gap-2 text-xs text-slate-400">
                            <span class="text-cyan-400">"→"</span>
                            {d}
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}
