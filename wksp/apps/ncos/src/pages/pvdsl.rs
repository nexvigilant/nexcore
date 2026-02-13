use crate::components::card::Card;
use leptos::prelude::*;

/// Mobile PVDSL console — pharmacovigilance DSL execution
/// Tier: T3 (domain — PVDSL)
#[component]
pub fn PvdslPage() -> impl IntoView {
    let code = RwSignal::new(String::new());
    let output = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    let examples: Vec<(&'static str, &'static str)> = vec![
        ("PRR Check", "prr(a=15, b=100, c=20, d=10000)"),
        (
            "Signal Pipeline",
            "signal_complete(a=15, b=100, c=20, d=10000)",
        ),
        ("Naranjo Quick", "naranjo(1, 2, 1, 2, -1, 0, 0, 1, 1, 1)"),
    ];

    view! {
        <div class="page pvdsl">
            <h1 class="page-title">"PVDSL Console"</h1>

            <Card title="Code Editor">
                <textarea
                    class="code-editor"
                    rows="6"
                    placeholder="Enter PVDSL expression..."
                    prop:value=move || code.get()
                    on:input=move |ev| code.set(event_target_value(&ev))
                ></textarea>

                <button class="btn-primary"
                    disabled=move || loading.get() || code.get().is_empty()
                    on:click=move |_| {
                        loading.set(true);
                    }
                >
                    {move || if loading.get() { "Executing..." } else { "Execute" }}
                </button>
            </Card>

            <Card title="Examples">
                <div class="example-list">
                    {examples.into_iter().map(|(name, expr)| {
                        let expr_owned = expr.to_string();
                        view! {
                            <button class="example-btn"
                                on:click=move |_| code.set(expr_owned.clone())
                            >
                                {name}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </Card>

            <Show when=move || !output.get().is_empty()>
                <Card title="Output">
                    <pre class="code-output">{move || output.get()}</pre>
                </Card>
            </Show>
        </div>
    }
}
