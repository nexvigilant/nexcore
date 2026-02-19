//! Google Identity Services (GIS) JS interop
//!
//! Bridges `window.google.accounts.id` via `js_sys::Reflect` — no
//! `#[wasm_bindgen]` extern blocks, just dynamic property access.
//! Only compiled under the `hydrate` feature (WASM client).

use js_sys::{Function, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// Helper: get a nested JS property, returning Err if missing or undefined.
fn get_defined(target: &JsValue, key: &str) -> Result<JsValue, String> {
    let val =
        Reflect::get(target, &JsValue::from_str(key)).map_err(|_| format!("{key} not found"))?;
    if val.is_undefined() || val.is_null() {
        return Err(format!("{key} is undefined"));
    }
    Ok(val)
}

/// Initialize Google Sign-In and render the branded button.
///
/// # Arguments
/// * `client_id` — OAuth 2.0 Web Client ID
/// * `callback` — Receives the credential JWT string on success
/// * `button_element_id` — DOM id of the container div for the button
pub fn initialize_google_sign_in(
    client_id: &str,
    callback: impl Fn(String) + 'static,
    button_element_id: &str,
) -> Result<(), String> {
    web_sys::console::log_1(&"[GIS-interop] initialize_google_sign_in called".into());
    let window = web_sys::window().ok_or("No window object")?;
    let google = get_defined(&window, "google")?;
    web_sys::console::log_1(&"[GIS-interop] window.google found".into());
    let accounts = get_defined(&google, "accounts")?;
    web_sys::console::log_1(&"[GIS-interop] google.accounts found".into());
    let id = get_defined(&accounts, "id")?;
    web_sys::console::log_1(&"[GIS-interop] google.accounts.id found".into());

    /* ---- google.accounts.id.initialize({ client_id, callback }) ---- */
    let init_fn: Function = get_defined(&id, "initialize")?
        .dyn_into()
        .map_err(|_| "initialize is not a function".to_string())?;

    let config = Object::new();
    Reflect::set(&config, &"client_id".into(), &JsValue::from_str(client_id))
        .map_err(|_| "set client_id".to_string())?;

    let closure = Closure::wrap(Box::new(move |response: JsValue| {
        if let Ok(credential) = Reflect::get(&response, &"credential".into()) {
            if let Some(token) = credential.as_string() {
                callback(token);
            }
        }
    }) as Box<dyn Fn(JsValue)>);

    Reflect::set(&config, &"callback".into(), closure.as_ref().unchecked_ref())
        .map_err(|_| "set callback".to_string())?;

    web_sys::console::log_1(&"[GIS-interop] calling google.accounts.id.initialize()...".into());
    init_fn
        .call1(&id, &config)
        .map_err(|e| format!("google.accounts.id.initialize() failed: {e:?}"))?;
    web_sys::console::log_1(&"[GIS-interop] initialize() succeeded".into());

    /* ---- google.accounts.id.renderButton(element, { theme, size }) ---- */
    let render_fn: Function = get_defined(&id, "renderButton")?
        .dyn_into()
        .map_err(|_| "renderButton is not a function".to_string())?;

    let document = window.document().ok_or("No document")?;
    let element = document
        .get_element_by_id(button_element_id)
        .ok_or_else(|| format!("Element #{button_element_id} not found"))?;

    let btn_opts = Object::new();
    Reflect::set(&btn_opts, &"theme".into(), &"filled_black".into())
        .map_err(|_| "set theme".to_string())?;
    Reflect::set(&btn_opts, &"size".into(), &"large".into())
        .map_err(|_| "set size".to_string())?;
    Reflect::set(&btn_opts, &"width".into(), &JsValue::from_f64(400.0))
        .map_err(|_| "set width".to_string())?;

    web_sys::console::log_1(
        &format!("[GIS-interop] calling renderButton on #{button_element_id}...").into(),
    );
    render_fn
        .call2(&id, &element, &btn_opts)
        .map_err(|e| format!("google.accounts.id.renderButton() failed: {e:?}"))?;
    web_sys::console::log_1(&"[GIS-interop] renderButton() succeeded!".into());

    /* Keep the closure alive for the lifetime of the page */
    closure.forget();

    Ok(())
}
