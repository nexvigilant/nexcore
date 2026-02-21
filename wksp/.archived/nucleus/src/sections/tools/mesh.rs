//! Full mesh navigator for cross-ecosystem connectivity

use leptos::prelude::*;

#[derive(Clone, Copy)]
struct MeshNode {
    label: &'static str,
    href: &'static str,
    zone: &'static str,
}

const NODES: &[MeshNode] = &[
    MeshNode {
        label: "Academy",
        href: "/academy",
        zone: "Enablement",
    },
    MeshNode {
        label: "Community",
        href: "/community",
        zone: "Enablement",
    },
    MeshNode {
        label: "Careers",
        href: "/careers",
        zone: "Enablement",
    },
    MeshNode {
        label: "Vigilance",
        href: "/vigilance",
        zone: "Safety",
    },
    MeshNode {
        label: "Regulatory",
        href: "/regulatory",
        zone: "Safety",
    },
    MeshNode {
        label: "Insights",
        href: "/insights",
        zone: "Intelligence",
    },
    MeshNode {
        label: "Solutions",
        href: "/solutions",
        zone: "Intelligence",
    },
    MeshNode {
        label: "Tools",
        href: "/tools",
        zone: "Engineering",
    },
    MeshNode {
        label: "App Store",
        href: "/tools/store",
        zone: "Engineering",
    },
    MeshNode {
        label: "Admin",
        href: "/admin",
        zone: "Control",
    },
    MeshNode {
        label: "Profile",
        href: "/profile",
        zone: "Identity",
    },
];

#[component]
pub fn MeshPage() -> impl IntoView {
    let selected_idx = RwSignal::new(0usize);
    let node_count = NODES.len();
    let edge_count = node_count * (node_count - 1);

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-10">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Ecosystem Mesh"</h1>
                </div>
                <p class="text-slate-400 max-w-3xl">
                    "Direct node-to-node pathways across the full ecosystem. Every domain can hop to every other domain."
                </p>
            </header>

            <div class="grid gap-8 lg:grid-cols-3">
                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                    <h2 class="text-[11px] font-bold font-mono text-cyan-400 uppercase tracking-[0.2em] mb-4">"Mesh Nodes"</h2>
                    <div class="space-y-2">
                        {NODES.iter().enumerate().map(|(idx, node)| view! {
                            <button
                                class=move || {
                                    if selected_idx.get() == idx {
                                        "w-full rounded-lg border border-cyan-500/40 bg-cyan-500/10 px-3 py-2 text-left"
                                    } else {
                                        "w-full rounded-lg border border-slate-800 bg-slate-950/60 px-3 py-2 text-left hover:border-slate-700"
                                    }
                                }
                                on:click=move |_| selected_idx.set(idx)
                            >
                                <div class="text-sm font-semibold text-white">{node.label}</div>
                                <div class="text-[10px] uppercase tracking-widest text-slate-500">{node.zone}</div>
                            </button>
                        }).collect_view()}
                    </div>
                </section>

                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5 lg:col-span-2">
                    <div class="flex items-center justify-between mb-5">
                        <h2 class="text-[11px] font-bold font-mono text-cyan-400 uppercase tracking-[0.2em]">
                            "Direct Links From Selected Node"
                        </h2>
                        <div class="text-[10px] font-mono uppercase tracking-widest text-slate-500">
                            {move || {
                                let src = NODES[selected_idx.get()];
                                format!(
                                    "{} nodes | {} directed edges | active: {}",
                                    node_count, edge_count, src.label
                                )
                            }}
                        </div>
                    </div>

                    <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        {move || {
                            let src_idx = selected_idx.get();
                            NODES.iter()
                                .enumerate()
                                .filter(|(idx, _)| *idx != src_idx)
                                .map(|(_, node)| view! {
                                    <a
                                        href=node.href
                                        class="rounded-lg border border-slate-800 bg-slate-950/70 px-4 py-3 hover:border-cyan-500/30 transition-colors"
                                    >
                                        <div class="text-sm font-semibold text-white">{node.label}</div>
                                        <div class="text-[10px] uppercase tracking-widest text-slate-500">{node.zone}</div>
                                    </a>
                                })
                                .collect_view()
                        }}
                    </div>
                </section>
            </div>
        </div>
    }
}
