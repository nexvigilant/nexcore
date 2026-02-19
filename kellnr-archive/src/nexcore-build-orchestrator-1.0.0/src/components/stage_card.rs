//! Individual stage card component.

use crate::pipeline::state::StageRunState;
use leptos::prelude::*;

/// Renders a single stage as a card with status, duration, and exit code.
#[component]
pub fn StageCard(
    /// The stage run state to display.
    stage: StageRunState,
) -> impl IntoView {
    let status_class = format!("status-{}", format!("{:?}", stage.status).to_lowercase());
    let duration_text = stage
        .duration
        .map(|d| d.display())
        .unwrap_or_else(|| "-".to_string());
    let exit_text = stage
        .exit_code
        .map(|c| format!("exit {c}"))
        .unwrap_or_default();

    view! {
        <div class="bg-gray-900 rounded p-3 border border-gray-700">
            <div class="flex items-center justify-between">
                <span class="font-medium">{stage.stage_id.0.clone()}</span>
                <span class={format!("text-sm font-bold {status_class}")}>
                    {format!("{:?}", stage.status)}
                </span>
            </div>
            <div class="flex items-center justify-between mt-2 text-xs text-gray-400">
                <span>{duration_text}</span>
                <span>{exit_text}</span>
            </div>
        </div>
    }
}
