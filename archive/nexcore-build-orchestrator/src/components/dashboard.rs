//! Main dashboard layout — shows pipeline controls, recent status, and workspace health.

use leptos::prelude::*;

/// Dashboard component — main layout with pipeline actions and summary.
#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold">"Dashboard"</h1>
                <div class="flex space-x-2">
                    <a
                        href="/api/run/validate-quick"
                        class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded text-sm"
                    >
                        "Run validate-quick"
                    </a>
                    <a
                        href="/api/run/validate"
                        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded text-sm"
                    >
                        "Run validate"
                    </a>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                    <h3 class="text-sm text-gray-400 uppercase">"Last Status"</h3>
                    <p class="text-2xl font-bold text-green-400 mt-1">"Ready"</p>
                </div>
                <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                    <h3 class="text-sm text-gray-400 uppercase">"Build Lock"</h3>
                    <p class="text-2xl font-bold text-gray-300 mt-1">"Available"</p>
                </div>
                <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
                    <h3 class="text-sm text-gray-400 uppercase">"Pipelines"</h3>
                    <p class="text-2xl font-bold text-gray-300 mt-1">"2 built-in"</p>
                </div>
            </div>

            <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
                <h2 class="text-lg font-semibold mb-4">"Available Pipelines"</h2>
                <div class="space-y-3">
                    <div class="flex items-center justify-between p-3 bg-gray-900 rounded">
                        <div>
                            <span class="font-medium">"validate"</span>
                            <span class="text-gray-400 text-sm ml-2">
                                "fmt > (clippy | test | docs) > build > coverage"
                            </span>
                        </div>
                        <span class="text-xs text-gray-500">"6 stages"</span>
                    </div>
                    <div class="flex items-center justify-between p-3 bg-gray-900 rounded">
                        <div>
                            <span class="font-medium">"validate-quick"</span>
                            <span class="text-gray-400 text-sm ml-2">
                                "check > clippy > test-core"
                            </span>
                        </div>
                        <span class="text-xs text-gray-500">"3 stages"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
