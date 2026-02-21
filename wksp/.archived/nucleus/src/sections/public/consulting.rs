//! Consulting page — detailed methodology, credentials, technology backing

use leptos::prelude::*;

#[component]
pub fn ConsultingPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-slate-950 selection:bg-cyan-500/30">
            // Hero
            <section class="relative py-24 px-6 text-center overflow-hidden">
                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] bg-cyan-600/5 rounded-full blur-[100px]"></div>
                <div class="relative z-10 max-w-3xl mx-auto">
                    <h2 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.4em] mb-4">"// CONSULTING"</h2>
                    <h1 class="text-5xl md:text-6xl font-black font-mono text-white uppercase tracking-tighter">"EXPERT PV ADVISORY"</h1>
                    <p class="mt-6 text-lg text-slate-400 font-mono max-w-xl mx-auto">
                        "CLINICAL DOMAIN EXPERTISE MEETS COMPUTATIONAL RIGOR. INDEPENDENT ANALYSIS WITHOUT CORPORATE BIAS."
                    </p>
                </div>
            </section>

            <section class="mx-auto max-w-5xl px-6 pb-32 space-y-20">
                // Methodology
                <div>
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-8">"// METHODOLOGY"</h3>
                    <div class="grid gap-6 md:grid-cols-3">
                        <PhaseCard number="01" title="DISCOVER" desc="Deep-dive into your current PV operations, data flows, and regulatory posture. We understand before we prescribe."/>
                        <PhaseCard number="02" title="ANALYZE" desc="Algorithmic assessment of your processes against industry benchmarks. 75 algorithms provide quantitative insight where others offer opinion."/>
                        <PhaseCard number="03" title="DELIVER" desc="Actionable recommendations with implementation support. Every finding comes with a remediation path and priority ranking."/>
                    </div>
                </div>

                // Credentials
                <div>
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// CREDENTIALS"</h3>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                        <div class="grid gap-8 md:grid-cols-2">
                            <div>
                                <h4 class="text-sm font-mono font-bold text-white uppercase mb-4">"DOMAIN EXPERTISE"</h4>
                                <ul class="space-y-2">
                                    <CredentialItem text="Doctor of Pharmacy (PharmD)"/>
                                    <CredentialItem text="Takeda pharmacovigilance experience"/>
                                    <CredentialItem text="ICH E2E/E2B(R3) implementation"/>
                                    <CredentialItem text="Signal detection & causality assessment"/>
                                    <CredentialItem text="ICSR processing & MedDRA coding"/>
                                </ul>
                            </div>
                            <div>
                                <h4 class="text-sm font-mono font-bold text-white uppercase mb-4">"TECHNOLOGY"</h4>
                                <ul class="space-y-2">
                                    <CredentialItem text="6 disproportionality methods (PRR, ROR, IC, EBGM, Chi-squared, BCPNN)"/>
                                    <CredentialItem text="15-layer PV Operating System (PVOS)"/>
                                    <CredentialItem text="Bayesian trust engine for evidence weighting"/>
                                    <CredentialItem text="Automated FAERS ETL pipeline"/>
                                    <CredentialItem text="Full-stack Rust engineering (143 crates)"/>
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>

                // Why independent
                <div>
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// WHY INDEPENDENT"</h3>
                    <div class="rounded-xl border border-cyan-500/20 bg-slate-900/50 p-8">
                        <p class="text-sm text-slate-300 font-mono leading-relaxed">
                            "Patient safety analysis should be free from the commercial pressures that can distort risk assessment. As an independent consultancy, our analysis is influenced only by the data and the science. We have no product to protect, no quarterly targets to meet, and no corporate hierarchy to navigate. This independence is not just a business model — it is a core value enshrined in our charter."
                        </p>
                    </div>
                </div>

                // CTA
                <div class="text-center">
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-4">"// ENGAGE"</h3>
                    <p class="text-sm text-slate-400 font-mono max-w-lg mx-auto mb-8">
                        "Start with a free 30-minute discovery call. No obligations, no sales pressure — just a conversation about your PV challenges."
                    </p>
                    <div class="flex flex-col sm:flex-row gap-4 justify-center">
                        <a href="/contact" class="px-10 py-4 bg-cyan-600 text-white font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-cyan-500 shadow-[0_0_20px_rgba(34,211,238,0.3)]">
                            "SCHEDULE DISCOVERY CALL"
                        </a>
                        <a href="/services" class="px-10 py-4 border border-slate-700 text-slate-300 font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-slate-900 hover:border-slate-500">
                            "VIEW SERVICE PACKAGES"
                        </a>
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
fn PhaseCard(number: &'static str, title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <span class="text-3xl font-black font-mono text-slate-800">{number}</span>
            <h4 class="mt-2 text-lg font-black font-mono text-white uppercase">{title}</h4>
            <p class="mt-3 text-sm text-slate-400 font-mono leading-relaxed">{desc}</p>
        </div>
    }
}

#[component]
fn CredentialItem(text: &'static str) -> impl IntoView {
    view! {
        <li class="flex items-start gap-2 text-sm font-mono text-slate-300">
            <span class="text-cyan-500 mt-0.5">"+"</span>
            <span>{text}</span>
        </li>
    }
}
