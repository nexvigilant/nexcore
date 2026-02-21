//! Regulatory overview — framework comparison and resources

use leptos::prelude::*;

#[component]
pub fn OverviewPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Regulatory Frameworks"</h1>
            <p class="mt-2 text-slate-400">"Compare pharmacovigilance regulations across major authorities."</p>

            <div class="mt-8 grid gap-4 md:grid-cols-3">
                <FrameworkCard
                    authority="FDA"
                    region="United States"
                    key_regs="21 CFR 312/314, FAERS, MedWatch"
                />
                <FrameworkCard
                    authority="EMA"
                    region="European Union"
                    key_regs="GVP Modules, EudraVigilance, PRAC"
                />
                <FrameworkCard
                    authority="ICH"
                    region="International"
                    key_regs="ICH E2A-E2F, MedDRA, CIOMS"
                />
            </div>
        </div>
    }
}

#[component]
fn FrameworkCard(authority: &'static str, region: &'static str, key_regs: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-colors">
            <h3 class="text-lg font-bold text-cyan-400">{authority}</h3>
            <p class="text-sm text-slate-500">{region}</p>
            <p class="mt-3 text-sm text-slate-300">{key_regs}</p>
        </div>
    }
}
