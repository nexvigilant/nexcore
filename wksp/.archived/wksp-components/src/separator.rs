use leptos::prelude::*;

/// Horizontal separator line
#[component]
pub fn Separator(
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let classes = format!("border-t border-slate-800 {class}");
    view! { <hr class=classes/> }
}
