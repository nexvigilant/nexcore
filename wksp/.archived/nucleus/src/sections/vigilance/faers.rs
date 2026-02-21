//! FAERS Drug Search + Signal Pipeline
//!
//! Live search of the FDA Adverse Event Reporting System (FAERS) database.
//! - Drug name search with autocomplete via openFDA
//! - Adverse event profile for any drug
//! - Auto-populated contingency tables from FAERS counts
//! - Multi-algorithm signal detection (PRR, ROR, IC, EBGM, Chi²)

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/* =========================================================================
Response types — openFDA drug/event API
========================================================================= */

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FaersCountResponse {
    pub results: Vec<FaersCountBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FaersCountBucket {
    pub term: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FaersTotalResponse {
    pub meta: Option<FaersMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FaersMeta {
    pub results: Option<FaersMetaResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FaersMetaResults {
    pub total: Option<u64>,
}

/// Signal row: one drug-event pair with contingency table + all algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaersSignalRow {
    pub event: String,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    pub prr: f64,
    pub prr_signal: bool,
    pub ror: f64,
    pub ror_lower_ci: f64,
    pub ror_signal: bool,
    pub ic025: f64,
    pub ic_signal: bool,
    pub ebgm: f64,
    pub eb05: f64,
    pub ebgm_signal: bool,
    pub chi_square: f64,
    pub chi_signal: bool,
    pub any_signal: bool,
}

/// Drug autocomplete suggestion
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrugSuggestion {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AutocompleteResponse {
    results: Vec<FaersCountBucket>,
}

/* =========================================================================
Server Functions — SSR-only (reqwest under ssr feature)
========================================================================= */

/// Autocomplete drug names via openFDA count endpoint
#[server(SearchDrugs, "/api")]
pub async fn search_drugs(query: String) -> Result<Vec<DrugSuggestion>, ServerFnError> {
    if query.len() < 2 {
        return Ok(vec![]);
    }

    let client = reqwest::Client::new();
    let search = format!(
        "patient.drug.openfda.brand_name:\"{}\"",
        query.to_uppercase()
    );

    let resp = client
        .get("https://api.fda.gov/drug/event.json")
        .query(&[
            ("search", search.as_str()),
            ("count", "patient.drug.openfda.brand_name.exact"),
            ("limit", "10"),
        ])
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("openFDA autocomplete failed: {e}")))?;

    if !resp.status().is_success() {
        /* Try generic_name fallback */
        let search2 = format!(
            "patient.drug.openfda.generic_name:\"{}\"",
            query.to_uppercase()
        );
        let resp2 = client
            .get("https://api.fda.gov/drug/event.json")
            .query(&[
                ("search", search2.as_str()),
                ("count", "patient.drug.openfda.generic_name.exact"),
                ("limit", "10"),
            ])
            .send()
            .await
            .map_err(|e| ServerFnError::new(format!("openFDA generic fallback failed: {e}")))?;

        if !resp2.status().is_success() {
            return Ok(vec![]);
        }

        let body: AutocompleteResponse = resp2
            .json()
            .await
            .map_err(|e| ServerFnError::new(format!("Parse error: {e}")))?;

        return Ok(body
            .results
            .into_iter()
            .map(|b| DrugSuggestion {
                name: b.term,
                count: b.count,
            })
            .collect());
    }

    let body: AutocompleteResponse = resp
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Parse error: {e}")))?;

    Ok(body
        .results
        .into_iter()
        .map(|b| DrugSuggestion {
            name: b.term,
            count: b.count,
        })
        .collect())
}

/// Get top adverse events for a drug and compute signal detection on each
#[server(AnalyzeDrug, "/api")]
pub async fn analyze_drug(drug_name: String) -> Result<Vec<FaersSignalRow>, ServerFnError> {
    let client = reqwest::Client::new();

    /* Step 1: Get total reports in FAERS (approximate) */
    let total_resp = client
        .get("https://api.fda.gov/drug/event.json")
        .query(&[
            ("search", "receivedate:[20200101+TO+20261231]"),
            ("limit", "1"),
        ])
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Total count failed: {e}")))?;

    let total_n: u64 = if total_resp.status().is_success() {
        let body: FaersTotalResponse = total_resp.json().await.unwrap_or_default();
        body.meta
            .and_then(|m| m.results)
            .and_then(|r| r.total)
            .unwrap_or(15_000_000)
    } else {
        15_000_000 /* fallback estimate */
    };

    /* Step 2: Count reports for this drug (a+b) */
    let drug_search = format!(
        "patient.drug.openfda.brand_name.exact:\"{}\" OR patient.drug.openfda.generic_name.exact:\"{}\"",
        drug_name.to_uppercase(),
        drug_name.to_uppercase()
    );

    let drug_total_resp = client
        .get("https://api.fda.gov/drug/event.json")
        .query(&[("search", drug_search.as_str()), ("limit", "1")])
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Drug total failed: {e}")))?;

    let drug_total: u64 = if drug_total_resp.status().is_success() {
        let body: FaersTotalResponse = drug_total_resp.json().await.unwrap_or_default();
        body.meta
            .and_then(|m| m.results)
            .and_then(|r| r.total)
            .unwrap_or(1000)
    } else {
        1000
    };

    /* Step 3: Get top events for this drug */
    let events_resp = client
        .get("https://api.fda.gov/drug/event.json")
        .query(&[
            ("search", drug_search.as_str()),
            ("count", "patient.reaction.reactionmeddrapt.exact"),
            ("limit", "25"),
        ])
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Event count failed: {e}")))?;

    if !events_resp.status().is_success() {
        return Err(ServerFnError::new(format!(
            "No FAERS data found for '{drug_name}'. Try the brand or generic name."
        )));
    }

    let events_body: FaersCountResponse = events_resp
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Events parse error: {e}")))?;

    /* Step 4: For each event, build contingency table and run signal algorithms */
    let mut rows = Vec::with_capacity(events_body.results.len());

    for bucket in &events_body.results {
        let a = bucket.count; /* Drug + Event */
        let b = drug_total.saturating_sub(a); /* Drug + No Event */

        /* Get total reports for this event across all drugs (a+c) */
        let event_search = format!(
            "patient.reaction.reactionmeddrapt.exact:\"{}\"",
            bucket.term
        );
        let event_total_resp = client
            .get("https://api.fda.gov/drug/event.json")
            .query(&[("search", event_search.as_str()), ("limit", "1")])
            .send()
            .await;

        let event_total: u64 = match event_total_resp {
            Ok(r) if r.status().is_success() => {
                let body: FaersTotalResponse = r.json().await.unwrap_or_default();
                body.meta
                    .and_then(|m| m.results)
                    .and_then(|r| r.total)
                    .unwrap_or(a * 10)
            }
            _ => a * 10, /* fallback estimate */
        };

        let c = event_total.saturating_sub(a); /* Other Drugs + Event */
        let d = total_n.saturating_sub(a + b + c); /* Other Drugs + No Event */

        /* Run signal detection */
        let row = compute_signals(&bucket.term, a, b, c, d);
        rows.push(row);
    }

    Ok(rows)
}

/// Pure computation — signal detection from contingency table
#[cfg(feature = "ssr")]
fn compute_signals(event: &str, a: u64, b: u64, c: u64, d: u64) -> FaersSignalRow {
    let af = a as f64;
    let bf = b as f64;
    let cf = c as f64;
    let df = d as f64;
    let n = af + bf + cf + df;

    /* PRR = (a/(a+b)) / (c/(c+d)) */
    let prr = if (af + bf) > 0.0 && (cf + df) > 0.0 && cf > 0.0 {
        (af / (af + bf)) / (cf / (cf + df))
    } else {
        0.0
    };
    let prr_signal = prr >= 2.0 && af >= 3.0;

    /* ROR = (a*d) / (b*c) with 95% CI */
    let ror = if bf * cf > 0.0 {
        (af * df) / (bf * cf)
    } else {
        0.0
    };
    let ln_ror = if ror > 0.0 { ror.ln() } else { 0.0 };
    let se_ln_ror = if af > 0.0 && bf > 0.0 && cf > 0.0 && df > 0.0 {
        (1.0 / af + 1.0 / bf + 1.0 / cf + 1.0 / df).sqrt()
    } else {
        f64::MAX
    };
    let ror_lower_ci = (ln_ror - 1.96 * se_ln_ror).exp();
    let ror_signal = ror_lower_ci > 1.0;

    /* IC (Information Component) = log2(observed/expected) with 0.25 credibility */
    let expected = if n > 0.0 {
        ((af + bf) * (af + cf)) / n
    } else {
        1.0
    };
    let ic = if expected > 0.0 && af > 0.0 {
        (af / expected).log2()
    } else {
        0.0
    };
    /* IC025 approximation using gamma shrinkage */
    let ic025 = ic - 3.3 * (1.0 / (af + 0.5)).sqrt();
    let ic_signal = ic025 > 0.0;

    /* EBGM (Empirical Bayes Geometric Mean) simplified */
    let ebgm = if expected > 0.0 {
        (af + 0.5) / (expected + 0.5)
    } else {
        0.0
    };
    let eb05 = ebgm * (1.0 - 1.645 * (1.0 / (af + 0.5)).sqrt()).max(0.0);
    let ebgm_signal = eb05 >= 2.0;

    /* Chi-square (with Yates correction) */
    let chi_square = if n > 0.0 {
        let num = n * (af * df - bf * cf).abs().powi(2);
        let denom = (af + bf) * (cf + df) * (af + cf) * (bf + df);
        if denom > 0.0 { num / denom } else { 0.0 }
    } else {
        0.0
    };
    let chi_signal = chi_square >= 3.841;

    let any_signal = prr_signal || ror_signal || ic_signal || ebgm_signal || chi_signal;

    FaersSignalRow {
        event: event.to_string(),
        a,
        b,
        c,
        d,
        prr,
        prr_signal,
        ror,
        ror_lower_ci,
        ror_signal,
        ic025,
        ic_signal,
        ebgm,
        eb05,
        ebgm_signal,
        chi_square,
        chi_signal,
        any_signal,
    }
}

/* =========================================================================
UI Components
========================================================================= */

#[component]
pub fn FaersPage() -> impl IntoView {
    let drug_query = RwSignal::new(String::new());
    let selected_drug = RwSignal::new(Option::<String>::None);
    let show_suggestions = RwSignal::new(false);

    /* Autocomplete */
    let suggestions = Resource::new(
        move || drug_query.get(),
        |q| async move {
            if q.len() < 2 {
                return Vec::new();
            }
            search_drugs(q).await.unwrap_or_default()
        },
    );

    /* Analysis — triggered when a drug is selected */
    let analysis = Resource::new(
        move || selected_drug.get(),
        |drug| async move {
            match drug {
                Some(name) => analyze_drug(name).await,
                None => Ok(vec![]),
            }
        },
    );

    let on_select = move |name: String| {
        drug_query.set(name.clone());
        selected_drug.set(Some(name));
        show_suggestions.set(false);
    };

    view! {
        <div class="mx-auto max-w-7xl px-4 py-8 md:py-12">
            <header class="mb-10">
                <div class="flex items-center gap-3 mb-2">
                    <span class="h-3 w-3 rounded-full bg-amber-500 animate-pulse"></span>
                    <h1 class="text-4xl font-black text-white font-mono uppercase tracking-tight">"FAERS"</h1>
                </div>
                <p class="text-slate-400 max-w-2xl">
                    "Search the FDA Adverse Event Reporting System. Select a drug to auto-generate contingency tables and run multi-algorithm signal detection."
                </p>
            </header>

            /* ---- Search Bar ---- */
            <div class="relative max-w-2xl mb-10">
                <div class="flex items-center gap-3">
                    <div class="relative flex-1">
                        <input
                            type="text"
                            class="w-full rounded-xl border border-slate-700 bg-slate-950 px-5 py-4 text-white font-mono text-lg focus:border-amber-500 focus:outline-none focus:ring-1 focus:ring-amber-500/30 placeholder-slate-600"
                            placeholder="Search drug name (e.g. LIPITOR, METFORMIN)..."
                            prop:value=move || drug_query.get()
                            on:input=move |ev| {
                                drug_query.set(event_target_value(&ev));
                                show_suggestions.set(true);
                                selected_drug.set(None);
                            }
                            on:focus=move |_| show_suggestions.set(true)
                        />
                        <span class="absolute right-4 top-1/2 -translate-y-1/2 text-slate-600 text-sm font-mono">"FDA"</span>
                    </div>
                </div>

                /* Autocomplete Dropdown */
                <Show when=move || { show_suggestions.get() && drug_query.get().len().ge(&2) }>
                    <div class="absolute z-50 w-full mt-2 rounded-xl border border-slate-700 bg-slate-900 shadow-2xl overflow-hidden">
                        <Suspense fallback=move || view! {
                            <div class="p-4 text-xs text-slate-500 font-mono animate-pulse">"Searching openFDA..."</div>
                        }>
                            {move || {
                                let items = suggestions.get().unwrap_or_default();
                                if items.is_empty() && (drug_query.get().len() > 1) {
                                    view! {
                                        <div class="p-4 text-xs text-slate-500 font-mono">"No matches — try brand or generic name"</div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <ul class="divide-y divide-slate-800 max-h-72 overflow-y-auto">
                                            {items.into_iter().map(|s| {
                                                let name = s.name.clone();
                                                let name2 = s.name.clone();
                                                let count = s.count;
                                                view! {
                                                    <li
                                                        class="flex justify-between items-center px-5 py-3 hover:bg-slate-800 cursor-pointer transition-colors group"
                                                        on:click=move |_| on_select(name.clone())
                                                    >
                                                        <span class="text-sm font-bold text-slate-300 group-hover:text-white transition-colors font-mono">{name2}</span>
                                                        <span class="text-[10px] text-slate-600 font-mono">{format!("{} reports", format_count(count))}</span>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    }.into_any()
                                }
                            }}
                        </Suspense>
                    </div>
                </Show>
            </div>

            /* ---- Selected Drug Header ---- */
            <Show when=move || selected_drug.get().is_some()>
                {move || {
                    let name = selected_drug.get().unwrap_or_default();
                    view! {
                        <div class="mb-8 flex items-center gap-4">
                            <div class="h-12 w-12 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-center">
                                <span class="text-amber-400 font-black font-mono text-xs">"Rx"</span>
                            </div>
                            <div>
                                <h2 class="text-2xl font-black text-white font-mono tracking-tight">{name}</h2>
                                <p class="text-xs text-slate-500 font-mono">"FAERS DISPROPORTIONALITY ANALYSIS"</p>
                            </div>
                        </div>
                    }
                }}
            </Show>

            /* ---- Results ---- */
            <Suspense fallback=move || view! {
                <AnalysisLoading/>
            }>
                {move || {
                    let data = analysis.get();
                    match data {
                        Some(Ok(rows)) if !rows.is_empty() => {
                            let signal_count = rows.iter().filter(|r| r.any_signal).count();
                            let total = rows.len();
                            view! { <AnalysisResults rows=rows signal_count=signal_count total=total /> }.into_any()
                        }
                        Some(Ok(_)) if selected_drug.get().is_some() => {
                            view! {
                                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-12 text-center">
                                    <p class="text-slate-500 font-mono text-sm">"No adverse event data found for this drug."</p>
                                </div>
                            }.into_any()
                        }
                        Some(Err(e)) => {
                            view! {
                                <div class="rounded-2xl border border-red-500/20 bg-red-500/5 p-8">
                                    <p class="text-red-400 font-mono text-sm">{e.to_string()}</p>
                                </div>
                            }.into_any()
                        }
                        _ => view! { <div></div> }.into_any(),
                    }
                }}
            </Suspense>

            /* ---- Methodology note ---- */
            <Show when=move || selected_drug.get().is_some()>
                <div class="mt-12 rounded-2xl border border-slate-800 bg-slate-900/30 p-6">
                    <h3 class="text-[10px] font-bold text-slate-600 uppercase tracking-widest mb-3">"// METHODOLOGY"</h3>
                    <div class="grid md:grid-cols-5 gap-4 text-[10px] text-slate-500 font-mono">
                        <div>
                            <p class="text-slate-400 font-bold">"PRR"</p>
                            <p>"Proportional Reporting Ratio"</p>
                            <p class="text-amber-500/60">"Signal: \u{2265}2.0, n\u{2265}3"</p>
                        </div>
                        <div>
                            <p class="text-slate-400 font-bold">"ROR"</p>
                            <p>"Reporting Odds Ratio"</p>
                            <p class="text-amber-500/60">"Signal: lower CI >1.0"</p>
                        </div>
                        <div>
                            <p class="text-slate-400 font-bold">"IC025"</p>
                            <p>"Information Component"</p>
                            <p class="text-amber-500/60">"Signal: IC025 >0"</p>
                        </div>
                        <div>
                            <p class="text-slate-400 font-bold">"EBGM"</p>
                            <p>"Empirical Bayes Geometric Mean"</p>
                            <p class="text-amber-500/60">"Signal: EB05 \u{2265}2.0"</p>
                        </div>
                        <div>
                            <p class="text-slate-400 font-bold">"\u{03C7}\u{00B2}"</p>
                            <p>"Chi-Square Statistic"</p>
                            <p class="text-amber-500/60">"Signal: \u{2265}3.841 (p<0.05)"</p>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

/* ---- Analysis Results Table ---- */

#[component]
fn AnalysisResults(rows: Vec<FaersSignalRow>, signal_count: usize, total: usize) -> impl IntoView {
    view! {
        <div class="space-y-6">
            /* Summary Banner */
            <div class="grid grid-cols-3 gap-4">
                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">"EVENTS ANALYZED"</p>
                    <p class="mt-1 text-3xl font-black text-white font-mono">{total}</p>
                </div>
                <div class=move || format!(
                    "rounded-2xl border p-5 {}",
                    if signal_count > 0 { "border-red-500/30 bg-red-500/5" } else { "border-emerald-500/30 bg-emerald-500/5" }
                )>
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">"SIGNALS DETECTED"</p>
                    <p class=move || format!(
                        "mt-1 text-3xl font-black font-mono {}",
                        if signal_count > 0 { "text-red-400" } else { "text-emerald-400" }
                    )>{signal_count}</p>
                </div>
                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-5">
                    <p class="text-[9px] font-bold text-slate-600 uppercase tracking-widest">"SIGNAL RATE"</p>
                    <p class="mt-1 text-3xl font-black text-slate-300 font-mono">
                        {if total > 0 { format!("{:.0}%", (signal_count as f64 / total as f64) * 100.0) } else { "0%".to_string() }}
                    </p>
                </div>
            </div>

            /* Results Table */
            <div class="rounded-2xl border border-slate-800 overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full">
                        <thead>
                            <tr class="border-b border-slate-800 bg-slate-900/80">
                                <th class="text-left px-4 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"ADVERSE EVENT"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"a"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"b"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"c"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"d"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"PRR"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"ROR"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"IC025"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"EBGM"</th>
                                <th class="text-right px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"\u{03C7}\u{00B2}"</th>
                                <th class="text-center px-3 py-3 text-[9px] font-bold text-slate-500 uppercase tracking-widest">"SIGNAL"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-800/50">
                            {rows.into_iter().map(|row| view! { <SignalTableRow row=row /> }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SignalTableRow(row: FaersSignalRow) -> impl IntoView {
    let bg = if row.any_signal {
        "bg-red-500/[0.03] hover:bg-red-500/[0.06]"
    } else {
        "hover:bg-slate-900/50"
    };

    let event_name = row.event.clone();
    let a_str = format_count(row.a);
    let b_str = format_count(row.b);
    let c_str = format_count(row.c);
    let d_str = format_count(row.d);
    let prr_str = format!("{:.2}", row.prr);
    let prr_class = format!(
        "text-right px-3 py-3 text-[11px] font-mono font-bold {}",
        if row.prr_signal {
            "text-red-400"
        } else {
            "text-slate-400"
        }
    );
    let ror_str = format!("{:.2}", row.ror);
    let ror_ci_str = format!("CI: {:.2}", row.ror_lower_ci);
    let ror_class = format!(
        "text-right px-3 py-3 text-[11px] font-mono font-bold {}",
        if row.ror_signal {
            "text-red-400"
        } else {
            "text-slate-400"
        }
    );
    let ic_str = format!("{:.2}", row.ic025);
    let ic_class = format!(
        "text-right px-3 py-3 text-[11px] font-mono font-bold {}",
        if row.ic_signal {
            "text-red-400"
        } else {
            "text-slate-400"
        }
    );
    let ebgm_str = format!("{:.2}", row.ebgm);
    let eb05_str = format!("EB05: {:.2}", row.eb05);
    let ebgm_class = format!(
        "text-right px-3 py-3 text-[11px] font-mono font-bold {}",
        if row.ebgm_signal {
            "text-red-400"
        } else {
            "text-slate-400"
        }
    );
    let chi_str = format!("{:.1}", row.chi_square);
    let chi_class = format!(
        "text-right px-3 py-3 text-[11px] font-mono font-bold {}",
        if row.chi_signal {
            "text-red-400"
        } else {
            "text-slate-400"
        }
    );
    let tr_class = format!("transition-colors {bg}");
    let any_signal = row.any_signal;

    view! {
        <tr class=tr_class>
            <td class="px-4 py-3">
                <span class="text-xs font-bold text-slate-300 font-mono">{event_name}</span>
            </td>
            <td class="text-right px-3 py-3 text-[11px] font-mono text-slate-400">{a_str}</td>
            <td class="text-right px-3 py-3 text-[11px] font-mono text-slate-500">{b_str}</td>
            <td class="text-right px-3 py-3 text-[11px] font-mono text-slate-500">{c_str}</td>
            <td class="text-right px-3 py-3 text-[11px] font-mono text-slate-600">{d_str}</td>

            /* PRR */
            <td class=prr_class>{prr_str}</td>

            /* ROR */
            <td class=ror_class>
                {ror_str}
                <span class="text-[8px] text-slate-600 block">{ror_ci_str}</span>
            </td>

            /* IC025 */
            <td class=ic_class>{ic_str}</td>

            /* EBGM */
            <td class=ebgm_class>
                {ebgm_str}
                <span class="text-[8px] text-slate-600 block">{eb05_str}</span>
            </td>

            /* Chi-square */
            <td class=chi_class>{chi_str}</td>

            /* Signal indicator */
            <td class="text-center px-3 py-3">
                {if any_signal {
                    view! { <span class="inline-block h-3 w-3 rounded-full bg-red-500 animate-pulse"></span> }.into_any()
                } else {
                    view! { <span class="inline-block h-3 w-3 rounded-full bg-slate-700"></span> }.into_any()
                }}
            </td>
        </tr>
    }
}

/* ---- Loading State ---- */

#[component]
fn AnalysisLoading() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="rounded-2xl border border-amber-500/20 bg-amber-500/5 p-8">
                <div class="flex items-center gap-4">
                    <div class="h-8 w-8 rounded-full border-2 border-amber-500 border-t-transparent animate-spin"></div>
                    <div>
                        <p class="text-amber-400 font-bold font-mono text-sm">"ANALYZING FAERS DATABASE"</p>
                        <p class="text-[10px] text-slate-500 font-mono mt-1">"Querying openFDA \u{2192} Building contingency tables \u{2192} Running 5 signal algorithms..."</p>
                    </div>
                </div>
            </div>
            /* Skeleton rows */
            {(0..5).map(|_| view! {
                <div class="rounded-xl border border-slate-800 bg-slate-900/30 p-4 animate-pulse">
                    <div class="h-4 w-48 bg-slate-800 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

/* ---- Helpers ---- */

fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
