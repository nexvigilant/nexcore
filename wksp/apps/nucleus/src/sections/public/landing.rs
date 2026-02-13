//! Landing page — primary marketing page

use leptos::prelude::*;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-slate-950 selection:bg-cyan-500/30 selection:text-cyan-200 overflow-hidden">
            // Hero section with high-tech visual effects
            <section class="relative min-h-[90vh] flex flex-col items-center justify-center px-6 py-24 text-center overflow-hidden">
                // Background visual elements
                <div class="absolute inset-0 z-0">
                    <div class="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-cyan-600/10 rounded-full blur-[120px] animate-pulse"></div>
                    <div class="absolute bottom-1/4 left-1/3 -translate-x-1/2 w-[600px] h-[600px] bg-violet-600/5 rounded-full blur-[100px]"></div>
                </div>

                // High-tech decorative grid
                <div class="absolute inset-0 z-[1] opacity-[0.05] bg-[linear-gradient(to_right,#808080_1px,transparent_1px),linear-gradient(to_bottom,#808080_1px,transparent_1px)] bg-[size:40px_40px]"></div>

                <div class="relative z-10 max-w-5xl mx-auto">
                    <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-cyan-500/30 bg-cyan-500/5 mb-8 animate-bounce">
                        <span class="h-1.5 w-1.5 rounded-full bg-cyan-400"></span>
                        <span class="text-[10px] font-mono font-black text-cyan-400 uppercase tracking-[0.2em]">"System Version 0.2.0 Active"</span>
                    </div>

                    <h1 class="text-6xl md:text-8xl font-black text-white font-mono tracking-tighter uppercase leading-[0.9]">
                        "EMPOWERMENT" <br/>
                        <span class="text-transparent bg-clip-text bg-gradient-to-r from-cyan-400 to-blue-500">"THROUGH VIGILANCE"</span>
                    </h1>

                    <p class="mt-8 max-w-2xl mx-auto text-lg md:text-xl text-slate-400 font-mono leading-relaxed">
                        "THE INDEPENDENT OPERATING SYSTEM FOR PHARMACEUTICAL INTELLIGENCE AND PROFESSIONAL AUTONOMY."
                    </p>

                    <div class="mt-12 flex flex-col sm:flex-row gap-6 justify-center">
                        <a href="/signup" class="group relative px-10 py-4 bg-cyan-600 text-white font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-cyan-500 shadow-[0_0_20px_rgba(34,211,238,0.3)]">
                            <span class="relative z-10">"INITIALIZE SESSION"</span>
                            <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300"></div>
                        </a>
                        <a href="/about" class="px-10 py-4 border border-slate-700 text-slate-300 font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-slate-900 hover:border-slate-500">
                            "READ PROTOCOLS"
                        </a>
                    </div>
                </div>

                // Bottom decorative line
                <div class="absolute bottom-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-slate-800 to-transparent"></div>
            </section>

            // Pillars section with high-fidelity glass cards
            <section class="relative mx-auto max-w-6xl px-6 py-32">
                <div class="text-center mb-20">
                    <h2 class="text-xs font-mono font-bold text-cyan-500 uppercase tracking-[0.4em] mb-4">"// CORE ARCHITECTURE"</h2>
                    <h3 class="text-4xl font-black text-white font-mono uppercase tracking-tighter">"TWO INSEPARABLE PILLARS"</h3>
                </div>

                <div class="grid gap-12 md:grid-cols-2">
                    <PillarCard 
                        title="VIGILANCE" 
                        desc="INDEPENDENT PHARMACEUTICAL OVERSIGHT PROTECTING PATIENTS THROUGH REAL-TIME SURVEILLANCE, CAUSALITY INVESTIGATION, AND SYSTEMIC ACCOUNTABILITY."
                        color="cyan"
                        symbol="σ"
                    />
                    <PillarCard 
                        title="EMPOWERMENT" 
                        desc="CAREER INFRASTRUCTURE, COGNITIVE TOOLS, AND A GLOBAL COMMUNITY ENABLING PROFESSIONALS TO ADVANCE WHILE ADVOCATING FOR PATIENT SAFETY."
                        color="amber"
                        symbol="π"
                    />
                </div>
            </section>

            // Ecosystem section
            <section class="mx-auto max-w-7xl px-6 py-32 border-t border-slate-900">
                <div class="flex flex-col md:flex-row justify-between items-end gap-8 mb-16">
                    <div class="max-w-xl">
                        <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.4em] mb-4">"// THE ECOSYSTEM"</h2>
                        <h3 class="text-4xl font-black text-white font-mono uppercase tracking-tighter">"MODULAR CAPABILITY LAYERS"</h3>
                    </div>
                    <p class="text-slate-500 font-mono text-sm max-w-xs leading-relaxed uppercase">
                        "EACH COMPONENT IS GROUNDED TO LEX PRIMITIVA, ENSURING CROSS-DOMAIN KNOWLEDGE TRANSFER."
                    </p>
                </div>

                <div class="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
                    <ComponentCard name="Academy" symbol="λ" desc="SKILLS-BASED TRAINING AND GLOBAL CERTIFICATION." color="emerald"/>
                    <ComponentCard name="Community" symbol="ς" desc="PROFESSIONAL NETWORKING AND DEEP COLLABORATION." color="violet"/>
                    <ComponentCard name="Careers" symbol="κ" desc="JOB ARCHITECTURE AND CAREER ADVANCEMENT TOOLS." color="amber"/>
                    <ComponentCard name="Patrol" symbol="∂" desc="REAL-TIME SAFETY MONITORING AND SIGNAL DETECTION." color="cyan"/>
                </div>
            </section>
        </div>
    }
}

