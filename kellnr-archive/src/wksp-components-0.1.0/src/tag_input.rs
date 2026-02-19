use leptos::prelude::*;

/// Tag/chip display component
#[component]
pub fn Tag(
    label: String,
    #[prop(optional)] on_remove: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <span class="inline-flex items-center gap-1 rounded-full bg-slate-800 px-3 py-1 text-xs font-medium text-slate-300">
            {label}
            {on_remove.map(|remove| view! {
                <button
                    class="ml-1 text-slate-500 hover:text-white transition-colors"
                    on:click=move |_| remove.run(())
                >
                    "x"
                </button>
            })}
        </span>
    }
}

/// Tag list container
#[component]
pub fn TagList(
    children: Children,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-2">
            {children()}
        </div>
    }
}
