//! Causality assessment page — WHO-UMC and Naranjo algorithms

use leptos::prelude::*;
use crate::api_client::{NaranjoCausality, WhoUmcCausality};

/// Server function to perform Naranjo causality assessment
#[server(RunNaranjo, "/api")]
pub async fn run_naranjo_action(answers: Vec<i32>) -> Result<NaranjoCausality, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.naranjo(&answers).await
        .map_err(ServerFnError::new)
}

#[component]
pub fn CausalityPage() -> impl IntoView {
    let naranjo_answers = RwSignal::new(vec![0; 10]);
    let naranjo_action = ServerAction::<RunNaranjo>::new();
    let result = naranjo_action.value();

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <header class="mb-10 text-center">
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Causality Assessment"</h1>
                <p class="mt-2 text-slate-400">"Scientific evaluation of the relationship between drug exposure and adverse events"</p>
            </header>

            <div class="grid gap-8 lg:grid-cols-1">
                <section class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                    <div class="flex justify-between items-center mb-8">
                        <h2 class="text-xl font-bold text-white font-mono uppercase tracking-wide">"Naranjo Algorithm"</h2>
                        <button 
                            on:click=move |_| {
                                naranjo_action.dispatch(RunNaranjo { answers: naranjo_answers.get() });
                            }
                            disabled=naranjo_action.pending()
                            class="rounded-lg bg-amber-600 px-8 py-2 text-sm font-bold text-white hover:bg-amber-500 transition-all shadow-lg shadow-amber-900/20 disabled:opacity-50 uppercase tracking-widest"
                        >
                            {move || if naranjo_action.pending().get() { "ASSESSING..." } else { "RUN ASSESSMENT" }}
                        </button>
                    </div>

                    <div class="space-y-6">
                        <NaranjoQuestion 
                            index=0
                            question="Are there previous conclusive reports on this reaction?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=1
                            question="Did the adverse event appear after the suspected drug was administered?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=2
                            question="Did the adverse reaction improve when the drug was discontinued?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=3
                            question="Did the adverse reaction reappear when the drug was readministered?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=4
                            question="Are there alternative causes (other than the drug) that could on their own have caused the reaction?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=5
                            question="Did the reaction reappear when a placebo was given?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=6
                            question="Was the drug detected in the blood (or other fluids) in concentrations known to be toxic?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=7
                            question="Was the reaction more severe when the dose was increased, or less severe when the dose was decreased?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=8
                            question="Did the patient have a similar reaction to the same or similar drugs in any previous exposure?" 
                            answers=naranjo_answers 
                        />
                        <NaranjoQuestion 
                            index=9
                            question="Was the adverse event confirmed by any objective evidence?" 
                            answers=naranjo_answers 
                        />
                    </div>

                    <Suspense fallback=|| ()>
                        {move || result.get().map(|res| match res {
                            Ok(data) => view! { <NaranjoResultView data=data /> }.into_any(),
                            Err(e) => view! { <div class="mt-8 p-4 rounded bg-red-500/10 border border-red-500/20 text-red-400 font-mono text-xs text-center">{e.to_string()}</div> }.into_any()
                        })}
                    </Suspense>
                </section>
            </div>
        </div>
    }
}

#[component]
fn NaranjoQuestion(
    index: usize,
    question: &'static str,
    answers: RwSignal<Vec<i32>>
) -> impl IntoView {
    view! {
        <div class="py-4 border-b border-slate-800 flex flex-col md:flex-row md:items-center justify-between gap-4">
            <p class="text-sm text-slate-300 max-w-xl">{question}</p>
            <div class="flex gap-2">
                <AnswerButton label="Yes" value=1 index=index answers=answers />
                <AnswerButton label="No" value=-1 index=index answers=answers />
                <AnswerButton label="Unknown" value=0 index=index answers=answers />
            </div>
        </div>
    }
}

#[component]
fn AnswerButton(
    label: &'static str,
    value: i32,
    index: usize,
    answers: RwSignal<Vec<i32>>
) -> impl IntoView {
    let is_active = move || answers.get()[index] == value;
    
    view! {
        <button 
            on:click=move |_| {
                answers.update(|a| a[index] = value);
            }
            class=move || {
                let color_class = if is_active() { 
                    "bg-cyan-600 text-white border-cyan-500" 
                } else { 
                    "bg-slate-950 text-slate-500 border-slate-700 hover:border-slate-500" 
                };
                format!("px-4 py-1.5 rounded-lg border text-xs font-bold transition-all font-mono uppercase {}", color_class)
            }
        >
            {label}
        </button>
    }
}

#[component]
fn NaranjoResultView(data: NaranjoCausality) -> impl IntoView {
    let category_color = match data.category.as_str() {
        "Definite" => "text-red-400 bg-red-500/10 border-red-500/20",
        "Probable" => "text-amber-400 bg-amber-500/10 border-amber-500/20",
        "Possible" => "text-yellow-400 bg-yellow-500/10 border-yellow-500/20",
        _ => "text-slate-400 bg-slate-500/10 border-slate-500/20",
    };

    view! {
        <div class="mt-10 p-6 rounded-xl bg-slate-950 border border-slate-800 flex flex-col items-center text-center">
            <p class="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em] mb-4">"Causality Assessment Result"</p>
            <h3 class=format!("text-2xl font-bold font-mono tracking-tighter px-6 py-2 rounded-lg border {}", category_color)>
                {data.category.to_uppercase()}
            </h3>
            <p class="mt-4 text-sm text-slate-400">"Total Score: " <span class="text-white font-mono">{data.score}</span></p>
            <div class="mt-8 flex gap-3">
                <button class="text-[10px] font-bold text-slate-500 hover:text-white transition-colors uppercase font-mono">"Save to Case"</button>
                <span class="text-slate-800">"|"</span>
                <button class="text-[10px] font-bold text-slate-500 hover:text-white transition-colors uppercase font-mono">"Export PDF"</button>
            </div>
        </div>
    }
}
