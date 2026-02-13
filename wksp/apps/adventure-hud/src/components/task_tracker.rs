//! TaskTracker - Displays current task progress

use leptos::prelude::*;
use crate::server::get_adventure_state::TaskInfo;

#[component]
pub fn TaskTracker(tasks: Vec<TaskInfo>) -> impl IntoView {
    let (completed, _pending): (Vec<_>, Vec<_>) = tasks
        .iter()
        .partition(|t| t.status == "completed");

    view! {
        <div class="task_tracker bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold text-blue-400 mb-4">"📋 Tasks"</h2>
            <ProgressBar completed=completed.len() total=tasks.len() />
            <TaskList tasks=tasks />
        </div>
    }
}

#[component]
fn ProgressBar(completed: usize, total: usize) -> impl IntoView {
    let pct = if total > 0 { (completed * 100) / total } else { 0 };
    let width = format!("{}%", pct);

    view! {
        <div class="mb-4">
            <div class="flex justify-between text-sm text-gray-400 mb-1">
                <span>{format!("{}/{} complete", completed, total)}</span>
                <span>{format!("{}%", pct)}</span>
            </div>
            <div class="w-full bg-gray-700 rounded-full h-2">
                <div class="bg-green-500 h-2 rounded-full" style:width=width></div>
            </div>
        </div>
    }
}

#[component]
fn TaskList(tasks: Vec<TaskInfo>) -> impl IntoView {
    view! {
        <div class="space-y-2 max-h-64 overflow-y-auto">
            {tasks.into_iter().map(render_task).collect::<Vec<_>>()}
        </div>
    }
}

fn render_task(task: TaskInfo) -> impl IntoView {
    let (icon, color) = status_style(&task.status);
    view! {
        <div class="flex items-center gap-2 p-2 bg-gray-700 rounded text-sm">
            <span class=format!("text-{}", color)>{icon}</span>
            <span class="flex-1 truncate">{task.subject}</span>
        </div>
    }
}

fn status_style(status: &str) -> (&'static str, &'static str) {
    match status {
        "completed" => ("✓", "green-400"),
        "in_progress" => ("⟳", "yellow-400"),
        _ => ("○", "gray-400"),
    }
}
