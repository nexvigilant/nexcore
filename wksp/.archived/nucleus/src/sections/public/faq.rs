//! FAQ page

use leptos::prelude::*;

#[component]
pub fn FaqPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Frequently Asked Questions"</h1>

            <div class="mt-8 space-y-4">
                <FaqItem
                    question="What is NexVigilant?"
                    answer="NexVigilant is an independent platform combining pharmaceutical safety intelligence with professional development tools for healthcare professionals."
                />
                <FaqItem
                    question="Who is NexVigilant for?"
                    answer="Healthcare professionals, pharmacovigilance specialists, drug safety scientists, regulatory affairs professionals, and anyone interested in drug safety."
                />
                <FaqItem
                    question="Is NexVigilant independent from pharmaceutical companies?"
                    answer="Yes. Absolute independence from pharmaceutical industry influence is a non-negotiable core value. We never suppress safety signals."
                />
                <FaqItem
                    question="What does the Academy include?"
                    answer="Structured learning across 15 PV domains, 1,462 knowledge-skill-behaviors, interactive assessments, and professional certificates."
                />
                <FaqItem
                    question="How much does it cost?"
                    answer="We offer a free tier with limited access, and Professional/Enterprise tiers with full platform access. See our membership page for details."
                />
            </div>
        </div>
    }
}

#[component]
fn FaqItem(question: &'static str, answer: &'static str) -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50">
            <button
                class="flex w-full items-center justify-between p-5 text-left"
                on:click=move |_| open.set(!open.get())
            >
                <span class="font-medium text-white">{question}</span>
                <span class=move || format!(
                    "text-slate-400 transition-transform {}",
                    if open.get() { "rotate-180" } else { "" }
                )>"v"</span>
            </button>
            <div class=move || format!(
                "overflow-hidden transition-all {}",
                if open.get() { "max-h-96" } else { "max-h-0" }
            )>
                <p class="px-5 pb-5 text-sm text-slate-400">{answer}</p>
            </div>
        </div>
    }
}