#[component]
fn PillarCard(
    title: &'static str,
    desc: &'static str,
    color: &'static str,
    symbol: &'static str,
) -> impl IntoView {
    let accent_color = match color {
        "cyan" => "text-cyan-400 border-cyan-500/20 shadow-cyan-900/10",
        "amber" => "text-amber-400 border-amber-500/20 shadow-amber-900/10",
        _ => "text-slate-400 border-slate-800 shadow-transparent",
    };

    view! {
        <div class=format!("glass-panel p-10 rounded-3xl border transition-all hover:scale-[1.02] group relative overflow-hidden {}", accent_color)>
            // Large background symbol
            <div class="absolute -bottom-8 -right-8 text-[120px] font-mono font-black opacity-5 group-hover:opacity-10 transition-opacity">
                {symbol}
            </div>

            <h3 class="text-3xl font-black font-mono tracking-tighter uppercase mb-6">{title}</h3>
            <p class="text-slate-400 font-mono leading-relaxed text-sm tracking-wide">
                {desc}
            </p>
            
            <div class="mt-12 flex items-center gap-3">
                <span class="h-px flex-1 bg-slate-800"></span>
                <span class="text-[10px] font-mono font-bold text-slate-600 tracking-widest uppercase">"SECURED BY NUCLEUS"</span>
            </div>
        </div>
    }
}

#[component]
fn ComponentCard(
    name: &'static str,
    symbol: &'static str,
    desc: &'static str,
    color: &'static str,
) -> impl IntoView {
    let hover_border = match color {
        "emerald" => "hover:border-emerald-500/30 hover:shadow-emerald-950/20",
        "violet" => "hover:border-violet-500/30 hover:shadow-violet-950/20",
        "amber" => "hover:border-amber-500/30 hover:shadow-amber-950/20",
        "cyan" => "hover:border-cyan-500/30 hover:shadow-cyan-950/20",
        _ => "hover:border-slate-700",
    };

    view! {
        <div class=format!("glass-panel p-8 rounded-2xl border border-slate-800/50 transition-all hover:-translate-y-1 shadow-xl {}", hover_border)>
            <div class="flex items-center gap-3 mb-6">
                <div class="h-10 w-10 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center">
                    <span class="text-white font-mono font-black text-xl">{symbol}</span>
                </div>
                <h3 class="text-xl font-black text-white font-mono tracking-tighter uppercase">{name}</h3>
            </div>
            <p class="text-xs text-slate-500 font-mono leading-relaxed uppercase tracking-wider">
                {desc}
            </p>
        </div>
    }
}