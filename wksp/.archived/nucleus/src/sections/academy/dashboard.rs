//! Academy dashboard page — student overview with learner state machine

use leptos::prelude::*;
use crate::api_client::{Enrollment, LearnerState, SignalRequest, SignalResult};

/// Server function to list student enrollments
#[server(ListEnrollments, "/api")]
pub async fn list_enrollments_action() -> Result<Vec<Enrollment>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.academy_list_enrollments().await
        .map_err(ServerFnError::new)
}

/// Server function to run signal detection on a 2x2 contingency table
#[server(RunSignalDetection, "/api")]
pub async fn run_signal_detection_action(a: u64, b: u64, c: u64, d: u64) -> Result<SignalResult, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = SignalRequest { a, b, c, d };
    client.signal_complete(&req).await
        .map_err(ServerFnError::new)
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let enrollments = Resource::new(|| (), |_| list_enrollments_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12 flex justify-between items-end">
                <div>
                    <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"ACADEMY"</h1>
                    <p class="mt-2 text-slate-400">"Your personal competency and certification track"</p>
                </div>
                <div class="flex gap-4 mb-1">
                    <StatBox label="Credits" value="0" />
                    <StatBox label="Mastery" value="0.0" />
                </div>
            </header>

            // Learner Journey State Machine
            <LearnerJourneyWidget current=LearnerState::Exploring />

            <div class="grid gap-8 lg:grid-cols-3">
                <div class="lg:col-span-2">
                    <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Active Pathways"</h2>
                    <Suspense fallback=|| view! { <div class="animate-pulse h-32 bg-slate-900/30 rounded-2xl"></div> }>
                        {move || enrollments.get().map(|result| match result {
                            Ok(list) => view! { <EnrollmentListView list=list /> }.into_any(),
                            Err(e) => view! { <div class="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-xs font-mono">{e.to_string()}</div> }.into_any()
                        })}
                    </Suspense>
                </div>

                <aside class="space-y-6">
                    <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-6">"Achievements"</h2>
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                        <p class="text-sm text-slate-500 italic">"No certificates issued yet."</p>
                        <a href="/academy/courses" class="mt-6 inline-block text-xs font-bold text-cyan-400 hover:text-cyan-300 transition-colors uppercase tracking-widest underline decoration-slate-800 underline-offset-8">
                            "Browse Curriculum"
                        </a>
                    </div>

                    // Signal Detection Practice Lab
                    <SignalPracticeLab />
                </aside>
            </div>
        </div>
    }
}

#[component]
fn StatBox(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="text-right px-4 border-r border-slate-800 last:border-0">
            <p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{label}</p>
            <p class="text-xl font-bold text-white font-mono">{value}</p>
        </div>
    }
}

#[component]
fn EnrollmentListView(list: Vec<Enrollment>) -> impl IntoView {
    if list.is_empty() {
        view! { 
            <div class="rounded-2xl border border-dashed border-slate-800 bg-slate-950/20 p-12 text-center">
                <p class="text-slate-500 font-mono text-sm">"NO ACTIVE ENROLLMENTS DETECTED"</p>
            </div>
        }.into_any()
    } else {
        view! {
            <div class="space-y-4">
                {list.into_iter().map(|e| view! { <EnrollmentItem enrollment=e /> }).collect_view()}
            </div>
        }.into_any()
    }
}

#[component]
fn EnrollmentItem(enrollment: Enrollment) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6 flex items-center justify-between">
            <div>
                <p class="text-xs font-bold text-cyan-500 font-mono tracking-wider mb-1">{enrollment.course_id.to_uppercase()}</p>
                <p class="text-sm font-bold text-slate-200">"Learning Pathway in Progress"</p>
            </div>
            <div class="w-32">
                <div class="flex justify-between items-center mb-2">
                    <span class="text-[10px] font-bold text-slate-500 font-mono">"PROGRESS"</span>
                    <span class="text-[10px] font-bold text-white font-mono">{(enrollment.progress * 100.0) as u32}"%"</span>
                </div>
                <div class="h-1 w-full bg-slate-800 rounded-full overflow-hidden">
                    <div class="h-full bg-cyan-500 transition-all duration-500" style=format!("width: {}%", enrollment.progress * 100.0)></div>
                </div>
            </div>
        </div>
    }
}

