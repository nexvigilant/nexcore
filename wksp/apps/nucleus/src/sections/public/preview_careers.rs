//! Public careers preview page

use leptos::prelude::*;

#[component]
pub fn CareersPreviewPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-16">
            <div class="text-center">
                <h1 class="text-4xl font-bold text-white">"NexVigilant Careers"</h1>
                <p class="mt-4 max-w-2xl mx-auto text-lg text-slate-400">
                    "Tools and assessments to navigate your pharmacovigilance career with confidence."
                </p>
            </div>

            <div class="mt-16">
                <h2 class="text-2xl font-bold text-white">"Career Assessment Tools"</h2>
                <div class="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    <AssessmentPreview name="Competency Assessment" desc="Map your skills across all 15 PV domains"/>
                    <AssessmentPreview name="Interview Preparation" desc="Practice questions tailored to PV roles"/>
                    <AssessmentPreview name="Skills Gap Analysis" desc="Identify areas for professional growth"/>
                    <AssessmentPreview name="Career Path Planner" desc="Chart your trajectory in drug safety"/>
                    <AssessmentPreview name="Salary Benchmarking" desc="Compare compensation across regions"/>
                    <AssessmentPreview name="Role Matching" desc="Find roles aligned with your strengths"/>
                </div>
            </div>

            <div class="mt-16 text-center">
                <a href="/signup" class="rounded-lg bg-amber-600 px-8 py-3 font-semibold text-white hover:bg-amber-500 transition-colors">
                    "Explore Career Tools"
                </a>
            </div>
        </div>
    }
}

#[component]
fn AssessmentPreview(name: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-amber-500/30 transition-colors">
            <h3 class="font-semibold text-white">{name}</h3>
            <p class="mt-1 text-sm text-slate-400">{desc}</p>
        </div>
    }
}
