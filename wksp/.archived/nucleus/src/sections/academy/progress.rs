//! Progress tracking — learning analytics

use leptos::prelude::*;

#[component]
pub fn ProgressPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Learning Progress"</h1>
            <p class="mt-2 text-slate-400">"Track your capability development across all domains"</p>

            <div class="mt-8 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
                <ProgressStat label="Overall" value="0%" color="cyan"/>
                <ProgressStat label="KSBs Assessed" value="0 / 1,462" color="emerald"/>
                <ProgressStat label="EPAs Completed" value="0 / 21" color="amber"/>
                <ProgressStat label="Study Streak" value="0 days" color="violet"/>
            </div>

            <div class="mt-12">
                <h2 class="text-xl font-semibold text-white">"Domain Progress"</h2>
                <div class="mt-4 space-y-3">
                    <DomainProgress code="D01" name="PV Foundations" ksbs=85 completed=0/>
                    <DomainProgress code="D02" name="Adverse Event Reporting" ksbs=92 completed=0/>
                    <DomainProgress code="D03" name="Case Processing" ksbs=110 completed=0/>
                    <DomainProgress code="D04" name="Literature Monitoring" ksbs=78 completed=0/>
                    <DomainProgress code="D05" name="Case Assessment" ksbs=120 completed=0/>
                    <DomainProgress code="D06" name="Aggregate Reporting" ksbs=95 completed=0/>
                    <DomainProgress code="D07" name="Risk Management" ksbs=105 completed=0/>
                    <DomainProgress code="D08" name="Signal Detection" ksbs=115 completed=0/>
                    <DomainProgress code="D09" name="Signal Evaluation" ksbs=98 completed=0/>
                    <DomainProgress code="D10" name="Benefit-Risk Assessment" ksbs=110 completed=0/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProgressStat(label: &'static str, value: &'static str, color: &'static str) -> impl IntoView {
    let text_class = format!("text-2xl font-bold text-{color}-400");
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-xs font-medium uppercase tracking-wider text-slate-500">{label}</p>
            <p class=text_class>{value}</p>
        </div>
    }
}

#[component]
fn DomainProgress(code: &'static str, name: &'static str, ksbs: u32, completed: u32) -> impl IntoView {
    let pct = if ksbs > 0 { (completed * 100) / ksbs } else { 0 };
    let width = format!("width: {pct}%");

    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-4">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <span class="rounded bg-cyan-500/10 px-2 py-0.5 text-xs font-medium text-cyan-400">{code}</span>
                    <span class="text-sm font-medium text-white">{name}</span>
                </div>
                <span class="text-xs text-slate-500">{format!("{completed}/{ksbs} KSBs")}</span>
            </div>
            <div class="mt-2 h-1.5 rounded-full bg-slate-800">
                <div class="h-full rounded-full bg-cyan-500 transition-all" style=width></div>
            </div>
        </div>
    }
}
