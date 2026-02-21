//! Text input component with live word count
//!
//! Tier: T2-C | Primitives: ς State, N Quantity

use leptos::prelude::*;

/// Feature labels for reference.
const FEATURE_NAMES: [&str; 5] = [
    "Zipf Deviation",
    "Entropy Std",
    "Burstiness",
    "Perplexity Var",
    "TTR Deviation",
];

#[component]
pub fn TextInput(
    text: RwSignal<String>,
) -> impl IntoView {
    let word_count = move || {
        text.get()
            .split_whitespace()
            .count()
    };

    view! {
        <div class="flex flex-col gap-2">
            <label class="text-sm font-semibold text-gray-300">"Paste text to analyze"</label>
            <textarea
                class="w-full h-64 p-3 bg-gray-800 border border-gray-600 rounded-lg text-gray-100 \
                       font-mono text-sm resize-y focus:border-blue-500 focus:outline-none"
                placeholder="Paste at least 50 words of text here to analyze for AI-generated content..."
                prop:value=move || text.get()
                on:input=move |ev| {
                    text.set(event_target_value(&ev));
                }
            />
            <div class="flex justify-between items-center text-sm text-gray-400">
                <span class="text-xs text-gray-500">
                    "Features: "
                    {FEATURE_NAMES.join(" | ")}
                </span>
                <span class=move || {
                    let count = word_count();
                    if count < 50 {
                        "text-red-400 font-semibold"
                    } else {
                        "text-green-400 font-semibold"
                    }
                }>
                    {move || format!("{} words", word_count())}
                </span>
            </div>
        </div>
    }
}
