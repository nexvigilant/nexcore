use leptos::prelude::*;

/// Full-page loading spinner
#[component]
pub fn LoadingScreen(
    #[prop(optional, default = "Loading...")] message: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex min-h-[50vh] flex-col items-center justify-center gap-4">
            <div class="h-10 w-10 animate-spin rounded-full border-4 border-slate-700 border-t-cyan-500"></div>
            <p class="text-sm text-slate-400">{message}</p>
        </div>
    }
}

/// Inline loading spinner
#[component]
pub fn Spinner(
    #[prop(optional, default = "md")] size: &'static str,
) -> impl IntoView {
    let dims = match size {
        "sm" => "h-4 w-4 border-2",
        "lg" => "h-8 w-8 border-4",
        _ => "h-6 w-6 border-2",
    };
    let classes = format!("animate-spin rounded-full border-slate-700 border-t-cyan-500 {dims}");
    view! {
        <div class=classes></div>
    }
}
