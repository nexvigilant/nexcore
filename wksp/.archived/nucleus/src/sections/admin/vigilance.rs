//! Admin: Vigilance management — PV system config, signal thresholds, Guardian

use leptos::prelude::*;

#[component]
pub fn VigilanceAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Vigilance Admin"</h1>
                <p class="mt-1 text-slate-400">"Configure pharmacovigilance system, signal thresholds, and Guardian engine"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Active Signals" value="0" color="text-amber-400"/>
                <Stat label="Guardian Status" value="LIVE" color="text-emerald-400"/>
                <Stat label="Threshold Alerts" value="0" color="text-red-400"/>
                <Stat label="PVDSL Rules" value="0" color="text-violet-400"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Signal Detection Thresholds"</h2>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden">
                    <table class="w-full text-sm">
                        <thead>
                            <tr class="border-b border-slate-800">
                                <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Metric"</th>
                                <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Default"</th>
                                <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Strict"</th>
                                <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Sensitive"</th>
                                <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Active"</th>
                            </tr>
                        </thead>
                        <tbody class="text-slate-300 font-mono">
                            <ThresholdRow metric="PRR" default_v=">= 2.0" strict=">= 3.0" sensitive=">= 1.5" active="Default"/>
                            <ThresholdRow metric="Chi-sq" default_v=">= 3.841" strict=">= 6.635" sensitive=">= 2.706" active="Default"/>
                            <ThresholdRow metric="ROR lower CI" default_v="> 1.0" strict="> 2.0" sensitive="> 1.0" active="Default"/>
                            <ThresholdRow metric="IC025" default_v="> 0" strict="> 1.0" sensitive="> -0.5" active="Default"/>
                            <ThresholdRow metric="EB05" default_v=">= 2.0" strict=">= 3.0" sensitive=">= 1.5" active="Default"/>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="grid gap-6 lg:grid-cols-2">
                <div>
                    <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Guardian Engine"</h2>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 space-y-3">
                        <ConfigRow label="Homeostasis Loop" value="Active"/>
                        <ConfigRow label="Sensors" value="12 registered"/>
                        <ConfigRow label="Actuators" value="3 (audit-log, cytokine, response)"/>
                        <ConfigRow label="Threat Level" value="Normal"/>
                        <ConfigRow label="Iterations" value="0"/>
                    </div>
                </div>
                <div>
                    <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"PVDSL Engine"</h2>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 space-y-3">
                        <ConfigRow label="Compiled Rules" value="0"/>
                        <ConfigRow label="Active Pipelines" value="0"/>
                        <ConfigRow label="Last Evaluation" value="Never"/>
                        <ConfigRow label="Error Rate" value="0%"/>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: &'static str, color: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</p>
            <p class=format!("mt-2 text-3xl font-bold font-mono {color}")>{value}</p>
        </div>
    }
}

#[component]
fn ThresholdRow(
    metric: &'static str,
    default_v: &'static str,
    strict: &'static str,
    sensitive: &'static str,
    active: &'static str,
) -> impl IntoView {
    view! {
        <tr class="border-b border-slate-800/50">
            <td class="p-4 text-white font-medium">{metric}</td>
            <td class="p-4">{default_v}</td>
            <td class="p-4">{strict}</td>
            <td class="p-4">{sensitive}</td>
            <td class="p-4">
                <span class="rounded bg-cyan-500/10 px-2 py-0.5 text-[10px] text-cyan-400 uppercase">{active}</span>
            </td>
        </tr>
    }
}

#[component]
fn ConfigRow(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-2 border-b border-slate-800/30 last:border-0">
            <span class="text-sm text-slate-400">{label}</span>
            <span class="text-sm text-white font-mono">{value}</span>
        </div>
    }
}
