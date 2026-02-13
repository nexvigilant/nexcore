//! AutoTicker - Tier: T2-C. Runs auto-clicks via setInterval.
#![allow(unused_variables, unused_mut)]

use crate::components::GameMetrics;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
fn setup_interval(
    cps: u64,
    set_score: WriteSignal<u64>,
    set_metrics: WriteSignal<GameMetrics>
) -> i32 {
    use wasm_bindgen::prelude::*;
    let cb = Closure::wrap(
        Box::new(move || {
            set_score.update(|s| {
                *s += cps;
            });
            set_metrics.update(|m| {
                m.auto_clicks += cps;
                m.seconds_played += 1;
            });
        }) as Box<dyn Fn()>
    );
    let window = web_sys::window().expect("window");
    let id = window
        .set_interval_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), 1000)
        .unwrap_or(0);
    cb.forget();
    id
}

#[component]
#[allow(unused_variables)]
pub fn AutoTicker(
    mut cps: ReadSignal<u64>,
    set_score: WriteSignal<u64>,
    set_metrics: WriteSignal<GameMetrics>
) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let c = cps.get();
        if c == 0 {
            return;
        }
        let id = setup_interval(c, set_score, set_metrics);
        on_cleanup(move || {
            web_sys::window().map(|w| w.clear_interval_with_handle(id));
        });
    });
    view! { <span></span> }
}
