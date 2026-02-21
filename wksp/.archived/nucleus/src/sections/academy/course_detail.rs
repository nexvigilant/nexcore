//! Course detail page — individual course view with modules and lessons

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn CourseDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.get().get("slug").unwrap_or_default();

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <nav class="text-sm text-slate-500">
                <a href="/academy" class="hover:text-cyan-400">"Academy"</a>
                " / "
                <a href="/academy/courses" class="hover:text-cyan-400">"Courses"</a>
                " / "
                <span class="text-slate-300">{slug}</span>
            </nav>

            <div class="mt-6">
                <h1 class="text-3xl font-bold text-white">{move || format!("Course: {}", slug())}</h1>
                <p class="mt-2 text-slate-400">"Loading course content..."</p>
            </div>

            <div class="mt-8 grid gap-6 lg:grid-cols-3">
                <div class="lg:col-span-2 space-y-4">
                    <h2 class="text-lg font-semibold text-white">"Modules"</h2>
                    <ModulePlaceholder number=1 title="Introduction" lessons=3/>
                    <ModulePlaceholder number=2 title="Core Concepts" lessons=5/>
                    <ModulePlaceholder number=3 title="Practical Application" lessons=4/>
                    <ModulePlaceholder number=4 title="Assessment" lessons=2/>
                </div>

                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 h-fit">
                    <h3 class="font-semibold text-white">"Course Info"</h3>
                    <dl class="mt-4 space-y-3 text-sm">
                        <div>
                            <dt class="text-slate-500">"Difficulty"</dt>
                            <dd class="text-slate-300">"Intermediate"</dd>
                        </div>
                        <div>
                            <dt class="text-slate-500">"Duration"</dt>
                            <dd class="text-slate-300">"~8 hours"</dd>
                        </div>
                        <div>
                            <dt class="text-slate-500">"Modules"</dt>
                            <dd class="text-slate-300">"4 modules, 14 lessons"</dd>
                        </div>
                    </dl>
                    <button class="mt-6 w-full rounded-lg bg-cyan-500 py-2.5 text-sm font-medium text-white hover:bg-cyan-400 transition-colors">
                        "Enroll"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ModulePlaceholder(number: u32, title: &'static str, lessons: u32) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-slate-800 bg-slate-900/50 p-4">
            <div class="flex items-center justify-between">
                <span class="font-medium text-white">{format!("Module {number}: {title}")}</span>
                <span class="text-xs text-slate-500">{format!("{lessons} lessons")}</span>
            </div>
        </div>
    }
}
