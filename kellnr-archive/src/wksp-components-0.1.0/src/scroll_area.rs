use leptos::prelude::*;

/// Scrollable container with custom scrollbar styling
#[component]
pub fn ScrollArea(
    #[prop(optional)] class: &'static str,
    #[prop(optional, default = "400px")] max_height: &'static str,
    children: Children,
) -> impl IntoView {
    let style = format!("max-height: {max_height}");
    let classes = format!(
        "overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-slate-900 {class}"
    );
    view! {
        <div class=classes style=style>
            {children()}
        </div>
    }
}
