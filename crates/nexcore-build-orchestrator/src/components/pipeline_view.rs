//! Pipeline stage progress visualization.

use crate::pipeline::state::PipelineRunState;
use leptos::prelude::*;

/// Renders the pipeline stages as a horizontal progress bar.
#[component]
pub fn PipelineView(
    /// The pipeline run state to display.
    state: PipelineRunState,
) -> impl IntoView {
    let stages = state.stages.clone();
    let total = stages.len();
    let completed = state.completed_count();
    let success = state.success_count();

    view! {
        <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <div class="flex items-center justify-between mb-4">
                <h2 class="text-lg font-semibold">
                    {format!("Pipeline: {}", state.definition_name)}
                </h2>
                <span class={format!("font-bold status-{}", format!("{:?}", state.status).to_lowercase())}>
                    {format!("{:?}", state.status)}
                </span>
            </div>

            <div class="mb-2 text-sm text-gray-400">
                {format!("{}/{} stages complete ({} passed)", completed, total, success)}
            </div>

            // Progress bar
            <div class="w-full bg-gray-700 rounded-full h-2 mb-4">
                <div
                    class="bg-green-500 h-2 rounded-full transition-all"
                    style={format!("width: {}%", if total > 0 { completed * 100 / total } else { 0 })}
                />
            </div>

            // Stage cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {stages.into_iter().map(|s| {
                    view! {
                        <super::stage_card::StageCard stage=s />
                    }
                }).collect::<Vec<_>>()}
            </div>

            // Duration
            {state.total_duration.map(|d| view! {
                <div class="mt-4 text-sm text-gray-400">
                    {format!("Total: {}", d.display())}
                </div>
            })}
        </div>
    }
}
