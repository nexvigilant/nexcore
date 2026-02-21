use leptos::prelude::*;

/// Popover with trigger and floating content
#[component]
pub fn Popover(
    trigger: Children,
    children: Children,
) -> impl IntoView {
    let is_open = RwSignal::new(false);

    view! {
        <div class="relative inline-block">
            <div on:click=move |_| is_open.set(!is_open.get())>
                {trigger()}
            </div>
            <div class=move || format!(
                "absolute left-1/2 z-50 mt-2 -translate-x-1/2 rounded-lg border border-slate-700 bg-slate-900 p-4 shadow-xl transition-all {}",
                if is_open.get() { "opacity-100 visible" } else { "opacity-0 invisible" }
            )>
                {children()}
            </div>
        </div>
    }
}
