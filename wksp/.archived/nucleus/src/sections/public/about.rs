//! About page — mission, vision, founder, technology

use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-slate-950 selection:bg-cyan-500/30">
            // Hero
            <section class="relative py-24 px-6 text-center overflow-hidden">
                <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] bg-cyan-600/5 rounded-full blur-[100px]"></div>
                <div class="relative z-10 max-w-3xl mx-auto">
                    <h2 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.4em] mb-4">"// ABOUT"</h2>
                    <h1 class="text-5xl md:text-6xl font-black font-mono text-white uppercase tracking-tighter">"NEXVIGILANT"</h1>
                    <p class="mt-6 text-lg text-slate-400 font-mono max-w-xl mx-auto">
                        "INDEPENDENT PHARMACEUTICAL INTELLIGENCE. PATIENT SAFETY AND PROFESSIONAL ADVANCEMENT DEMAND INDEPENDENCE, VIGILANCE, AND EMPOWERMENT."
                    </p>
                </div>
            </section>

            <section class="mx-auto max-w-5xl px-6 pb-32 space-y-20">
                // Two Pillars
                <div>
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-8">"// TWO INSEPARABLE PILLARS"</h3>
                    <div class="grid gap-8 md:grid-cols-2">
                        <div class="rounded-xl border border-cyan-500/20 bg-slate-900/50 p-6 backdrop-blur-sm">
                            <h4 class="text-lg font-black font-mono text-cyan-400 uppercase">"VIGILANCE"</h4>
                            <p class="mt-3 text-sm text-slate-400 font-mono leading-relaxed">
                                "Real-time pharmaceutical surveillance, signal detection, causality investigation, and systemic accountability. Protecting patients through algorithmic intelligence and independent oversight."
                            </p>
                        </div>
                        <div class="rounded-xl border border-amber-500/20 bg-slate-900/50 p-6 backdrop-blur-sm">
                            <h4 class="text-lg font-black font-mono text-amber-400 uppercase">"EMPOWERMENT"</h4>
                            <p class="mt-3 text-sm text-slate-400 font-mono leading-relaxed">
                                "Career infrastructure, cognitive tools, and a global community enabling professionals to advance while advocating for patient safety. Education and growth without corporate gatekeeping."
                            </p>
                        </div>
                    </div>
                </div>

                // Mission & Vision
                <div class="grid gap-8 md:grid-cols-2">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.3em] mb-4">"// MISSION"</h3>
                        <p class="text-sm text-slate-300 font-mono leading-relaxed">
                            "To establish the first independent pharmaceutical intelligence infrastructure that simultaneously strengthens drug safety monitoring and empowers individual healthcare professionals."
                        </p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <h3 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.3em] mb-4">"// VISION"</h3>
                        <p class="text-sm text-slate-300 font-mono leading-relaxed">
                            "A world where pharmaceutical safety monitoring is transparent, accessible, and computationally rigorous — and where the professionals doing this critical work have the tools and community they deserve."
                        </p>
                    </div>
                </div>

                // Founder
                <div>
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// FOUNDER"</h3>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                        <h4 class="text-xl font-black font-mono text-white uppercase">"MATTHEW CAMPION, PHARMD"</h4>
                        <p class="mt-1 text-sm font-mono text-cyan-500">"Founder & Chief Executive"</p>
                        <p class="mt-4 text-sm text-slate-400 font-mono leading-relaxed">
                            "Clinical pharmacist turned systems architect. Former pharmacovigilance professional at Takeda, now building the independent infrastructure the industry needs. Combines deep PV domain expertise with full-stack Rust engineering to create tools that serve patients and professionals — not corporate interests."
                        </p>
                    </div>
                </div>

                // Technology
                <div>
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// TECHNOLOGY"</h3>
                    <div class="grid gap-4 sm:grid-cols-2 md:grid-cols-4">
                        <StatBlock number="143" label="Rust Crates"/>
                        <StatBlock number="75" label="Algorithms"/>
                        <StatBlock number="32" label="Original IP"/>
                        <StatBlock number="4,400+" label="Tests"/>
                    </div>
                </div>

                // Values
                <div>
                    <h3 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em] mb-6">"// CORE VALUES"</h3>
                    <div class="grid gap-3 sm:grid-cols-2 md:grid-cols-3">
                        <ValueItem label="Independence" desc="Freedom from corporate influence in safety analysis"/>
                        <ValueItem label="Patient Safety" desc="P0 priority — always the non-negotiable foundation"/>
                        <ValueItem label="Scientific Rigor" desc="Mathematical ground truth over opinion"/>
                        <ValueItem label="Transparency" desc="Open algorithms, auditable methods"/>
                        <ValueItem label="Professional Growth" desc="Empowering individuals alongside institutions"/>
                        <ValueItem label="Accessibility" desc="World-class tools available to all practitioners"/>
                        <ValueItem label="Community" desc="Collective intelligence amplifying individual capability"/>
                    </div>
                </div>
            </section>
        </div>
    }
}

#[component]
fn StatBlock(number: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
            <div class="text-3xl font-black font-mono text-cyan-400">{number}</div>
            <div class="mt-1 text-[10px] font-mono text-slate-500 uppercase tracking-widest">{label}</div>
        </div>
    }
}

#[component]
fn ValueItem(label: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-900/30 p-4">
            <h4 class="text-sm font-mono font-bold text-white uppercase">{label}</h4>
            <p class="mt-1 text-xs text-slate-500 font-mono">{desc}</p>
        </div>
    }
}
