//! get_user server function

use leptos::prelude::*;

#[server]
pub async fn get_user() -> Result<String, ServerFnError> {
    Ok("Hello from server".to_string())
}
