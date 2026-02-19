//! Admin: Affiliate program applications

use leptos::prelude::*;

struct AffiliateApp {
    date: &'static str,
    name: &'static str,
    org: &'static str,
    email: &'static str,
    reach: &'static str,
    status: &'static str,
    channel: &'static str,
}

const APPLICATIONS: &[AffiliateApp] = &[
    AffiliateApp {
        date: "2026-02-15",
        name: "Dr. Rebecca Torres",
        org: "PV Academy Online",
        email: "r.torres@pvacademy.com",
        reach: "12,000",
        status: "Pending",
        channel: "Blog",
    },
    AffiliateApp {
        date: "2026-02-14",
        name: "Michael Chen",
        org: "PharmEd Institute",
        email: "m.chen@pharmed.edu",
        reach: "45,000",
        status: "Pending",
        channel: "LMS",
    },
    AffiliateApp {
        date: "2026-02-13",
        name: "Anna Bergstrom",
        org: "Nordic Safety Training",
        email: "a.berg@nordsafe.se",
        reach: "8,500",
        status: "Pending",
        channel: "Newsletter",
    },
    AffiliateApp {
        date: "2026-02-12",
        name: "Dr. Kwame Asante",
        org: "African PV Network",
        email: "k.asante@afripv.org",
        reach: "22,000",
        status: "Approved",
        channel: "Community",
    },
    AffiliateApp {
        date: "2026-02-10",
        name: "Laura Rossi",
        org: "EuroVigilance Training",
        email: "l.rossi@eurovigtrain.eu",
        reach: "35,000",
        status: "Approved",
        channel: "Webinars",
    },
    AffiliateApp {
        date: "2026-02-08",
        name: "James Cooper",
        org: "PharmaCompliance UK",
        email: "j.cooper@pharmacomp.uk",
        reach: "5,200",
        status: "Rejected",
        channel: "Blog",
    },
];

#[component]
pub fn AffiliateApplicationsPage() -> impl IntoView {
    let active_tab = RwSignal::new("all");
    let pending = APPLICATIONS
        .iter()
        .filter(|a| a.status == "Pending")
        .count();
    let approved = APPLICATIONS
        .iter()
        .filter(|a| a.status == "Approved")
        .count();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Affiliate Applications"</h1>
                    <p class="mt-1 text-slate-400">"Review and manage affiliate program applications."</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} Dashboard"</a>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-3">
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Pending"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{pending.to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Approved"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{approved.to_string()}</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Total Applications"</p>
                    <p class="text-2xl font-black text-white font-mono mt-2">{APPLICATIONS.len().to_string()}</p>
                </div>
            </div>

            /* Tabs */
            <div class="mt-8 flex gap-2 border-b border-slate-800 pb-3">
                {["all", "pending", "approved", "rejected"].into_iter().map(|tab| {
                    view! {
                        <button
                            class=move || if active_tab.get() == tab {
                                "rounded-lg px-3 py-1.5 text-[10px] font-bold text-amber-400 bg-amber-500/10 font-mono uppercase tracking-widest"
                            } else {
                                "rounded-lg px-3 py-1.5 text-[10px] text-slate-500 hover:text-white font-mono uppercase tracking-widest transition-colors"
                            }
                            on:click=move |_| active_tab.set(tab)
                        >{tab}</button>
                    }
                }).collect_view()}
            </div>

            /* Applications table */
            <div class="mt-6 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Date"</th>
                            <th class="px-4 py-3">"Applicant"</th>
                            <th class="px-4 py-3">"Channel"</th>
                            <th class="px-4 py-3 text-right">"Reach"</th>
                            <th class="px-4 py-3">"Status"</th>
                            <th class="px-4 py-3">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {APPLICATIONS.iter().map(|a| {
                            let status_cls = match a.status {
                                "Pending" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                                "Approved" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                                "Rejected" => "text-red-400 bg-red-500/10 border-red-500/20",
                                _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                            };
                            view! {
                                <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono">{a.date}</td>
                                    <td class="px-4 py-3">
                                        <p class="text-sm font-medium text-white">{a.name}</p>
                                        <p class="text-[10px] text-slate-500 font-mono">{a.org}</p>
                                    </td>
                                    <td class="px-4 py-3 text-xs text-slate-400 font-mono">{a.channel}</td>
                                    <td class="px-4 py-3 text-xs text-slate-400 font-mono text-right">{a.reach}</td>
                                    <td class="px-4 py-3">
                                        <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {status_cls}")>{a.status}</span>
                                    </td>
                                    <td class="px-4 py-3">
                                        <button class="rounded border border-slate-700 px-2 py-1 text-[9px] font-bold text-slate-400 hover:text-white transition-colors uppercase font-mono">"Review"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
