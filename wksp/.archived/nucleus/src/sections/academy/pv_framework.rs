//! PV KSB Framework explorer powered by embedded workbook import.

use super::pv_ksb_framework::{
    framework_stats, sheet_counts, workbook, PvCpaDomainRow, PvDomainRow, PvEpaDomainRow, PvKsbRow,
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegulatorFeedStatus {
    pub authority: String,
    pub status: String,
    pub source_url: String,
    pub headline: String,
    pub observed_at: String,
    pub checked_at: String,
    pub is_new: bool,
}

#[derive(Debug, Deserialize)]
struct FdaEnforcementResponse {
    #[serde(default)]
    results: Vec<FdaEnforcementItem>,
}

#[derive(Debug, Deserialize)]
struct FdaEnforcementItem {
    #[serde(default)]
    product_description: String,
    #[serde(default)]
    report_date: String,
}

#[derive(Debug, Deserialize)]
struct EmaDhpcItem {
    #[serde(default, alias = "Title")]
    title: String,
    #[serde(default, alias = "First published")]
    first_published: String,
}

#[derive(Debug, Deserialize)]
struct GovUkSearchResponse {
    #[serde(default)]
    results: Vec<GovUkResult>,
}

#[derive(Debug, Deserialize)]
struct GovUkResult {
    #[serde(default)]
    title: String,
    #[serde(default)]
    public_timestamp: String,
}

static REGULATOR_SIGNATURES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn novelty_cache() -> &'static Mutex<HashMap<String, String>> {
    REGULATOR_SIGNATURES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn apply_novelty(mut rows: Vec<RegulatorFeedStatus>) -> Vec<RegulatorFeedStatus> {
    let mut cache = novelty_cache()
        .lock()
        .expect("novelty cache mutex poisoned");
    for row in &mut rows {
        let signature = format!("{}|{}|{}", row.status, row.headline, row.observed_at);
        let prev = cache.get(&row.authority).cloned().unwrap_or_default();
        row.is_new = !prev.is_empty() && prev != signature;
        cache.insert(row.authority.clone(), signature);
    }
    rows
}

fn regulator_signature(item: &RegulatorFeedStatus) -> String {
    format!(
        "{}|{}|{}|{}",
        item.authority, item.status, item.headline, item.observed_at
    )
}

#[cfg(feature = "hydrate")]
fn load_acknowledged_signatures() -> HashSet<String> {
    let Some(window) = web_sys::window() else {
        return HashSet::new();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return HashSet::new();
    };
    let Ok(Some(raw)) = storage.get_item("pv_framework_ack_v1") else {
        return HashSet::new();
    };
    serde_json::from_str::<Vec<String>>(&raw)
        .map(|v| v.into_iter().collect())
        .unwrap_or_default()
}

#[cfg(feature = "hydrate")]
fn save_acknowledged_signatures(set: &HashSet<String>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    let payload = serde_json::to_string(&set.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_else(|_| "[]".to_string());
    let _ = storage.set_item("pv_framework_ack_v1", &payload);
}

#[server(CheckRegulatorUpdates, "/api")]
pub async fn check_regulator_updates() -> Result<Vec<RegulatorFeedStatus>, ServerFnError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| ServerFnError::new(format!("HTTP client init failed: {e}")))?;

    let checked_at = chrono::Utc::now().to_rfc3339();
    let mut out = Vec::new();

    let fda_url = "https://api.fda.gov/drug/enforcement.json?limit=1&sort=report_date:desc";
    match client.get(fda_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<FdaEnforcementResponse>().await
        {
            Ok(parsed) => {
                if let Some(item) = parsed.results.first() {
                    out.push(RegulatorFeedStatus {
                        authority: "FDA".to_string(),
                        status: "ok".to_string(),
                        source_url: fda_url.to_string(),
                        headline: item.product_description.clone(),
                        observed_at: item.report_date.clone(),
                        checked_at: checked_at.clone(),
                        is_new: false,
                    });
                } else {
                    out.push(RegulatorFeedStatus {
                        authority: "FDA".to_string(),
                        status: "empty".to_string(),
                        source_url: fda_url.to_string(),
                        headline: "No recent enforcement rows returned".to_string(),
                        observed_at: String::new(),
                        checked_at: checked_at.clone(),
                        is_new: false,
                    });
                }
            }
            Err(e) => out.push(RegulatorFeedStatus {
                authority: "FDA".to_string(),
                status: "error".to_string(),
                source_url: fda_url.to_string(),
                headline: format!("Parse failed: {e}"),
                observed_at: String::new(),
                checked_at: checked_at.clone(),
                is_new: false,
            }),
        },
        Ok(resp) => out.push(RegulatorFeedStatus {
            authority: "FDA".to_string(),
            status: "error".to_string(),
            source_url: fda_url.to_string(),
            headline: format!("HTTP {}", resp.status()),
            observed_at: String::new(),
            checked_at: checked_at.clone(),
            is_new: false,
        }),
        Err(e) => out.push(RegulatorFeedStatus {
            authority: "FDA".to_string(),
            status: "error".to_string(),
            source_url: fda_url.to_string(),
            headline: format!("Request failed: {e}"),
            observed_at: String::new(),
            checked_at: checked_at.clone(),
            is_new: false,
        }),
    }

    let ema_url = "https://www.ema.europa.eu/en/documents/report/dhpc-output-json-report_en.json";
    match client.get(ema_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Vec<EmaDhpcItem>>().await {
            Ok(rows) => {
                if let Some(item) = rows.first() {
                    out.push(RegulatorFeedStatus {
                        authority: "EMA".to_string(),
                        status: "ok".to_string(),
                        source_url: ema_url.to_string(),
                        headline: item.title.clone(),
                        observed_at: item.first_published.clone(),
                        checked_at: checked_at.clone(),
                        is_new: false,
                    });
                } else {
                    out.push(RegulatorFeedStatus {
                        authority: "EMA".to_string(),
                        status: "empty".to_string(),
                        source_url: ema_url.to_string(),
                        headline: "No DHPC rows returned".to_string(),
                        observed_at: String::new(),
                        checked_at: checked_at.clone(),
                        is_new: false,
                    });
                }
            }
            Err(e) => out.push(RegulatorFeedStatus {
                authority: "EMA".to_string(),
                status: "error".to_string(),
                source_url: ema_url.to_string(),
                headline: format!("Parse failed: {e}"),
                observed_at: String::new(),
                checked_at: checked_at.clone(),
                is_new: false,
            }),
        },
        Ok(resp) => out.push(RegulatorFeedStatus {
            authority: "EMA".to_string(),
            status: "error".to_string(),
            source_url: ema_url.to_string(),
            headline: format!("HTTP {}", resp.status()),
            observed_at: String::new(),
            checked_at: checked_at.clone(),
            is_new: false,
        }),
        Err(e) => out.push(RegulatorFeedStatus {
            authority: "EMA".to_string(),
            status: "error".to_string(),
            source_url: ema_url.to_string(),
            headline: format!("Request failed: {e}"),
            observed_at: String::new(),
            checked_at: checked_at.clone(),
            is_new: false,
        }),
    }

    let mhra_url = "https://www.gov.uk/api/search.json";
    match client
        .get(mhra_url)
        .query(&[
            (
                "filter_organisations",
                "medicines-and-healthcare-products-regulatory-agency",
            ),
            ("count", "1"),
            ("order", "-public_timestamp"),
            ("fields", "title,public_timestamp"),
        ])
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<GovUkSearchResponse>().await {
            Ok(rows) => {
                if let Some(item) = rows.results.first() {
                    out.push(RegulatorFeedStatus {
                        authority: "MHRA".to_string(),
                        status: "ok".to_string(),
                        source_url: mhra_url.to_string(),
                        headline: item.title.clone(),
                        observed_at: item.public_timestamp.clone(),
                        checked_at: checked_at.clone(),
                        is_new: false,
                    });
                } else {
                    out.push(RegulatorFeedStatus {
                        authority: "MHRA".to_string(),
                        status: "empty".to_string(),
                        source_url: mhra_url.to_string(),
                        headline: "No MHRA search results".to_string(),
                        observed_at: String::new(),
                        checked_at: checked_at.clone(),
                        is_new: false,
                    });
                }
            }
            Err(e) => out.push(RegulatorFeedStatus {
                authority: "MHRA".to_string(),
                status: "error".to_string(),
                source_url: mhra_url.to_string(),
                headline: format!("Parse failed: {e}"),
                observed_at: String::new(),
                checked_at: checked_at.clone(),
                is_new: false,
            }),
        },
        Ok(resp) => out.push(RegulatorFeedStatus {
            authority: "MHRA".to_string(),
            status: "error".to_string(),
            source_url: mhra_url.to_string(),
            headline: format!("HTTP {}", resp.status()),
            observed_at: String::new(),
            checked_at: checked_at.clone(),
            is_new: false,
        }),
        Err(e) => out.push(RegulatorFeedStatus {
            authority: "MHRA".to_string(),
            status: "error".to_string(),
            source_url: mhra_url.to_string(),
            headline: format!("Request failed: {e}"),
            observed_at: String::new(),
            checked_at: checked_at.clone(),
            is_new: false,
        }),
    }

    for (authority, url) in [
        (
            "PMDA",
            "https://www.pmda.go.jp/english/safety/info-services/drugs/calling-attention/safety-information/0001.html",
        ),
        ("TGA", "https://www.tga.gov.au/safety/safety-alerts"),
    ] {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => out.push(RegulatorFeedStatus {
                authority: authority.to_string(),
                status: "ok".to_string(),
                source_url: url.to_string(),
                headline: "Endpoint reachable".to_string(),
                observed_at: String::new(),
                checked_at: checked_at.clone(),
                is_new: false,
            }),
            Ok(resp) => out.push(RegulatorFeedStatus {
                authority: authority.to_string(),
                status: "error".to_string(),
                source_url: url.to_string(),
                headline: format!("HTTP {}", resp.status()),
                observed_at: String::new(),
                checked_at: checked_at.clone(),
                is_new: false,
            }),
            Err(e) => out.push(RegulatorFeedStatus {
                authority: authority.to_string(),
                status: "error".to_string(),
                source_url: url.to_string(),
                headline: format!("Request failed: {e}"),
                observed_at: String::new(),
                checked_at: checked_at.clone(),
                is_new: false,
            }),
        }
    }

    Ok(apply_novelty(out))
}

