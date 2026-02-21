//! Public academy preview page

use leptos::prelude::*;

#[component]
pub fn AcademyPreviewPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-16">
            <div class="text-center">
                <h1 class="text-4xl font-bold text-white">"NexVigilant Academy"</h1>
                <p class="mt-4 max-w-2xl mx-auto text-lg text-slate-400">
                    "Skills-based learning aligned with real-world competencies. Master pharmacovigilance from fundamentals to advanced signal detection."
                </p>
            </div>

            <div class="mt-16 grid gap-8 md:grid-cols-3">
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-center">
                    <div class="text-3xl font-bold text-emerald-400">"15"</div>
                    <p class="mt-1 text-sm text-slate-400">"PV Domains"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-center">
                    <div class="text-3xl font-bold text-emerald-400">"1,462"</div>
                    <p class="mt-1 text-sm text-slate-400">"Knowledge, Skills & Behaviors"</p>
                </div>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-center">
                    <div class="text-3xl font-bold text-emerald-400">"21"</div>
                    <p class="mt-1 text-sm text-slate-400">"Entrustable Activities"</p>
                </div>
            </div>

            <div class="mt-16">
                <h2 class="text-2xl font-bold text-white">"How It Works"</h2>
                <div class="mt-8 grid gap-6 md:grid-cols-3">
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <div class="text-lg font-bold text-emerald-400">"1. Assess"</div>
                        <p class="mt-2 text-sm text-slate-400">"Take a baseline competency assessment to understand where you stand across all 15 domains."</p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <div class="text-lg font-bold text-emerald-400">"2. Learn"</div>
                        <p class="mt-2 text-sm text-slate-400">"Follow personalized learning pathways with interactive content, case studies, and practical exercises."</p>
                    </div>
                    <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                        <div class="text-lg font-bold text-emerald-400">"3. Certify"</div>
                        <p class="mt-2 text-sm text-slate-400">"Demonstrate mastery and earn certificates recognized by the PV professional community."</p>
                    </div>
                </div>
            </div>

            <div class="mt-16 text-center">
                <a href="/signup" class="rounded-lg bg-emerald-600 px-8 py-3 font-semibold text-white hover:bg-emerald-500 transition-colors">
                    "Start Learning"
                </a>
            </div>
        </div>
    }
}
