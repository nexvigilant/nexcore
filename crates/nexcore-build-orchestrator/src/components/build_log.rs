//! Live log output component.

use crate::types::LogChunk;
use leptos::prelude::*;

/// Renders a scrollable log output panel.
#[component]
pub fn BuildLog(
    /// Log chunks to display.
    logs: Vec<LogChunk>,
) -> impl IntoView {
    view! {
        <div class="bg-gray-900 rounded-lg border border-gray-700 overflow-hidden">
            <div class="px-4 py-2 bg-gray-800 border-b border-gray-700">
                <span class="text-sm font-medium">"Build Output"</span>
                <span class="text-xs text-gray-500 ml-2">
                    {format!("{} chunks", logs.len())}
                </span>
            </div>
            <div class="p-4 max-h-96 overflow-y-auto log-output">
                {if logs.is_empty() {
                    view! { <p class="text-gray-500 italic">"No output yet"</p> }.into_any()
                } else {
                    view! {
                        <pre class="whitespace-pre-wrap text-gray-300">
                            {logs.into_iter().map(|chunk| {
                                let color = if chunk.is_stderr { "text-red-400" } else { "text-gray-300" };
                                view! {
                                    <span class={color}>{chunk.content}</span>
                                }
                            }).collect::<Vec<_>>()}
                        </pre>
                    }.into_any()
                }}
            </div>
        </div>
    }
}
