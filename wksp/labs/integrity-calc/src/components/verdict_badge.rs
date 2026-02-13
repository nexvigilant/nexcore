//! Verdict badge — green/red classification display
//!
//! Tier: T2-C | Primitives: ∂ Boundary, ς State

use leptos::prelude::*;
use crate::pipeline::PipelineResult;
use crate::pipeline::classify::Verdict;

#[component]
pub fn VerdictBadge(result: PipelineResult) -> impl IntoView {
    let is_human = result.verdict == Verdict::Human;

    let badge_class = if is_human {
        "bg-green-900 border-green-500 text-green-300"
    } else {
        "bg-red-900 border-red-500 text-red-300"
    };

    let verdict_text = if is_human {
        "Human Written"
    } else {
        "AI Generated"
    };

    let prob_pct = result.probability * 100.0;
    let conf_pct = result.confidence * 100.0;

    view! {
        <div class=format!("rounded-lg border-2 p-4 {badge_class}")>
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <span class="text-2xl">
                        {if is_human { "\u{2705}" } else { "\u{1F916}" }}
                    </span>
                    <span class="text-xl font-bold">{verdict_text}</span>
                </div>
            </div>
            <div class="mt-2 flex flex-wrap gap-x-6 gap-y-1 text-sm">
                <span>
                    "Probability: "
                    <span class="font-mono font-semibold">{format!("{prob_pct:.1}%")}</span>
                </span>
                <span>
                    "Confidence: "
                    <span class="font-mono font-semibold">{format!("{conf_pct:.1}%")}</span>
                </span>
                <span>
                    "Bloom "
                    {result.bloom_level}
                    ": "
                    {result.bloom_name.clone()}
                </span>
                <span>
                    "Preset: "
                    {result.preset_name.clone()}
                </span>
            </div>
        </div>
    }
}
