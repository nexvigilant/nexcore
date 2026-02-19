use leptos::prelude::*;

/// Tooltip wrapping a trigger element
#[component]
pub fn Tooltip(
    text: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="group relative inline-block">
            {children()}
            <div class="pointer-events-none absolute bottom-full left-1/2 -translate-x-1/2 mb-2 rounded-md bg-slate-700 px-3 py-1.5 text-xs text-white opacity-0 transition-opacity group-hover:opacity-100 whitespace-nowrap">
                {text}
                <div class="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-slate-700"/>
            </div>
        </div>
    }
}
