/// SOS — The Vigilance Machine (Mobile)
///
/// Primitive Foundation: λ Location + μ Mapping + ∂ Boundary + ς State + ν Frequency + σ Sequence
///
/// CSR Leptos app wrapped in Capacitor for iOS/Android.
/// All data flows through nexcore-api — this is a pure presentation layer.

mod api;
mod app;
mod components;
mod pages;

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount::mount_to_body(app::App);
}
