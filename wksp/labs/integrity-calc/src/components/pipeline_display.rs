//! Pipeline details — collapsible step-by-step view
//!
//! Tier: T2-C | Primitives: σ Sequence, N Quantity

use leptos::prelude::*;
use crate::pipeline::PipelineResult;

#[component]
pub fn PipelineDisplay(result: PipelineResult) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <h3 class="text-sm font-bold text-gray-300 uppercase tracking-wide mb-3">
                "Pipeline Details"
            </h3>

            // Stage 1: Tokenization
            <details class="mb-1">
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1">
                    "Stage 1: Tokenization"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("Total tokens: {}", result.total_tokens)}</div>
                    <div>{format!("Unique tokens: {}", result.unique_tokens)}</div>
                    <div>{format!("Type-Token Ratio: {:.4}", result.ttr)}</div>
                    <div>{format!("TTR deviation from 0.7: {:.4}", result.ttr_deviation)}</div>
                </div>
            </details>

            // Stage 2a: Zipf
            <details class="mb-1">
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1">
                    "Stage 2a: Zipf\u{2019}s Law Deviation"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("Zipf alpha: {:.4}", result.zipf_alpha)}</div>
                    <div>{format!("R\u{00B2}: {:.4}", result.zipf_r_squared)}</div>
                    <div>{format!("Deviation from 1.0: {:.4}", result.zipf_deviation)}</div>
                    <div class="text-gray-500">"Human text: alpha \u{2248} 1.0 (Zipf\u{2019}s law)"</div>
                </div>
            </details>

            // Stage 2b: Entropy
            <details class="mb-1">
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1">
                    "Stage 2b: Shannon Entropy Variance"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("Mean entropy: {:.4}", result.entropy_mean)}</div>
                    <div>{format!("Std deviation: {:.4}", result.entropy_std)}</div>
                    <div>{format!("Windows analyzed: {}", result.entropy_window_count)}</div>
                    <div class="text-gray-500">"Low std = suspiciously uniform (AI-like)"</div>
                </div>
            </details>

            // Stage 2c: Burstiness
            <details class="mb-1">
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1">
                    "Stage 2c: Burstiness Coefficient"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("Coefficient: {:.4}", result.burstiness_coeff)}</div>
                    <div>{format!("Tokens with repeats: {}", result.burstiness_tokens_analyzed)}</div>
                    <div class="text-gray-500">"B > 0 = bursty (human), B \u{2248} 0 = smooth (AI)"</div>
                </div>
            </details>

            // Stage 2d: Perplexity
            <details class="mb-1">
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1">
                    "Stage 2d: Perplexity Variance"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("Mean sentence entropy: {:.4}", result.perplexity_mean)}</div>
                    <div>{format!("Variance: {:.4}", result.perplexity_var)}</div>
                    <div>{format!("Sentences: {}", result.perplexity_sentence_count)}</div>
                    <div class="text-gray-500">"Low variance = consistent surprise (AI-like)"</div>
                </div>
            </details>

            // Stage 2e: TTR
            <details class="mb-1">
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1">
                    "Stage 2e: TTR Deviation"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("TTR: {:.4}", result.ttr)}</div>
                    <div>{format!("Deviation from 0.7 baseline: {:.4}", result.ttr_deviation)}</div>
                    <div>{format!("Normalized: {:.4}", result.normalized[4])}</div>
                    <div class="text-gray-500">"Human baseline TTR \u{2248} 0.7"</div>
                </div>
            </details>

            // Stage 3: Aggregation
            <details class="mb-1">
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1">
                    "Stage 3: Aggregation (Beer-Lambert + Hill)"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("Beer-Lambert sum: {:.4}", result.beer_lambert_score)}</div>
                    <div>{format!("Composite (normalized): {:.4}", result.composite)}</div>
                    <div>{format!("Hill amplified (K=0.5, n=2.5): {:.4}", result.hill_score)}</div>
                    <div class="text-gray-500">"Hill sharpens distinction near decision boundary"</div>
                </div>
            </details>

            // Stage 4: Classification
            <details class="mb-1" open>
                <summary class="cursor-pointer text-sm text-gray-300 hover:text-white py-1 font-semibold">
                    "Stage 4: Classification (Arrhenius Gate)"
                </summary>
                <div class="pl-6 py-2 text-xs text-gray-400 font-mono space-y-1">
                    <div>{format!("Arrhenius prob: {:.4}", result.probability)}</div>
                    <div>{format!("Threshold: {:.4}", result.threshold)}</div>
                    <div class="font-semibold text-gray-200">
                        {format!("Verdict: {}", result.verdict)}
                    </div>
                    <div>{format!("Confidence: {:.1}%", result.confidence * 100.0)}</div>
                    <div class="text-gray-500">
                        {format!(
                            "Ea={}, scale={} | prob = exp(-Ea / (hill \u{00D7} scale))",
                            3.0, 10.0
                        )}
                    </div>
                </div>
            </details>
        </div>
    }
}
