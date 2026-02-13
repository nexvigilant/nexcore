//! Admin: Website leads and contact form submissions

use leptos::prelude::*;

#[component]
pub fn WebsiteLeadsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Website Leads"</h1>
            <p class="mt-1 text-slate-400">"Contact form submissions, demo requests, and trial signups."</p>

            <div class="mt-6 rounded-xl border border-slate-800 overflow-hidden">
                <table class="w-full text-left text-sm">
                    <thead class="bg-slate-900/80 text-xs uppercase text-slate-500">
                        <tr>
                            <th class="px-4 py-3">"Date"</th>
                            <th class="px-4 py-3">"Name"</th>
                            <th class="px-4 py-3">"Email"</th>
                            <th class="px-4 py-3">"Type"</th>
                            <th class="px-4 py-3">"Status"</th>
                        </tr>
                    </thead>
                    <tbody class="text-slate-300">
                        <tr class="border-t border-slate-800">
                            <td class="px-4 py-3" colspan="5">
                                <span class="text-slate-500">"No leads captured yet."</span>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
