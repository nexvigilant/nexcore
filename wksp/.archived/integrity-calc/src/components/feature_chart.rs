//! Feature analysis bar chart — 5 horizontal bars
//!
//! Tier: T2-C | Primitives: N Quantity, κ Comparison

use leptos::prelude::*;
use crate::pipeline::PipelineResult;

const FEATURE_NAMES: [&str; 5] = [
    "Zipf Deviation",
    "Entropy Std",
    "Burstiness",
    "Perplexity Var",
    "TTR Deviation",
];

const BAR_COLORS: [&str; 5] = [
    "bg-blue-500",
    "bg-purple-500",
    "bg-teal-500",
    "bg-orange-500",
    "bg-pink-500",
];

#[component]
pub fn FeatureChart(result: PipelineResult) -> impl IntoView {
    let bars: Vec<_> = (0..5)
        .map(|i| {
            let val = result.normalized[i];
            let weight = result.weights[i];
            let pct = (val * 100.0).clamp(0.0, 100.0);
            let color = BAR_COLORS[i];
            let name = FEATURE_NAMES[i];

            view! {
                <div class="flex items-center gap-2 text-sm">
                    <span class="w-28 text-gray-400 text-right text-xs">{name}</span>
                    <div class="flex-1 bg-gray-700 rounded-full h-4 overflow-hidden">
                        <div
                            class=format!("{color} h-full rounded-full transition-all duration-500")
                            style=format!("width: {pct:.0}%")
                        />
                    </div>
                    <span class="w-24 text-gray-300 text-xs font-mono">
                        {format!("{val:.2} (w={weight:.1})")}
                    </span>
                </div>
            }
        })
        .collect();

    view! {
        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <h3 class="text-sm font-bold text-gray-300 uppercase tracking-wide mb-3">
                "Feature Analysis"
            </h3>
            <div class="flex flex-col gap-2">
                {bars}
            </div>
            <div class="mt-3 pt-2 border-t border-gray-700 text-xs text-gray-400 font-mono">
                {format!(
                    "Beer-Lambert: {:.3}  Composite: {:.3}  Hill: {:.3}",
                    result.beer_lambert_score, result.composite, result.hill_score
                )}
            </div>
        </div>
    }
}
