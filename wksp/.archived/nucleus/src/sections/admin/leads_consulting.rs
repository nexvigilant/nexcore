//! Admin: Consulting lead management

use leptos::prelude::*;

struct ConsultingLead {
    date: &'static str,
    company: &'static str,
    contact: &'static str,
    email: &'static str,
    service: &'static str,
    stage: &'static str,
    value: &'static str,
}

const LEADS: &[ConsultingLead] = &[
    ConsultingLead {
        date: "2026-02-15",
        company: "PharmaCo Inc",
        contact: "Dr. Sarah Mitchell",
        email: "s.mitchell@pharma.co",
        service: "PV System Audit",
        stage: "New",
        value: "$45,000",
    },
    ConsultingLead {
        date: "2026-02-14",
        company: "MediSafe EU",
        contact: "James Park",
        email: "j.park@medisafe.eu",
        service: "Signal Detection Setup",
        stage: "Qualifying",
        value: "$28,000",
    },
    ConsultingLead {
        date: "2026-02-14",
        company: "Nordic PV AB",
        contact: "Dr. Henrik Larsson",
        email: "h.larsson@nordicpv.se",
        service: "QPPV Services",
        stage: "Qualifying",
        value: "$120,000",
    },
    ConsultingLead {
        date: "2026-02-13",
        company: "DrugWatch India",
        contact: "Priya Sharma",
        email: "p.sharma@drugwatch.in",
        service: "Regulatory Intelligence",
        stage: "Proposal Sent",
        value: "$35,000",
    },
    ConsultingLead {
        date: "2026-02-12",
        company: "BioPharma Ltd",
        contact: "Emily Chen",
        email: "e.chen@biopharma.com",
        service: "PV Training Program",
        stage: "Proposal Sent",
        value: "$18,000",
    },
    ConsultingLead {
        date: "2026-02-11",
        company: "SafeMed GmbH",
        contact: "Thomas Weber",
        email: "t.weber@safemed.de",
        service: "GVP Compliance Review",
        stage: "Won",
        value: "$52,000",
    },
    ConsultingLead {
        date: "2026-02-10",
        company: "PV Global",
        contact: "Dr. Ahmed Hassan",
        email: "a.hassan@pvglobal.com",
        service: "Signal Management",
        stage: "Won",
        value: "$38,000",
    },
    ConsultingLead {
        date: "2026-02-09",
        company: "VigilanceIO",
        contact: "Maria Santos",
        email: "m.santos@vigil.io",
        service: "ICSR Processing",
        stage: "Won",
        value: "$24,000",
    },
];

#[component]
pub fn LeadsConsultingPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Consulting Leads"</h1>
                    <p class="mt-1 text-slate-400">"Inbound consulting requests and engagement pipeline."</p>
                </div>
                <a href="/admin/leads" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"\u{2190} All Leads"</a>
            </div>

            /* Pipeline stats */
            <div class="mt-6 grid gap-4 md:grid-cols-4">
                {[("New", "1", "text-cyan-400 border-cyan-500/20 bg-cyan-500/5"),
                  ("Qualifying", "2", "text-amber-400 border-amber-500/20 bg-amber-500/5"),
                  ("Proposal Sent", "2", "text-violet-400 border-violet-500/20 bg-violet-500/5"),
                  ("Won", "3", "text-emerald-400 border-emerald-500/20 bg-emerald-500/5")]
                    .into_iter().map(|(stage, count, cls)| view! {
                        <div class=format!("rounded-xl border p-5 {cls}")>
                            <p class="text-[9px] font-bold uppercase tracking-widest font-mono">{stage}</p>
                            <p class="text-2xl font-black font-mono mt-2">{count}</p>
                        </div>
                    }).collect_view()}
            </div>

            /* Leads table */
            <div class="mt-8 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Date"</th>
                            <th class="px-4 py-3">"Company"</th>
                            <th class="px-4 py-3">"Contact"</th>
                            <th class="px-4 py-3">"Service"</th>
                            <th class="px-4 py-3">"Stage"</th>
                            <th class="px-4 py-3 text-right">"Value"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {LEADS.iter().map(|l| {
                            let stage_cls = match l.stage {
                                "New" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                                "Qualifying" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                                "Proposal Sent" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
                                "Won" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                                _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                            };
                            view! {
                                <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono">{l.date}</td>
                                    <td class="px-4 py-3 text-sm font-medium text-white">{l.company}</td>
                                    <td class="px-4 py-3">
                                        <p class="text-sm text-slate-300">{l.contact}</p>
                                        <p class="text-[10px] text-slate-500 font-mono">{l.email}</p>
                                    </td>
                                    <td class="px-4 py-3 text-xs text-slate-400 font-mono">{l.service}</td>
                                    <td class="px-4 py-3">
                                        <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {stage_cls}")>{l.stage}</span>
                                    </td>
                                    <td class="px-4 py-3 text-sm font-bold text-emerald-400 font-mono text-right">{l.value}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            <div class="mt-4 flex items-center justify-between text-[10px] text-slate-600 font-mono">
                <span>{format!("{} leads", LEADS.len())}</span>
                <span>"Pipeline value: $360,000"</span>
            </div>
        </div>
    }
}
