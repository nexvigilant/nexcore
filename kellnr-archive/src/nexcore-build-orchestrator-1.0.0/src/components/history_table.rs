//! Past runs history table.

use crate::pipeline::state::PipelineRunState;
use leptos::prelude::*;

/// Renders a table of past pipeline runs.
#[component]
pub fn HistoryTable(
    /// List of past runs (newest first).
    runs: Vec<PipelineRunState>,
) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
            <table class="w-full text-sm">
                <thead class="bg-gray-900">
                    <tr>
                        <th class="px-4 py-2 text-left text-gray-400">"Run ID"</th>
                        <th class="px-4 py-2 text-left text-gray-400">"Pipeline"</th>
                        <th class="px-4 py-2 text-left text-gray-400">"Status"</th>
                        <th class="px-4 py-2 text-left text-gray-400">"Duration"</th>
                        <th class="px-4 py-2 text-left text-gray-400">"Date"</th>
                    </tr>
                </thead>
                <tbody>
                    {if runs.is_empty() {
                        view! {
                            <tr>
                                <td colspan="5" class="px-4 py-8 text-center text-gray-500">
                                    "No pipeline runs recorded"
                                </td>
                            </tr>
                        }.into_any()
                    } else {
                        view! {
                            {runs.into_iter().map(|run| {
                                let status_class = format!("status-{}", format!("{:?}", run.status).to_lowercase());
                                let duration = run.total_duration.map(|d| d.display()).unwrap_or_else(|| "-".to_string());
                                let date = run.started_at.format("%Y-%m-%d %H:%M").to_string();
                                view! {
                                    <tr class="border-t border-gray-700 hover:bg-gray-750">
                                        <td class="px-4 py-2 font-mono text-xs">{run.id.0.clone()}</td>
                                        <td class="px-4 py-2">{run.definition_name.clone()}</td>
                                        <td class={format!("px-4 py-2 font-bold {status_class}")}>
                                            {format!("{:?}", run.status)}
                                        </td>
                                        <td class="px-4 py-2">{duration}</td>
                                        <td class="px-4 py-2 text-gray-400">{date}</td>
                                    </tr>
                                }
                            }).collect::<Vec<_>>()}
                        }.into_any()
                    }}
                </tbody>
            </table>
        </div>
    }
}
