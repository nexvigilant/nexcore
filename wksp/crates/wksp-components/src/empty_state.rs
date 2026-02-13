use leptos::prelude::*;

/// Empty state placeholder for when there's no content
#[component]
pub fn EmptyState(
    title: &'static str,
    #[prop(optional)] description: &'static str,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-12 text-center">
            <div class="mb-4 rounded-full bg-slate-800 p-4">
                <span class="text-2xl text-slate-500">"--"</span>
            </div>
            <h3 class="mb-1 text-lg font-medium text-slate-200">{title}</h3>
            {if !description.is_empty() {
                Some(view! { <p class="mb-4 max-w-sm text-sm text-slate-400">{description}</p> })
            } else {
                None
            }}
            {children.map(|c| c())}
        </div>
    }
}
