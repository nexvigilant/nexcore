//! Onboarding profile setup — role, experience, goals with reactive form

use leptos::prelude::*;

#[component]
pub fn ProfileSetupPage() -> impl IntoView {
    let role = RwSignal::new(String::from("Drug Safety Associate"));
    let experience = RwSignal::new(String::from("0-1 years"));
    let goal = RwSignal::new(String::from("Learn PV fundamentals"));

    let is_complete = Signal::derive(move || {
        !role.get().is_empty() && !experience.get().is_empty() && !goal.get().is_empty()
    });

    view! {
        <div class="mx-auto max-w-xl px-4 py-12">
            // Step indicator
            <div class="flex items-center justify-center gap-3 mb-8">
                <StepDot number=1 label="Welcome" completed=true active=false/>
                <div class="h-px w-8 bg-cyan-500"></div>
                <StepDot number=2 label="Profile" completed=false active=true/>
                <div class="h-px w-8 bg-slate-800"></div>
                <StepDot number=3 label="Interests" completed=false active=false/>
            </div>

            <ProgressBar step=2 total=3/>

            <h1 class="mt-8 text-3xl font-bold text-white font-mono tracking-tight">"About You"</h1>
            <p class="mt-2 text-slate-400">"Help us personalize your experience."</p>

            <div class="mt-8 space-y-6">
                <div>
                    <label class="block text-xs font-bold uppercase tracking-widest text-slate-500 mb-2">"Current Role"</label>
                    <select
                        prop:value=move || role.get()
                        on:change=move |ev| role.set(event_target_value(&ev))
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white focus:border-cyan-500 focus:outline-none transition-colors"
                    >
                        <option>"Drug Safety Associate"</option>
                        <option>"PV Scientist"</option>
                        <option>"Regulatory Affairs Specialist"</option>
                        <option>"Clinical Safety Officer"</option>
                        <option>"Student / Career Changer"</option>
                        <option>"Other"</option>
                    </select>
                </div>

                <div>
                    <label class="block text-xs font-bold uppercase tracking-widest text-slate-500 mb-2">"Years of PV Experience"</label>
                    <select
                        prop:value=move || experience.get()
                        on:change=move |ev| experience.set(event_target_value(&ev))
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white focus:border-cyan-500 focus:outline-none transition-colors"
                    >
                        <option>"0-1 years"</option>
                        <option>"2-4 years"</option>
                        <option>"5-9 years"</option>
                        <option>"10+ years"</option>
                    </select>
                </div>

                <div>
                    <label class="block text-xs font-bold uppercase tracking-widest text-slate-500 mb-2">"Primary Goal"</label>
                    <select
                        prop:value=move || goal.get()
                        on:change=move |ev| goal.set(event_target_value(&ev))
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-2.5 text-sm text-white focus:border-cyan-500 focus:outline-none transition-colors"
                    >
                        <option>"Learn PV fundamentals"</option>
                        <option>"Advance my career"</option>
                        <option>"Connect with peers"</option>
                        <option>"Stay current on safety intelligence"</option>
                    </select>
                </div>
            </div>

            // Summary preview
            <div class="mt-8 rounded-lg border border-slate-800 bg-slate-900/30 p-4">
                <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-2">"Profile Preview"</p>
                <p class="text-sm text-slate-300">
                    {move || role.get()}" · "{move || experience.get()}" · "{move || goal.get()}
                </p>
            </div>

            <div class="mt-8 flex justify-between items-center">
                <a href="/onboarding" class="text-sm text-slate-400 hover:text-white transition-colors font-mono">"BACK"</a>
                <a
                    href="/onboarding/interests"
                    class=move || if is_complete.get() {
                        "rounded-lg bg-cyan-500 px-6 py-2.5 font-bold text-white hover:bg-cyan-400 transition-colors shadow-lg shadow-cyan-900/20"
                    } else {
                        "rounded-lg bg-slate-700 px-6 py-2.5 font-bold text-slate-400 cursor-not-allowed"
                    }
                >
                    "Continue"
                </a>
            </div>
        </div>
    }
}

#[component]
fn StepDot(number: u32, label: &'static str, completed: bool, active: bool) -> impl IntoView {
    let dot_class = if active {
        "h-8 w-8 rounded-full bg-cyan-500 flex items-center justify-center text-xs font-bold text-white"
    } else if completed {
        "h-8 w-8 rounded-full bg-cyan-500/20 border border-cyan-500 flex items-center justify-center text-xs font-bold text-cyan-400"
    } else {
        "h-8 w-8 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-xs font-bold text-slate-500"
    };

    let label_class = if active || completed {
        "text-xs text-cyan-400 font-mono mt-1"
    } else {
        "text-xs text-slate-600 font-mono mt-1"
    };

    view! {
        <div class="flex flex-col items-center">
            <div class=dot_class>
                {if completed { "✓".to_string() } else { number.to_string() }}
            </div>
            <span class=label_class>{label}</span>
        </div>
    }
}

#[component]
fn ProgressBar(step: u32, total: u32) -> impl IntoView {
    let pct = (step as f32 / total as f32) * 100.0;
    view! {
        <div class="h-1 w-full rounded-full bg-slate-800">
            <div class="h-1 rounded-full bg-cyan-500 transition-all" style=format!("width: {pct}%")/>
        </div>
    }
}
