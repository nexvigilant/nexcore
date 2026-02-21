//! Academy skills browser — explore KSBs across 15 PV domains
//!
//! Wired to NexCore KSB taxonomy via server function. Each domain shows
//! its dominant primitive, PVOS layer alignment, and transfer confidence.

use leptos::prelude::*;
use crate::api_client::KsbDomainSummary;

/// Server function to fetch KSB domain taxonomy from NexCore API.
#[server(ListKsbDomains, "/api")]
pub async fn list_ksb_domains_action() -> Result<Vec<KsbDomainSummary>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.academy_ksb_domains().await
        .map_err(ServerFnError::new)
}

#[component]
pub fn SkillsPage() -> impl IntoView {
    let domains = Resource::new(|| (), |_| list_ksb_domains_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <header class="mb-8">
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"KSB Taxonomy"</h1>
                <p class="mt-2 text-slate-400">"Explore 1,462 Knowledge, Skills, and Behaviors across 15 pharmacovigilance domains — each grounded to Lex Primitiva."</p>
            </header>

            <Suspense fallback=|| view! { <LoadingGrid /> }>
                {move || domains.get().map(|result| match result {
                    Ok(list) => view! { <DomainBrowser domains=list /> }.into_any(),
                    Err(e) => view! {
                        <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-6 text-red-400 font-mono text-sm">
                            "Failed to load KSB taxonomy: " {e.to_string()}
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn LoadingGrid() -> impl IntoView {
    view! {
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {(0..6).map(|_| view! {
                <div class="rounded-xl border border-slate-800 bg-slate-900/30 p-5 h-52 animate-pulse">
                    <div class="h-4 w-16 bg-slate-800 rounded mb-4"></div>
                    <div class="h-5 w-48 bg-slate-800 rounded mb-3"></div>
                    <div class="h-3 w-full bg-slate-800 rounded mb-1"></div>
                    <div class="h-3 w-2/3 bg-slate-800 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

#[component]
fn DomainBrowser(domains: Vec<KsbDomainSummary>) -> impl IntoView {
    let domains = StoredValue::new(domains);
    let search_query = RwSignal::new(String::new());
    let selected_domain = RwSignal::new(String::from("All"));

    let filtered = Signal::derive(move || {
        let query = search_query.get().to_lowercase();
        let filter = selected_domain.get();
        domains.get_value().into_iter()
            .filter(|d| {
                let domain_match = filter == "All" || d.code == filter;
                let search_match = query.is_empty()
                    || d.name.to_lowercase().contains(&query)
                    || d.code.to_lowercase().contains(&query)
                    || d.example_ksbs.iter().any(|k| k.to_lowercase().contains(&query));
                domain_match && search_match
            })
            .collect::<Vec<_>>()
    });

    let total_ksbs = Signal::derive(move || {
        filtered.get().iter().map(|d| d.ksb_count).sum::<u32>()
    });

    let all_domains = domains.get_value();

    view! {
        <div class="flex flex-col gap-4 sm:flex-row sm:items-center mb-4">
            <input
                type="text"
                placeholder="Search domains, KSBs..."
                prop:value=move || search_query.get()
                on:input=move |ev| search_query.set(event_target_value(&ev))
                class="flex-1 rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none font-mono"
            />
            <select
                on:change=move |ev| selected_domain.set(event_target_value(&ev))
                class="rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white"
            >
                <option value="All">"All Domains"</option>
                {all_domains.into_iter().map(|d| {
                    let label = format!("{} — {}", d.code, d.name);
                    let val = d.code.clone();
                    view! { <option value=val>{label}</option> }
                }).collect_view()}
            </select>
        </div>

        <p class="mb-6 text-xs text-slate-500 font-mono">
            {move || format!("{} domains · {} KSBs", filtered.get().len(), total_ksbs.get())}
        </p>

        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {move || filtered.get().into_iter().map(|d| {
                view! { <DomainCard domain=d /> }
            }).collect_view()}
        </div>
    }
}

#[component]
fn DomainCard(domain: KsbDomainSummary) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let detail_href = format!("/academy/skills/{}", domain.code.to_lowercase());

    let primitive_symbol = primitive_to_symbol(&domain.dominant_primitive);
    let cognitive_symbol = primitive_to_symbol(&domain.cognitive_primitive);

    let confidence_pct = format!("{:.0}%", domain.transfer_confidence * 100.0);
    let confidence_color = if domain.transfer_confidence >= 0.80 {
        "text-emerald-400"
    } else if domain.transfer_confidence >= 0.70 {
        "text-cyan-400"
    } else {
        "text-amber-400"
    };

    let code = domain.code.clone();
    let name = domain.name.clone();
    let ksb_label = format!("{} KSBs", domain.ksb_count);
    let pvos_badge = domain.pvos_layer.clone();

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-cyan-500/20 transition-colors">
            <div class="flex items-center justify-between mb-3">
                <div class="flex items-center gap-2">
                    <span class="rounded bg-cyan-500/10 px-2 py-0.5 text-xs font-bold font-mono text-cyan-400">{code}</span>
                    {pvos_badge.map(|layer| {
                        view! {
                            <span class="rounded bg-violet-500/10 px-1.5 py-0.5 text-[10px] font-mono text-violet-400">{layer}</span>
                        }
                    })}
                </div>
                <span class="text-xs text-slate-500 font-mono">{ksb_label}</span>
            </div>

            <h3 class="font-semibold text-white text-sm leading-snug">{name}</h3>

            // Primitive badges
            <div class="mt-3 flex items-center gap-3 text-[10px]">
                <span class="flex items-center gap-1 text-slate-400" title="Operational primitive">
                    <span class="text-cyan-400 font-mono text-sm">{primitive_symbol}</span>
                    " ops"
                </span>
                <span class="flex items-center gap-1 text-slate-400" title="Cognitive primitive">
                    <span class="text-amber-400 font-mono text-sm">{cognitive_symbol}</span>
                    " cog"
                </span>
                <span class=format!("ml-auto font-mono font-bold {confidence_color}") title="Transfer confidence">
                    {confidence_pct}
                </span>
            </div>

            <button
                on:click=move |_| expanded.update(|v| *v = !*v)
                class="mt-3 text-xs text-slate-500 hover:text-cyan-400 transition-colors font-mono"
            >
                {move || if expanded.get() { "Hide examples" } else { "Show examples" }}
            </button>

            {move || expanded.get().then(|| {
                let ksbs = domain.example_ksbs.clone();
                view! {
                    <ul class="mt-3 space-y-1.5">
                        {ksbs.into_iter().map(|ksb| {
                            view! {
                                <li class="text-xs text-slate-400 pl-3 border-l border-slate-800">{ksb}</li>
                            }
                        }).collect_view()}
                    </ul>
                }
            })}

            <a href=detail_href class="mt-4 block text-xs text-cyan-400 hover:text-cyan-300 font-mono transition-colors">
                "Browse all →"
            </a>
        </div>
    }
}

/// Map primitive Debug name to its Lex Primitiva Unicode symbol.
fn primitive_to_symbol(name: &str) -> &'static str {
    match name {
        "Sequence" => "σ",
        "Mapping" => "μ",
        "State" => "ς",
        "Recursion" => "ρ",
        "Void" => "∅",
        "Boundary" => "∂",
        "Frequency" => "ν",
        "Existence" => "∃",
        "Persistence" => "π",
        "Causality" => "→",
        "Comparison" => "κ",
        "Quantity" => "N",
        "Location" => "λ",
        "Irreversibility" => "∝",
        "Sum" => "Σ",
        "Product" => "×",
        _ => "?",
    }
}
