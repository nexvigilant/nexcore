//! Admin: Careers management — assessment tools, mentoring, career paths

use leptos::prelude::*;

#[component]
pub fn CareersAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Careers Admin"</h1>
                <p class="mt-1 text-slate-400">"Manage assessment tools, mentoring programs, and career development resources"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Assessment Tools" value="14" sub="active"/>
                <Stat label="Mentoring Pairs" value="0" sub="matched"/>
                <Stat label="Completions" value="0" sub="this month"/>
                <Stat label="Avg Score" value="—" sub="across tools"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Assessment Tools"</h2>
                <div class="space-y-2">
                    <ToolRow name="Competency Framework" status="Active" completions=0/>
                    <ToolRow name="Maturity Model" status="Active" completions=0/>
                    <ToolRow name="Value Proposition Canvas" status="Active" completions=0/>
                    <ToolRow name="Interview Preparation" status="Active" completions=0/>
                    <ToolRow name="Advisory Readiness" status="Active" completions=0/>
                    <ToolRow name="Board Competencies" status="Active" completions=0/>
                    <ToolRow name="Board Effectiveness" status="Active" completions=0/>
                    <ToolRow name="Change Readiness" status="Active" completions=0/>
                    <ToolRow name="Fellowship Evaluator" status="Active" completions=0/>
                    <ToolRow name="Hidden Job Market" status="Active" completions=0/>
                    <ToolRow name="Performance Conditions" status="Active" completions=0/>
                    <ToolRow name="Signal Decision Matrix" status="Active" completions=0/>
                    <ToolRow name="Startup Health Check" status="Active" completions=0/>
                    <ToolRow name="Assessment Hub" status="Active" completions=0/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Mentoring Program"</h2>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-center">
                    <p class="text-slate-400">"No mentoring pairs yet. Pairs are created when users request mentors through the careers section."</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: &'static str, sub: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</p>
            <p class="mt-2 text-3xl font-bold font-mono text-cyan-400">{value}</p>
            <p class="mt-1 text-xs text-slate-500">{sub}</p>
        </div>
    }
}

#[component]
fn ToolRow(name: &'static str, status: &'static str, completions: u32) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <div class="h-2 w-2 rounded-full bg-emerald-400"></div>
                <span class="text-sm text-white font-medium">{name}</span>
            </div>
            <div class="flex items-center gap-6">
                <span class="text-xs text-slate-500 font-mono">{format!("{} completions", completions)}</span>
                <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
            </div>
        </div>
    }
}
