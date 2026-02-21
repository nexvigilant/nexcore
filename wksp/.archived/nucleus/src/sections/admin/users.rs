//! Admin: User management — accounts, roles, permissions, waitlist

use leptos::prelude::*;

#[component]
pub fn UsersAdminPage() -> impl IntoView {
    let active_tab = RwSignal::new("accounts");

    view! {
        <div class="mx-auto max-w-6xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"User Admin"</h1>
                    <p class="mt-1 text-slate-400">"Accounts, roles, permissions, and waitlist"</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors">"\u{2190} Dashboard"</a>
            </div>

            <div class="mt-6 flex gap-4 border-b border-slate-800 pb-4">
                {["accounts", "roles", "waitlist", "leads"].into_iter().map(|tab| {
                    view! {
                        <button
                            class=move || { if active_tab.get() == tab {
                                "text-sm font-medium text-amber-400 border-b-2 border-amber-400 pb-1"
                            } else {
                                "text-sm text-slate-400 hover:text-white transition-colors"
                            }}
                            on:click=move |_| active_tab.set(tab)
                        >{tab.to_uppercase()}</button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <Show when=move || active_tab.get() == "accounts">
                <div class="mt-6">
                    <div class="flex items-center justify-between">
                        <h2 class="text-lg font-semibold text-white">"User Accounts"</h2>
                        <div class="flex gap-2">
                            <input type="search" placeholder="Search users..."
                                class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-white placeholder:text-slate-500 focus:border-amber-500 focus:outline-none"/>
                            <button class="rounded-lg bg-amber-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-amber-500">"+ Invite User"</button>
                        </div>
                    </div>
                    <div class="mt-4 overflow-x-auto">
                        <table class="w-full text-left text-sm">
                            <thead>
                                <tr class="border-b border-slate-700">
                                    <th class="pb-3 text-slate-500">"Email"</th>
                                    <th class="pb-3 text-slate-500">"Role"</th>
                                    <th class="pb-3 text-slate-500">"Status"</th>
                                    <th class="pb-3 text-slate-500">"Joined"</th>
                                    <th class="pb-3 text-slate-500">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td class="py-6 text-center text-slate-500" colspan="5">"No users registered"</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "roles">
                <div class="mt-6">
                    <h2 class="text-lg font-semibold text-white">"Roles & Permissions"</h2>
                    <div class="mt-4 space-y-3">
                        {[("Admin", "Full platform access", "text-red-400"),
                          ("Moderator", "Community moderation + content review", "text-amber-400"),
                          ("Instructor", "Academy course management", "text-cyan-400"),
                          ("Member", "Standard access to all member features", "text-emerald-400"),
                          ("Free", "Limited access to public content", "text-slate-400")].into_iter().map(|(role, desc, color)| {
                            view! {
                                <div class="flex items-center justify-between rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                                    <div>
                                        <p class=format!("font-medium {color}")>{role}</p>
                                        <p class="text-xs text-slate-500">{desc}</p>
                                    </div>
                                    <button class="text-xs text-amber-400">"Edit"</button>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "waitlist">
                <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                    <p class="text-2xl font-bold text-amber-400">"0"</p>
                    <p class="mt-2 text-slate-400">"People on waitlist"</p>
                    <p class="mt-2 text-sm text-slate-500">"Waitlist signups from the marketing site."</p>
                </div>
            </Show>

            <Show when=move || active_tab.get() == "leads">
                <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-8 text-center">
                    <p class="text-2xl font-bold text-cyan-400">"0"</p>
                    <p class="mt-2 text-slate-400">"Website leads"</p>
                    <p class="mt-2 text-sm text-slate-500">"Contact form submissions and consulting inquiries."</p>
                </div>
            </Show>
        </div>
    }
}
