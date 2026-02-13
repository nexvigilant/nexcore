use leptos::prelude::*;

/// Dialog/sheet overlay (extends modal with more variants)
#[component]
pub fn Dialog(
    open: RwSignal<bool>,
    #[prop(optional)] title: &'static str,
    #[prop(optional, default = "md")] size: &'static str,
    children: Children,
) -> impl IntoView {
    let max_w = match size {
        "sm" => "max-w-sm",
        "lg" => "max-w-2xl",
        "xl" => "max-w-4xl",
        "full" => "max-w-full mx-4",
        _ => "max-w-lg",
    };

    view! {
        <div class=move || format!(
            "fixed inset-0 z-50 flex items-center justify-center transition-all {}",
            if open.get() { "visible opacity-100" } else { "invisible opacity-0" }
        )>
            <div
                class="absolute inset-0 bg-black/60 backdrop-blur-sm"
                on:click=move |_| open.set(false)
            />
            <div class=format!(
                "relative w-full {max_w} rounded-xl border border-slate-700 bg-slate-900 p-6 shadow-2xl"
            )>
                {if !title.is_empty() {
                    Some(view! {
                        <div class="mb-4 flex items-center justify-between">
                            <h2 class="text-lg font-semibold text-white">{title}</h2>
                            <button
                                class="rounded-lg p-1 text-slate-400 hover:bg-slate-800 hover:text-white transition-colors"
                                on:click=move |_| open.set(false)
                            >
                                "X"
                            </button>
                        </div>
                    })
                } else {
                    None
                }}
                {children()}
            </div>
        </div>
    }
}

/// Dialog footer with action buttons
#[component]
pub fn DialogFooter(
    children: Children,
) -> impl IntoView {
    view! {
        <div class="mt-6 flex justify-end gap-3">
            {children()}
        </div>
    }
}
