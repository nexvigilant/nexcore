//! Schedule — book consultations and demos

use leptos::prelude::*;

#[component]
pub fn SchedulePage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Schedule a Session"</h1>
            <p class="mt-3 text-slate-400">
                "Book a consultation, demo, or training session with our team."
            </p>

            <div class="mt-8 space-y-4">
                <ScheduleOption
                    title="Platform Demo"
                    duration="30 min"
                    desc="See Nucleus in action — Academy, signals, Guardian, and more."
                />
                <ScheduleOption
                    title="PV Consulting"
                    duration="60 min"
                    desc="One-on-one session with a pharmacovigilance specialist."
                />
                <ScheduleOption
                    title="Enterprise Onboarding"
                    duration="45 min"
                    desc="Set up your team with custom pathways and analytics."
                />
            </div>

            <p class="mt-8 text-sm text-slate-500">
                "Calendar integration coming soon. Contact us at "
                <a href="/contact" class="text-cyan-400 hover:text-cyan-300">"contact page"</a>
                " to schedule."
            </p>
        </div>
    }
}

#[component]
fn ScheduleOption(title: &'static str, duration: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-cyan-500/30 transition-colors cursor-pointer">
            <div>
                <h3 class="font-semibold text-white">{title}</h3>
                <p class="mt-1 text-sm text-slate-400">{desc}</p>
            </div>
            <span class="shrink-0 rounded-full bg-slate-800 px-3 py-1 text-xs text-slate-400">{duration}</span>
        </div>
    }
}
