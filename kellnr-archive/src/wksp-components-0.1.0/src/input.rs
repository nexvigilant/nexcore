use leptos::prelude::*;

/// Touch-friendly number input for mobile (44px min target)
/// Tier: T2-P (State + Boundary)
#[component]
pub fn NumberInput(
    #[prop(optional)] label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] step: &'static str,
    #[prop(optional)] decimal: bool,
) -> impl IntoView {
    let mode = if decimal { "decimal" } else { "numeric" };
    let step_val = if step.is_empty() { "1" } else { step };
    view! {
        <div class="input-group">
            {if !label.is_empty() {
                Some(view! { <label class="input-label">{label}</label> })
            } else {
                None
            }}
            <input
                type="number"
                inputmode=mode
                class="input-field"
                placeholder=placeholder
                step=step_val
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
            />
        </div>
    }
}

/// Touch-friendly text input
/// Tier: T2-P (State + Boundary)
#[component]
pub fn TextInput(
    #[prop(optional)] label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] input_type: &'static str,
) -> impl IntoView {
    let field_type = if input_type.is_empty() {
        "text"
    } else {
        input_type
    };
    view! {
        <div class="input-group">
            {if !label.is_empty() {
                Some(view! { <label class="input-label">{label}</label> })
            } else {
                None
            }}
            <input
                type=field_type
                class="input-field"
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
            />
        </div>
    }
}