/// Interactive signal detection practice widget.
///
/// Learners input a 2x2 contingency table (a, b, c, d) and get real-time
/// disproportionality metrics from NexCore — bridging D08 learning to practice.
#[component]
fn SignalPracticeLab() -> impl IntoView {
    let (a_val, set_a) = signal(String::from("15"));
    let (b_val, set_b) = signal(String::from("100"));
    let (c_val, set_c) = signal(String::from("20"));
    let (d_val, set_d) = signal(String::from("10000"));
    let (result, set_result) = signal(None::<Result<SignalResult, ServerFnError>>);
    let (loading, set_loading) = signal(false);

    let run_analysis = Action::new(move |_: &()| {
        let a_str = a_val.get();
        let b_str = b_val.get();
        let c_str = c_val.get();
        let d_str = d_val.get();
        async move {
            let a = a_str.parse::<u64>().unwrap_or(0);
            let b = b_str.parse::<u64>().unwrap_or(0);
            let c = c_str.parse::<u64>().unwrap_or(0);
            let d = d_str.parse::<u64>().unwrap_or(0);
            set_loading.set(true);
            let res = run_signal_detection_action(a, b, c, d).await;
            set_result.set(Some(res));
            set_loading.set(false);
        }
    });

    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/30 p-5">
            <div class="flex items-center gap-2 mb-4">
                <span class="text-cyan-400 font-mono text-sm">"∂"</span>
                <h3 class="text-xs font-bold text-slate-400 uppercase tracking-widest">"Signal Lab"</h3>
            </div>

            <p class="text-[10px] text-slate-500 mb-4">"Enter a 2×2 contingency table to run disproportionality analysis."</p>

            // 2x2 grid inputs
            <div class="grid grid-cols-2 gap-2 mb-3">
                <div>
                    <label class="text-[10px] text-slate-600 font-mono block mb-1">"a (drug+event)"</label>
                    <input type="number" class="w-full bg-slate-950 border border-slate-700 rounded-lg px-2 py-1.5 text-xs text-white font-mono focus:border-cyan-500 focus:outline-none"
                        prop:value=move || a_val.get()
                        on:input=move |ev| set_a.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] text-slate-600 font-mono block mb-1">"b (drug+no event)"</label>
                    <input type="number" class="w-full bg-slate-950 border border-slate-700 rounded-lg px-2 py-1.5 text-xs text-white font-mono focus:border-cyan-500 focus:outline-none"
                        prop:value=move || b_val.get()
                        on:input=move |ev| set_b.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] text-slate-600 font-mono block mb-1">"c (no drug+event)"</label>
                    <input type="number" class="w-full bg-slate-950 border border-slate-700 rounded-lg px-2 py-1.5 text-xs text-white font-mono focus:border-cyan-500 focus:outline-none"
                        prop:value=move || c_val.get()
                        on:input=move |ev| set_c.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] text-slate-600 font-mono block mb-1">"d (no drug+no event)"</label>
                    <input type="number" class="w-full bg-slate-950 border border-slate-700 rounded-lg px-2 py-1.5 text-xs text-white font-mono focus:border-cyan-500 focus:outline-none"
                        prop:value=move || d_val.get()
                        on:input=move |ev| set_d.set(event_target_value(&ev))
                    />
                </div>
            </div>

            <button
                class="w-full rounded-lg bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 text-xs font-bold font-mono py-2 hover:bg-cyan-500/20 transition-colors uppercase tracking-widest disabled:opacity-50"
                disabled=move || loading.get()
                on:click=move |_| { run_analysis.dispatch(()); }
            >
                {move || if loading.get() { "Analyzing..." } else { "Run Detection" }}
            </button>

            // Results
            {move || result.get().map(|res| match res {
                Ok(sr) => {
                    let prr_class = if sr.prr_signal { "text-red-400" } else { "text-emerald-400" };
                    let ror_class = if sr.ror_signal { "text-red-400" } else { "text-emerald-400" };
                    let ic_class = if sr.ic_signal { "text-red-400" } else { "text-emerald-400" };
                    let ebgm_class = if sr.ebgm_signal { "text-red-400" } else { "text-emerald-400" };
                    let chi_class = if sr.chi_signal { "text-red-400" } else { "text-emerald-400" };
                    let verdict_class = if sr.any_signal {
                        "text-red-400 border-red-500/30 bg-red-500/5"
                    } else {
                        "text-emerald-400 border-emerald-500/30 bg-emerald-500/5"
                    };
                    let verdict_text = if sr.any_signal { "SIGNAL DETECTED" } else { "NO SIGNAL" };

                    view! {
                        <div class="mt-4 space-y-2">
                            <div class=format!("rounded-lg border p-2 text-center text-xs font-bold font-mono {verdict_class}")>
                                {verdict_text}
                            </div>
                            <div class="grid grid-cols-2 gap-x-3 gap-y-1 text-[10px] font-mono">
                                <span class="text-slate-500">"PRR"</span>
                                <span class=prr_class>{format!("{:.2}", sr.prr)}</span>
                                <span class="text-slate-500">"ROR"</span>
                                <span class=ror_class>{format!("{:.2} (CI: {:.2})", sr.ror, sr.ror_lower_ci)}</span>
                                <span class="text-slate-500">"IC"</span>
                                <span class=ic_class>{format!("{:.2} (IC025: {:.2})", sr.ic, sr.ic025)}</span>
                                <span class="text-slate-500">"EBGM"</span>
                                <span class=ebgm_class>{format!("{:.2} (EB05: {:.2})", sr.ebgm, sr.eb05)}</span>
                                <span class="text-slate-500">"χ²"</span>
                                <span class=chi_class>{format!("{:.2}", sr.chi_square)}</span>
                            </div>
                        </div>
                    }.into_any()
                },
                Err(e) => view! {
                    <div class="mt-4 text-[10px] text-red-400 font-mono rounded-lg border border-red-500/20 bg-red-500/5 p-2">
                        {e.to_string()}
                    </div>
                }.into_any()
            })}
        </div>
    }
}

