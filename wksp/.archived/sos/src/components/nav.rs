/// Bottom tab navigation — 4 primary tabs + "More" overflow
/// σ Sequence of tabs + ς State (overlay open/closed)
///
/// Leptos 0.8: <A> sets aria-current="page" on active links.
/// Style via CSS `a[aria-current]` selector.
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn BottomNav() -> impl IntoView {
    let (more_open, set_more_open) = signal(false);

    view! {
        <nav class="bottom-nav">
            <A href="/" attr:class="nav-tab">
                <span class="nav-icon">"H"</span>
                <span class="nav-label">"Home"</span>
            </A>
            <A href="/signals" attr:class="nav-tab">
                <span class="nav-icon">"S"</span>
                <span class="nav-label">"Signals"</span>
            </A>
            <A href="/guardian" attr:class="nav-tab">
                <span class="nav-icon">"G"</span>
                <span class="nav-label">"Guardian"</span>
            </A>
            <A href="/brain" attr:class="nav-tab">
                <span class="nav-icon">"B"</span>
                <span class="nav-label">"Brain"</span>
            </A>
            <button
                class="nav-tab nav-more-btn"
                class:nav-more-active=more_open
                on:click=move |_| set_more_open.update(|v| *v = !*v)
            >
                <span class="nav-icon">"+"</span>
                <span class="nav-label">"More"</span>
            </button>
        </nav>

        // More overlay
        {move || {
            if more_open.get() {
                Some(view! {
                    <div class="more-backdrop" on:click=move |_| set_more_open.set(false)></div>
                    <div class="more-menu">
                        <div class="more-header">"More"</div>
                        <A href="/pvos" attr:class="more-item" on:click=move |_| set_more_open.set(false)>
                            <span class="more-icon">"P"</span>
                            <span class="more-label">"PVOS"</span>
                            <span class="more-desc">"15-Layer Operating System"</span>
                        </A>
                        <A href="/skills" attr:class="more-item" on:click=move |_| set_more_open.set(false)>
                            <span class="more-icon">"K"</span>
                            <span class="more-label">"Skills"</span>
                            <span class="more-desc">"94 Capabilities"</span>
                        </A>
                        <A href="/academy" attr:class="more-item" on:click=move |_| set_more_open.set(false)>
                            <span class="more-icon">"A"</span>
                            <span class="more-label">"Academy"</span>
                            <span class="more-desc">"Professional Development"</span>
                        </A>
                        <A href="/settings" attr:class="more-item" on:click=move |_| set_more_open.set(false)>
                            <span class="more-icon">"*"</span>
                            <span class="more-label">"Settings"</span>
                            <span class="more-desc">"API URL & Configuration"</span>
                        </A>
                    </div>
                })
            } else {
                None
            }
        }}
    }
}
