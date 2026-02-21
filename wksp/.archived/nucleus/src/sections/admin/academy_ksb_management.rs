//! Admin: KSB Management — browse, search, and manage all KSBs

use leptos::prelude::*;

#[component]
pub fn AcademyKsbManagementPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"KSB Management"</h1>
                    <p class="mt-1 text-slate-400">"Browse, search, and manage all Knowledge, Skills, and Behaviours"</p>
                </div>
                <a href="/admin/academy/ksb-builder" class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">"New KSB"</a>
            </div>
            <div class="flex gap-4">
                <input type="text" class="flex-1 bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50" placeholder="Search KSBs..."/>
                <select class="bg-slate-900/50 border border-slate-700/50 rounded-lg px-4 py-2 text-white focus:outline-none focus:border-cyan-500/50">
                    <option>"All Types"</option>
                    <option>"Knowledge"</option>
                    <option>"Skills"</option>
                    <option>"Behaviours"</option>
                </select>
            </div>
            <div class="grid gap-4 sm:grid-cols-3">
                <Stat label="Knowledge" value="52"/>
                <Stat label="Skills" value="64"/>
                <Stat label="Behaviours" value="40"/>
            </div>
            <div class="rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="border-b border-slate-800">
                            <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"ID"</th>
                            <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Type"</th>
                            <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Title"</th>
                            <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Area"</th>
                            <th class="text-left text-xs font-bold uppercase tracking-widest text-slate-500 p-4">"Level"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        <KsbRow id="K001" ksb_type="Knowledge" title="PV regulatory framework" area="Regulatory" level="Understand"/>
                        <KsbRow id="K002" ksb_type="Knowledge" title="Signal detection methods" area="Signal Mgmt" level="Analyze"/>
                        <KsbRow id="S001" ksb_type="Skill" title="Run PRR analysis" area="Signal Mgmt" level="Apply"/>
                        <KsbRow id="S002" ksb_type="Skill" title="Draft ICSR narrative" area="Case Processing" level="Create"/>
                        <KsbRow id="B001" ksb_type="Behaviour" title="Escalate critical findings" area="Quality" level="Evaluate"/>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: &'static str) -> impl IntoView {
    view! { <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5"><p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</p><p class="mt-2 text-3xl font-bold font-mono text-cyan-400">{value}</p></div> }
}
#[component]
fn KsbRow(
    id: &'static str,
    ksb_type: &'static str,
    title: &'static str,
    area: &'static str,
    level: &'static str,
) -> impl IntoView {
    let color = match ksb_type {
        "Knowledge" => "text-blue-400 bg-blue-500/10",
        "Skill" => "text-emerald-400 bg-emerald-500/10",
        _ => "text-violet-400 bg-violet-500/10",
    };
    view! {
        <tr class="border-b border-slate-800/50 hover:bg-slate-800/30">
            <td class="p-4 font-mono text-xs text-slate-500">{id}</td>
            <td class="p-4"><span class={format!("px-2 py-0.5 rounded text-[10px] font-mono uppercase {}", color)}>{ksb_type}</span></td>
            <td class="p-4 text-white">{title}</td>
            <td class="p-4 text-slate-400">{area}</td>
            <td class="p-4 text-slate-400">{level}</td>
        </tr>
    }
}
