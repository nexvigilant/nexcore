//! Admin: Platform settings — config, billing, integrations

use leptos::prelude::*;

#[component]
pub fn SettingsAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-white">"Platform Settings"</h1>
                    <p class="mt-1 text-slate-400">"Configuration, billing, and integrations"</p>
                </div>
                <a href="/admin" class="text-sm text-slate-400 hover:text-white transition-colors">"\u{2190} Dashboard"</a>
            </div>

            // General
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"General"</h2>
                <div class="mt-4 space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-400">"Platform Name"</label>
                        <input type="text" value="NexVigilant Nucleus"
                            class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white focus:border-amber-500 focus:outline-none"/>
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-400">"Support Email"</label>
                        <input type="email" placeholder="support@nexvigilant.com"
                            class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white placeholder:text-slate-600 focus:border-amber-500 focus:outline-none"/>
                    </div>
                </div>
            </div>

            // Billing
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Billing"</h2>
                <div class="mt-4 grid gap-4 sm:grid-cols-3">
                    <div class="rounded-lg bg-slate-800/50 p-4 text-center">
                        <p class="text-xs text-slate-500">"MRR"</p>
                        <p class="mt-1 text-2xl font-bold text-emerald-400">"$0"</p>
                    </div>
                    <div class="rounded-lg bg-slate-800/50 p-4 text-center">
                        <p class="text-xs text-slate-500">"Active Subscriptions"</p>
                        <p class="mt-1 text-2xl font-bold text-cyan-400">"0"</p>
                    </div>
                    <div class="rounded-lg bg-slate-800/50 p-4 text-center">
                        <p class="text-xs text-slate-500">"Churn Rate"</p>
                        <p class="mt-1 text-2xl font-bold text-amber-400">"0%"</p>
                    </div>
                </div>
            </div>

            // Integrations
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Integrations"</h2>
                <div class="mt-4 space-y-3">
                    {[("Firebase Auth", "Authentication provider", true),
                      ("Firestore", "Database", true),
                      ("Stripe", "Payment processing", false),
                      ("SendGrid", "Email delivery", false),
                      ("nexcore API", "Vigilance backend", true)].into_iter().map(|(name, desc, connected)| {
                        view! {
                            <div class="flex items-center justify-between rounded-lg border border-slate-800 bg-slate-900/30 px-4 py-3">
                                <div>
                                    <p class="font-medium text-white">{name}</p>
                                    <p class="text-xs text-slate-500">{desc}</p>
                                </div>
                                {if connected {
                                    view! { <span class="rounded-full bg-emerald-500/10 px-2.5 py-0.5 text-xs font-medium text-emerald-400">"Connected"</span> }.into_any()
                                } else {
                                    view! { <button class="rounded-lg border border-slate-600 px-3 py-1 text-xs text-slate-400 hover:border-slate-500">"Connect"</button> }.into_any()
                                }}
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Danger zone
            <div class="mt-6 rounded-xl border border-red-500/30 bg-red-500/5 p-6">
                <h2 class="text-lg font-semibold text-red-400">"Danger Zone"</h2>
                <div class="mt-4 space-y-3">
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-sm text-white">"Maintenance Mode"</p>
                            <p class="text-xs text-slate-500">"Take platform offline for maintenance"</p>
                        </div>
                        <button class="rounded-lg border border-red-500/50 px-3 py-1.5 text-xs text-red-400 hover:bg-red-500/10">"Enable"</button>
                    </div>
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-sm text-white">"Export All Data"</p>
                            <p class="text-xs text-slate-500">"Download complete platform data"</p>
                        </div>
                        <button class="rounded-lg border border-slate-600 px-3 py-1.5 text-xs text-slate-400 hover:border-slate-500">"Export"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
