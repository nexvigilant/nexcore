//! Unified Academy -> Guardian bridge map.

use super::gvp_data::{guardian_seed_for_module, GVP_MODULES};
use super::pv_ksb_framework::{all_cpas, all_epas, workbook};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LaunchProfile {
    name: String,
    layer: String,
    item: String,
    code: String,
    rationale: String,
    href: String,
}

#[cfg(feature = "hydrate")]
fn load_launch_profiles() -> Vec<LaunchProfile> {
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Vec::new();
    };
    let Ok(Some(raw)) = storage.get_item("academy_guardian_launch_profiles_v1") else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<LaunchProfile>>(&raw).unwrap_or_default()
}

#[cfg(feature = "hydrate")]
fn save_launch_profiles(profiles: &[LaunchProfile]) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let payload = serde_json::to_string(profiles).unwrap_or_else(|_| "[]".to_string());
    let _ = storage.set_item("academy_guardian_launch_profiles_v1", &payload);
}

#[component]
pub fn GuardianBridgePage() -> impl IntoView {
    let wb = workbook();
    let epas = all_epas();
    let cpas = all_cpas();
    let epas_store = StoredValue::new(epas.clone());
    let cpas_store = StoredValue::new(cpas.clone());

    let selected_layer = RwSignal::new(String::from("gvp"));
    let selected_item = RwSignal::new(String::from("I"));
    let profile_name = RwSignal::new(String::new());
    let profiles = RwSignal::new({
        #[cfg(feature = "hydrate")]
        {
            load_launch_profiles()
        }
        #[cfg(not(feature = "hydrate"))]
        {
            Vec::<LaunchProfile>::new()
        }
    });

    let recommended = Signal::derive(move || {
        let layer = selected_layer.get();
        let item = selected_item.get();
        if layer == "gvp" {
            let (drug, event, count) = guardian_seed_for_module(&item);
            (
                format!("GVP Module {}", item),
                format!(
                    "Regulatory module intent translated to seeded vigilance scenario ({drug}/{event}/{count})."
                ),
                format!(
                    "/vigilance/guardian?module={}&drug={}&event={}&count={}",
                    item, drug, event, count
                ),
            )
        } else if layer == "epa" {
            if let Some(e) = epas_store
                .get_value()
                .into_iter()
                .find(|e| e.epa_id.eq_ignore_ascii_case(&item))
            {
                let event = e.focus_area.to_ascii_lowercase().replace(' ', "-");
                (
                    e.epa_id.clone(),
                    format!("Entrustable activity execution for '{}'.", e.epa_name),
                    format!(
                        "/vigilance/guardian?module={}&event={}&count=2",
                        e.epa_id, event
                    ),
                )
            } else {
                (
                    "EPA Unknown".to_string(),
                    "EPA selection not found.".to_string(),
                    "/vigilance/guardian".to_string(),
                )
            }
        } else if let Some(c) = cpas_store
            .get_value()
            .into_iter()
            .find(|c| c.cpa_id.eq_ignore_ascii_case(&item))
        {
            let event = c.focus_area.to_ascii_lowercase().replace(' ', "-");
            (
                c.cpa_id.clone(),
                format!("Capability progression execution for '{}'.", c.cpa_name),
                format!(
                    "/vigilance/guardian?module={}&event={}&count=2",
                    c.cpa_id, event
                ),
            )
        } else {
            (
                "CPA Unknown".to_string(),
                "CPA selection not found.".to_string(),
                "/vigilance/guardian".to_string(),
            )
        }
    });

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-8">
                <p class="text-[11px] font-bold text-emerald-400 uppercase tracking-[0.2em] font-mono">"Execution Bridge"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"Academy ↔ Guardian Integration Map"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl">
                    "One execution surface connecting workbook-aligned learning assets to operational Guardian actions."
                </p>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/academy/pv-framework" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 uppercase tracking-widest font-mono">"Framework"</a>
                    <a href="/academy/epa-tracks" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 uppercase tracking-widest font-mono">"EPA Tracks"</a>
                    <a href="/academy/cpa-tracks" class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs font-bold text-amber-300 hover:text-amber-200 uppercase tracking-widest font-mono">"CPA Tracks"</a>
                    <a href="/academy/evidence-ledger" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 uppercase tracking-widest font-mono">"Evidence Ledger"</a>
                    <a href="/vigilance/guardian" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">"Open Guardian"</a>
                </div>
            </header>

            <section class="mb-6 rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Connection Layers"</h2>
                <div class="mt-4 grid gap-3 md:grid-cols-4">
                    <LayerCard label="GVP Modules" value=GVP_MODULES.len().to_string() description="Regulatory module intent" />
                    <LayerCard label="EPA Tracks" value=epas.len().to_string() description="Entrustable execution units" />
                    <LayerCard label="CPA Tracks" value=cpas.len().to_string() description="Capability progression" />
                    <LayerCard label="Domains" value=wb.domain_overview.len().to_string() description="Knowledge substrate" />
                </div>
            </section>

            <section class="mb-6 rounded-2xl border border-emerald-500/20 bg-emerald-500/5 p-5">
                <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Recommended Guardian Launch"</h2>
                <div class="mt-4 grid gap-3 md:grid-cols-3">
                    <select
                        prop:value=move || selected_layer.get()
                        on:change=move |ev| {
                            let next = event_target_value(&ev);
                            selected_layer.set(next.clone());
                            if next == "gvp" {
                                selected_item.set("I".to_string());
                            } else if next == "epa" {
                                if let Some(first) = epas_store.get_value().first() {
                                    selected_item.set(first.epa_id.clone());
                                }
                            } else if let Some(first) = cpas_store.get_value().first() {
                                selected_item.set(first.cpa_id.clone());
                            }
                        }
                        class="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-emerald-500 focus:outline-none"
                    >
                        <option value="gvp">"GVP Module"</option>
                        <option value="epa">"EPA Track"</option>
                        <option value="cpa">"CPA Track"</option>
                    </select>
                    <select
                        prop:value=move || selected_item.get()
                        on:change=move |ev| selected_item.set(event_target_value(&ev))
                        class="md:col-span-2 rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-emerald-500 focus:outline-none"
                    >
                        {move || {
                            if selected_layer.get() == "gvp" {
                                GVP_MODULES.iter().map(|m| {
                                    let value = m.code.to_string();
                                    let label = format!("Module {} — {}", m.code, m.title);
                                    view! { <option value=value>{label}</option> }
                                }).collect_view()
                            } else if selected_layer.get() == "epa" {
                                epas_store.get_value().into_iter().map(|e| {
                                    let value = e.epa_id.clone();
                                    let label = format!("{} — {}", e.epa_id, e.epa_name);
                                    view! { <option value=value>{label}</option> }
                                }).collect_view()
                            } else {
                                cpas_store.get_value().into_iter().map(|c| {
                                    let value = c.cpa_id.clone();
                                    let label = format!("{} — {}", c.cpa_id, c.cpa_name);
                                    view! { <option value=value>{label}</option> }
                                }).collect_view()
                            }
                        }}
                    </select>
                </div>
                {move || {
                    let (code, rationale, href) = recommended.get();
                    view! {
                        <div class="mt-4 rounded-xl border border-emerald-500/20 bg-slate-950/40 p-4">
                            <p class="text-[10px] font-bold text-emerald-300 uppercase tracking-widest font-mono">{code}</p>
                            <p class="mt-1 text-sm text-slate-300">{rationale}</p>
                            <a href=href class="mt-3 inline-flex rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">
                                "Launch Recommended Path"
                            </a>
                            <div class="mt-4 flex flex-col gap-3 md:flex-row md:items-center">
                                <input
                                    type="text"
                                    prop:value=move || profile_name.get()
                                    on:input=move |ev| profile_name.set(event_target_value(&ev))
                                    placeholder="Profile name (e.g., Signal Triage Sprint)"
                                    class="w-full md:flex-1 rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white placeholder:text-slate-500 focus:border-emerald-500 focus:outline-none"
                                />
                                <button
                                    on:click=move |_| {
                                        let name = profile_name.get().trim().to_string();
                                        if name.is_empty() {
                                            return;
                                        }
                                        let (code, rationale, href) = recommended.get();
                                        let mut next = profiles.get();
                                        let profile = LaunchProfile {
                                            name: name.clone(),
                                            layer: selected_layer.get(),
                                            item: selected_item.get(),
                                            code,
                                            rationale,
                                            href,
                                        };
                                        if let Some(i) = next
                                            .iter()
                                            .position(|p| p.name.eq_ignore_ascii_case(&name))
                                        {
                                            next[i] = profile;
                                        } else {
                                            next.insert(0, profile);
                                        }
                                        profiles.set(next.clone());
                                        profile_name.set(String::new());
                                        #[cfg(feature = "hydrate")]
                                        save_launch_profiles(&next);
                                    }
                                    class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono"
                                >
                                    "Save Launch Profile"
                                </button>
                            </div>
                            <div class="mt-4 space-y-2">
                                <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">
                                    "Saved Profiles"
                                </p>
                                {move || {
                                    let rows = profiles.get();
                                    if rows.is_empty() {
                                        view! {
                                            <p class="text-xs text-slate-500">
                                                "No saved profiles yet."
                                            </p>
                                        }
                                            .into_any()
                                    } else {
                                        rows.into_iter()
                                            .map(|profile| {
                                                let apply_layer = profile.layer.clone();
                                                let apply_item = profile.item.clone();
                                                let remove_name = profile.name.clone();
                                                let profile_href = profile.href.clone();
                                                view! {
                                                    <article class="rounded-lg border border-slate-800 bg-slate-900/40 px-3 py-2">
                                                        <div class="flex flex-wrap items-center justify-between gap-2">
                                                            <div class="min-w-0">
                                                                <p class="text-[10px] text-slate-500 font-mono uppercase tracking-widest">
                                                                    {profile.code}
                                                                </p>
                                                                <p class="text-sm text-white truncate">
                                                                    {profile.name}
                                                                </p>
                                                            </div>
                                                            <div class="flex items-center gap-2">
                                                                <button
                                                                    on:click=move |_| {
                                                                        selected_layer.set(apply_layer.clone());
                                                                        selected_item.set(apply_item.clone());
                                                                    }
                                                                    class="rounded border border-cyan-500/30 bg-cyan-500/10 px-2 py-1 text-[10px] font-bold text-cyan-300 hover:text-cyan-200 uppercase tracking-widest font-mono"
                                                                >
                                                                    "Apply"
                                                                </button>
                                                                <a
                                                                    href=profile_href
                                                                    class="rounded border border-emerald-500/30 bg-emerald-500/10 px-2 py-1 text-[10px] font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono"
                                                                >
                                                                    "Launch"
                                                                </a>
                                                                <button
                                                                    on:click=move |_| {
                                                                        let mut next = profiles.get();
                                                                        next.retain(|p| !p.name.eq_ignore_ascii_case(&remove_name));
                                                                        profiles.set(next.clone());
                                                                        #[cfg(feature = "hydrate")]
                                                                        save_launch_profiles(&next);
                                                                    }
                                                                    class="rounded border border-rose-500/30 bg-rose-500/10 px-2 py-1 text-[10px] font-bold text-rose-300 hover:text-rose-200 uppercase tracking-widest font-mono"
                                                                >
                                                                    "Delete"
                                                                </button>
                                                            </div>
                                                        </div>
                                                    </article>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                }}
                            </div>
                        </div>
                    }
                }}
            </section>

            <div class="grid gap-6 lg:grid-cols-3">
                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"GVP → Guardian"</h2>
                    <div class="mt-4 space-y-2 max-h-[480px] overflow-y-auto pr-1">
                        {GVP_MODULES.iter().map(|m| {
                            let (drug, event, count) = guardian_seed_for_module(m.code);
                            let href = format!("/vigilance/guardian?module={}&drug={}&event={}&count={}", m.code, drug, event, count);
                            view! {
                                <BridgeRow
                                    code=format!("M{}", m.code)
                                    title=m.title.to_string()
                                    href=href
                                />
                            }
                        }).collect_view()}
                    </div>
                </section>

                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"EPA → Guardian"</h2>
                    <div class="mt-4 space-y-2 max-h-[480px] overflow-y-auto pr-1">
                        {epas.into_iter().map(|e| {
                            let href = format!(
                                "/vigilance/guardian?module={}&event={}&count=2",
                                e.epa_id,
                                e.focus_area.to_ascii_lowercase().replace(' ', "-")
                            );
                            view! {
                                <BridgeRow
                                    code=e.epa_id
                                    title=e.epa_name
                                    href=href
                                />
                            }
                        }).collect_view()}
                    </div>
                </section>

                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"CPA → Guardian"</h2>
                    <div class="mt-4 space-y-2 max-h-[480px] overflow-y-auto pr-1">
                        {cpas.into_iter().map(|c| {
                            let href = format!(
                                "/vigilance/guardian?module={}&event={}&count=2",
                                c.cpa_id,
                                c.focus_area.to_ascii_lowercase().replace(' ', "-")
                            );
                            view! {
                                <BridgeRow
                                    code=c.cpa_id
                                    title=c.cpa_name
                                    href=href
                                />
                            }
                        }).collect_view()}
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn LayerCard(label: &'static str, value: String, description: &'static str) -> impl IntoView {
    view! {
        <article class="rounded-xl border border-slate-800 bg-slate-950/40 p-4 text-center">
            <p class="text-[10px] text-slate-500 font-bold uppercase tracking-widest font-mono">{label}</p>
            <p class="mt-1 text-2xl font-bold text-white font-mono">{value}</p>
            <p class="mt-1 text-[10px] text-slate-500">{description}</p>
        </article>
    }
}

#[component]
fn BridgeRow(code: String, title: String, href: String) -> impl IntoView {
    view! {
        <article class="rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2">
            <div class="flex items-center justify-between gap-3">
                <div class="min-w-0">
                    <p class="text-[10px] text-slate-500 font-mono uppercase tracking-widest">{code}</p>
                    <p class="text-sm text-white truncate">{title}</p>
                </div>
                <a href=href class="text-[10px] font-bold text-emerald-300 hover:text-emerald-200 uppercase tracking-widest font-mono">
                    "Launch"
                </a>
            </div>
        </article>
    }
}
