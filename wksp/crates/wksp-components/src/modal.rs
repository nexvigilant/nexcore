use leptos::prelude::*;

/// Bottom sheet modal for mobile (slides up from bottom)
/// Tier: T2-C (State + Boundary + Sequence)
///
/// Wrap with <Show> at the call site to control visibility.
/// Children are standard Leptos Children (FnOnce), safe because
/// each Show render creates a fresh BottomSheet instance.
#[component]
pub fn BottomSheet(
    on_close: Callback<()>,
    #[prop(optional)] title: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="modal-overlay" on:click=move |_| on_close.run(())>
            <div class="bottom-sheet" on:click=move |ev| ev.stop_propagation()>
                <div class="bottom-sheet-handle"></div>
                {if !title.is_empty() {
                    Some(view! { <h3 class="bottom-sheet-title">{title}</h3> })
                } else {
                    None
                }}
                <div class="bottom-sheet-content">
                    {children()}
                </div>
            </div>
        </div>
    }
}
