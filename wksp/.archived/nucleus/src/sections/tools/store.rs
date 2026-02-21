//! App Store page migrated from legacy NCOS

use leptos::prelude::*;

struct EcosystemApp {
    name: &'static str,
    description: &'static str,
    href: &'static str,
    tier: &'static str,
}

#[component]
pub fn StorePage() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let apps = [
        EcosystemApp {
            name: "Nucleus",
            description: "Unified portal for vigilance, academy, community, and tools.",
            href: "/",
            tier: "production",
        },
        EcosystemApp {
            name: "Adventure HUD",
            description: "Game HUD with metric tracking.",
            href: "/tools",
            tier: "experimental",
        },
        EcosystemApp {
            name: "Borrow Miner",
            description: "Ore mining game with FDA signal checks.",
            href: "/tools",
            tier: "experimental",
        },
        EcosystemApp {
            name: "Education Machine",
            description: "Educational content experiences.",
            href: "/academy",
            tier: "experimental",
        },
        EcosystemApp {
            name: "Ferro Clicker",
            description: "Clicker-based interaction sandbox.",
            href: "/tools",
            tier: "experimental",
        },
        EcosystemApp {
            name: "Ferro Explore",
            description: "Ferrostack exploration interface.",
            href: "/tools",
            tier: "experimental",
        },
        EcosystemApp {
            name: "NexCore Watch",
            description: "Watch companion application surface.",
            href: "/tools",
            tier: "experimental",
        },
    ];

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-10">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                    <h1 class="text-3xl md:text-4xl font-black text-white font-mono uppercase tracking-tight">
                        "Ecosystem App Store"
                    </h1>
                </div>
                <p class="text-slate-400 max-w-3xl">
                    "Transferred from NCOS. Browse available workspace applications from inside Nucleus."
                </p>
            </header>

            <div class="mb-8">
                <input
                    type="search"
                    placeholder="Search apps..."
                    prop:value=move || query.get()
                    on:input=move |ev| query.set(event_target_value(&ev))
                    class="w-full rounded-xl border border-slate-800 bg-slate-900/60 px-4 py-3 text-sm text-white placeholder:text-slate-500 focus:border-cyan-500 focus:outline-none"
                />
            </div>

            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {move || {
                    let search = query.get().to_lowercase();
                    apps.iter()
                        .filter(|app| {
                            search.is_empty()
                                || app.name.to_lowercase().contains(&search)
                                || app.description.to_lowercase().contains(&search)
                        })
                        .map(|app| {
                            let tier_class = match app.tier {
                                "production" => "bg-emerald-500/10 text-emerald-400 border-emerald-500/30",
                                _ => "bg-amber-500/10 text-amber-400 border-amber-500/30",
                            };
                            view! {
                                <a
                                    href=app.href
                                    class="glass-panel rounded-2xl border border-slate-800 p-6 hover:border-cyan-500/30 transition-all"
                                >
                                    <div class="mb-3 flex items-start justify-between gap-3">
                                        <h2 class="text-xl font-bold text-white">{app.name}</h2>
                                        <span class=format!("rounded-full border px-2 py-1 text-[10px] font-bold uppercase tracking-widest {tier_class}")>
                                            {app.tier}
                                        </span>
                                    </div>
                                    <p class="text-sm text-slate-400">{app.description}</p>
                                    <div class="mt-5 text-[10px] font-bold uppercase tracking-[0.2em] text-cyan-400">
                                        "Open in Nucleus →"
                                    </div>
                                </a>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}
