use leptos::prelude::*;

/// Tab bar container
#[component]
pub fn Tabs(
    /// Currently active tab index
    active: RwSignal<usize>,
    /// Tab labels
    labels: Vec<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div>
            <div class="flex border-b border-slate-800">
                {labels.into_iter().enumerate().map(|(i, label)| {
                    view! {
                        <button
                            class=move || format!(
                                "px-4 py-3 text-sm font-medium transition-colors border-b-2 -mb-px {}",
                                if active.get() == i {
                                    "border-cyan-500 text-cyan-400"
                                } else {
                                    "border-transparent text-slate-400 hover:text-white"
                                }
                            )
                            on:click=move |_| active.set(i)
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <div class="mt-4">
                {children()}
            </div>
        </div>
    }
}
