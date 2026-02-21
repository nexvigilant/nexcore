//! Free trial signup page

use leptos::prelude::*;

#[component]
pub fn TrialPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-2xl px-4 py-16 text-center">
            <h1 class="text-4xl font-bold text-white">"Start Your Free Trial"</h1>
            <p class="mt-3 text-lg text-slate-400">
                "14 days of full access to Nucleus Academy, Career Tools, and Community."
            </p>

            <div class="mt-8 rounded-xl border border-cyan-500/30 bg-slate-900/50 p-8">
                <h2 class="text-xl font-semibold text-white">"Professional Trial"</h2>
                <ul class="mt-4 space-y-2 text-left text-sm text-slate-300">
                    <li class="flex items-center gap-2">
                        <span class="text-emerald-400">"✓"</span> "Full Academy access — all 15 PV domains"
                    </li>
                    <li class="flex items-center gap-2">
                        <span class="text-emerald-400">"✓"</span> "14 career assessment tools"
                    </li>
                    <li class="flex items-center gap-2">
                        <span class="text-emerald-400">"✓"</span> "Community circles and messaging"
                    </li>
                    <li class="flex items-center gap-2">
                        <span class="text-emerald-400">"✓"</span> "Signal detection playground"
                    </li>
                    <li class="flex items-center gap-2">
                        <span class="text-emerald-400">"✓"</span> "No credit card required"
                    </li>
                </ul>

                <a href="/signup" class="mt-6 inline-block rounded-lg bg-cyan-500 px-8 py-3 font-medium text-white hover:bg-cyan-400 transition-colors">
                    "Start Free Trial"
                </a>
            </div>
        </div>
    }
}
