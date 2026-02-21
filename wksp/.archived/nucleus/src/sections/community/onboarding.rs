//! Community onboarding — new member welcome flow

use leptos::prelude::*;

#[component]
pub fn OnboardingPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-2xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Welcome to the Community"</h1>
            <p class="mt-2 text-slate-400">"Let's set up your community profile in a few quick steps."</p>

            <div class="mt-8 space-y-6">
                <Step number=1 title="Choose Your Interests" active=true>
                    <div class="flex flex-wrap gap-2">
                        <InterestTag label="Signal Detection"/>
                        <InterestTag label="ICSR Processing"/>
                        <InterestTag label="Regulatory Affairs"/>
                        <InterestTag label="Risk Management"/>
                        <InterestTag label="Clinical Safety"/>
                        <InterestTag label="Data Science"/>
                        <InterestTag label="AI in PV"/>
                        <InterestTag label="Career Growth"/>
                    </div>
                </Step>

                <Step number=2 title="Join Circles" active=false>
                    <p class="text-sm text-slate-500">"We'll suggest circles based on your interests."</p>
                </Step>

                <Step number=3 title="Introduce Yourself" active=false>
                    <p class="text-sm text-slate-500">"Write a brief introduction for the community."</p>
                </Step>
            </div>

            <button class="mt-8 rounded-lg bg-cyan-500 px-8 py-3 font-medium text-white hover:bg-cyan-400 transition-colors">
                "Continue"
            </button>
        </div>
    }
}

#[component]
fn Step(number: u32, title: &'static str, active: bool, children: Children) -> impl IntoView {
    let border = if active { "border-cyan-500/50" } else { "border-slate-800 opacity-50" };
    view! {
        <div class=format!("rounded-xl border {border} bg-slate-900/50 p-5")>
            <div class="flex items-center gap-3">
                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-slate-800 text-xs font-bold text-slate-400">{number}</span>
                <h2 class="font-semibold text-white">{title}</h2>
            </div>
            <div class="mt-4">{children()}</div>
        </div>
    }
}

#[component]
fn InterestTag(label: &'static str) -> impl IntoView {
    let selected = RwSignal::new(false);
    view! {
        <button
            class=move || if selected.get() {
                "rounded-full border border-cyan-500 bg-cyan-500/20 px-3 py-1 text-xs text-cyan-400"
            } else {
                "rounded-full border border-slate-700 px-3 py-1 text-xs text-slate-400 hover:border-slate-500"
            }
            on:click=move |_| selected.set(!selected.get())
        >
            {label}
        </button>
    }
}
