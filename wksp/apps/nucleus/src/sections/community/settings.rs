//! Community settings — notification preferences and privacy

use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Community Settings"</h1>
            <p class="mt-2 text-slate-400">"Manage your community preferences, notifications, and privacy."</p>

            <div class="mt-8 space-y-6">
                <SettingsSection title="Notifications">
                    <ToggleRow label="New replies to my posts" default_on=true/>
                    <ToggleRow label="Likes on my posts" default_on=true/>
                    <ToggleRow label="New followers" default_on=true/>
                    <ToggleRow label="Circle invitations" default_on=true/>
                    <ToggleRow label="Direct messages" default_on=true/>
                    <ToggleRow label="Weekly digest email" default_on=false/>
                </SettingsSection>

                <SettingsSection title="Privacy">
                    <ToggleRow label="Show profile to non-members" default_on=true/>
                    <ToggleRow label="Allow direct messages from anyone" default_on=false/>
                    <ToggleRow label="Show online status" default_on=true/>
                </SettingsSection>
            </div>

            <button class="mt-8 rounded-lg bg-cyan-500 px-6 py-2.5 font-medium text-white hover:bg-cyan-400 transition-colors">
                "Save Settings"
            </button>
        </div>
    }
}

#[component]
fn SettingsSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
            <h2 class="text-lg font-semibold text-white">{title}</h2>
            <div class="mt-4 space-y-3">
                {children()}
            </div>
        </div>
    }
}

#[component]
fn ToggleRow(label: &'static str, default_on: bool) -> impl IntoView {
    let enabled = RwSignal::new(default_on);
    view! {
        <div class="flex items-center justify-between">
            <span class="text-sm text-slate-300">{label}</span>
            <button
                class=move || if enabled.get() {
                    "h-6 w-10 rounded-full bg-cyan-500 relative transition-colors"
                } else {
                    "h-6 w-10 rounded-full bg-slate-700 relative transition-colors"
                }
                on:click=move |_| enabled.set(!enabled.get())
            >
                <span class=move || format!(
                    "absolute top-0.5 h-5 w-5 rounded-full bg-white transition-transform {}",
                    if enabled.get() { "left-[18px]" } else { "left-0.5" }
                )/>
            </button>
        </div>
    }
}
