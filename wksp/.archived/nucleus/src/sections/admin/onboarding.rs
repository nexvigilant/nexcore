//! Admin: Onboarding management — user flows, welcome sequences, feature tours

use leptos::prelude::*;

#[component]
pub fn OnboardingAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Onboarding Admin"</h1>
                <p class="mt-1 text-slate-400">"Configure user onboarding flows, welcome sequences, and feature tours"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <Stat label="Active Flows" value="3" sub="onboarding sequences"/>
                <Stat label="Completion Rate" value="—" sub="avg across flows"/>
                <Stat label="Avg Time" value="—" sub="to complete onboarding"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Onboarding Flows"</h2>
                <div class="space-y-4">
                    <FlowCard
                        name="New Member Welcome"
                        steps=5
                        desc="Profile setup, interest selection, circle recommendations, first post prompt, mentor match"
                        status="Active"
                    />
                    <FlowCard
                        name="Academy Enrollment"
                        steps=4
                        desc="Skill assessment, pathway recommendation, first course selection, learning goal setup"
                        status="Active"
                    />
                    <FlowCard
                        name="PV Professional"
                        steps=6
                        desc="Role verification, experience level, specialization, tool preferences, dashboard customization, signal setup"
                        status="Active"
                    />
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Feature Tours"</h2>
                <div class="grid gap-4 lg:grid-cols-2">
                    <TourCard name="Community Tour" screens=8 trigger="First community visit"/>
                    <TourCard name="Academy Tour" screens=6 trigger="First academy visit"/>
                    <TourCard name="Vigilance Tour" screens=10 trigger="First vigilance visit"/>
                    <TourCard name="Careers Tour" screens=5 trigger="First careers visit"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Welcome Emails"</h2>
                <div class="space-y-2">
                    <EmailRow name="Welcome Email" delay="Immediate" status="Active"/>
                    <EmailRow name="Getting Started Guide" delay="Day 1" status="Active"/>
                    <EmailRow name="Feature Highlights" delay="Day 3" status="Active"/>
                    <EmailRow name="Community Invitation" delay="Day 7" status="Active"/>
                    <EmailRow name="Progress Check-in" delay="Day 14" status="Active"/>
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
fn FlowCard(
    name: &'static str,
    steps: u32,
    desc: &'static str,
    status: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <h3 class="text-sm font-bold text-white">{name}</h3>
                    <span class="text-xs text-slate-500 font-mono">{format!("{} steps", steps)}</span>
                </div>
                <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
            </div>
            <p class="mt-2 text-sm text-slate-400">{desc}</p>
        </div>
    }
}

#[component]
fn TourCard(name: &'static str, screens: u32, trigger: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <h3 class="text-sm font-bold text-white">{name}</h3>
            <div class="mt-2 flex justify-between text-xs">
                <span class="text-slate-500">{format!("{} screens", screens)}</span>
                <span class="text-slate-400 font-mono">{trigger}</span>
            </div>
        </div>
    }
}

#[component]
fn EmailRow(name: &'static str, delay: &'static str, status: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <span class="text-sm text-white font-medium">{name}</span>
            </div>
            <div class="flex items-center gap-4">
                <span class="text-xs text-slate-500 font-mono">{delay}</span>
                <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
            </div>
        </div>
    }
}
