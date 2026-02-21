//! Enterprise readiness page for pharma adoption

use leptos::prelude::*;

#[component]
pub fn EnterpriseReadinessPage() -> impl IntoView {
    let pillars = [
        (
            "Regulatory + Validation",
            "Computerized system validation package with URS, risk matrix, and IQ/OQ/PQ templates.",
        ),
        (
            "Security + Privacy",
            "Access control model, boundary documentation, and security posture for technical due diligence.",
        ),
        (
            "Integration + Data Flow",
            "Staged rollout plan with API contract mapping and migration runbook.",
        ),
        (
            "Auditability + Governance",
            "Traceability expectations and review-ready evidence model across critical workflows.",
        ),
        (
            "Change Management",
            "Role-based enablement, operating playbooks, and adoption checkpoints.",
        ),
        (
            "Value Realization",
            "Baseline metrics and measurement plan for cycle-time and throughput impact.",
        ),
    ];

    view! {
        <div class="min-h-screen bg-slate-950">
            <div class="mx-auto max-w-5xl px-4 py-14">
                <header class="mb-10">
                    <h1 class="text-4xl font-black font-mono text-white uppercase tracking-tight">
                        "Enterprise Adoption Readiness"
                    </h1>
                    <p class="mt-4 text-slate-400 max-w-3xl">
                        "A procurement and QA-facing readiness package for pharmaceutical technology adoption."
                    </p>
                </header>

                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-6">
                    <h2 class="text-sm font-bold font-mono uppercase tracking-[0.2em] text-cyan-400">
                        "Readiness Pillars"
                    </h2>
                    <div class="mt-4 grid gap-3 md:grid-cols-2">
                        {pillars.into_iter().map(|(title, desc)| view! {
                            <div class="rounded-xl border border-slate-800 bg-slate-950/60 p-4">
                                <h3 class="text-white font-semibold">{title}</h3>
                                <p class="mt-2 text-sm text-slate-400">{desc}</p>
                            </div>
                        }).collect_view()}
                    </div>
                </section>

                <section class="mt-8 rounded-2xl border border-slate-800 bg-slate-900/40 p-6">
                    <h2 class="text-sm font-bold font-mono uppercase tracking-[0.2em] text-cyan-400">
                        "Download Procurement One-Pager"
                    </h2>
                    <p class="mt-3 text-sm text-slate-400">
                        "Share this with procurement, quality, and information security stakeholders."
                    </p>
                    <div class="mt-5 flex flex-col gap-3 sm:flex-row">
                        <a
                            href="/enterprise-readiness-one-pager.md"
                            download="nucleus-enterprise-readiness-one-pager.md"
                            class="rounded-lg bg-cyan-600 px-6 py-3 text-center text-xs font-bold uppercase tracking-widest text-white hover:bg-cyan-500 transition-colors"
                        >
                            "Download One-Pager"
                        </a>
                        <a
                            href="/contact"
                            class="rounded-lg border border-slate-700 px-6 py-3 text-center text-xs font-bold uppercase tracking-widest text-slate-300 hover:bg-slate-800 transition-colors"
                        >
                            "Request Enterprise Workshop"
                        </a>
                    </div>
                </section>
            </div>
        </div>
    }
}
