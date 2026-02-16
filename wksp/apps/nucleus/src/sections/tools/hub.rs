//! Tools Hub — access engineering capability accelerators

use super::code_gen::CodeGenStudio;
use leptos::prelude::*;

#[component]
pub fn HubPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <div class="flex items-center gap-4 mb-4">
                    <div class="p-3 rounded-xl bg-cyan-500/10 border border-cyan-500/20">
                        <span class="text-3xl text-cyan-400">"🛠"</span>
                    </div>
                    <div>
                        <h1 class="text-4xl md:text-5xl font-extrabold font-mono text-white uppercase tracking-tight">
                            "Engineering Studio"
                        </h1>
                        <p class="text-slate-400 font-medium max-w-2xl">
                            "Capability accelerators for automated code generation, AI-assisted debugging, and performance optimization."
                        </p>
                    </div>
                </div>
            </header>

            <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
                <ToolCard
                    title="Code Generator"
                    icon="λ"
                    desc="Automated generation of Rust boilerplate, API clients, and PVDSL logic."
                    href="/tools/codegen"
                    color="text-amber-400"
                />
                <ToolCard
                    title="Debug Assistant"
                    icon="∂"
                    desc="AI-powered analysis of stack traces, logs, and logical anomalies."
                    href="/tools/debug"
                    color="text-red-400"
                />
                <ToolCard
                    title="Performance Analyzer"
                    icon="ν"
                    desc="Optimization analysis for async workflows and high-throughput data pipelines."
                    href="/tools/perf"
                    color="text-emerald-400"
                />
                <ToolCard
                    title="Brain Storage"
                    icon="ρ"
                    desc="Working memory and persistent artifacts from AI sessions."
                    href="/tools/storage"
                    color="text-gold"
                />
                <ToolCard
                    title="API Explorer"
                    icon="μ"
                    desc="Interactive documentation and testing for NexCore telemetry endpoints."
                    href="/tools/api-explorer"
                    color="text-cyan-400"
                />
                <ToolCard
                    title="Architecture Visualizer"
                    icon="κ"
                    desc="Decompose system designs into fundamental primitives (Lex Primitiva)."
                    href="/tools/visualizer"
                    color="text-violet-400"
                />
                <ToolCard
                    title="Primitive Forge"
                    icon="σ"
                    desc="Game-theory roguelike for symbol collection and code forging."
                    href="/forge"
                    color="text-amber-500"
                />
                <ToolCard
                    title="Registry HUD"
                    icon="ς"
                    desc="Monitor Kellogg registry status and crate lifecycle events."
                    href="/tools/registry"
                    color="text-slate-400"
                />
            </div>

            <section class="mt-14">
                <div class="mb-5 flex items-end justify-between gap-4">
                    <div>
                        <h2 class="text-2xl md:text-3xl font-extrabold font-mono text-white uppercase tracking-tight">
                            "Embedded Code Studio"
                        </h2>
                        <p class="text-slate-400 text-sm">
                            "Generate and iterate directly here without leaving the Engineering Studio."
                        </p>
                    </div>
                    <a
                        href="/tools/codegen"
                        class="text-[10px] font-bold font-mono uppercase tracking-[0.2em] text-cyan-400 hover:text-cyan-300 transition-colors"
                    >
                        "Open Dedicated View →"
                    </a>
                </div>

                <div class="rounded-2xl border border-slate-800 bg-slate-900/30 p-4 md:p-6">
                    <CodeGenStudio/>
                </div>
            </section>

            <div class="mt-16 rounded-3xl border border-slate-800 bg-slate-900/30 p-10 flex flex-col items-center text-center">
                <div class="h-12 w-12 rounded-full bg-cyan-500/10 flex items-center justify-center text-cyan-400 text-xl font-mono mb-6">
                    "ρ"
                </div>
                <h3 class="text-2xl font-bold text-white mb-4">"Grounding to Lex Primitiva"</h3>
                <p class="text-slate-400 max-w-2xl leading-relaxed">
                    "Every tool in the Engineering Studio is designed to reduce entropy and ensure that our software
                    architectures remain grounded to the 16 Lex Primitiva symbols."
                </p>
            </div>
        </div>
    }
}

#[component]
fn ToolCard(
    title: &'static str,
    icon: &'static str,
    desc: &'static str,
    href: &'static str,
    color: &'static str,
) -> impl IntoView {
    view! {
        <a href=href class="glass-panel p-8 rounded-2xl border border-slate-800 hover:border-cyan-500/30 transition-all group flex flex-col h-full">
            <div class=format!("text-3xl font-mono mb-6 {color} group-hover:scale-110 transition-transform w-fit")>
                {icon}
            </div>
            <h3 class="text-xl font-bold text-white group-hover:text-cyan-400 transition-colors mb-3">
                {title}
            </h3>
            <p class="text-sm text-slate-400 leading-relaxed">
                {desc}
            </p>
            <div class="mt-auto pt-8 flex items-center text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono group-hover:text-cyan-400 transition-colors">
                "Launch tool" <span class="ml-2 group-hover:translate-x-1 transition-transform">"→"</span>
            </div>
        </a>
    }
}
