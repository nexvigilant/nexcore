//! Admin: Framework browser — visual exploration of EPA/CPA hierarchy

use leptos::prelude::*;

struct TreeItem {
    level: u32,
    label: &'static str,
    node_type: &'static str,
    children: u32,
    ksbs: u32,
}

const TREE: &[TreeItem] = &[
    TreeItem {
        level: 0,
        label: "Signal Detection & Management",
        node_type: "Area",
        children: 4,
        ksbs: 42,
    },
    TreeItem {
        level: 1,
        label: "EPA 1.1: Detect signals from spontaneous reports",
        node_type: "EPA",
        children: 2,
        ksbs: 12,
    },
    TreeItem {
        level: 2,
        label: "CPA 1.1.1: Run disproportionality analysis (PRR, ROR)",
        node_type: "CPA",
        children: 3,
        ksbs: 6,
    },
    TreeItem {
        level: 2,
        label: "CPA 1.1.2: Interpret signal metrics and confidence intervals",
        node_type: "CPA",
        children: 4,
        ksbs: 6,
    },
    TreeItem {
        level: 1,
        label: "EPA 1.2: Validate and prioritize signals",
        node_type: "EPA",
        children: 2,
        ksbs: 10,
    },
    TreeItem {
        level: 2,
        label: "CPA 1.2.1: Assess clinical significance of detected signal",
        node_type: "CPA",
        children: 2,
        ksbs: 5,
    },
    TreeItem {
        level: 2,
        label: "CPA 1.2.2: Document signal evaluation and rationale",
        node_type: "CPA",
        children: 3,
        ksbs: 5,
    },
    TreeItem {
        level: 1,
        label: "EPA 1.3: Signal tracking and communication",
        node_type: "EPA",
        children: 2,
        ksbs: 8,
    },
    TreeItem {
        level: 2,
        label: "CPA 1.3.1: Maintain signal tracking database",
        node_type: "CPA",
        children: 2,
        ksbs: 4,
    },
    TreeItem {
        level: 2,
        label: "CPA 1.3.2: Communicate signals to stakeholders",
        node_type: "CPA",
        children: 2,
        ksbs: 4,
    },
    TreeItem {
        level: 1,
        label: "EPA 1.4: Advanced signal methods (data mining, ML)",
        node_type: "EPA",
        children: 2,
        ksbs: 12,
    },
    TreeItem {
        level: 0,
        label: "Individual Case Safety Reports",
        node_type: "Area",
        children: 3,
        ksbs: 36,
    },
    TreeItem {
        level: 1,
        label: "EPA 2.1: Process ICSRs from all sources",
        node_type: "EPA",
        children: 3,
        ksbs: 14,
    },
    TreeItem {
        level: 1,
        label: "EPA 2.2: Medical review and assessment",
        node_type: "EPA",
        children: 2,
        ksbs: 12,
    },
    TreeItem {
        level: 1,
        label: "EPA 2.3: Regulatory submission of ICSRs",
        node_type: "EPA",
        children: 2,
        ksbs: 10,
    },
    TreeItem {
        level: 0,
        label: "Aggregate Safety Reporting",
        node_type: "Area",
        children: 4,
        ksbs: 34,
    },
    TreeItem {
        level: 0,
        label: "Risk Management",
        node_type: "Area",
        children: 3,
        ksbs: 28,
    },
    TreeItem {
        level: 0,
        label: "Benefit-Risk Assessment",
        node_type: "Area",
        children: 3,
        ksbs: 26,
    },
    TreeItem {
        level: 0,
        label: "Regulatory Intelligence",
        node_type: "Area",
        children: 4,
        ksbs: 38,
    },
    TreeItem {
        level: 0,
        label: "PV System & Quality",
        node_type: "Area",
        children: 3,
        ksbs: 30,
    },
];

#[component]
pub fn AcademyFrameworkBrowserPage() -> impl IntoView {
    let search = RwSignal::new(String::new());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div>
                <a href="/admin/academy/framework" class="text-cyan-400 hover:text-cyan-300 text-sm font-mono">"\u{2190} Back to Framework"</a>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight mt-2">"Framework Browser"</h1>
                <p class="mt-1 text-slate-400">"Visual exploration of the competency hierarchy \u{2014} Areas, EPAs, CPAs, and KSBs."</p>
            </div>

            /* Legend */
            <div class="mt-6 flex gap-3 flex-wrap">
                {[("Area", "text-amber-400 bg-amber-500/10 border-amber-500/20"),
                  ("EPA", "text-cyan-400 bg-cyan-500/10 border-cyan-500/20"),
                  ("CPA", "text-violet-400 bg-violet-500/10 border-violet-500/20"),
                  ("KSB", "text-emerald-400 bg-emerald-500/10 border-emerald-500/20")]
                    .into_iter().map(|(label, cls)| view! {
                        <span class=format!("rounded-full border px-3 py-1 text-[10px] font-bold font-mono uppercase {cls}")>{label}</span>
                    }).collect_view()}
            </div>

            /* Search */
            <div class="mt-4">
                <input
                    type="text"
                    placeholder="Search competencies, EPAs, CPAs, KSBs..."
                    class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
            </div>

            /* Tree */
            <div class="mt-6 space-y-1">
                {TREE.iter().map(|item| {
                    let color = match item.node_type {
                        "Area" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
                        "EPA" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
                        "CPA" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
                        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
                    };
                    let font_cls = if item.level == 0 { "font-bold" } else { "font-medium" };
                    let margin = format!("margin-left:{}rem", item.level * 2);
                    view! {
                        <div
                            class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-3 hover:bg-slate-800/30 transition-colors cursor-pointer"
                            style=margin
                        >
                            <div class="flex items-center gap-3 min-w-0">
                                <span class=format!("rounded px-2 py-0.5 text-[10px] font-bold font-mono uppercase border shrink-0 {color}")>{item.node_type}</span>
                                <span class=format!("text-sm text-white truncate {font_cls}")>{item.label}</span>
                            </div>
                            <div class="flex items-center gap-4 shrink-0 text-xs font-mono">
                                <span class="text-slate-500">{format!("{} children", item.children)}</span>
                                <span class="text-cyan-400">{format!("{} KSBs", item.ksbs)}</span>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="mt-4 text-[10px] text-slate-600 font-mono">
                {format!("{} items \u{00B7} 7 Areas \u{00B7} 24 EPAs \u{00B7} 48 CPAs", TREE.len())}
            </div>
        </div>
    }
}
