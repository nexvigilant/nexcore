//! Data view pattern — loading/error/content states for server data
//!
//! Wraps the Leptos Resource + Suspense + ErrorBoundary pattern
//! into a reusable component for pages that fetch data.

use leptos::prelude::*;

/// Render data with automatic loading and error states.
///
/// Usage:
/// ```rust,ignore
/// <DataView data=my_resource let:items>
///     {/* render items here */}
/// </DataView>
/// ```
#[component]
pub fn DataView<T, V>(
    /// The async data to display
    data: Resource<Result<T, String>>,
    /// Render function for the loaded data
    children: fn(&T) -> V,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
    V: IntoView + 'static,
{
    view! {
        <Suspense fallback=move || view! { <LoadingState/> }>
            {move || {
                data.get().map(|result| match result {
                    Ok(ref value) => children(value).into_any(),
                    Err(ref e) => view! { <ErrorState message=e.clone()/> }.into_any(),
                })
            }}
        </Suspense>
    }
}

/// Loading spinner placeholder
#[component]
pub fn LoadingState() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center py-16">
            <div class="h-8 w-8 animate-spin rounded-full border-2 border-slate-700 border-t-cyan-500"/>
        </div>
    }
}

/// Error display with retry hint
#[component]
pub fn ErrorState(message: String) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-16 text-center">
            <div class="text-3xl text-slate-600">"!"</div>
            <p class="mt-4 text-sm text-slate-400">"Something went wrong"</p>
            <p class="mt-1 text-xs text-slate-600">{message}</p>
        </div>
    }
}

/// Empty state for when data loads but contains no items
#[component]
pub fn EmptyState(
    message: &'static str,
    #[prop(optional)] action_label: Option<&'static str>,
    #[prop(optional)] action_href: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-16 text-center">
            <p class="text-sm text-slate-500">{message}</p>
            {action_label.zip(action_href).map(|(label, href)| view! {
                <a href=href class="mt-3 text-sm text-cyan-400 hover:text-cyan-300">{label}</a>
            })}
        </div>
    }
}
