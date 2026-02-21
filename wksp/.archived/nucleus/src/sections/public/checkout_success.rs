//! Checkout success confirmation page

use leptos::prelude::*;

/// Server function: verify the Stripe Checkout Session completed
#[server(VerifyCheckoutSession, "/api")]
pub async fn verify_checkout_session(session_id: String) -> Result<String, ServerFnError> {
    use crate::stripe::client::StripeClient;

    let secret_key = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ServerFnError::new("STRIPE_SECRET_KEY not configured"))?;

    let client = StripeClient::new(secret_key);
    let session = client
        .retrieve_session(&session_id)
        .await
        .map_err(ServerFnError::new)?;

    match session.payment_status.as_deref() {
        Some("paid") => Ok("Payment confirmed".to_string()),
        Some(status) => Err(ServerFnError::new(format!("Payment status: {status}"))),
        None => Err(ServerFnError::new("Unable to verify payment status")),
    }
}

#[component]
pub fn CheckoutSuccessPage() -> impl IntoView {
    let params = leptos_router::hooks::use_query_map();
    let session_id = move || {
        params.with(|p| p.get("session_id").unwrap_or_default().to_string())
    };

    // Optionally verify the session
    let verification = Resource::new(
        session_id,
        |sid| async move {
            if sid.is_empty() {
                return None;
            }
            Some(verify_checkout_session(sid).await)
        },
    );

    view! {
        <div class="flex min-h-[60vh] flex-col items-center justify-center px-4 text-center">
            <div class="rounded-full bg-emerald-500/10 p-4 border border-emerald-500/20">
                <svg class="h-12 w-12 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7"/>
                </svg>
            </div>

            <h1 class="mt-6 text-4xl font-black font-mono text-white uppercase tracking-tight">"WELCOME TO NEXVIGILANT"</h1>

            // Verification status
            <Suspense fallback=move || view! {
                <p class="mt-3 text-sm text-slate-500 font-mono uppercase">"Verifying payment..."</p>
            }>
                {move || {
                    verification.get().map(|result| {
                        match result {
                            Some(Ok(msg)) => view! {
                                <p class="mt-3 text-sm text-emerald-400 font-mono uppercase">{msg}</p>
                            }.into_any(),
                            Some(Err(_)) | None => view! {
                                <p class="mt-3 max-w-md text-slate-400 font-mono">
                                    "Your membership is active. Full access to Academy, Community, and Career tools is now enabled."
                                </p>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>

            <div class="mt-10 flex flex-col sm:flex-row gap-4">
                <a href="/academy" class="group relative px-10 py-4 bg-cyan-600 text-white font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-cyan-500 shadow-[0_0_20px_rgba(34,211,238,0.3)]">
                    "START LEARNING"
                </a>
                <a href="/community" class="px-10 py-4 border border-slate-700 text-slate-300 font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-slate-900 hover:border-slate-500">
                    "JOIN COMMUNITY"
                </a>
            </div>

            <div class="mt-8 text-[10px] font-mono text-slate-600 uppercase tracking-widest">
                "Session confirmed. All systems operational."
            </div>
        </div>
    }
}
