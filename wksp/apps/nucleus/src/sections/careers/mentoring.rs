//! Mentoring framework — find mentors or become one

use leptos::prelude::*;

#[component]
pub fn MentoringPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Mentoring Framework"</h1>
            <p class="mt-2 text-slate-400">"Connect with experienced PV professionals or share your expertise with others."</p>

            <div class="mt-8 grid gap-6 md:grid-cols-2">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-semibold text-cyan-400">"Find a Mentor"</h2>
                    <p class="mt-2 text-sm text-slate-400">"Get paired with an experienced professional who can guide your career development."</p>
                    <ul class="mt-4 space-y-2 text-sm text-slate-300">
                        <li>"1:1 career guidance"</li>
                        <li>"Domain-specific expertise"</li>
                        <li>"Network expansion"</li>
                    </ul>
                    <button class="mt-4 rounded-lg bg-cyan-500 px-6 py-2 text-sm font-medium text-white hover:bg-cyan-400 transition-colors">
                        "Request Mentor"
                    </button>
                </div>

                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-semibold text-amber-400">"Become a Mentor"</h2>
                    <p class="mt-2 text-sm text-slate-400">"Share your knowledge and help the next generation of PV professionals grow."</p>
                    <ul class="mt-4 space-y-2 text-sm text-slate-300">
                        <li>"Give back to the community"</li>
                        <li>"Sharpen your own skills"</li>
                        <li>"Build your reputation"</li>
                    </ul>
                    <button class="mt-4 rounded-lg border border-amber-500/50 px-6 py-2 text-sm font-medium text-amber-400 hover:bg-amber-500/10 transition-colors">
                        "Apply to Mentor"
                    </button>
                </div>
            </div>
        </div>
    }
}
