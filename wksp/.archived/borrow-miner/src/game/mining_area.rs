//! Mining area - Main game canvas with particles and floating scores

use leptos::prelude::*;
use crate::{AMBER_GOLD, DEEP_SLATE, SLATE_DIM};
use super::{GameState, OreType, Particle, FloatingScore};

#[component]
pub fn MiningArea() -> impl IntoView {
    let state = expect_context::<GameState>();

    view! {
        <main style=MAIN_STYLE>
            <ParticleLayer particles=state.particles />
            <FloatingScoreLayer scores=state.floating_scores />
            <MiningPrompt last_ore=state.last_ore />
            <AnimationController state=state />
        </main>
    }
}

const MAIN_STYLE: &str = "flex: 1; display: flex; flex-direction: column; \
    align-items: center; justify-content: center; \
    min-height: 60vh; position: relative; overflow: hidden;";

#[component]
fn ParticleLayer(particles: RwSignal<Vec<Particle>>) -> impl IntoView {
    view! {
        <div style="position: absolute; inset: 0; pointer-events: none;">
            {move || particles.get().into_iter().map(particle_div).collect_view()}
        </div>
    }
}

fn particle_div(p: Particle) -> impl IntoView {
    let style = particle_style(&p);
    view! { <div style=style></div> }
}

fn particle_style(p: &Particle) -> String {
    format!(
        "position: absolute; left: {}%; top: {}%; \
         width: {}px; height: {}px; background: {}; \
         border-radius: 50%; opacity: {};",
        p.x, p.y, p.size, p.size, p.color, p.life
    )
}

#[component]
fn FloatingScoreLayer(scores: RwSignal<Vec<FloatingScore>>) -> impl IntoView {
    view! {
        <div style="position: absolute; inset: 0; pointer-events: none;">
            {move || scores.get().into_iter().map(score_div).collect_view()}
        </div>
    }
}

fn score_div(s: FloatingScore) -> impl IntoView {
    let style = score_style(&s);
    let text = format!("+{}", s.value.0);
    view! { <div style=style>{text}</div> }
}

fn score_style(s: &FloatingScore) -> String {
    format!(
        "position: absolute; left: {}%; top: {}%; color: {}; \
         font-size: 1.5rem; font-weight: bold; opacity: {}; \
         transform: translateX(-50%);",
        s.x, s.y, AMBER_GOLD, s.life
    )
}

#[component]
fn MiningPrompt(last_ore: RwSignal<Option<OreType>>) -> impl IntoView {
    view! {
        <div style="text-align: center;">
            <PickaxeIcon />
            <HelpText />
            {move || last_ore.get().map(|ore| view! { <LastOreDisplay ore=ore /> })}
        </div>
    }
}

#[component]
fn PickaxeIcon() -> impl IntoView {
    let style = format!(
        "font-size: 4rem; margin-bottom: 1rem; filter: drop-shadow(0 0 20px {});",
        AMBER_GOLD
    );
    view! { <div style=style>"⛏️"</div> }
}

#[component]
fn HelpText() -> impl IntoView {
    let style = format!("color: {}; font-size: 1.25rem;", SLATE_DIM);
    view! { <div style=style>"Click anywhere to mine"</div> }
}

#[component]
fn LastOreDisplay(#[prop(into)] ore: OreType) -> impl IntoView {
    let outer = last_ore_outer_style();
    let inner = format!("color: {};", ore.color());
    let text = format!("{} {} (+{})", ore.symbol(), ore.name(), ore.base_value());
    view! {
        <div style=outer>
            <span style=inner>{text}</span>
        </div>
    }
}

fn last_ore_outer_style() -> String {
    format!(
        "margin-top: 1rem; padding: 0.5rem 1rem; \
         background: {}; border-radius: 8px; display: inline-block;",
        DEEP_SLATE
    )
}

/// Animation controller - sets up timers for particle updates
#[component]
fn AnimationController(state: GameState) -> impl IntoView {
    // Consume state to satisfy unused warning on non-wasm
    let _ = &state;
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::prelude::*;

        // Animation loop
        let s1 = state.clone();
        Effect::new(move |_| {
            let s = s1.clone();
            let f: Box<dyn Fn()> = Box::new(move || s.tick(0.016));
            let cb = Closure::wrap(f);
            let id = do_set_interval(&cb, 16);
            cb.forget();
            on_cleanup(move || do_clear_interval(id));
        });

        // Combo decay
        Effect::new(move |_| {
            let s = state.clone();
            let f: Box<dyn Fn()> = Box::new(move || s.decay_combo());
            let cb = Closure::wrap(f);
            let id = do_set_interval(&cb, 2000);
            cb.forget();
            on_cleanup(move || do_clear_interval(id));
        });
    }

    view! { <div style="display: none;"></div> }
}

#[cfg(target_arch = "wasm32")]
fn do_set_interval(cb: &leptos::wasm_bindgen::closure::Closure<dyn Fn()>, ms: i32) -> Option<i32> {
    use leptos::wasm_bindgen::JsCast;
    leptos::web_sys::window()?
        .set_interval_with_callback_and_timeout_and_arguments_0(cb.as_ref().unchecked_ref(), ms)
        .ok()
}

#[cfg(target_arch = "wasm32")]
fn do_clear_interval(id: Option<i32>) {
    if let (Some(w), Some(id)) = (leptos::web_sys::window(), id) {
        w.clear_interval_with_handle(id);
    }
}
