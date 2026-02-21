//! Onboarding welcome — first step of the multi-step onboarding flow

use leptos::prelude::*;

#[component]
pub fn WelcomePage() -> impl IntoView {
    view! {
        <div class="flex min-h-[80vh] flex-col items-center justify-center px-4">
            // Step indicator
            <div class="flex items-center gap-3 mb-12">
                <StepDot number=1 label="Welcome" active=true/>
                <div class="h-px w-8 bg-slate-800"></div>
                <StepDot number=2 label="Profile" active=false/>
                <div class="h-px w-8 bg-slate-800"></div>
                <StepDot number=3 label="Interests" active=false/>
            </div>

            <h1 class="text-4xl font-bold text-white font-mono tracking-tight">"Welcome to Nucleus"</h1>
            <p class="mt-3 max-w-md text-center text-lg text-slate-400">
                "Your platform for pharmacovigilance education, career development, and community."
            </p>

            <div class="mt-10 w-full max-w-md space-y-4">
                <FeatureCard
                    icon="σ"
                    title="Academy"
                    desc="Structured learning across 15 PV domains with 1,462 KSBs"
                />
                <FeatureCard
                    icon="Σ"
                    title="Community"
                    desc="Connect with fellow PV professionals in topic-based circles"
                />
                <FeatureCard
                    icon="→"
                    title="Careers"
                    desc="Competency assessments, interview prep, and career pathing"
                />
            </div>

            <a
                href="/onboarding/profile"
                class="mt-10 rounded-lg bg-cyan-500 px-8 py-3 font-bold text-white hover:bg-cyan-400 transition-colors shadow-lg shadow-cyan-900/20"
            >
                "Get Started"
            </a>
        </div>
    }
}

#[component]
fn StepDot(number: u32, label: &'static str, active: bool) -> impl IntoView {
    let (dot_class, label_class) = if active {
        ("h-8 w-8 rounded-full bg-cyan-500 flex items-center justify-center text-xs font-bold text-white",
         "text-xs text-cyan-400 font-mono mt-1")
    } else {
        ("h-8 w-8 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-xs font-bold text-slate-500",
         "text-xs text-slate-600 font-mono mt-1")
    };

    view! {
        <div class="flex flex-col items-center">
            <div class=dot_class>{number.to_string()}</div>
            <span class=label_class>{label}</span>
        </div>
    }
}

#[component]
fn FeatureCard(icon: &'static str, title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-start gap-4 rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <span class="h-10 w-10 shrink-0 rounded-lg bg-cyan-500/10 flex items-center justify-center text-cyan-400 font-mono text-lg">{icon}</span>
            <div>
                <h3 class="font-semibold text-white">{title}</h3>
                <p class="mt-1 text-sm text-slate-400">{desc}</p>
            </div>
        </div>
    }
}
