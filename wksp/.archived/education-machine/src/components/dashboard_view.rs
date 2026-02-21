//! Main dashboard component — Stats | Subjects | Reviews

use leptos::prelude::*;

use crate::components::subject_card::SubjectCard;
use crate::components::review_queue::ReviewQueue;
use crate::server::get_learner::get_learner;
use crate::server::get_subjects::get_subjects;

/// Dashboard view — the main landing page.
#[component]
pub fn DashboardView() -> impl IntoView {
    let learner = Resource::new(|| (), |_| get_learner());
    let subjects = Resource::new(|| (), |_| get_subjects());

    view! {
        <div class="min-h-screen bg-gray-900">
            // Header
            <header class="bg-gray-800 border-b border-gray-700 px-6 py-4">
                <div class="max-w-6xl mx-auto flex justify-between items-center">
                    <div>
                        <h1 class="text-2xl font-bold text-cyan-400">"Education Machine"</h1>
                        <p class="text-gray-500 text-sm">"Primitive-first learning engine"</p>
                    </div>
                    <Suspense fallback=move || view! { <span class="text-gray-500">"..."</span> }>
                        {move || {
                            learner.get().map(|result| {
                                match result {
                                    Ok(l) => view! {
                                        <span class="text-gray-300">{l.name}</span>
                                    }.into_any(),
                                    Err(_) => view! {
                                        <span class="text-gray-500">"Guest"</span>
                                    }.into_any()
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </header>

            <main class="max-w-6xl mx-auto px-6 py-8 space-y-8">
                // Stats row
                <Suspense fallback=move || view! { <div class="text-gray-500">"Loading stats..."</div> }>
                    {move || {
                        learner.get().map(|result| {
                            match result {
                                Ok(l) => view! {
                                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                        <StatCard label="Subjects" value={l.enrollment_count.to_string()} color="text-cyan-400" />
                                        <StatCard label="Mastered" value={l.mastered_count.to_string()} color="text-green-400" />
                                        <StatCard label="Developing" value={l.developing_count.to_string()} color="text-yellow-400" />
                                        <StatCard label="Reviews Due" value={l.reviews_due.to_string()} color="text-purple-400" />
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="text-red-400">{format!("Error: {e}")}</p>
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>

                // Subjects grid
                <section>
                    <h2 class="text-xl font-semibold text-white mb-4">"Subjects"</h2>
                    <Suspense fallback=move || view! { <div class="text-gray-500">"Loading subjects..."</div> }>
                        {move || {
                            subjects.get().map(|result| {
                                match result {
                                    Ok(subs) => view! {
                                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                            {subs.into_iter().map(|s| {
                                                view! {
                                                    <SubjectCard
                                                        id=s.id
                                                        name=s.name
                                                        description=s.description
                                                        lesson_count=s.lesson_count
                                                        mastery=s.mastery
                                                        phase=s.phase
                                                        tags=s.tags
                                                    />
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

                // Review queue
                <section>
                    <ReviewQueue />
                </section>
            </main>
        </div>
    }
}

/// Stat card for the dashboard header
#[component]
fn StatCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] color: String,
) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <p class={format!("text-2xl font-bold {color}")}>{value}</p>
            <p class="text-gray-500 text-sm">{label}</p>
        </div>
    }
}
