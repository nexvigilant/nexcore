//! Crate health panel — shows workspace scan results.

use crate::workspace::target::WorkspaceScan;
use leptos::prelude::*;

/// Renders workspace health overview.
#[component]
pub fn WorkspaceHealth(
    /// Workspace scan data.
    scan: WorkspaceScan,
) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
            <h2 class="text-lg font-semibold mb-4">"Workspace Health"</h2>

            <div class="grid grid-cols-3 gap-4 mb-4">
                <div class="text-center">
                    <div class="text-2xl font-bold">{scan.crate_count}</div>
                    <div class="text-xs text-gray-400">"Total Crates"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold text-green-400">{scan.clean_count()}</div>
                    <div class="text-xs text-gray-400">"Clean"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold text-yellow-400">{scan.dirty_count()}</div>
                    <div class="text-xs text-gray-400">"Dirty"</div>
                </div>
            </div>

            <div class="text-xs text-gray-500">
                {format!("Scanned: {}", scan.scanned_at.format("%Y-%m-%d %H:%M:%S UTC"))}
            </div>
        </div>
    }
}
