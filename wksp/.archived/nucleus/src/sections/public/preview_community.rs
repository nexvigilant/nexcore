//! Public community preview page

use leptos::prelude::*;

#[component]
pub fn CommunityPreviewPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-16">
            <div class="text-center">
                <h1 class="text-4xl font-bold text-white">"NexVigilant Community"</h1>
                <p class="mt-4 max-w-2xl mx-auto text-lg text-slate-400">
                    "Connect with pharmacovigilance professionals worldwide. Share knowledge, find mentors, and advance together."
                </p>
            </div>

            <div class="mt-16 grid gap-8 md:grid-cols-2">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                    <h3 class="text-xl font-bold text-violet-400">"Professional Circles"</h3>
                    <p class="mt-3 text-slate-400">"Join topic-focused groups: Signal Detection, Benefit-Risk, Regulatory Affairs, Clinical Safety, and more."</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                    <h3 class="text-xl font-bold text-violet-400">"Mentorship"</h3>
                    <p class="mt-3 text-slate-400">"Connect with experienced professionals for guidance on career transitions and skill development."</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                    <h3 class="text-xl font-bold text-violet-400">"Knowledge Sharing"</h3>
                    <p class="mt-3 text-slate-400">"Post insights, ask questions, share case studies, and learn from the collective expertise."</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-8">
                    <h3 class="text-xl font-bold text-violet-400">"Events"</h3>
                    <p class="mt-3 text-slate-400">"Webinars, journal clubs, workshops, and networking events for continuous professional growth."</p>
                </div>
            </div>

            <div class="mt-16 text-center">
                <a href="/signup" class="rounded-lg bg-violet-600 px-8 py-3 font-semibold text-white hover:bg-violet-500 transition-colors">
                    "Join the Community"
                </a>
            </div>
        </div>
    }
}
