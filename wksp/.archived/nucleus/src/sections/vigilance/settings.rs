//! Vigilance settings — API configuration, preferences

use leptos::prelude::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let api_url = RwSignal::new(String::from("http://localhost:3030"));
    let api_key = RwSignal::new(String::new());
    let save_msg = RwSignal::new(String::new());
    let dark_mode = RwSignal::new(true);
    let threshold_profile = RwSignal::new(String::from("default"));

    view! {
        <div class="mx-auto max-w-3xl px-4 py-8">
            <h1 class="text-3xl font-bold text-white">"Vigilance Settings"</h1>
            <p class="mt-2 text-slate-400">"API configuration and preferences"</p>

            // API connection
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"API Connection"</h2>
                <div class="mt-4 space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-400">"API URL"</label>
                        <input type="url"
                            class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white placeholder:text-slate-600 focus:border-amber-500 focus:outline-none"
                            placeholder="https://nexcore-api-xxx.run.app"
                            prop:value=move || api_url.get()
                            on:input=move |ev| api_url.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-400">"API Key"</label>
                        <input type="password"
                            class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white placeholder:text-slate-600 focus:border-amber-500 focus:outline-none"
                            placeholder="X-API-Key header value"
                            prop:value=move || api_key.get()
                            on:input=move |ev| api_key.set(event_target_value(&ev))
                        />
                    </div>
                    <button
                        class="w-full rounded-lg bg-amber-600 px-4 py-2.5 text-sm font-medium text-white hover:bg-amber-500 transition-colors"
                        on:click=move |_| {
                            save_msg.set("Settings saved to session".to_string());
                        }
                    >"Save Connection"</button>
                </div>
            </div>

            // Signal detection defaults
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Signal Detection Defaults"</h2>
                <div class="mt-4">
                    <label class="block text-sm font-medium text-slate-400">"Default Threshold Profile"</label>
                    <select
                        class="mt-1 w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-white focus:border-amber-500 focus:outline-none"
                        on:change=move |ev| threshold_profile.set(event_target_value(&ev))
                    >
                        <option value="default" selected>"Default (PRR \u{2265} 2.0, n \u{2265} 3)"</option>
                        <option value="strict">"Strict (PRR \u{2265} 3.0, n \u{2265} 5)"</option>
                        <option value="sensitive">"Sensitive (PRR \u{2265} 1.5, n \u{2265} 2)"</option>
                    </select>
                </div>

                <div class="mt-6">
                    <h3 class="text-sm font-medium text-slate-400">"Threshold Comparison"</h3>
                    <div class="mt-2 overflow-x-auto">
                        <table class="w-full text-left text-sm">
                            <thead>
                                <tr class="border-b border-slate-700">
                                    <th class="pb-2 text-slate-500">"Metric"</th>
                                    <th class="pb-2 text-slate-500">"Default"</th>
                                    <th class="pb-2 text-slate-500">"Strict"</th>
                                    <th class="pb-2 text-slate-500">"Sensitive"</th>
                                </tr>
                            </thead>
                            <tbody class="text-slate-300">
                                <tr class="border-b border-slate-800"><td class="py-1.5">"PRR"</td><td>"\u{2265} 2.0"</td><td>"\u{2265} 3.0"</td><td>"\u{2265} 1.5"</td></tr>
                                <tr class="border-b border-slate-800"><td class="py-1.5">"Chi\u{00B2}"</td><td>"\u{2265} 3.841"</td><td>"\u{2265} 6.635"</td><td>"\u{2265} 2.706"</td></tr>
                                <tr class="border-b border-slate-800"><td class="py-1.5">"n"</td><td>"\u{2265} 3"</td><td>"\u{2265} 5"</td><td>"\u{2265} 2"</td></tr>
                                <tr class="border-b border-slate-800"><td class="py-1.5">"ROR CI"</td><td>"> 1.0"</td><td>"> 2.0"</td><td>"> 1.0"</td></tr>
                                <tr class="border-b border-slate-800"><td class="py-1.5">"IC025"</td><td>"> 0"</td><td>"> 1.0"</td><td>"> -0.5"</td></tr>
                                <tr><td class="py-1.5">"EB05"</td><td>"\u{2265} 2.0"</td><td>"\u{2265} 3.0"</td><td>"\u{2265} 1.5"</td></tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            // Appearance
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"Appearance"</h2>
                <div class="mt-4 flex items-center justify-between">
                    <span class="text-sm text-slate-300">"Dark Mode"</span>
                    <button
                        class=move || { if dark_mode.get() {
                            "rounded-full bg-amber-600 px-4 py-1.5 text-xs font-medium text-white"
                        } else {
                            "rounded-full border border-slate-600 px-4 py-1.5 text-xs text-slate-400"
                        }}
                        on:click=move |_| dark_mode.update(|v| *v = !*v)
                    >
                        {move || if dark_mode.get() { "ON" } else { "OFF" }}
                    </button>
                </div>
            </div>

            // About
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <h2 class="text-lg font-semibold text-white">"About"</h2>
                <div class="mt-3 space-y-1 text-sm text-slate-400">
                    <p>"Nucleus v0.1.0"</p>
                    <p>"NexVigilant Platform \u{2014} Empowerment Through Vigilance"</p>
                    <p>"Built with Leptos 0.7 + Axum (100% Rust)"</p>
                </div>
            </div>

            // Success message
            <Show when=move || !save_msg.get().is_empty()>
                <div class="mt-4 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-400">
                    {move || save_msg.get()}
                </div>
            </Show>
        </div>
    }
}
