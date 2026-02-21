//! Admin: New course — course creation form

use leptos::prelude::*;

#[component]
pub fn AcademyCoursesNewPage() -> impl IntoView {
    let title = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <div>
                <a href="/admin/academy/courses" class="text-cyan-400 hover:text-cyan-300 text-sm font-mono">"\u{2190} Back to Courses"</a>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight mt-2">"New Course"</h1>
                <p class="mt-1 text-slate-400">"Create a new Academy course with modules, assessments, and KSB mappings."</p>
            </div>

            /* Form */
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6 space-y-6">
                <div>
                    <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Course Title"</label>
                    <input
                        type="text"
                        placeholder="e.g., Introduction to Pharmacovigilance"
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                        prop:value=move || title.get()
                        on:input=move |ev| title.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"URL Slug"</label>
                    <input
                        type="text"
                        placeholder="intro-to-pv"
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                        prop:value=move || slug.get()
                        on:input=move |ev| slug.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Description"</label>
                    <textarea
                        class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono h-28 resize-none placeholder:text-slate-600"
                        placeholder="Course description..."
                        prop:value=move || description.get()
                        on:input=move |ev| description.set(event_target_value(&ev))
                    />
                </div>
                <div class="grid gap-4 sm:grid-cols-2">
                    <div>
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Domain"</label>
                        <select class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono">
                            <option>"Signal Detection"</option>
                            <option>"Case Processing"</option>
                            <option>"Aggregate Reporting"</option>
                            <option>"Risk Management"</option>
                            <option>"Benefit-Risk"</option>
                            <option>"Regulatory"</option>
                            <option>"PV Systems"</option>
                        </select>
                    </div>
                    <div>
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Difficulty"</label>
                        <select class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono">
                            <option>"Beginner"</option>
                            <option>"Intermediate"</option>
                            <option>"Advanced"</option>
                            <option>"Expert"</option>
                        </select>
                    </div>
                </div>
                <div class="grid gap-4 sm:grid-cols-2">
                    <div>
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Estimated Duration"</label>
                        <input
                            type="text"
                            placeholder="e.g., 12 hours"
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                        />
                    </div>
                    <div>
                        <label class="text-[10px] text-slate-500 font-bold uppercase ml-1 block mb-1.5 font-mono tracking-widest">"Prerequisites"</label>
                        <input
                            type="text"
                            placeholder="e.g., PV Fundamentals"
                            class="w-full rounded-lg border border-slate-700 bg-slate-950 px-4 py-3 text-sm text-white focus:border-cyan-500 focus:outline-none font-mono placeholder:text-slate-600"
                        />
                    </div>
                </div>

                /* Module builder placeholder */
                <div class="rounded-lg border border-dashed border-slate-700 bg-slate-950/50 p-6 text-center">
                    <p class="text-sm text-slate-500 font-mono">"Module builder will appear after course creation"</p>
                    <p class="text-[10px] text-slate-600 mt-1">"Add modules, lessons, assessments, and KSB mappings"</p>
                </div>

                /* Actions */
                <div class="flex justify-end gap-3 pt-2">
                    <a href="/admin/academy/courses" class="px-4 py-2.5 border border-slate-700 text-slate-400 hover:text-white rounded-lg text-xs font-bold transition-colors font-mono uppercase">"Cancel"</a>
                    <button class="px-4 py-2.5 border border-slate-700 text-slate-400 hover:text-white rounded-lg text-xs font-bold transition-colors font-mono uppercase">"Save Draft"</button>
                    <button class="px-6 py-2.5 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-xs font-bold transition-colors font-mono uppercase">"Create Course"</button>
                </div>
            </div>
        </div>
    }
}
