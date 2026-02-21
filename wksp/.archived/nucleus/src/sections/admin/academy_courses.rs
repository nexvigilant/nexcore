//! Admin: Academy courses CRUD

use leptos::prelude::*;

#[component]
pub fn AcademyCoursesPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Course Management"</h1>
                    <p class="mt-1 text-slate-400">"Create, edit, and publish Academy courses."</p>
                </div>
                <button class="rounded-lg bg-cyan-500 px-4 py-2 text-sm font-medium text-white hover:bg-cyan-400 transition-colors">
                    "+ New Course"
                </button>
            </div>

            <div class="mt-6 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-xs uppercase text-slate-500">
                        <tr>
                            <th class="px-4 py-3">"Title"</th>
                            <th class="px-4 py-3">"Difficulty"</th>
                            <th class="px-4 py-3">"Modules"</th>
                            <th class="px-4 py-3">"Status"</th>
                            <th class="px-4 py-3">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        <tr class="border-t border-slate-800">
                            <td class="px-4 py-3" colspan="5">
                                <span class="text-slate-500">"No courses yet. Create your first course."</span>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
