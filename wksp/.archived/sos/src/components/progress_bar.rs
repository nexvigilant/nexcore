/// Progress bar — N Quantity (percentage) + μ Mapping (value -> width)
use leptos::prelude::*;

#[component]
pub fn ProgressBar(
    /// Current value (0-max)
    value: u32,
    /// Maximum value
    max: u32,
) -> impl IntoView {
    let pct = if max > 0 { (value * 100) / max } else { 0 };
    let width = format!("{}%", pct);

    let bar_class = if pct >= 80 {
        "progress-fill progress-high"
    } else if pct >= 50 {
        "progress-fill progress-mid"
    } else {
        "progress-fill progress-low"
    };

    view! {
        <div class="progress-bar">
            <div class=bar_class style=format!("width: {width}")></div>
            <span class="progress-label">{format!("{pct}%")}</span>
        </div>
    }
}
