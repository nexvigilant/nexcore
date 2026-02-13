//! Value proposition builder — articulate your professional value

use leptos::prelude::*;

#[component]
pub fn ValuePropPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Value Proposition Builder"</h1>
            <p class="mt-2 text-slate-400">"Craft a compelling professional value proposition for your PV career."</p>

            <div class="mt-8 space-y-6">
                <FieldSection
                    label="Target Role"
                    placeholder="e.g., Senior Drug Safety Scientist"
                    help="What position are you targeting?"
                />
                <FieldSection
                    label="Key Differentiators"
                    placeholder="e.g., Signal detection expertise, cross-functional leadership"
                    help="What makes you stand out from other candidates?"
                />
                <FieldSection
                    label="Quantified Achievements"
                    placeholder="e.g., Processed 500+ ICSRs/month, reduced signal detection time by 30%"
                    help="Numbers speak louder — quantify your impact."
                />
                <FieldSection
                    label="Domain Expertise"
                    placeholder="e.g., Oncology, biologics, real-world evidence"
                    help="Which therapeutic areas and methodologies are you strongest in?"
                />
            </div>

            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="font-semibold text-white">"Generated Value Proposition"</h2>
                <p class="mt-3 text-sm italic text-slate-400">"Fill in the fields above to generate your value proposition statement."</p>
            </div>
        </div>
    }
}

#[component]
fn FieldSection(label: &'static str, placeholder: &'static str, help: &'static str) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-white">{label}</label>
            <p class="mt-0.5 text-xs text-slate-500">{help}</p>
            <input
                type="text"
                placeholder=placeholder
                class="mt-2 w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
            />
        </div>
    }
}
