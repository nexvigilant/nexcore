//! Find Your Home — guided quiz to match members to relevant circles

use leptos::prelude::*;

#[component]
pub fn FindYourHomePage() -> impl IntoView {
    let (current_step, set_current_step) = signal(0u32);
    let (selected_focus, set_selected_focus) = signal(Vec::<&'static str>::new());
    let (experience_level, set_experience_level) = signal(Option::<&'static str>::None);
    let (selected_interests, set_selected_interests) = signal(Vec::<&'static str>::new());
    let (show_results, set_show_results) = signal(false);

    view! {
        <div class="mx-auto max-w-3xl px-4 py-12">
            <div class="mb-12 text-center">
                <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"FIND YOUR HOME"</h1>
                <p class="mt-3 text-slate-400 max-w-lg mx-auto">"Answer a few questions and we'll match you with the circles where you'll thrive"</p>
            </div>

            /* Progress indicator */
            <div class="flex items-center justify-center gap-2 mb-12">
                {(0..3u32).map(|i| {
                    let step_class = move || {
                        if current_step.get() >= i {
                            "h-1.5 w-12 rounded-full bg-cyan-500 transition-all"
                        } else {
                            "h-1.5 w-12 rounded-full bg-slate-800 transition-all"
                        }
                    };
                    view! {
                        <div class=step_class></div>
                    }
                }).collect_view()}
            </div>

            <Show when=move || !show_results.get()>
                /* Step 1: PV Focus Area */
                <Show when=move || current_step.get() == 0>
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                        <div class="flex items-center gap-3 mb-2">
                            <span class="flex h-7 w-7 items-center justify-center rounded-full bg-cyan-500/20 text-xs font-bold text-cyan-400 font-mono">"1"</span>
                            <h2 class="text-xl font-bold text-white">"What areas of pharmacovigilance are you focused on?"</h2>
                        </div>
                        <p class="text-sm text-slate-500 mb-6 ml-10">"Select all that apply"</p>

                        <div class="flex flex-wrap gap-2 ml-10">
                            <FocusOption label="Signal Detection" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="ICSR Processing" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="Benefit-Risk Assessment" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="Regulatory Submissions" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="Literature Monitoring" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="Risk Management" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="Clinical Safety" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="Medical Devices" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="AI & Automation" selected=selected_focus set_selected=set_selected_focus />
                            <FocusOption label="Quality Systems" selected=selected_focus set_selected=set_selected_focus />
                        </div>

                        <div class="mt-8 flex justify-end">
                            <button
                                class="rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20 disabled:opacity-50"
                                disabled=move || selected_focus.get().is_empty()
                                on:click=move |_| set_current_step.set(1)
                            >
                                "NEXT"
                            </button>
                        </div>
                    </div>
                </Show>

                /* Step 2: Experience Level */
                <Show when=move || current_step.get() == 1>
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                        <div class="flex items-center gap-3 mb-2">
                            <span class="flex h-7 w-7 items-center justify-center rounded-full bg-cyan-500/20 text-xs font-bold text-cyan-400 font-mono">"2"</span>
                            <h2 class="text-xl font-bold text-white">"What is your experience level?"</h2>
                        </div>
                        <p class="text-sm text-slate-500 mb-6 ml-10">"This helps us recommend the right depth of discussion"</p>

                        <div class="space-y-3 ml-10">
                            <ExperienceOption
                                level="student"
                                title="Student / Career Explorer"
                                description="Learning the fundamentals of drug safety"
                                selected=experience_level
                                set_selected=set_experience_level
                            />
                            <ExperienceOption
                                level="early"
                                title="Early Career (0-3 years)"
                                description="Building core PV competencies"
                                selected=experience_level
                                set_selected=set_experience_level
                            />
                            <ExperienceOption
                                level="mid"
                                title="Mid Career (3-8 years)"
                                description="Deepening expertise, leading projects"
                                selected=experience_level
                                set_selected=set_experience_level
                            />
                            <ExperienceOption
                                level="senior"
                                title="Senior / Leadership (8+ years)"
                                description="Strategic oversight, mentoring others"
                                selected=experience_level
                                set_selected=set_experience_level
                            />
                        </div>

                        <div class="mt-8 flex justify-between">
                            <button
                                class="rounded-lg px-4 py-2 text-sm font-bold text-slate-400 hover:text-white transition-colors"
                                on:click=move |_| set_current_step.set(0)
                            >
                                "BACK"
                            </button>
                            <button
                                class="rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20 disabled:opacity-50"
                                disabled=move || experience_level.get().is_none()
                                on:click=move |_| set_current_step.set(2)
                            >
                                "NEXT"
                            </button>
                        </div>
                    </div>
                </Show>

                /* Step 3: Interests */
                <Show when=move || current_step.get() == 2>
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                        <div class="flex items-center gap-3 mb-2">
                            <span class="flex h-7 w-7 items-center justify-center rounded-full bg-cyan-500/20 text-xs font-bold text-cyan-400 font-mono">"3"</span>
                            <h2 class="text-xl font-bold text-white">"What else interests you?"</h2>
                        </div>
                        <p class="text-sm text-slate-500 mb-6 ml-10">"Beyond PV — what other topics would you like to explore?"</p>

                        <div class="flex flex-wrap gap-2 ml-10">
                            <FocusOption label="Career Growth" selected=selected_interests set_selected=set_selected_interests />
                            <FocusOption label="Industry News" selected=selected_interests set_selected=set_selected_interests />
                            <FocusOption label="Networking Events" selected=selected_interests set_selected=set_selected_interests />
                            <FocusOption label="Job Opportunities" selected=selected_interests set_selected=set_selected_interests />
                            <FocusOption label="Research Collaboration" selected=selected_interests set_selected=set_selected_interests />
                            <FocusOption label="Mentorship" selected=selected_interests set_selected=set_selected_interests />
                            <FocusOption label="Regulatory Updates" selected=selected_interests set_selected=set_selected_interests />
                            <FocusOption label="Technology Trends" selected=selected_interests set_selected=set_selected_interests />
                        </div>

                        <div class="mt-8 flex justify-between">
                            <button
                                class="rounded-lg px-4 py-2 text-sm font-bold text-slate-400 hover:text-white transition-colors"
                                on:click=move |_| set_current_step.set(1)
                            >
                                "BACK"
                            </button>
                            <button
                                class="rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20"
                                on:click=move |_| set_show_results.set(true)
                            >
                                "FIND MY CIRCLES"
                            </button>
                        </div>
                    </div>
                </Show>
            </Show>

            /* Results */
            <Show when=move || show_results.get()>
                <div class="text-center mb-10">
                    <h2 class="text-2xl font-bold text-white font-mono uppercase tracking-tight">"YOUR MATCHES"</h2>
                    <p class="mt-2 text-slate-400">"Based on your answers, here are the circles where you belong"</p>
                </div>

                <div class="space-y-4">
                    <MatchResult
                        name="Signal Detection Practitioners"
                        match_pct=94
                        members=89
                        description="Advanced techniques in disproportionality analysis, data mining, and emerging signal methodologies"
                    />
                    <MatchResult
                        name="AI & Automation in PV"
                        match_pct=87
                        members=67
                        description="Exploring NLP, machine learning, and automation tools transforming drug safety workflows"
                    />
                    <MatchResult
                        name="Risk Management Circle"
                        match_pct=76
                        members=54
                        description="RMPs, REMS, benefit-risk frameworks, and proactive risk minimization strategies"
                    />
                    <MatchResult
                        name="PV Career Network"
                        match_pct=68
                        members=134
                        description="Job opportunities, salary benchmarks, career advice, and professional development"
                    />
                    <MatchResult
                        name="Regulatory Affairs Hub"
                        match_pct=61
                        members=72
                        description="ICH guidelines, regional submissions, PSUR/PBRER writing, and regulatory intelligence"
                    />
                </div>

                <div class="mt-10 flex justify-center gap-4">
                    <button
                        class="rounded-lg border border-slate-700 px-6 py-2.5 text-sm font-bold text-slate-300 hover:bg-slate-800 hover:text-white transition-all"
                        on:click=move |_| {
                            set_show_results.set(false);
                            set_current_step.set(0);
                            set_selected_focus.set(Vec::new());
                            set_experience_level.set(None);
                            set_selected_interests.set(Vec::new());
                        }
                    >
                        "RETAKE QUIZ"
                    </button>
                    <a
                        href="/community/circles"
                        class="rounded-lg bg-cyan-600 px-6 py-2.5 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20"
                    >
                        "BROWSE ALL CIRCLES"
                    </a>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn FocusOption(
    label: &'static str,
    selected: ReadSignal<Vec<&'static str>>,
    set_selected: WriteSignal<Vec<&'static str>>,
) -> impl IntoView {
    let is_selected = move || selected.get().contains(&label);

    view! {
        <button
            class=move || if is_selected() {
                "rounded-full border border-cyan-500 bg-cyan-500/20 px-4 py-2 text-sm text-cyan-400 font-medium transition-all"
            } else {
                "rounded-full border border-slate-700 px-4 py-2 text-sm text-slate-400 hover:border-slate-500 hover:text-white transition-all"
            }
            on:click=move |_| {
                let mut current = selected.get();
                if let Some(pos) = current.iter().position(|&x| x == label) {
                    current.remove(pos);
                } else {
                    current.push(label);
                }
                set_selected.set(current);
            }
        >
            {label}
        </button>
    }
}

#[component]
fn ExperienceOption(
    level: &'static str,
    title: &'static str,
    description: &'static str,
    selected: ReadSignal<Option<&'static str>>,
    set_selected: WriteSignal<Option<&'static str>>,
) -> impl IntoView {
    let is_selected = move || selected.get() == Some(level);

    view! {
        <button
            class=move || if is_selected() {
                "w-full rounded-xl border border-cyan-500/50 bg-cyan-500/10 p-4 text-left transition-all"
            } else {
                "w-full rounded-xl border border-slate-800 bg-slate-900/30 p-4 text-left hover:border-slate-700 transition-all"
            }
            on:click=move |_| set_selected.set(Some(level))
        >
            <div class="flex items-center gap-3">
                <div class=move || if is_selected() {
                    "h-4 w-4 rounded-full border-2 border-cyan-500 bg-cyan-500 transition-all"
                } else {
                    "h-4 w-4 rounded-full border-2 border-slate-600 transition-all"
                } />
                <div>
                    <p class="text-sm font-bold text-white">{title}</p>
                    <p class="text-xs text-slate-500">{description}</p>
                </div>
            </div>
        </button>
    }
}

#[component]
fn MatchResult(
    name: &'static str,
    match_pct: u32,
    members: u32,
    description: &'static str,
) -> impl IntoView {
    let pct_color = if match_pct >= 85 {
        "text-emerald-400"
    } else if match_pct >= 70 {
        "text-cyan-400"
    } else {
        "text-amber-400"
    };

    let bar_color = if match_pct >= 85 {
        "bg-emerald-500"
    } else if match_pct >= 70 {
        "bg-cyan-500"
    } else {
        "bg-amber-500"
    };

    let joined = RwSignal::new(false);

    view! {
        <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6 hover:border-slate-700 transition-all group">
            <div class="flex items-start justify-between">
                <div class="flex-1">
                    <div class="flex items-center gap-3 mb-1">
                        <h3 class="text-lg font-bold text-white group-hover:text-cyan-400 transition-colors">{name}</h3>
                        <span class=format!("text-sm font-bold font-mono {pct_color}")>{format!("{match_pct}%")}" match"</span>
                    </div>
                    <p class="text-sm text-slate-400 leading-relaxed">{description}</p>
                    <p class="mt-2 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest">{members}" members"</p>
                </div>
                <button
                    class=move || if joined.get() {
                        "ml-4 shrink-0 rounded-lg bg-emerald-500/10 border border-emerald-500/20 px-5 py-2 text-xs font-bold text-emerald-400 uppercase tracking-widest"
                    } else {
                        "ml-4 shrink-0 rounded-lg border border-violet-500/50 px-5 py-2 text-xs font-bold text-violet-400 hover:bg-violet-500 hover:text-white transition-all uppercase tracking-widest"
                    }
                    on:click=move |_| joined.set(true)
                >
                    {move || if joined.get() { "JOINED" } else { "JOIN" }}
                </button>
            </div>
            <div class="mt-4 h-1.5 w-full rounded-full bg-slate-800">
                <div
                    class=format!("h-1.5 rounded-full {} transition-all", bar_color)
                    style=format!("width: {}%", match_pct)
                />
            </div>
        </div>
    }
}
