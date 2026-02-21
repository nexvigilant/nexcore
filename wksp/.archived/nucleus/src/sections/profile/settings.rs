//! Profile settings page — user preferences and account configuration

use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Settings"</h1>
            <p class="mt-2 text-slate-400">"Manage your account preferences and notification settings."</p>

            <div class="mt-8 space-y-6">
                /* Notifications */
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-semibold text-white font-mono uppercase tracking-tight">"Notifications"</h2>
                    <div class="mt-4 space-y-4">
                        <ToggleRow label="Email notifications" description="Receive updates about your courses and assessments" default_on=true />
                        <ToggleRow label="Community mentions" description="Get notified when someone mentions you" default_on=true />
                        <ToggleRow label="Signal alerts" description="Alerts for new pharmacovigilance signals" default_on=false />
                    </div>
                </div>

                /* Appearance */
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-semibold text-white font-mono uppercase tracking-tight">"Appearance"</h2>
                    <div class="mt-4 space-y-4">
                        <ToggleRow label="Compact mode" description="Reduce spacing in lists and tables" default_on=false />
                        <ToggleRow label="Show primitives" description="Display Lex Primitiva symbols alongside concepts" default_on=true />
                    </div>
                </div>

                /* Account */
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <h2 class="text-lg font-semibold text-white font-mono uppercase tracking-tight">"Account"</h2>
                    <p class="mt-2 text-sm text-slate-500">"Account management is handled through your authentication provider."</p>
                    <a
                        href="/profile"
                        class="mt-4 inline-block rounded-lg bg-slate-800 px-5 py-2 text-xs font-bold text-slate-300 hover:bg-slate-700 transition-colors font-mono uppercase tracking-widest"
                    >
                        "View Profile"
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ToggleRow(label: &'static str, description: &'static str, default_on: bool) -> impl IntoView {
    let (enabled, set_enabled) = signal(default_on);

    let toggle_class = move || {
        if enabled.get() {
            "relative inline-flex h-6 w-11 items-center rounded-full bg-cyan-600 transition-colors cursor-pointer"
        } else {
            "relative inline-flex h-6 w-11 items-center rounded-full bg-slate-700 transition-colors cursor-pointer"
        }
    };

    let knob_class = move || {
        if enabled.get() {
            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-6"
        } else {
            "inline-block h-4 w-4 transform rounded-full bg-white transition-transform translate-x-1"
        }
    };

    view! {
        <div class="flex items-center justify-between">
            <div>
                <p class="text-sm font-medium text-white">{label}</p>
                <p class="text-xs text-slate-500">{description}</p>
            </div>
            <button
                on:click=move |_| set_enabled.set(!enabled.get())
                class=toggle_class
            >
                <span class=knob_class></span>
            </button>
        </div>
    }
}
