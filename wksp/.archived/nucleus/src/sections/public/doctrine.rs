//! Doctrine — NexVigilant's principles and non-negotiables

use leptos::prelude::*;

#[component]
pub fn DoctrinePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Our Doctrine"</h1>
            <p class="mt-3 text-lg text-slate-400 italic">"Empowerment Through Vigilance"</p>

            <div class="mt-10 space-y-8">
                <Principle
                    title="Never Suppress Safety Signals"
                    desc="Patient safety is absolute. No commercial pressure, partnership, or convenience justifies hiding or downplaying a safety signal. Transparency is non-negotiable."
                />
                <Principle
                    title="Absolute Pharma Independence"
                    desc="NexVigilant maintains complete independence from pharmaceutical industry influence. Our analysis is unbiased because our funding never depends on specific outcomes."
                />
                <Principle
                    title="Capability-Based Advancement"
                    desc="We measure professionals by what they can do, not just what they know. Competency is demonstrated through practice, not credentials alone."
                />
                <Principle
                    title="Education as Right"
                    desc="PV knowledge boundaries should include, not exclude. We lower barriers to professional development while maintaining rigor."
                />
                <Principle
                    title="Data-Driven Decisions"
                    desc="Every claim is backed by quantifiable evidence. Disproportionality analysis, Bayesian trust scoring, and statistical rigor underpin everything we do."
                />
            </div>
        </div>
    }
}

#[component]
fn Principle(title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="border-l-2 border-cyan-500/50 pl-6">
            <h2 class="text-xl font-semibold text-white">{title}</h2>
            <p class="mt-2 text-sm leading-relaxed text-slate-400">{desc}</p>
        </div>
    }
}
