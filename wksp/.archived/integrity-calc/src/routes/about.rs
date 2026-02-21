//! About page — algorithm explanation

use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-900 text-gray-100">
            <header class="border-b border-gray-700 bg-gray-900/80 backdrop-blur sticky top-0 z-10">
                <div class="max-w-3xl mx-auto px-4 py-4 flex items-center justify-between">
                    <h1 class="text-2xl font-bold text-white">"How It Works"</h1>
                    <a href="/" class="text-sm text-blue-400 hover:text-blue-300 transition-colors">
                        "\u{2190} Back to Calculator"
                    </a>
                </div>
            </header>

            <main class="max-w-3xl mx-auto px-4 py-8 space-y-8">
                // Overview
                <section>
                    <h2 class="text-xl font-bold text-white mb-3">"Overview"</h2>
                    <p class="text-gray-300 leading-relaxed">
                        "The Integrity Calculator detects AI-generated text through 5 statistical features "
                        "aggregated via chemistry-inspired primitives. It does not use machine learning \u{2014} "
                        "instead, it measures distributional properties of text that differ between human "
                        "and AI writing."
                    </p>
                </section>

                // 5 Features
                <section>
                    <h2 class="text-xl font-bold text-white mb-3">"5 Detection Features"</h2>
                    <div class="space-y-4">
                        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                            <h3 class="font-semibold text-blue-400">"1. Zipf\u{2019}s Law Deviation"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "Human text follows Zipf\u{2019}s law: word frequency \u{221D} 1/rank\u{1D45}. "
                                "LLM text deviates because softmax attention smooths the distribution. "
                                "We compute log-log regression and measure |\u{03B1} - 1.0|."
                            </p>
                        </div>
                        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                            <h3 class="font-semibold text-purple-400">"2. Shannon Entropy Variance"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "Human text has high entropy variance: creative bursts alternate with simple passages. "
                                "LLM text has suspiciously uniform entropy across sliding windows. "
                                "Low standard deviation = suspicious."
                            </p>
                        </div>
                        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                            <h3 class="font-semibold text-teal-400">"3. Burstiness Coefficient"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "Human text is bursty \u{2014} topic words cluster together. "
                                "B = (\u{03C3} - \u{03BC}) / (\u{03C3} + \u{03BC}) of inter-arrival times. "
                                "B > 0 = bursty (human), B \u{2248} 0 = smooth (AI)."
                            </p>
                        </div>
                        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                            <h3 class="font-semibold text-orange-400">"4. Perplexity Variance"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "Per-sentence Shannon entropy measures surprise. Human writing varies \u{2014} "
                                "some sentences are predictable, others creative. LLMs produce "
                                "consistent perplexity across sentences."
                            </p>
                        </div>
                        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                            <h3 class="font-semibold text-pink-400">"5. TTR Deviation"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "Type-Token Ratio = unique words / total words. Human baseline \u{2248} 0.7. "
                                "Deviation from this baseline indicates non-human vocabulary patterns."
                            </p>
                        </div>
                    </div>
                </section>

                // Chemistry Pipeline
                <section>
                    <h2 class="text-xl font-bold text-white mb-3">"Chemistry Aggregation Pipeline"</h2>
                    <div class="space-y-4">
                        <div class="bg-gray-800 rounded-lg p-4 border-l-4 border-yellow-500">
                            <h3 class="font-semibold text-yellow-400">"Step 1: Beer-Lambert Weighted Sum"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "A = \u{03A3}(\u{03B5}\u{1D62} \u{00D7} feature\u{1D62}) \u{2014} Linear combination "
                                "where each feature contributes proportionally to its weight (absorptivity). "
                                "Weights: Zipf=2.5, Entropy=2.0, Burstiness=1.8, Perplexity=2.2, TTR=1.5."
                            </p>
                        </div>
                        <div class="bg-gray-800 rounded-lg p-4 border-l-4 border-yellow-500">
                            <h3 class="font-semibold text-yellow-400">"Step 2: Hill Cooperative Amplification"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "Y = x\u{207F}\u{1D34} / (K\u{207F}\u{1D34} + x\u{207F}\u{1D34}) \u{2014} "
                                "Sigmoidal response with K=0.5, nH=2.5. Sharpens distinction near "
                                "the decision boundary. Gradual at extremes, steep near K."
                            </p>
                        </div>
                        <div class="bg-gray-800 rounded-lg p-4 border-l-4 border-yellow-500">
                            <h3 class="font-semibold text-yellow-400">"Step 3: Arrhenius Threshold Gate"</h3>
                            <p class="text-sm text-gray-400 mt-1">
                                "p = exp(-Ea / (score \u{00D7} scale)) \u{2014} "
                                "Maps Hill score to probability through activation energy barrier. "
                                "Ea=3.0, scale=10.0. Compared against Bloom-adapted threshold."
                            </p>
                        </div>
                    </div>
                </section>

                // Bloom Adaptation
                <section>
                    <h2 class="text-xl font-bold text-white mb-3">"Bloom Taxonomy Adaptation"</h2>
                    <p class="text-gray-300 leading-relaxed mb-3">
                        "Higher cognitive levels require more original thought, so detection "
                        "thresholds are lower (stricter). The system adapts expectations based "
                        "on what kind of thinking the text should demonstrate."
                    </p>
                    <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                        <table class="w-full text-sm">
                            <thead>
                                <tr class="text-gray-400 border-b border-gray-700">
                                    <th class="text-left py-1">"Level"</th>
                                    <th class="text-left py-1">"Name"</th>
                                    <th class="text-right py-1">"PV Ed"</th>
                                    <th class="text-right py-1">"Strict"</th>
                                    <th class="text-right py-1">"Lenient"</th>
                                </tr>
                            </thead>
                            <tbody class="text-gray-300 font-mono">
                                <tr><td>"L1"</td><td>"Remember"</td><td class="text-right">"0.66"</td><td class="text-right">"0.63"</td><td class="text-right">"0.68"</td></tr>
                                <tr><td>"L2"</td><td>"Understand"</td><td class="text-right">"0.65"</td><td class="text-right">"0.62"</td><td class="text-right">"0.67"</td></tr>
                                <tr><td>"L3"</td><td>"Apply"</td><td class="text-right">"0.64"</td><td class="text-right">"0.62"</td><td class="text-right">"0.67"</td></tr>
                                <tr><td>"L4"</td><td>"Analyze"</td><td class="text-right">"0.64"</td><td class="text-right">"0.62"</td><td class="text-right">"0.67"</td></tr>
                                <tr><td>"L5"</td><td>"Evaluate"</td><td class="text-right">"0.64"</td><td class="text-right">"0.61"</td><td class="text-right">"0.66"</td></tr>
                                <tr><td>"L6"</td><td>"Create"</td><td class="text-right">"0.64"</td><td class="text-right">"0.61"</td><td class="text-right">"0.66"</td></tr>
                                <tr><td>"L7"</td><td>"Meta-Create"</td><td class="text-right">"0.63"</td><td class="text-right">"0.60"</td><td class="text-right">"0.66"</td></tr>
                            </tbody>
                        </table>
                    </div>
                </section>

                // Accuracy
                <section>
                    <h2 class="text-xl font-bold text-white mb-3">"Accuracy"</h2>
                    <p class="text-gray-300 leading-relaxed">
                        "Experiment 1 validated 70% accuracy with recalibrated Bloom thresholds "
                        "on a 40-sample KSB fixture corpus (~80-120 words per sample). "
                        "Arrhenius probabilities cluster in [0.57, 0.68] for short text, "
                        "making discrimination challenging. The system is best suited as one "
                        "signal among many, not a sole arbiter."
                    </p>
                </section>
            </main>

            <footer class="border-t border-gray-700 mt-12 py-4 text-center text-xs text-gray-500">
                "Integrity Calculator \u{2022} NexVigilant"
            </footer>
        </div>
    }
}
