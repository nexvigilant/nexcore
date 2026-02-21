//! Lesson content viewer page

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::LessonStepView;
use crate::server::get_lessons::get_lesson_detail;

/// Lesson page: renders full lesson content with steps.
#[component]
pub fn LessonPage() -> impl IntoView {
    let params = use_params_map();
    let lesson_id = move || {
        params.read().get("id").unwrap_or_default()
    };

    let lid = lesson_id();

    let lesson = Resource::new(
        move || lid.clone(),
        get_lesson_detail,
    );

    view! {
        <div class="min-h-screen bg-gray-900">
            <header class="bg-gray-800 border-b border-gray-700 px-6 py-4">
                <div class="max-w-4xl mx-auto flex items-center gap-4">
                    <Suspense fallback=move || view! { <span class="text-gray-500">"..."</span> }>
                        {move || {
                            lesson.get().map(|result| {
                                match result {
                                    Ok(l) => view! {
                                        <a href={format!("/subject/{}", l.subject_id)}
                                           class="text-gray-400 hover:text-cyan-400 transition-colors">
                                            "← Back to Subject"
                                        </a>
                                        <div>
                                            <h1 class="text-xl font-bold text-white">{l.title.clone()}</h1>
                                            <span class="text-gray-500 text-sm">{l.difficulty.clone()}</span>
                                        </div>
                                    }.into_any(),
                                    Err(_) => view! {
                                        <a href="/" class="text-gray-400 hover:text-cyan-400 transition-colors">
                                            "← Dashboard"
                                        </a>
                                        <h1 class="text-xl font-bold text-gray-500">"Lesson"</h1>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </header>

            <main class="max-w-4xl mx-auto px-6 py-8">
                <Suspense fallback=move || view! { <div class="text-gray-500">"Loading lesson..."</div> }>
                    {move || {
                        lesson.get().map(|result| {
                            match result {
                                Ok(l) => view! {
                                    <div class="space-y-6">
                                        {l.steps.into_iter().enumerate().map(|(i, step)| {
                                            view! {
                                                <LessonStepView step=step number={i + 1} />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="text-red-400">{format!("Error: {e}")}</p>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </main>
        </div>
    }
}
