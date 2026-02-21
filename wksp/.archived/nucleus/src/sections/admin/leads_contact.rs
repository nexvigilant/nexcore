//! Admin: Contact form submissions

use leptos::prelude::*;

struct ContactSubmission {
    date: &'static str,
    name: &'static str,
    email: &'static str,
    subject: &'static str,
    status: &'static str,
    message_preview: &'static str,
}

const SUBMISSIONS: &[ContactSubmission] = &[
    ContactSubmission {
        date: "2026-02-15",
        name: "Dr. Klaus Weber",
        email: "k.weber@pharma-eu.de",
        subject: "PV System Audit Support",
        status: "New",
        message_preview: "We need help preparing for our upcoming EMA GVP inspection...",
    },
    ContactSubmission {
        date: "2026-02-15",
        name: "Jennifer Liu",
        email: "j.liu@meditech.com",
        subject: "Enterprise Pricing",
        status: "New",
        message_preview: "Interested in enterprise licensing for our 200+ person PV team...",
    },
    ContactSubmission {
        date: "2026-02-14",
        name: "Dr. Rajesh Gupta",
        email: "r.gupta@safepharma.in",
        subject: "Academy Partnership",
        status: "Replied",
        message_preview: "We'd like to discuss integrating your academy content with our LMS...",
    },
    ContactSubmission {
        date: "2026-02-14",
        name: "Anna Kowalski",
        email: "a.kowalski@pvteam.pl",
        subject: "Signal Detection Demo",
        status: "Replied",
        message_preview: "Can we schedule a demo of your signal detection capabilities?",
    },
    ContactSubmission {
        date: "2026-02-13",
        name: "Marcus Thompson",
        email: "m.thompson@biotrial.co.uk",
        subject: "Clinical Trial Safety",
        status: "Closed",
        message_preview: "Looking for a platform to manage clinical trial safety reporting...",
    },
    ContactSubmission {
        date: "2026-02-12",
        name: "Dr. Sophie Martin",
        email: "s.martin@agence-med.fr",
        subject: "Regulatory Compliance",
        status: "Closed",
        message_preview: "Need tools for French ANSM regulatory compliance requirements...",
    },
];

#[component]
pub fn LeadsContactPage() -> impl IntoView {
    let new_count = SUBMISSIONS.iter().filter(|s| s.status == "New").count();
    let replied = SUBMISSIONS.iter().filter(|s| s.status == "Replied").count();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Contact Submissions"</h1>
                    <p class="mt-1 text-slate-400">"Messages from the public contact form."</p>
                </div>
                <a href="/admin/leads" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} All Leads"</a>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 md:grid-cols-3">
                <div class="rounded-xl border border-cyan-500/20 bg-cyan-500/5 p-5">
                    <p class="text-[9px] font-bold text-cyan-400 uppercase tracking-widest font-mono">"New"</p>
                    <p class="text-2xl font-black text-cyan-400 font-mono mt-2">{new_count.to_string()}</p>
                </div>
                <div class="rounded-xl border border-amber-500/20 bg-amber-500/5 p-5">
                    <p class="text-[9px] font-bold text-amber-400 uppercase tracking-widest font-mono">"Replied"</p>
                    <p class="text-2xl font-black text-amber-400 font-mono mt-2">{replied.to_string()}</p>
                </div>
                <div class="rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                    <p class="text-[9px] font-bold text-emerald-400 uppercase tracking-widest font-mono">"Total"</p>
                    <p class="text-2xl font-black text-emerald-400 font-mono mt-2">{SUBMISSIONS.len().to_string()}</p>
                </div>
            </div>

            /* Submissions table */
            <div class="mt-8 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Date"</th>
                            <th class="px-4 py-3">"Contact"</th>
                            <th class="px-4 py-3">"Subject"</th>
                            <th class="px-4 py-3">"Status"</th>
                            <th class="px-4 py-3">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {SUBMISSIONS.iter().map(|s| {
                            let status_cls = match s.status {
                                "New" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                                "Replied" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                                "Closed" => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                                _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                            };
                            view! {
                                <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono">{s.date}</td>
                                    <td class="px-4 py-3">
                                        <p class="text-sm font-medium text-white">{s.name}</p>
                                        <p class="text-[10px] text-slate-500 font-mono">{s.email}</p>
                                    </td>
                                    <td class="px-4 py-3">
                                        <p class="text-sm text-white">{s.subject}</p>
                                        <p class="text-[10px] text-slate-500 mt-0.5 truncate max-w-xs">{s.message_preview}</p>
                                    </td>
                                    <td class="px-4 py-3">
                                        <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {status_cls}")>{s.status}</span>
                                    </td>
                                    <td class="px-4 py-3">
                                        <button class="rounded border border-slate-700 px-2 py-1 text-[9px] font-bold text-slate-400 hover:text-white transition-colors uppercase font-mono">"Reply"</button>
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
