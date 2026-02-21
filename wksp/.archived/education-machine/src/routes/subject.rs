//! Subject detail page — lesson list + progress

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::{PhaseIndicator, ProgressBar};
use crate::server::get_lessons::get_lessons;
use crate::server::get_subjects::get_subjects;

/// Subject detail page: shows subject info and lesson list.
#[component]
pub fn SubjectPage() -> impl IntoView {
    let params = use_params_map();
    let subject_id = move || {
        params.read().get("id").unwrap_or_default()
    };

    let sid = subject_id();
    let sid_for_lessons = sid.clone();
    let sid_for_subject = sid.clone();
    let sid_for_assess = sid.clone();

    let subject = Resource::new(
        move || sid_for_subject.clone(),
        |id| async move {
            let subs = get_subjects().await?;
            let found = subs.into_iter().find(|s| s.id == id);
            Ok::<_, ServerFnError>(found)
        },
    );

    let lessons = Resource::new(
        move || sid_for_lessons.clone(),
        get_lessons,
    );

    view! {
        <div class="min-h-screen bg-gray-900">
            // Back nav
            <header class="bg-gray-800 border-b border-gray-700 px-6 py-4">
                <div class="max-w-4xl mx-auto flex items-center gap-4">
                    <a href="/" class="text-gray-400 hover:text-cyan-400 transition-colors">
                        "← Dashboard"
                    </a>
                    <Suspense fallback=move || view! { <span class="text-gray-500">"..."</span> }>
                        {move || {
                            subject.get().map(|result| {
                                match result {
                                    Ok(Some(s)) => view! {
                                        <h1 class="text-xl font-bold text-white">{s.name}</h1>
                                    }.into_any(),
                                    _ => view! {
                                        <h1 class="text-xl font-bold text-gray-500">"Subject"</h1>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </header>

            <main class="max-w-4xl mx-auto px-6 py-8 space-y-8">
                // Subject detail card
                <Suspense fallback=move || view! { <div class="text-gray-500">"Loading subject..."</div> }>
                    {move || {
                        subject.get().map(|result| {
                            match result {
                                Ok(Some(s)) => view! {
                                    <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
                                        <p class="text-gray-400 mb-4">{s.description}</p>
                                        <ProgressBar value=s.mastery />
                                        <div class="mt-3">
                                            <PhaseIndicator current_phase=s.phase />
                                        </div>
                                        <div class="mt-4 flex gap-4 text-sm">
                                            <span class="text-gray-500">
                                                {format!("{} lessons", s.lesson_count)}
                                            </span>
                                            <a
                                                href={format!("/assess/{}", sid_for_assess)}
                                                class="text-cyan-400 hover:text-cyan-300 transition-colors"
                                            >
                                                "Take Assessment →"
                                            </a>
                                        </div>
                                    </div>
                                }.into_any(),
                                Ok(None) => view! {
                                    <p class="text-yellow-400">"Subject not found"</p>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="text-red-400">{format!("Error: {e}")}</p>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>

                // Lesson list
                <section>
                    <h2 class="text-xl font-semibold text-white mb-4">"Lessons"</h2>
                    <Suspense fallback=move || view! { <div class="text-gray-500">"Loading lessons..."</div> }>
                        {move || {
                            lessons.get().map(|result| {
                                match result {
                                    Ok(items) => view! {
                                        <div class="space-y-3">
                                            {items.into_iter().enumerate().map(|(i, lesson)| {
                                                let status_color = if lesson.completed {
                                                    "text-green-400"
                                                } else {
                                                    "text-gray-500"
                                                };
                                                let status_text = if lesson.completed {
                                                    "✓ Complete"
                                                } else {
                                                    "○ In Progress"
                                                };
                                                let difficulty_color = match lesson.difficulty.as_str() {
                                                    "Hard" => "text-red-400 bg-red-900/30",
                                                    "Medium" => "text-yellow-400 bg-yellow-900/30",
                                                    _ => "text-green-400 bg-green-900/30",
                                                };
                                                let href = format!("/lesson/{}", lesson.id);
                                                view! {
                                                    <a href={href}
                                                       class="block bg-gray-800 rounded-lg p-4 hover:bg-gray-750 transition-colors border border-gray-700 hover:border-cyan-600">
                                                        <div class="flex justify-between items-start">
                                                            <div>
                                                                <span class="text-gray-500 text-xs">{format!("Lesson {}", i + 1)}</span>
                                                                <h3 class="text-white font-medium">{lesson.title}</h3>
                                                                <p class="text-gray-400 text-sm">{lesson.description}</p>
                                                            </div>
                                                            <div class="text-right flex flex-col items-end gap-1">
                                                                <span class={format!("text-xs px-2 py-0.5 rounded {difficulty_color}")}>
                                                                    {lesson.difficulty}
                                                                </span>
                                                                <span class={format!("text-xs {status_color}")}>{status_text}</span>
                                                                <span class="text-gray-600 text-xs">{format!("{} steps", lesson.step_count)}</span>
                                                            </div>
                                                        </div>
                                                    </a>
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
                </section>
            </main>
        </div>
    }
}
