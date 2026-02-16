//! Services page — 4 PV consulting service cards

use leptos::prelude::*;

#[component]
pub fn ServicesPage() -> impl IntoView {
    let blocker_checks = RwSignal::new(vec![false; 10]);
    let checked_count =
        Signal::derive(move || blocker_checks.with(|items| items.iter().filter(|v| **v).count()));

    let blockers = [
        (
            "GxP/CSV VALIDATION BURDEN",
            "Pharma teams reject tools that cannot survive validation and SOP review.",
            "We ship a validation package: URS, risk matrix, IQ/OQ/PQ protocol templates, and traceability mapping.",
        ),
        (
            "SECURITY + PRIVACY RISK",
            "No adoption if data handling is unclear for PV case data and user access.",
            "We provide tenant isolation posture, encryption model, role-based access controls, and boundary documentation.",
        ),
        (
            "INTEGRATION FRICTION",
            "If it does not connect to existing safety systems, it becomes shelfware.",
            "We implement staged integration with API mapping, data contracts, and migration runbooks.",
        ),
        (
            "WEAK AUDIT TRAIL",
            "Regulated teams need deterministic evidence for who changed what and when.",
            "Nucleus surfaces review-ready auditability expectations and workflow artifacts tied to reporting.",
        ),
        (
            "AI BLACK-BOX FEAR",
            "Signal or triage recommendations without rationale are blocked by governance.",
            "We expose method provenance, threshold logic, and human override checkpoints.",
        ),
        (
            "NO REGULATORY FIT",
            "Teams abandon tools that do not map to ICH/GVP workflows.",
            "We align features to PV operations and provide guidance paths in regulatory and vigilance modules.",
        ),
        (
            "CHANGE-MANAGEMENT FAILURE",
            "Adoption dies when users are not trained and workflows are not embedded.",
            "We provide role-based onboarding, admin workflow pages, and operating playbooks.",
        ),
        (
            "UNCLEAR BUSINESS CASE",
            "Without quantified benefit, procurement will not approve expansion.",
            "We define baseline metrics (cycle time, case throughput, signal latency) and track deltas.",
        ),
        (
            "GLOBAL OP MODEL GAPS",
            "Global pharma needs consistent governance across local affiliates.",
            "We support centralized controls with configurable section-level processes and oversight hubs.",
        ),
        (
            "VENDOR RISK / LOCK-IN",
            "Procurement blocks vendors with poor portability or unclear continuity.",
            "We provide architecture transparency, export pathways, and transition documentation.",
        ),
    ];

    view! {
        <div class="min-h-screen bg-slate-950 selection:bg-cyan-500/30">
            // Hero
            <section class="relative py-24 px-6 text-center overflow-hidden">
                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] bg-cyan-600/5 rounded-full blur-[100px]"></div>
                <div class="relative z-10 max-w-3xl mx-auto">
                    <h2 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.4em] mb-4">"// SERVICES"</h2>
                    <h1 class="text-5xl md:text-6xl font-black font-mono text-white uppercase tracking-tighter">"PV CONSULTING"</h1>
                    <p class="mt-6 text-lg text-slate-400 font-mono max-w-xl mx-auto">
                        "INDEPENDENT PHARMACOVIGILANCE CONSULTING POWERED BY 75 ALGORITHMS AND DEEP DOMAIN EXPERTISE."
                    </p>
                </div>
            </section>

            // Services grid
            <section class="mx-auto max-w-6xl px-6 pb-32">
                <div class="grid gap-8 md:grid-cols-2">
                    <ServiceCard
                        title="E2B(R3) READINESS ASSESSMENT"
                        price="$5,000 - $10,000"
                        timeline="2-4 weeks"
                        desc="Comprehensive gap analysis of your E2B(R3) compliance posture. We audit your current ICSR processing workflows, data mapping, and submission pipelines against the latest ICH standards."
                        deliverables=vec![
                            "Gap analysis report",
                            "Remediation roadmap",
                            "Data mapping validation",
                            "Submission pipeline audit",
                        ]
                    />
                    <ServiceCard
                        title="SIGNAL DETECTION AUDIT"
                        price="$7,500 - $15,000"
                        timeline="3-6 weeks"
                        desc="Algorithm-powered review of your signal detection methodology. We benchmark your approach against 6 disproportionality methods (PRR, ROR, IC, EBGM, Chi-squared, BCPNN) and identify blind spots."
                        deliverables=vec![
                            "Methodology benchmark report",
                            "Algorithm comparison analysis",
                            "Threshold optimization",
                            "Detection gap identification",
                        ]
                    />
                    <ServiceCard
                        title="FRACTIONAL QPPV SUPPORT"
                        price="$3,000 - $5,000/mo"
                        timeline="Ongoing retainer"
                        desc="Senior PV expertise on demand. Ideal for small pharma and biotech companies needing qualified oversight without the cost of a full-time QPPV. Includes regulatory intelligence and signal management support."
                        deliverables=vec![
                            "Regulatory intelligence briefings",
                            "Signal management oversight",
                            "PSMF maintenance support",
                            "Authority interaction guidance",
                        ]
                    />
                    <ServiceCard
                        title="PV PROCESS OPTIMIZATION"
                        price="$10,000 - $20,000"
                        timeline="4-8 weeks"
                        desc="End-to-end review and optimization of your pharmacovigilance processes. From case intake to regulatory submission, we identify bottlenecks and implement algorithmic improvements."
                        deliverables=vec![
                            "Process flow documentation",
                            "Bottleneck analysis",
                            "Automation recommendations",
                            "KPI framework design",
                        ]
                    />
                </div>

                // Enterprise adoption blockers and mitigations
                <section class="mt-20 rounded-2xl border border-slate-800 bg-slate-900/40 p-8 md:p-10">
                    <div class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
                        <div>
                            <h3 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.3em] mb-3">
                                "// ADOPTION RISK MAP"
                            </h3>
                            <h2 class="text-2xl md:text-3xl font-black font-mono text-white uppercase tracking-tight">
                                "10 REASONS PHARMA WON'T ADOPT — AND HOW WE FIX THEM"
                            </h2>
                        </div>
                        <div class="rounded-xl border border-cyan-500/30 bg-cyan-500/10 px-4 py-3">
                            <p class="text-[10px] font-mono font-bold uppercase tracking-[0.2em] text-cyan-300">
                                "Readiness Score"
                            </p>
                            <p class="text-2xl font-black font-mono text-white">
                                {move || format!("{}/10", checked_count.get())}
                            </p>
                        </div>
                    </div>

                    <p class="mt-4 text-sm text-slate-400 font-mono max-w-4xl">
                        "Use this checklist in discovery calls. Every item should be closed before full deployment."
                    </p>

                    <div class="mt-8 grid gap-4">
                        {blockers.into_iter().enumerate().map(|(idx, (title, risk, fix))| {
                            let is_checked = Signal::derive(move || {
                                blocker_checks.with(|items| items.get(idx).copied().unwrap_or(false))
                            });
                            view! {
                                <div class="rounded-xl border border-slate-800 bg-slate-950/60 p-5">
                                    <div class="flex items-start gap-3">
                                        <button
                                            class=move || {
                                                if is_checked.get() {
                                                    "mt-0.5 h-5 w-5 rounded border border-emerald-400 bg-emerald-500/20 text-emerald-300 text-xs font-bold"
                                                } else {
                                                    "mt-0.5 h-5 w-5 rounded border border-slate-600 bg-slate-900 text-slate-500 text-xs font-bold"
                                                }
                                            }
                                            on:click=move |_| {
                                                blocker_checks.update(|items| {
                                                    if let Some(slot) = items.get_mut(idx) {
                                                        *slot = !*slot;
                                                    }
                                                });
                                            }
                                            title="Mark as mitigated"
                                        >
                                            {move || if is_checked.get() { "✓" } else { "" }}
                                        </button>
                                        <div class="flex-1">
                                            <h4 class="text-sm font-mono font-black text-white uppercase tracking-wide">{title}</h4>
                                            <p class="mt-2 text-sm text-rose-300/90 font-mono">
                                                <span class="text-rose-400 font-bold">"Why adoption fails: "</span>
                                                {risk}
                                            </p>
                                            <p class="mt-2 text-sm text-emerald-300/90 font-mono">
                                                <span class="text-emerald-400 font-bold">"Fix: "</span>
                                                {fix}
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <div class="mt-8 flex flex-col gap-3 sm:flex-row">
                        <a href="/enterprise-readiness" class="px-8 py-3 bg-white/5 border border-white/20 text-white font-mono font-black text-xs uppercase tracking-widest rounded hover:bg-white/10 transition-colors">
                            "OPEN ENTERPRISE READINESS CENTER"
                        </a>
                        <a href="/verify" class="px-8 py-3 bg-cyan-600 text-white font-mono font-black text-xs uppercase tracking-widest rounded hover:bg-cyan-500 transition-colors">
                            "OPEN TRUST + VALIDATION CENTER"
                        </a>
                        <a href="/contact" class="px-8 py-3 border border-slate-700 text-slate-300 font-mono font-black text-xs uppercase tracking-widest rounded hover:bg-slate-900 transition-colors">
                            "BOOK ADOPTION READINESS WORKSHOP"
                        </a>
                    </div>
                </section>

                // CTA
                <div class="mt-16 text-center">
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-4">"// READY TO START?"</h3>
                    <p class="text-sm text-slate-400 font-mono max-w-lg mx-auto mb-8">
                        "Every engagement begins with a free 30-minute discovery call to understand your needs and scope the work."
                    </p>
                    <div class="flex flex-col sm:flex-row gap-4 justify-center">
                        <a href="/contact" class="px-10 py-4 bg-cyan-600 text-white font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-cyan-500 shadow-[0_0_20px_rgba(34,211,238,0.3)]">
                            "REQUEST CONSULTATION"
                        </a>
                        <a href="/consulting" class="px-10 py-4 border border-slate-700 text-slate-300 font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-slate-900 hover:border-slate-500">
                            "LEARN MORE"
                        </a>
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
fn ServiceCard(
    title: &'static str,
    price: &'static str,
    timeline: &'static str,
    desc: &'static str,
    deliverables: Vec<&'static str>,
) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 backdrop-blur-sm p-8 flex flex-col hover:border-slate-700 transition-colors">
            <h3 class="text-lg font-black font-mono text-white uppercase tracking-tight">{title}</h3>

            <div class="mt-4 flex gap-4">
                <div class="inline-flex items-center gap-1 px-2 py-1 rounded border border-cyan-500/20 bg-cyan-500/5">
                    <span class="text-[10px] font-mono font-bold text-cyan-400 uppercase">{price}</span>
                </div>
                <div class="inline-flex items-center gap-1 px-2 py-1 rounded border border-slate-700 bg-slate-800/30">
                    <span class="text-[10px] font-mono font-bold text-slate-400 uppercase">{timeline}</span>
                </div>
            </div>

            <p class="mt-4 text-sm text-slate-400 font-mono leading-relaxed flex-1">{desc}</p>

            <div class="mt-6">
                <h4 class="text-[10px] font-mono font-bold text-slate-500 uppercase tracking-widest mb-3">"DELIVERABLES"</h4>
                <ul class="space-y-2">
                    {deliverables.into_iter().map(|d| view! {
                        <li class="flex items-start gap-2 text-sm font-mono text-slate-300">
                            <span class="text-cyan-500 mt-0.5">"+"</span>
                            <span>{d}</span>
                        </li>
                    }).collect::<Vec<_>>()}
                </ul>
            </div>

            <a href="/contact" class="mt-6 block text-center px-6 py-3 border border-slate-700 text-slate-300 font-mono font-bold text-xs uppercase tracking-widest rounded transition-all hover:bg-slate-900 hover:border-slate-500">
                "INQUIRE"
            </a>
        </div>
    }
}
