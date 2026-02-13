//! Admin: Academy management — courses, KSBs, EPAs, learners, analytics

use leptos::prelude::*;

#[component]
pub fn AcademyAdminPage() -> impl IntoView {
    let active_tab = RwSignal::new("courses");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Academy Admin"</h1>
                    <p class="mt-1 text-slate-400">"Manage courses, KSBs, EPAs, and learner progress"</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors">"\u{2190} Dashboard"</a>
            </div>

            // Tab navigation
            <div class="mt-6 flex gap-4 overflow-x-auto border-b border-slate-800 pb-4">
                {["courses", "ksbs", "epas", "learners", "analytics", "framework"].into_iter().map(|tab| {
                    view! {
                        <button
                            class=move || { if active_tab.get() == tab {
                                "whitespace-nowrap text-sm font-medium text-amber-400 border-b-2 border-amber-400 pb-1"
                            } else {
                                "whitespace-nowrap text-sm text-slate-400 hover:text-white transition-colors"
                            }}
                            on:click=move |_| active_tab.set(tab)
                        >{tab.replace("_", " ").to_uppercase()}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Courses tab
            <Show when=move || active_tab.get() == "courses">
                <CoursesAdmin/>
            </Show>
            <Show when=move || active_tab.get() == "ksbs">
                <KsbsAdmin/>
            </Show>
            <Show when=move || active_tab.get() == "epas">
                <EpasAdmin/>
            </Show>
            <Show when=move || active_tab.get() == "learners">
                <LearnersAdmin/>
            </Show>
            <Show when=move || active_tab.get() == "analytics">
                <AcademyAnalytics/>
            </Show>
            <Show when=move || active_tab.get() == "framework">
                <FrameworkAdmin/>
            </Show>
        </div>
    }
}

#[component]
fn CoursesAdmin() -> impl IntoView {
    view! {
        <div class="mt-6">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-semibold text-white">"Capability Pathways"</h2>
                <button class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500">"+ New Pathway"</button>
            </div>
            <div class="mt-4 space-y-3">
                {["D01: PV Foundations", "D02: AE Reporting", "D03: Case Processing",
                  "D04: Literature Surveillance", "D05: Case Assessment",
                  "D06: Aggregate Reporting", "D07: Risk Management",
                  "D08: Signal Detection", "D09: Signal Evaluation"].into_iter().map(|course| {
                    view! {
                        <div class="flex items-center justify-between rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                            <div>
                                <p class="font-medium text-white">{course}</p>
                                <p class="text-xs text-slate-500">"0 enrolled \u{00B7} 0 modules"</p>
                            </div>
                            <div class="flex gap-2">
                                <button class="text-xs text-amber-400 hover:text-amber-300">"Edit"</button>
                                <button class="text-xs text-slate-400 hover:text-white">"View"</button>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn KsbsAdmin() -> impl IntoView {
    view! {
        <div class="mt-6">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-semibold text-white">"KSB Registry (1,462 KSBs)"</h2>
                <div class="flex gap-2">
                    <input type="search" placeholder="Search KSBs..."
                        class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-white placeholder:text-slate-500 focus:border-amber-500 focus:outline-none"/>
                    <button class="rounded-lg bg-amber-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-amber-500">"+ Add KSB"</button>
                </div>
            </div>
            <div class="mt-4 rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-center">
                <p class="text-slate-400">"1,462 KSBs across 15 domains"</p>
                <p class="mt-2 text-sm text-slate-500">"Use search or domain filter to browse. Each KSB maps to Bloom/Dreyfus levels."</p>
            </div>
        </div>
    }
}

#[component]
fn EpasAdmin() -> impl IntoView {
    view! {
        <div class="mt-6">
            <h2 class="text-lg font-semibold text-white">"Entrustable Professional Activities (21 EPAs)"</h2>
            <div class="mt-4 grid gap-3 sm:grid-cols-2">
                {(1..=10).map(|i| {
                    let name = match i {
                        1 => "Process ICSRs",
                        2 => "Assess causality",
                        3 => "Detect signals",
                        4 => "Write aggregate reports",
                        5 => "Conduct literature surveillance",
                        6 => "Manage risk minimization",
                        7 => "Evaluate benefit-risk",
                        8 => "Support audits & inspections",
                        9 => "Train PV team members",
                        10 => "Manage PV systems",
                        _ => "",
                    };
                    view! {
                        <div class="rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                            <p class="text-sm font-medium text-white">{format!("EPA-{i:02}: {name}")}</p>
                            <p class="text-xs text-slate-500">"0 learners enrolled"</p>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn LearnersAdmin() -> impl IntoView {
    view! {
        <div class="mt-6">
            <div class="flex items-center justify-between">
                <h2 class="text-lg font-semibold text-white">"Learners"</h2>
                <input type="search" placeholder="Search learners..."
                    class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-white placeholder:text-slate-500 focus:border-amber-500 focus:outline-none"/>
            </div>
            <div class="mt-4 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                <p class="text-slate-400">"No learners enrolled yet"</p>
                <p class="mt-2 text-sm text-slate-500">"Learners will appear here once they enroll in capability pathways."</p>
            </div>
        </div>
    }
}

#[component]
fn AcademyAnalytics() -> impl IntoView {
    view! {
        <div class="mt-6">
            <h2 class="text-lg font-semibold text-white">"Academy Analytics"</h2>
            <div class="mt-4 grid gap-4 sm:grid-cols-3">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                    <p class="text-xs text-slate-500">"Completion Rate"</p>
                    <p class="mt-2 text-3xl font-bold text-emerald-400">"0%"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                    <p class="text-xs text-slate-500">"Avg. Time to Complete"</p>
                    <p class="mt-2 text-3xl font-bold text-cyan-400">"\u{2014}"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 text-center">
                    <p class="text-xs text-slate-500">"Certificates Issued"</p>
                    <p class="mt-2 text-3xl font-bold text-amber-400">"0"</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn FrameworkAdmin() -> impl IntoView {
    view! {
        <div class="mt-6">
            <h2 class="text-lg font-semibold text-white">"PV Competency Framework"</h2>
            <p class="mt-2 text-sm text-slate-400">"15 domains, 1,462 KSBs, 21 EPAs, Bloom/Dreyfus levels"</p>
            <div class="mt-4 grid gap-3 sm:grid-cols-3">
                {["D01: Foundations", "D02: AE Reporting", "D03: Case Processing",
                  "D04: Literature", "D05: Case Assessment", "D06: Aggregate",
                  "D07: Risk Management", "D08: Signal Detection", "D09: Signal Eval",
                  "D10: Benefit-Risk", "D11: Technology", "D12: Quality",
                  "D13: Leadership", "D14: Communication", "D15: Regulation"].into_iter().map(|domain| {
                    view! {
                        <div class="rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3 hover:border-slate-700 transition-colors cursor-pointer">
                            <p class="text-sm font-medium text-white">{domain}</p>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
