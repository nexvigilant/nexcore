//! User profile — account details, subscription, preferences

use leptos::prelude::*;
use crate::auth::use_auth;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let auth = use_auth();

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Profile"</h1>
            <p class="mt-2 text-slate-400">"Your account and preferences"</p>

            // Account info
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Account"</h2>
                <div class="mt-4 flex items-center gap-4">
                    <div class="flex h-16 w-16 items-center justify-center rounded-full bg-amber-500/10 text-2xl font-bold text-amber-400">
                        {move || {
                            auth.user.get()
                                .and_then(|u| u.display_name)
                                .map(|n| n.chars().next().unwrap_or('?').to_string())
                                .unwrap_or_else(|| "?".to_string())
                        }}
                    </div>
                    <div>
                        <p class="text-lg font-medium text-white">{move || {
                            auth.user.get()
                                .and_then(|u| u.display_name)
                                .unwrap_or_else(|| "Anonymous".to_string())
                        }}</p>
                        <p class="text-sm text-slate-400">{move || {
                            auth.user.get()
                                .map(|u| u.email)
                                .unwrap_or_default()
                        }}</p>
                    </div>
                </div>
            </div>

            // Subscription
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Subscription"</h2>
                <div class="mt-4 flex items-center justify-between">
                    <div>
                        <p class="text-sm text-white">"Current Plan"</p>
                        <p class="text-xs text-slate-500">"Free tier"</p>
                    </div>
                    <button class="rounded-lg bg-amber-600 px-4 py-2 text-sm font-medium text-white hover:bg-amber-500 transition-colors">"Upgrade"</button>
                </div>
            </div>

            // Competency snapshot
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Competency Snapshot"</h2>
                <div class="mt-4 grid gap-3 sm:grid-cols-2">
                    <div class="rounded-lg bg-slate-800/50 p-4 text-center">
                        <p class="text-xs text-slate-500">"Courses Enrolled"</p>
                        <p class="mt-1 text-2xl font-bold text-cyan-400">"0"</p>
                    </div>
                    <div class="rounded-lg bg-slate-800/50 p-4 text-center">
                        <p class="text-xs text-slate-500">"Certificates"</p>
                        <p class="mt-1 text-2xl font-bold text-amber-400">"0"</p>
                    </div>
                    <div class="rounded-lg bg-slate-800/50 p-4 text-center">
                        <p class="text-xs text-slate-500">"Community Posts"</p>
                        <p class="mt-1 text-2xl font-bold text-violet-400">"0"</p>
                    </div>
                    <div class="rounded-lg bg-slate-800/50 p-4 text-center">
                        <p class="text-xs text-slate-500">"Connections"</p>
                        <p class="mt-1 text-2xl font-bold text-emerald-400">"0"</p>
                    </div>
                </div>
            </div>

            // Preferences
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Preferences"</h2>
                <div class="mt-4 space-y-3">
                    <div class="flex items-center justify-between rounded-lg bg-slate-800/50 px-4 py-3">
                        <span class="text-sm text-slate-300">"Email notifications"</span>
                        <span class="text-xs text-emerald-400">"On"</span>
                    </div>
                    <div class="flex items-center justify-between rounded-lg bg-slate-800/50 px-4 py-3">
                        <span class="text-sm text-slate-300">"Public profile"</span>
                        <span class="text-xs text-emerald-400">"On"</span>
                    </div>
                    <div class="flex items-center justify-between rounded-lg bg-slate-800/50 px-4 py-3">
                        <span class="text-sm text-slate-300">"Theme"</span>
                        <span class="text-xs text-slate-400">"Dark"</span>
                    </div>
                </div>
            </div>

            // Account actions
            <div class="mt-6 rounded-xl border border-red-500/20 bg-red-500/5 p-6">
                <h2 class="text-lg font-semibold text-red-400">"Account Actions"</h2>
                <div class="mt-4 flex gap-3">
                    <button
                        class="rounded-lg border border-slate-600 px-4 py-2 text-sm text-slate-300 hover:border-slate-500 transition-colors"
                        on:click=move |_| auth.sign_out()
                    >"Sign Out"</button>
                    <button class="rounded-lg border border-red-500/50 px-4 py-2 text-sm text-red-400 hover:bg-red-500/10 transition-colors">"Delete Account"</button>
                </div>
            </div>
        </div>
    }
}
