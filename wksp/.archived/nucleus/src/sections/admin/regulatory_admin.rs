//! Admin: Regulatory management — guideline sources, framework config, compliance tracking

use leptos::prelude::*;

#[component]
pub fn RegulatoryAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Regulatory Admin"</h1>
                <p class="mt-1 text-slate-400">"Manage guideline sources, regulatory frameworks, and compliance tracking"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Guidelines" value="894+" sub="ICH terms indexed"/>
                <Stat label="Frameworks" value="3" sub="ICH, CIOMS, EMA"/>
                <Stat label="Updates" value="0" sub="pending review"/>
                <Stat label="Compliance" value="—" sub="score"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Guideline Sources"</h2>
                <div class="space-y-2">
                    <SourceRow name="ICH Guidelines" count=894 status="Synced" last_sync="Auto-indexed"/>
                    <SourceRow name="CIOMS Working Groups" count=12 status="Active" last_sync="Manual"/>
                    <SourceRow name="EMA Guidance Documents" count=45 status="Active" last_sync="Manual"/>
                    <SourceRow name="FDA Guidance (CDER)" count=38 status="Active" last_sync="Manual"/>
                    <SourceRow name="WHO-UMC Resources" count=8 status="Active" last_sync="Manual"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Regulatory Frameworks"</h2>
                <div class="grid gap-4 lg:grid-cols-3">
                    <FrameworkCard
                        name="ICH"
                        desc="International Council for Harmonisation — E2A through E2F, M1-M13"
                        modules=24
                    />
                    <FrameworkCard
                        name="CIOMS"
                        desc="Council for International Organizations of Medical Sciences — Working Groups I-XIII"
                        modules=13
                    />
                    <FrameworkCard
                        name="EMA"
                        desc="European Medicines Agency — GVP Modules I-XVI, Annexes"
                        modules=16
                    />
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Compliance Tracking"</h2>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-center">
                    <p class="text-slate-400">"Compliance tracking activates when users complete regulatory assessments. Track adherence to ICH E2E pharmacovigilance planning requirements."</p>
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
fn SourceRow(
    name: &'static str,
    count: u32,
    status: &'static str,
    last_sync: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <div class="h-2 w-2 rounded-full bg-emerald-400"></div>
                <span class="text-sm text-white font-medium">{name}</span>
                <span class="text-xs text-slate-500 font-mono">{format!("{} items", count)}</span>
            </div>
            <div class="flex items-center gap-4">
                <span class="text-xs text-slate-500">{last_sync}</span>
                <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
            </div>
        </div>
    }
}

#[component]
fn FrameworkCard(name: &'static str, desc: &'static str, modules: u32) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <h3 class="text-lg font-bold text-white font-mono">{name}</h3>
            <p class="mt-2 text-sm text-slate-400 leading-relaxed">{desc}</p>
            <p class="mt-3 text-xs text-slate-500 font-mono">{format!("{} modules/sections", modules)}</p>
        </div>
    }
}