#[component]
pub fn PvFrameworkPage() -> impl IntoView {
    let stats = framework_stats();
    let sheets = sheet_counts();
    let wb = workbook();

    let domains = StoredValue::new(wb.domain_overview.clone());
    let ksbs = StoredValue::new(wb.capability_components.clone());
    let epa_map = StoredValue::new(wb.epa_domain_mapping.clone());
    let cpa_map = StoredValue::new(wb.cpa_domain_mapping.clone());
    let sheet_data = StoredValue::new(wb.all_sheets.clone());

    let domain_query = RwSignal::new(String::new());
    let selected_domain = RwSignal::new(String::from("ALL"));
    let ksb_query = RwSignal::new(String::new());
    let ksb_type_filter = RwSignal::new(String::from("all"));
    let refresh_nonce = RwSignal::new(0u64);
    let regulator_updates =
        Resource::new(move || refresh_nonce.get(), |_| check_regulator_updates());
    let previous_updates = RwSignal::new(HashMap::<String, RegulatorFeedStatus>::new());
    let regulator_change_map = RwSignal::new(HashMap::<String, Vec<String>>::new());
    let regulator_filter = RwSignal::new(String::from("all"));
    let acknowledged_signatures = RwSignal::new(HashSet::<String>::new());
    let ack_loaded = RwSignal::new(false);
    let selected_sheet = RwSignal::new(
        wb.sheet_names
            .first()
            .cloned()
            .unwrap_or_else(|| "Domain Overview".to_string()),
    );
    let sheet_row_query = RwSignal::new(String::new());

    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        Effect::new(move |_| {
            let cb = Closure::<dyn FnMut()>::new(move || {
                refresh_nonce.update(|n| *n += 1);
            });
            if let Some(window) = web_sys::window() {
                let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    300_000,
                );
                cb.forget();
            }
        });

        Effect::new(move |_| {
            if !ack_loaded.get() {
                acknowledged_signatures.set(load_acknowledged_signatures());
                ack_loaded.set(true);
            }
        });

        Effect::new(move |_| {
            if ack_loaded.get() {
                let set = acknowledged_signatures.get();
                save_acknowledged_signatures(&set);
            }
        });
    }

    Effect::new(move |_| {
        if let Some(Ok(items)) = regulator_updates.get() {
            let mut next_prev = HashMap::new();
            let mut changes = HashMap::new();
            let prev = previous_updates.get_untracked();

            for item in &items {
                let mut changed = Vec::new();
                if let Some(old) = prev.get(&item.authority) {
                    if old.status != item.status {
                        changed.push("status".to_string());
                    }
                    if old.headline != item.headline {
                        changed.push("headline".to_string());
                    }
                    if old.observed_at != item.observed_at {
                        changed.push("observed_at".to_string());
                    }
                }
                changes.insert(item.authority.clone(), changed);
                next_prev.insert(item.authority.clone(), item.clone());
            }

            regulator_change_map.set(changes);
            previous_updates.set(next_prev);
        }
    });

    let filtered_domains = Signal::derive(move || {
        let q = domain_query.get().to_ascii_lowercase();
        domains
            .get_value()
            .into_iter()
            .filter(|d| {
                q.is_empty()
                    || d.domain_id.to_ascii_lowercase().contains(&q)
                    || d.domain_name.to_ascii_lowercase().contains(&q)
                    || d.definition.to_ascii_lowercase().contains(&q)
            })
            .collect::<Vec<_>>()
    });

    let active_domain = Signal::derive(move || {
        let selected = selected_domain.get();
        if selected == "ALL" {
            None
        } else {
            domains
                .get_value()
                .into_iter()
                .find(|d| d.domain_id == selected)
        }
    });

    let filtered_ksbs = Signal::derive(move || {
        let selected = selected_domain.get();
        let query = ksb_query.get().to_ascii_lowercase();
        let kind = ksb_type_filter.get();

        ksbs.get_value()
            .into_iter()
            .filter(|k| selected == "ALL" || k.domain_id == selected)
            .filter(|k| {
                let t = k.ksb_type.to_ascii_lowercase();
                kind == "all" || t == kind
            })
            .filter(|k| {
                if query.is_empty() {
                    true
                } else {
                    k.ksb_id.to_ascii_lowercase().contains(&query)
                        || k.item_name.to_ascii_lowercase().contains(&query)
                        || k.item_description.to_ascii_lowercase().contains(&query)
                        || k.major_section.to_ascii_lowercase().contains(&query)
                        || k.section.to_ascii_lowercase().contains(&query)
                        || k.keywords.to_ascii_lowercase().contains(&query)
                }
            })
            .take(120)
            .collect::<Vec<_>>()
    });

    let ksb_count_text = Signal::derive(move || {
        let n = filtered_ksbs.get().len();
        if n >= 120 {
            "Showing first 120 matches".to_string()
        } else {
            format!("{} matching KSBs", n)
        }
    });

    let type_counts = Signal::derive(move || {
        let mut k = 0usize;
        let mut s = 0usize;
        let mut b = 0usize;
        for row in filtered_ksbs.get() {
            match row.ksb_type.to_ascii_lowercase().as_str() {
                "knowledge" => k += 1,
                "skill" => s += 1,
                "behavior" => b += 1,
                _ => {}
            }
        }
        (k, s, b)
    });

    let mapped_epas = Signal::derive(move || -> Vec<PvEpaDomainRow> {
        let selected = selected_domain.get();
        if selected == "ALL" {
            return Vec::new();
        }
        epa_map
            .get_value()
            .into_iter()
            .filter(|m| m.domain_id == selected)
            .collect::<Vec<_>>()
    });

    let mapped_cpas = Signal::derive(move || -> Vec<PvCpaDomainRow> {
        let selected = selected_domain.get();
        if selected == "ALL" {
            return Vec::new();
        }
        cpa_map
            .get_value()
            .into_iter()
            .filter(|m| m.domain_id == selected)
            .collect::<Vec<_>>()
    });

    let inspected_rows = Signal::derive(move || -> Vec<BTreeMap<String, String>> {
        let sheet = selected_sheet.get();
        let query = sheet_row_query.get().to_ascii_lowercase();
        let rows = sheet_data
            .get_value()
            .get(&sheet)
            .cloned()
            .unwrap_or_default();

        rows.into_iter()
            .filter(|row| {
                if query.is_empty() {
                    true
                } else {
                    row.iter().any(|(k, v)| {
                        k.to_ascii_lowercase().contains(&query)
                            || v.to_ascii_lowercase().contains(&query)
                    })
                }
            })
            .take(50)
            .collect::<Vec<_>>()
    });

    let inspected_columns = Signal::derive(move || {
        inspected_rows
            .get()
            .first()
            .map(|row| row.keys().take(8).cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    });

    view! {
        <div class="mx-auto max-w-7xl px-4 py-12">
            <header class="mb-8">
                <p class="text-[11px] font-bold text-cyan-400 uppercase tracking-[0.2em] font-mono">"Workbook Integration"</p>
                <h1 class="mt-2 text-4xl font-bold text-white font-mono uppercase tracking-tight">"PV KSB Framework (Master 2025-12-08)"</h1>
                <p class="mt-3 text-slate-400 max-w-4xl">
                    "Embedded framework with live regulator heartbeat checks. No external workbook file is required at runtime."
                </p>
                <p class="mt-2 text-xs font-mono text-slate-500">
                    {format!("Source: {} | Imported: {}", wb.source_file, wb.generated_at)}
                </p>
                <div class="mt-4 flex flex-wrap gap-3">
                    <a href="/academy/skills" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono">
                        "Open KSB Taxonomy"
                    </a>
                    <a href="/academy/epa-tracks" class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-4 py-2 text-xs font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono">
                        "EPA Tracks"
                    </a>
                    <a href="/academy/cpa-tracks" class="rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs font-bold text-amber-300 hover:text-amber-200 transition-colors uppercase tracking-widest font-mono">
                        "CPA Tracks"
                    </a>
                    <a href="/academy/guardian-bridge" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Bridge Map"
                    </a>
                    <a href="/academy/gvp-practicum" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Launch Practicum"
                    </a>
                    <a href="/vigilance/guardian" class="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-2 text-xs font-bold text-emerald-300 hover:text-emerald-200 transition-colors uppercase tracking-widest font-mono">
                        "Open Guardian"
                    </a>
                </div>
            </header>

            <section class="mb-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-6">
                <Stat label="Sheets" value=stats.sheet_count />
                <Stat label="Domains" value=stats.domain_count />
                <Stat label="KSBs" value=stats.ksb_count />
                <Stat label="EPAs" value=stats.epa_count />
                <Stat label="CPAs" value=stats.cpa_count />
                <Stat label="Integration Edges" value=stats.integration_edges />
            </section>

            <section class="mb-6 rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <div class="flex items-center justify-between gap-3">
                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Regulator Update Heartbeat"</h2>
                    <div class="flex items-center gap-2">
                        <select
                            prop:value=move || regulator_filter.get()
                            on:change=move |ev| regulator_filter.set(event_target_value(&ev))
                            class="rounded-lg border border-slate-700 bg-slate-950 px-2 py-1.5 text-[10px] font-bold text-slate-200 uppercase tracking-widest font-mono"
                        >
                            <option value="all">"All"</option>
                            <option value="new">"New Only"</option>
                            <option value="error">"Errors"</option>
                        </select>
                        <button
                            on:click=move |_| refresh_nonce.update(|n| *n += 1)
                            class="rounded-lg border border-cyan-500/30 bg-cyan-500/10 px-3 py-1.5 text-[10px] font-bold text-cyan-300 hover:text-cyan-200 transition-colors uppercase tracking-widest font-mono"
                        >
                            "Refresh Now"
                        </button>
                    </div>
                </div>
                <p class="mt-1 text-xs text-slate-500 font-mono">"Auto-refresh every 5 minutes. FDA/EMA/MHRA latest content + PMDA/TGA endpoint checks."</p>
                <Suspense fallback=|| view! {
                    <div class="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-5">
                        {(0..5).map(|_| view! {
                            <div class="h-24 rounded-lg border border-slate-800 bg-slate-950/40 animate-pulse"></div>
                        }).collect_view()}
                    </div>
                }>
                    {move || regulator_updates.get().map(|result| match result {
                        Ok(items) => {
                            let ack = acknowledged_signatures.get();
                            let unacked_new_count = items
                                .iter()
                                .filter(|i| i.is_new && !ack.contains(&regulator_signature(i)))
                                .count();
                            let new_count = items.iter().filter(|i| i.is_new).count();
                            let error_count = items.iter().filter(|i| i.status == "error").count();
                            let ok_count = items.iter().filter(|i| i.status == "ok").count();
                            let active_filter = regulator_filter.get();
                            let mut filtered = items
                                .into_iter()
                                .filter(|i| {
                                    if active_filter == "new" {
                                        i.is_new
                                    } else if active_filter == "error" {
                                        i.status == "error"
                                    } else {
                                        true
                                    }
                                })
                                .collect::<Vec<_>>();
                            filtered.sort_by_key(|i| (
                                !(i.is_new && !ack.contains(&regulator_signature(i))),
                                i.status != "error",
                                i.authority.clone(),
                            ));
                            let new_signatures = filtered
                                .iter()
                                .filter(|item| item.is_new)
                                .map(regulator_signature)
                                .collect::<Vec<_>>();
                            view! {
                                <>
                                    <div class="mt-3 flex flex-wrap gap-2 text-[10px] font-mono">
                                        <span class="rounded-full border border-emerald-500/20 bg-emerald-500/5 px-2.5 py-1 text-emerald-300">{format!("OK: {}", ok_count)}</span>
                                        <span class="rounded-full border border-cyan-500/20 bg-cyan-500/5 px-2.5 py-1 text-cyan-300">{format!("NEW: {}", new_count)}</span>
                                        <span class="rounded-full border border-blue-500/20 bg-blue-500/5 px-2.5 py-1 text-blue-300">{format!("UNACKED NEW: {}", unacked_new_count)}</span>
                                        <span class="rounded-full border border-red-500/20 bg-red-500/5 px-2.5 py-1 text-red-300">{format!("ERROR: {}", error_count)}</span>
                                        <button
                                            on:click=move |_| {
                                                acknowledged_signatures.update(|set| {
                                                    for signature in &new_signatures {
                                                        set.insert(signature.clone());
                                                    }
                                                });
                                            }
                                            class="rounded-full border border-slate-700 bg-slate-900/60 px-2.5 py-1 text-slate-300 hover:text-white transition-colors uppercase tracking-widest"
                                        >
                                            "Acknowledge New"
                                        </button>
                                    </div>
                                    <div class="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-5">
                                        {filtered.into_iter().map(|item| {
                                            let signature = regulator_signature(&item);
                                            let acknowledged = ack.contains(&signature);
                                            let sig_for_ack = signature.clone();
                                            let changes = regulator_change_map
                                                .get()
                                                .get(&item.authority)
                                                .cloned()
                                                .unwrap_or_default();
                                            view! {
                                                <RegulatorCard
                                                    item=item
                                                    changes=changes
                                                    acknowledged=acknowledged
                                                    ack_signature=sig_for_ack
                                                    acknowledged_signatures=acknowledged_signatures
                                                />
                                            }
                                        }).collect_view()}
                                    </div>
                                </>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <p class="mt-3 text-xs text-red-400 font-mono">{e.to_string()}</p>
                        }.into_any(),
                    })}
                </Suspense>
            </section>

            <section class="mb-6 rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Interactive Controls"</h2>
                <div class="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                    <input
                        type="text"
                        placeholder="Search domains..."
                        prop:value=move || domain_query.get()
                        on:input=move |ev| domain_query.set(event_target_value(&ev))
                        class="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                    />
                    <select
                        prop:value=move || selected_domain.get()
                        on:change=move |ev| selected_domain.set(event_target_value(&ev))
                        class="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-cyan-500 focus:outline-none"
                    >
                        <option value="ALL">"All Domains"</option>
                        {move || filtered_domains.get().into_iter().map(|d| {
                            let label = format!("{} — {}", d.domain_id, d.domain_name);
                            let value = d.domain_id;
                            view! { <option value=value>{label}</option> }
                        }).collect_view()}
                    </select>
                    <input
                        type="text"
                        placeholder="Search KSB corpus..."
                        prop:value=move || ksb_query.get()
                        on:input=move |ev| ksb_query.set(event_target_value(&ev))
                        class="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                    />
                    <select
                        prop:value=move || ksb_type_filter.get()
                        on:change=move |ev| ksb_type_filter.set(event_target_value(&ev))
                        class="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-cyan-500 focus:outline-none"
                    >
                        <option value="all">"All Types"</option>
                        <option value="knowledge">"Knowledge"</option>
                        <option value="skill">"Skill"</option>
                        <option value="behavior">"Behavior"</option>
                    </select>
                </div>

                <div class="mt-3 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                    <select
                        prop:value=move || selected_sheet.get()
                        on:change=move |ev| selected_sheet.set(event_target_value(&ev))
                        class="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-cyan-500 focus:outline-none"
                    >
                        {wb.sheet_names.iter().map(|name| {
                            let value = name.clone();
                            let display = name.clone();
                            view! { <option value=value>{display}</option> }
                        }).collect_view()}
                    </select>
                    <input
                        type="text"
                        placeholder="Search selected sheet rows..."
                        prop:value=move || sheet_row_query.get()
                        on:input=move |ev| sheet_row_query.set(event_target_value(&ev))
                        class="md:col-span-2 xl:col-span-3 rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                    />
                </div>

                <div class="mt-4 flex flex-wrap items-center gap-3 text-xs font-mono">
                    <span class="rounded-full border border-slate-700 bg-slate-900/50 px-3 py-1 text-slate-300">{move || ksb_count_text.get()}</span>
                    <span class="rounded-full border border-cyan-500/20 bg-cyan-500/5 px-3 py-1 text-cyan-300">{move || { let (k, _, _) = type_counts.get(); format!("Knowledge: {}", k) }}</span>
                    <span class="rounded-full border border-amber-500/20 bg-amber-500/5 px-3 py-1 text-amber-300">{move || { let (_, s, _) = type_counts.get(); format!("Skill: {}", s) }}</span>
                    <span class="rounded-full border border-violet-500/20 bg-violet-500/5 px-3 py-1 text-violet-300">{move || { let (_, _, b) = type_counts.get(); format!("Behavior: {}", b) }}</span>
                </div>
            </section>

            <div class="grid gap-6 lg:grid-cols-3 mb-6">
                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5 lg:col-span-2">
                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Filtered KSB Corpus"</h2>
                    <div class="mt-4 grid gap-3 md:grid-cols-2">
                        {move || filtered_ksbs.get().into_iter().map(|row| view! { <KsbCard row=row /> }).collect_view()}
                    </div>
                </section>

                <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5 lg:col-span-1">
                    <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Active Domain"</h2>
                    <div class="mt-4">
                        {move || match active_domain.get() {
                            Some(d) => view! { <DomainSummaryCard domain=d /> }.into_any(),
                            None => view! {
                                <div class="rounded-lg border border-slate-800 bg-slate-950/40 p-4">
                                    <p class="text-xs text-slate-500 font-mono">"Select a domain to inspect EPA/CPA mapping and launch guided Guardian execution."</p>
                                </div>
                            }.into_any(),
                        }}
                    </div>

                    <h3 class="mt-5 text-xs font-bold text-slate-500 uppercase tracking-widest font-mono">"EPA Mapping"</h3>
                    <div class="mt-2 space-y-2 max-h-44 overflow-y-auto pr-1">
                        {move || mapped_epas.get().into_iter().map(|m| view! { <MappingRow label=m.epa_id subtitle=m.epa_name role=m.role /> }).collect_view()}
                    </div>

                    <h3 class="mt-5 text-xs font-bold text-slate-500 uppercase tracking-widest font-mono">"CPA Mapping"</h3>
                    <div class="mt-2 space-y-2 max-h-44 overflow-y-auto pr-1">
                        {move || mapped_cpas.get().into_iter().map(|m| view! { <MappingRow label=m.cpa_id subtitle=m.cpa_name role=m.role /> }).collect_view()}
                    </div>
                </section>
            </div>

            <section class="mb-6 rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Sheet Inspector"</h2>
                <p class="mt-1 text-xs text-slate-500 font-mono">{move || format!("Sheet: {} | Preview: up to 50 filtered rows", selected_sheet.get())}</p>
                <div class="mt-4 overflow-x-auto">
                    <table class="w-full text-xs">
                        <thead>
                            <tr class="border-b border-slate-800 text-left">
                                {move || inspected_columns.get().into_iter().map(|col| view! {
                                    <th class="pb-2 pr-4 text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono whitespace-nowrap">{col}</th>
                                }).collect_view()}
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-900">
                            {move || {
                                let cols = inspected_columns.get();
                                inspected_rows.get().into_iter().map(|row| {
                                    view! {
                                        <tr>
                                            {cols.iter().map(|col| {
                                                let value = row.get(col).cloned().unwrap_or_default();
                                                view! {
                                                    <td class="py-2 pr-4 text-slate-300 align-top min-w-[140px] max-w-[280px]">
                                                        <span class="line-clamp-2">{value}</span>
                                                    </td>
                                                }
                                            }).collect_view()}
                                        </tr>
                                    }
                                }).collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </section>

            <section class="rounded-2xl border border-slate-800 bg-slate-900/40 p-5">
                <h2 class="text-sm font-bold text-white uppercase tracking-widest font-mono">"Sheet Row Counts"</h2>
                <div class="mt-4 grid gap-2 md:grid-cols-2 xl:grid-cols-3">
                    {sheets.into_iter().map(|(name, rows)| view! {
                        <div class="flex items-center justify-between rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2">
                            <span class="text-xs text-slate-300 truncate pr-3">{name}</span>
                            <span class="text-[10px] font-mono text-slate-500">{rows}</span>
                        </div>
                    }).collect_view()}
                </div>
            </section>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: usize) -> impl IntoView {
    view! {
        <article class="rounded-xl border border-slate-800 bg-slate-900/40 p-4 text-center">
            <p class="text-[10px] text-slate-500 font-bold uppercase tracking-widest font-mono">{label}</p>
            <p class="mt-1 text-2xl font-bold text-white font-mono">{value}</p>
        </article>
    }
}

#[component]
fn RegulatorCard(
    item: RegulatorFeedStatus,
    changes: Vec<String>,
    acknowledged: bool,
    ack_signature: String,
    acknowledged_signatures: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let cls = match item.status.as_str() {
        "ok" => "text-emerald-400 border-emerald-500/20 bg-emerald-500/5",
        "empty" => "text-amber-400 border-amber-500/20 bg-amber-500/5",
        _ => "text-red-400 border-red-500/20 bg-red-500/5",
    };

    view! {
        <article class=format!("rounded-lg border p-3 {}", cls)>
            <div class="flex items-center justify-between gap-2">
                <span class="text-[10px] font-bold uppercase tracking-widest font-mono">{item.authority.clone()}</span>
                <div class="flex items-center gap-2">
                    {if item.is_new && !acknowledged {
                        view! {
                            <span class="rounded-full border border-cyan-500/30 bg-cyan-500/10 px-2 py-0.5 text-[8px] font-bold uppercase tracking-widest font-mono text-cyan-300">"NEW"</span>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                    <span class="text-[9px] uppercase tracking-widest font-mono opacity-80">{item.status.clone()}</span>
                </div>
            </div>
            <p class="mt-2 text-[11px] leading-snug text-slate-200 line-clamp-3">{item.headline.clone()}</p>
            {if !changes.is_empty() {
                let summary = changes.join(", ");
                view! {
                    <p class="mt-1 text-[9px] text-cyan-300 font-mono uppercase tracking-widest">
                        {format!("Changed: {}", summary)}
                    </p>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <p class="mt-2 text-[9px] text-slate-500 font-mono">{item.observed_at.clone()}</p>
            {if item.is_new && !acknowledged {
                view! {
                    <button
                        on:click=move |_| {
                            acknowledged_signatures.update(|set| {
                                set.insert(ack_signature.clone());
                            });
                        }
                        class="mt-2 inline-flex rounded border border-slate-700 bg-slate-900/60 px-2 py-1 text-[9px] font-bold uppercase tracking-widest font-mono text-slate-300 hover:text-white transition-colors"
                    >
                        "Acknowledge"
                    </button>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <a href=item.source_url target="_blank" rel="noopener noreferrer" class="mt-2 inline-flex text-[9px] font-bold uppercase tracking-widest font-mono text-cyan-300 hover:text-cyan-200">
                "Source ↗"
            </a>
        </article>
    }
}

#[component]
fn KsbCard(row: PvKsbRow) -> impl IntoView {
    let kind = row.ksb_type.to_ascii_lowercase();
    let badge_cls = match kind.as_str() {
        "knowledge" => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
        "skill" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
        "behavior" => "text-violet-400 bg-violet-500/10 border-violet-500/20",
        _ => "text-slate-400 bg-slate-800 border-slate-700",
    };
    let domain = row.domain_id.clone();
    let title = row.item_name.clone();
    let desc = row.item_description.clone();
    let sec = if row.section.is_empty() {
        row.major_section.clone()
    } else {
        row.section.clone()
    };
    let ksb_href = format!("/academy/skills/{}", row.ksb_id);
    let guardian_href = format!(
        "/vigilance/guardian?module={}&event={}&count=1",
        domain,
        url_encode_fragment(&title)
    );

    view! {
        <article class="rounded-xl border border-slate-800 bg-slate-950/40 p-4">
            <div class="flex items-start justify-between gap-3">
                <span class=format!("rounded-full border px-2 py-0.5 text-[10px] font-bold uppercase tracking-widest font-mono {}", badge_cls)>
                    {row.ksb_type.clone()}
                </span>
                <span class="text-[10px] font-mono text-slate-500">{domain}</span>
            </div>
            <h3 class="mt-2 text-sm font-semibold text-white leading-snug">{title}</h3>
            <p class="mt-1 text-[11px] text-slate-400 line-clamp-3">{desc}</p>
            <p class="mt-2 text-[10px] font-mono text-slate-500 uppercase tracking-widest">{sec}</p>
            <div class="mt-3 flex items-center justify-between">
                <a href=ksb_href class="text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest font-mono">
                    "Open KSB"
                </a>
                <a href=guardian_href class="text-xs font-bold text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest font-mono">
                    "Launch Guardian"
                </a>
            </div>
        </article>
    }
}

#[component]
fn DomainSummaryCard(domain: PvDomainRow) -> impl IntoView {
    let guardian_href = format!(
        "/vigilance/guardian?module={}&event=domain-{}-execution&count=3",
        domain.domain_id,
        domain.domain_id.to_ascii_lowercase()
    );

    view! {
        <article class="rounded-lg border border-slate-800 bg-slate-950/40 p-4">
            <p class="text-[10px] font-bold text-cyan-400 uppercase tracking-widest font-mono">{domain.domain_id.clone()}</p>
            <h3 class="mt-1 text-sm font-semibold text-white">{domain.domain_name.clone()}</h3>
            <p class="mt-1 text-xs text-slate-400 line-clamp-4">{domain.definition.clone()}</p>
            <p class="mt-2 text-[10px] font-mono text-slate-500">{format!("Workbook Total KSBs: {}", domain.total_ksbs)}</p>
            <a href=guardian_href class="mt-3 inline-flex text-xs font-bold text-emerald-400 hover:text-emerald-300 transition-colors uppercase tracking-widest font-mono">
                "Launch Domain in Guardian"
            </a>
        </article>
    }
}

#[component]
fn MappingRow(label: String, subtitle: String, role: String) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-950/40 px-3 py-2">
            <div class="flex items-center justify-between gap-2">
                <span class="text-[10px] font-bold text-slate-300 font-mono uppercase tracking-widest">{label}</span>
                <span class="text-[9px] text-slate-500 font-mono uppercase">{role}</span>
            </div>
            <p class="mt-1 text-xs text-slate-400 line-clamp-2">{subtitle}</p>
        </div>
    }
}

fn url_encode_fragment(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push('-');
        }
    }
    out
}
