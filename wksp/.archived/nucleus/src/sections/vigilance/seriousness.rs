//! Seriousness Classification — ICH E2A criteria for adverse event seriousness.
//! Healthcare professionals use this to determine if an event requires expedited reporting.

use crate::api_client::{SeriousnessRequest, SeriousnessResult};
use leptos::prelude::*;

#[server(ClassifySeriousness, "/api")]
pub async fn classify_seriousness(
    event_description: String,
    resulted_in_death: bool,
    life_threatening: bool,
    hospitalization: bool,
    disability: bool,
    congenital_anomaly: bool,
    other_medically_important: bool,
) -> Result<SeriousnessResult, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);
    let req = SeriousnessRequest {
        event_description,
        resulted_in_death,
        life_threatening,
        hospitalization,
        disability,
        congenital_anomaly,
        other_medically_important,
    };
    client.seriousness(&req).await.map_err(ServerFnError::new)
}

#[component]
pub fn SeriousnessPage() -> impl IntoView {
    let description = RwSignal::new(String::new());
    let death = RwSignal::new(false);
    let life_threat = RwSignal::new(false);
    let hospital = RwSignal::new(false);
    let disability = RwSignal::new(false);
    let congenital = RwSignal::new(false);
    let other = RwSignal::new(false);

    let action = ServerAction::<ClassifySeriousness>::new();
    let result = action.value();

    /* Local seriousness preview (client-side, before server call) */
    let any_serious = Signal::derive(move || {
        death.get()
            || life_threat.get()
            || hospital.get()
            || disability.get()
            || congenital.get()
            || other.get()
    });

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <header class="mb-8 text-center">
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">
                    "Seriousness Classification"
                </h1>
                <p class="mt-2 text-slate-400">
                    "ICH E2A criteria — determine if an adverse event meets seriousness thresholds for expedited reporting"
                </p>
            </header>

            <div class="grid gap-6 lg:grid-cols-2">
                /* Input section */
                <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Event Details"</h2>

                    <div class="mb-6">
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5">"Event Description"</label>
                        <textarea
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-amber-500 focus:outline-none font-mono h-24 resize-none placeholder:text-slate-600"
                            placeholder="Describe the adverse event..."
                            prop:value=move || description.get()
                            on:input=move |ev| description.set(event_target_value(&ev))
                        />
                    </div>

                    <h3 class="text-xs font-bold text-slate-400 mb-4 uppercase tracking-wide">"ICH E2A Seriousness Criteria"</h3>
                    <div class="space-y-3">
                        <CriterionCheck label="Results in death" signal=death icon="\u{2620}" />
                        <CriterionCheck label="Is life-threatening" signal=life_threat icon="\u{26a0}" />
                        <CriterionCheck label="Requires or prolongs hospitalization" signal=hospital icon="\u{1f3e5}" />
                        <CriterionCheck label="Results in persistent or significant disability/incapacity" signal=disability icon="\u{267f}" />
                        <CriterionCheck label="Is a congenital anomaly/birth defect" signal=congenital icon="\u{1f476}" />
                        <CriterionCheck label="Other medically important condition" signal=other icon="\u{2139}" />
                    </div>

                    <button
                        on:click=move |_| {
                            action.dispatch(ClassifySeriousness {
                                event_description: description.get(),
                                resulted_in_death: death.get(),
                                life_threatening: life_threat.get(),
                                hospitalization: hospital.get(),
                                disability: disability.get(),
                                congenital_anomaly: congenital.get(),
                                other_medically_important: other.get(),
                            });
                        }
                        disabled=action.pending()
                        class="w-full mt-6 rounded-lg bg-amber-600 px-4 py-3 text-sm font-bold text-white hover:bg-amber-500 transition-all disabled:opacity-50 uppercase tracking-widest"
                    >
                        {move || if action.pending().get() { "CLASSIFYING..." } else { "CLASSIFY EVENT" }}
                    </button>
                </section>

                /* Result / Preview section */
                <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Classification"</h2>

                    /* Live preview */
                    <div class="mb-6">
                        <div class=move || {
                            if any_serious.get() {
                                "rounded-xl border-2 border-red-500/40 bg-red-500/5 p-6 text-center transition-all"
                            } else {
                                "rounded-xl border-2 border-green-500/20 bg-green-500/5 p-6 text-center transition-all"
                            }
                        }>
                            <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-2">"Live Assessment"</p>
                            <p class=move || {
                                if any_serious.get() {
                                    "text-2xl font-bold font-mono text-red-400 tracking-tighter"
                                } else {
                                    "text-2xl font-bold font-mono text-green-400 tracking-tighter"
                                }
                            }>
                                {move || if any_serious.get() { "SERIOUS" } else { "NON-SERIOUS" }}
                            </p>
                            <p class="mt-2 text-xs text-slate-500">
                                {move || {
                                    let mut criteria = Vec::new();
                                    if death.get() { criteria.push("Death"); }
                                    if life_threat.get() { criteria.push("Life-threatening"); }
                                    if hospital.get() { criteria.push("Hospitalization"); }
                                    if disability.get() { criteria.push("Disability"); }
                                    if congenital.get() { criteria.push("Congenital anomaly"); }
                                    if other.get() { criteria.push("Other medically important"); }
                                    if criteria.is_empty() {
                                        "No seriousness criteria met".to_string()
                                    } else {
                                        criteria.join(" \u{2022} ")
                                    }
                                }}
                            </p>
                        </div>
                    </div>

                    /* Regulatory implications */
                    <div class="space-y-3">
                        <RegImplication
                            label="Expedited Reporting"
                            description="Serious + unexpected = 15-day report (FDA) / 15-day (EMA)"
                            active=any_serious
                        />
                        <RegImplication
                            label="PBRER Inclusion"
                            description="All serious events must be included in Periodic Benefit-Risk Evaluation Report"
                            active=any_serious
                        />
                        <RegImplication
                            label="Signal Evaluation"
                            description="Serious events contribute disproportionately to signal detection metrics"
                            active=any_serious
                        />
                    </div>

                    /* Server result */
                    <Suspense fallback=|| ()>
                        {move || result.get().map(|res| match res {
                            Ok(data) => view! { <SeriousnessResultView data=data /> }.into_any(),
                            Err(e) => view! {
                                <div class="mt-4 p-3 rounded bg-red-500/10 border border-red-500/20 text-red-400 font-mono text-xs text-center">
                                    {e.to_string()}
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                </section>
            </div>
        </div>
    }
}

#[component]
fn CriterionCheck(
    label: &'static str,
    signal: RwSignal<bool>,
    icon: &'static str,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| signal.update(|v| *v = !*v)
            class=move || {
                if signal.get() {
                    "w-full flex items-center gap-3 px-4 py-3 rounded-lg border border-red-500/40 bg-red-500/10 text-left transition-all"
                } else {
                    "w-full flex items-center gap-3 px-4 py-3 rounded-lg border border-slate-700 bg-slate-950 text-left hover:border-slate-500 transition-all"
                }
            }
        >
            <span class="text-lg">{icon}</span>
            <span class=move || {
                if signal.get() { "text-sm text-red-300 font-mono" } else { "text-sm text-slate-400 font-mono" }
            }>{label}</span>
            <span class="ml-auto text-xs font-bold font-mono">
                {move || if signal.get() { "\u{2713}" } else { "" }}
            </span>
        </button>
    }
}

#[component]
fn RegImplication(
    label: &'static str,
    description: &'static str,
    active: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class=move || {
            if active.get() {
                "rounded-lg border border-amber-500/30 bg-amber-500/5 p-3 transition-all"
            } else {
                "rounded-lg border border-slate-800 bg-slate-950/50 p-3 transition-all opacity-40"
            }
        }>
            <p class="text-xs font-bold text-slate-300 font-mono uppercase">{label}</p>
            <p class="text-[10px] text-slate-500 mt-0.5">{description}</p>
        </div>
    }
}

#[component]
fn SeriousnessResultView(data: SeriousnessResult) -> impl IntoView {
    view! {
        <div class="mt-6 p-4 rounded-xl bg-slate-950 border border-slate-800">
            <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-2">"Server Classification"</p>
            <p class={format!("text-lg font-bold font-mono {}", if data.is_serious { "text-red-400" } else { "text-green-400" })}>
                {data.classification}
            </p>
            {if !data.criteria_met.is_empty() {
                view! {
                    <div class="mt-3 flex flex-wrap gap-1">
                        {data.criteria_met.into_iter().map(|c| view! {
                            <span class="px-2 py-0.5 rounded-full bg-red-500/10 border border-red-500/20 text-red-400 text-[10px] font-mono">{c}</span>
                        }).collect_view()}
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}
