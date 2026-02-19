//! Admin: AI course generation — generate course content from KSB framework

use leptos::prelude::*;

#[component]
pub fn AcademyCoursesGeneratePage() -> impl IntoView {
    let selected_domain = RwSignal::new(String::from("signal-detection"));
    let selected_level = RwSignal::new(String::from("intermediate"));
    let selected_duration = RwSignal::new(String::from("8"));
    let generating = RwSignal::new(false);

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <div>
                <a href="/admin/academy/courses" class="text-cyan-400 hover:text-cyan-300 text-sm font-mono">"\u{2190} Back to Courses"</a>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight mt-2">"Generate Course"</h1>
                <p class="mt-1 text-slate-400">"AI-powered course generation from the competency framework."</p>
            </div>

            /* Generation form */
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6 space-y-6">
                <div>
                    <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Target Competency Area"</label>
                    <select
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono"
                        prop:value=move || selected_domain.get()
                        on:change=move |ev| selected_domain.set(event_target_value(&ev))
                    >
                        <option value="signal-detection">"Signal Detection & Management"</option>
                        <option value="case-processing">"Individual Case Safety Reports"</option>
                        <option value="aggregate-reporting">"Aggregate Safety Reporting"</option>
                        <option value="risk-management">"Risk Management & Minimization"</option>
                        <option value="benefit-risk">"Benefit-Risk Assessment"</option>
                        <option value="regulatory">"Regulatory Intelligence & Compliance"</option>
                        <option value="pv-systems">"PV System Quality & Governance"</option>
                    </select>
                </div>
                <div class="grid gap-4 sm:grid-cols-2">
                    <div>
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Target Level"</label>
                        <select
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono"
                            prop:value=move || selected_level.get()
                            on:change=move |ev| selected_level.set(event_target_value(&ev))
                        >
                            <option value="beginner">"Beginner"</option>
                            <option value="intermediate">"Intermediate"</option>
                            <option value="advanced">"Advanced"</option>
                        </select>
                    </div>
                    <div>
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Target Duration (hours)"</label>
                        <select
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono"
                            prop:value=move || selected_duration.get()
                            on:change=move |ev| selected_duration.set(event_target_value(&ev))
                        >
                            <option value="4">"4 hours (short)"</option>
                            <option value="8">"8 hours (standard)"</option>
                            <option value="16">"16 hours (comprehensive)"</option>
                            <option value="24">"24 hours (masterclass)"</option>
                        </select>
                    </div>
                </div>

                /* KSB Preview */
                <div>
                    <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"KSBs to Cover"</label>
                    <div class="rounded-lg border border-slate-700 bg-slate-950 p-4">
                        <div class="flex flex-wrap gap-2">
                            {["K1: Signal detection principles", "K2: Disproportionality analysis", "S1: Run PRR/ROR calculations",
                              "S2: Interpret signal metrics", "B1: Evidence-based decision making", "B2: Regulatory awareness"]
                                .into_iter().map(|ksb| view! {
                                    <span class="rounded-full border border-slate-700 bg-slate-900 px-3 py-1 text-[10px] text-slate-300 font-mono">{ksb}</span>
                                }).collect_view()}
                        </div>
                    </div>
                </div>

                /* Advanced options */
                <div class="rounded-lg border border-slate-800 bg-slate-950/50 p-4 space-y-3">
                    <p class="text-[10px] text-slate-500 font-bold uppercase tracking-widest font-mono">"Advanced Options"</p>
                    <div class="grid gap-3 sm:grid-cols-2 text-sm">
                        <label class="flex items-center gap-2 text-slate-400">
                            <input type="checkbox" class="rounded border-slate-600 bg-slate-900" checked />
                            " Include assessments"
                        </label>
                        <label class="flex items-center gap-2 text-slate-400">
                            <input type="checkbox" class="rounded border-slate-600 bg-slate-900" checked />
                            " Include case studies"
                        </label>
                        <label class="flex items-center gap-2 text-slate-400">
                            <input type="checkbox" class="rounded border-slate-600 bg-slate-900" />
                            " Include regulatory references"
                        </label>
                        <label class="flex items-center gap-2 text-slate-400">
                            <input type="checkbox" class="rounded border-slate-600 bg-slate-900" />
                            " Map to ICH guidelines"
                        </label>
                    </div>
                </div>

                <button
                    on:click=move |_| generating.set(true)
                    disabled=generating
                    class="w-full rounded-lg bg-amber-600 px-6 py-3.5 text-sm font-bold text-white hover:bg-amber-500 transition-all disabled:opacity-50 uppercase tracking-widest font-mono"
                >
                    {move || if generating.get() { "GENERATING COURSE OUTLINE..." } else { "GENERATE COURSE OUTLINE" }}
                </button>
            </div>

            /* Info */
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-5">
                <p class="text-[10px] text-slate-500 font-bold uppercase tracking-widest font-mono mb-2">"How It Works"</p>
                <div class="space-y-2 text-xs text-slate-400">
                    <p>"1. Select the target competency area and difficulty level"</p>
                    <p>"2. AI analyzes the KSB framework to identify relevant knowledge, skills, and behaviors"</p>
                    <p>"3. Course outline is generated with modules, assessments, and learning objectives"</p>
                    <p>"4. Review and edit the generated outline before publishing"</p>
                </div>
            </div>
        </div>
    }
}
