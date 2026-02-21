//! Configuration panel — Bloom level radio + preset dropdown
//!
//! Tier: T2-C | Primitives: ∂ Boundary, κ Comparison

use leptos::prelude::*;
use crate::pipeline::bloom::BloomThresholds;

#[component]
pub fn ConfigPanel(
    bloom_level: RwSignal<u8>,
    preset: RwSignal<String>,
) -> impl IntoView {
    let bloom_labels = [
        (1u8, "L1: Remember"),
        (2, "L2: Understand"),
        (3, "L3: Apply"),
        (4, "L4: Analyze"),
        (5, "L5: Evaluate"),
        (6, "L6: Create"),
        (7, "L7: Meta-Create"),
    ];

    let threshold = move || {
        let bt = BloomThresholds::from_name(&preset.get());
        bt.threshold_for_level(bloom_level.get()).unwrap_or(0.64)
    };

    view! {
        <div class="flex flex-col gap-3 bg-gray-800 rounded-lg p-4 border border-gray-700">
            <h3 class="text-sm font-bold text-gray-300 uppercase tracking-wide">"Configuration"</h3>

            // Bloom level radio buttons
            <fieldset class="flex flex-col gap-1">
                {bloom_labels.into_iter().map(|(level, label)| {
                    let id = format!("bloom-{level}");
                    let id2 = id.clone();
                    view! {
                        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer \
                                      hover:text-white transition-colors" for=id>
                            <input
                                type="radio"
                                name="bloom"
                                id=id2
                                class="text-blue-500 focus:ring-blue-500"
                                prop:checked=move || bloom_level.get() == level
                                on:change=move |_| bloom_level.set(level)
                            />
                            {label}
                        </label>
                    }
                }).collect::<Vec<_>>()}
            </fieldset>

            // Preset dropdown
            <div class="flex flex-col gap-1 mt-2">
                <label class="text-xs text-gray-400 font-semibold">"Preset"</label>
                <select
                    class="bg-gray-700 text-gray-200 text-sm rounded px-2 py-1 border border-gray-600 \
                           focus:border-blue-500 focus:outline-none"
                    on:change=move |ev| {
                        preset.set(event_target_value(&ev));
                    }
                >
                    <option value="pv_education" selected=move || preset.get() == "pv_education">
                        "PV Education"
                    </option>
                    <option value="strict" selected=move || preset.get() == "strict">
                        "Strict"
                    </option>
                    <option value="lenient" selected=move || preset.get() == "lenient">
                        "Lenient"
                    </option>
                </select>
            </div>

            // Threshold preview
            <div class="mt-2 px-3 py-2 bg-gray-900 rounded text-center">
                <span class="text-xs text-gray-400">"Threshold: "</span>
                <span class="text-lg font-mono font-bold text-yellow-400">
                    {move || format!("{:.2}", threshold())}
                </span>
            </div>
        </div>
    }
}
