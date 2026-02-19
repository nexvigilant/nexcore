use leptos::prelude::*;

/// Loading skeleton placeholder
#[component]
pub fn Skeleton(
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let classes = format!("animate-pulse rounded-md bg-slate-800 {class}");
    view! { <div class=classes/> }
}

/// Text skeleton line
#[component]
pub fn SkeletonText(
    #[prop(optional)] lines: u32,
) -> impl IntoView {
    let count = if lines == 0 { 3 } else { lines };
    view! {
        <div class="space-y-2">
            {(0..count).map(|i| {
                let width = if i == count - 1 { "w-3/4" } else { "w-full" };
                let classes = format!("animate-pulse rounded bg-slate-800 h-4 {width}");
                view! { <div class=classes/> }
            }).collect::<Vec<_>>()}
        </div>
    }
}
