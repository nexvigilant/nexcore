//! Certificate verification page

use leptos::prelude::*;

#[component]
pub fn VerifyPage() -> impl IntoView {
    let code = RwSignal::new(String::new());
    let result = RwSignal::new(Option::<&'static str>::None);

    let on_verify = move |_| {
        let c = code.get();
        if c.is_empty() {
            result.set(Some("Please enter a verification code."));
        } else {
            // TODO: Wire to Firestore certificate lookup
            result.set(Some("Certificate verification will be available when connected to the backend."));
        }
    };

    view! {
        <div class="mx-auto max-w-xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Verify Certificate"</h1>
            <p class="mt-3 text-slate-400">
                "Enter a certificate verification code to confirm its authenticity."
            </p>

            <div class="mt-8 space-y-4">
                <input
                    type="text"
                    placeholder="Enter verification code (e.g., NV-CERT-2026-XXXX)"
                    class="w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                    on:input=move |ev| code.set(event_target_value(&ev))
                />
                <button
                    class="w-full rounded-lg bg-cyan-500 px-6 py-3 font-medium text-white hover:bg-cyan-400 transition-colors"
                    on:click=on_verify
                >
                    "Verify"
                </button>
            </div>

            {move || result.get().map(|msg| view! {
                <div class="mt-6 rounded-lg border border-slate-700 bg-slate-900/50 p-4 text-sm text-slate-300">
                    {msg}
                </div>
            })}
        </div>
    }
}
