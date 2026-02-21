//! Assessment page — take questions and view results

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::assessment_form::{AssessmentResultView, QuestionView};
use crate::server::submit_assessment::{
    get_assessment_questions, submit_assessment, AssessmentResultData,
};

/// Assessment page: loads questions and handles submission.
#[component]
pub fn AssessmentPage() -> impl IntoView {
    let params = use_params_map();
    let subject_id = move || {
        params.read().get("subject_id").unwrap_or_default()
    };

    let sid = subject_id();
    let sid_for_questions = sid.clone();
    let sid_for_back = sid.clone();
    let sid_for_action = sid.clone();

    let questions = Resource::new(
        move || sid_for_questions.clone(),
        get_assessment_questions,
    );

    let (result, set_result) = signal(None::<AssessmentResultData>);
    let (submitting, set_submitting) = signal(false);

    // Use Action for the submission (Fn-safe, not FnOnce)
    let submit_action = Action::new(move |_: &()| {
        let sid = sid_for_action.clone();
        let answers = "[]".to_string();
        async move {
            submit_assessment(sid, answers).await.ok()
        }
    });

    // Watch action value to update signals
    Effect::new(move || {
        if let Some(Some(res)) = submit_action.value().get() {
            set_result.set(Some(res));
            set_submitting.set(false);
        }
    });

    view! {
        <div class="min-h-screen bg-gray-900">
            <header class="bg-gray-800 border-b border-gray-700 px-6 py-4">
                <div class="max-w-4xl mx-auto flex items-center gap-4">
                    <a href={format!("/subject/{}", sid_for_back)}
                       class="text-gray-400 hover:text-cyan-400 transition-colors">
                        "← Back to Subject"
                    </a>
                    <h1 class="text-xl font-bold text-white">"Assessment"</h1>
                </div>
            </header>

            <main class="max-w-4xl mx-auto px-6 py-8 space-y-6">
                // Show result if submitted
                {move || {
                    result.get().map(|res| {
                        view! { <AssessmentResultView result=res /> }
                    })
                }}

                // Show questions if not yet submitted
                <Show
                    when=move || result.get().is_none()
                    fallback=|| view! {
                        <div class="text-center">
                            <a href="/" class="text-cyan-400 hover:text-cyan-300">
                                "← Return to Dashboard"
                            </a>
                        </div>
                    }
                >
                    <Suspense fallback=move || view! { <div class="text-gray-500">"Loading questions..."</div> }>
                        {move || {
                            questions.get().map(|result| {
                                match result {
                                    Ok(qs) => view! {
                                        <div class="space-y-4">
                                            {qs.into_iter().enumerate().map(|(i, q)| {
                                                view! {
                                                    <QuestionView question=q number={i + 1} />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                        <div class="flex justify-end">
                                            <button
                                                on:click=move |_| {
                                                    set_submitting.set(true);
                                                    submit_action.dispatch(());
                                                }
                                                disabled=move || submitting.get()
                                                class="bg-cyan-600 hover:bg-cyan-500 text-white font-medium px-6 py-3 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                            >
                                                {move || if submitting.get() { "Submitting..." } else { "Submit Assessment" }}
                                            </button>
                                        </div>
                                    }.into_any(),
                                    Err(e) => view! {
                                        <p class="text-red-400">{format!("Error: {e}")}</p>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </Show>
            </main>
        </div>
    }
}
