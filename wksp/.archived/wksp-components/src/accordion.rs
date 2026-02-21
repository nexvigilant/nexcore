use leptos::prelude::*;

/// Accordion item with expandable content
#[component]
pub fn AccordionItem(
    title: &'static str,
    #[prop(optional)] default_open: bool,
    children: Children,
) -> impl IntoView {
    let is_open = RwSignal::new(default_open);

    view! {
        <div class="border-b border-slate-800">
            <button
                class="flex w-full items-center justify-between py-4 text-left text-sm font-medium text-slate-200 hover:text-white transition-colors"
                on:click=move |_| is_open.set(!is_open.get())
            >
                <span>{title}</span>
                <span class=move || format!(
                    "text-slate-400 transition-transform {}",
                    if is_open.get() { "rotate-180" } else { "" }
                )>"v"</span>
            </button>
            <div class=move || format!(
                "overflow-hidden transition-all {}",
                if is_open.get() { "max-h-96 pb-4" } else { "max-h-0" }
            )>
                <div class="text-sm text-slate-400">
                    {children()}
                </div>
            </div>
        </div>
    }
}
