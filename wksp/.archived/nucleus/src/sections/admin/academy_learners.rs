//! Admin: Academy learner management

use leptos::prelude::*;

#[component]
pub fn AcademyLearnersPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Learner Management"</h1>
            <p class="mt-1 text-slate-400">"View enrollments, progress, and learner analytics."</p>

            <div class="mt-6 flex gap-4">
                <input
                    type="text"
                    placeholder="Search learners..."
                    class="flex-1 rounded-lg border border-slate-700 bg-slate-800 px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                />
            </div>

            <div class="mt-6 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-xs uppercase text-slate-500">
                        <tr>
                            <th class="px-4 py-3">"Name"</th>
                            <th class="px-4 py-3">"Enrolled Courses"</th>
                            <th class="px-4 py-3">"Avg Progress"</th>
                            <th class="px-4 py-3">"Certificates"</th>
                            <th class="px-4 py-3">"Last Active"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        <tr class="border-t border-slate-800">
                            <td class="px-4 py-3" colspan="5">
                                <span class="text-slate-500">"No learners enrolled yet."</span>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
