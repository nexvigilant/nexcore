//! Admin: Academy resources — manage learning materials, documents, and references

use leptos::prelude::*;

struct Resource {
    name: &'static str,
    res_type: &'static str,
    domain: &'static str,
    size: &'static str,
    downloads: u32,
    uploaded: &'static str,
}

const RESOURCES: &[Resource] = &[
    Resource {
        name: "ICH E2E Pharmacovigilance Planning",
        res_type: "Document",
        domain: "Regulatory",
        size: "2.4 MB",
        downloads: 342,
        uploaded: "2026-01-15",
    },
    Resource {
        name: "Signal Detection Algorithms Explained",
        res_type: "Video",
        domain: "Signal Detection",
        size: "156 MB",
        downloads: 892,
        uploaded: "2026-01-20",
    },
    Resource {
        name: "ICSR Processing Checklist",
        res_type: "Template",
        domain: "Case Processing",
        size: "45 KB",
        downloads: 567,
        uploaded: "2026-01-22",
    },
    Resource {
        name: "GVP Module IX Reference Card",
        res_type: "Reference",
        domain: "Signal Detection",
        size: "180 KB",
        downloads: 1204,
        uploaded: "2026-01-10",
    },
    Resource {
        name: "MedDRA Coding Quick Guide",
        res_type: "Document",
        domain: "Case Processing",
        size: "1.8 MB",
        downloads: 743,
        uploaded: "2026-02-01",
    },
    Resource {
        name: "PBRER Template (EMA Format)",
        res_type: "Template",
        domain: "Aggregate Reporting",
        size: "92 KB",
        downloads: 456,
        uploaded: "2026-02-05",
    },
    Resource {
        name: "Benefit-Risk Assessment Workshop",
        res_type: "Video",
        domain: "Benefit-Risk",
        size: "234 MB",
        downloads: 312,
        uploaded: "2026-02-08",
    },
    Resource {
        name: "RMP Section Templates",
        res_type: "Template",
        domain: "Risk Management",
        size: "67 KB",
        downloads: 389,
        uploaded: "2026-02-10",
    },
    Resource {
        name: "WHO-UMC Causality Categories",
        res_type: "Reference",
        domain: "Case Processing",
        size: "120 KB",
        downloads: 678,
        uploaded: "2026-01-28",
    },
    Resource {
        name: "QPPV Responsibilities Overview",
        res_type: "Document",
        domain: "PV Systems",
        size: "3.1 MB",
        downloads: 234,
        uploaded: "2026-02-12",
    },
];

#[component]
pub fn AcademyResourcesPage() -> impl IntoView {
    let docs = RESOURCES
        .iter()
        .filter(|r| r.res_type == "Document")
        .count();
    let videos = RESOURCES.iter().filter(|r| r.res_type == "Video").count();
    let templates = RESOURCES
        .iter()
        .filter(|r| r.res_type == "Template")
        .count();
    let refs = RESOURCES
        .iter()
        .filter(|r| r.res_type == "Reference")
        .count();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Academy Resources"</h1>
                    <p class="mt-1 text-slate-400">"Manage learning materials, reference documents, and supplementary content."</p>
                </div>
                <button class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-xs font-bold transition-colors font-mono uppercase">"Upload Resource"</button>
            </div>

            /* Stats */
            <div class="mt-6 grid gap-4 sm:grid-cols-4">
                {[("Documents", docs, "text-blue-400 border-blue-500/20 bg-blue-500/5"),
                  ("Videos", videos, "text-violet-400 border-violet-500/20 bg-violet-500/5"),
                  ("Templates", templates, "text-emerald-400 border-emerald-500/20 bg-emerald-500/5"),
                  ("References", refs, "text-amber-400 border-amber-500/20 bg-amber-500/5")]
                    .into_iter().map(|(label, count, cls)| view! {
                        <div class=format!("rounded-xl border p-5 {cls}")>
                            <p class="text-[9px] font-bold uppercase tracking-widest font-mono">{label}</p>
                            <p class="text-2xl font-black font-mono mt-2">{count.to_string()}</p>
                        </div>
                    }).collect_view()}
            </div>

            /* Resource table */
            <div class="mt-8 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-[10px] uppercase text-slate-500 font-mono tracking-widest">
                        <tr>
                            <th class="px-4 py-3">"Name"</th>
                            <th class="px-4 py-3">"Type"</th>
                            <th class="px-4 py-3">"Domain"</th>
                            <th class="px-4 py-3 text-right">"Size"</th>
                            <th class="px-4 py-3 text-right">"Downloads"</th>
                            <th class="px-4 py-3">"Uploaded"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        {RESOURCES.iter().map(|r| {
                            let type_cls = match r.res_type {
                                "Document" => "text-blue-400 bg-blue-500/10 border-blue-500/20",
                                "Video" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
                                "Template" => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
                                "Reference" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                                _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                            };
                            view! {
                                <tr class="border-t border-slate-800 hover:bg-slate-800/30 transition-colors">
                                    <td class="px-4 py-3 text-sm font-medium text-white">{r.name}</td>
                                    <td class="px-4 py-3">
                                        <span class=format!("rounded-full border px-2 py-0.5 text-[9px] font-bold font-mono uppercase {type_cls}")>{r.res_type}</span>
                                    </td>
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono">{r.domain}</td>
                                    <td class="px-4 py-3 text-xs text-slate-500 font-mono text-right">{r.size}</td>
                                    <td class="px-4 py-3 text-xs text-slate-400 font-mono text-right">{r.downloads.to_string()}</td>
                                    <td class="px-4 py-3 text-[10px] text-slate-600 font-mono">{r.uploaded}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>

            <div class="mt-4 text-[10px] text-slate-600 font-mono">
                {format!("{} resources \u{00B7} {} total downloads", RESOURCES.len(), RESOURCES.iter().map(|r| r.downloads).sum::<u32>())}
            </div>
        </div>
    }
}