/// Visual state machine showing the learner's journey through 5 states.
///
/// Each state is a node in a horizontal pipeline with LP symbols,
/// highlighting the current state and dimming future states.
#[component]
fn LearnerJourneyWidget(current: LearnerState) -> impl IntoView {
    let states = LearnerState::ALL;

    view! {
        <div class="mb-10 rounded-2xl border border-slate-800 bg-slate-900/30 p-6">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest">"Your Journey"</h2>
                <span class="text-xs font-mono text-cyan-400">{format!("{}% complete", current.progress())}</span>
            </div>

            // Progress bar
            <div class="h-1 w-full rounded-full bg-slate-800 mb-6">
                <div class="h-1 rounded-full bg-gradient-to-r from-cyan-500 to-emerald-400 transition-all" style=format!("width: {}%", current.progress())></div>
            </div>

            // State nodes
            <div class="flex items-center justify-between">
                {states.into_iter().enumerate().map(|(i, state)| {
                    let is_current = state == current;
                    let is_past = (state.progress()) < current.progress();
                    let is_future = !is_current && !is_past;

                    let node_class = if is_current {
                        "h-12 w-12 rounded-xl bg-cyan-500 flex items-center justify-center text-white font-mono text-lg shadow-lg shadow-cyan-500/20"
                    } else if is_past {
                        "h-12 w-12 rounded-xl bg-cyan-500/20 border border-cyan-500/30 flex items-center justify-center text-cyan-400 font-mono text-lg"
                    } else {
                        "h-12 w-12 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center text-slate-500 font-mono text-lg"
                    };

                    let label_class = if is_current {
                        "text-[10px] font-bold text-cyan-400 font-mono mt-2 text-center"
                    } else if is_past {
                        "text-[10px] font-bold text-slate-400 font-mono mt-2 text-center"
                    } else {
                        "text-[10px] font-bold text-slate-600 font-mono mt-2 text-center"
                    };

                    let connector = (i < 4).then(|| {
                        let line_class = if is_past {
                            "flex-1 h-px bg-cyan-500/30 mx-2"
                        } else {
                            "flex-1 h-px bg-slate-800 mx-2"
                        };
                        view! { <div class=line_class></div> }
                    });

                    let _ = is_future; // suppress unused warning

                    view! {
                        <div class="flex flex-col items-center">
                            <div class=node_class>{state.symbol()}</div>
                            <span class=label_class>{state.label()}</span>
                        </div>
                        {connector}
                    }
                }).collect_view()}
            </div>
        </div>
    }
}