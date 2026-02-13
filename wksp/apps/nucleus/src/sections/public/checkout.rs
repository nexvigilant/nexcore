//! Checkout flow — creates Stripe Checkout Session and redirects

use leptos::prelude::*;

/// Plan metadata: name, price display, env var suffix
fn plan_info(plan: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match plan {
        "community" => Some(("Community", "$19/mo", "COMMUNITY")),
        "professional" => Some(("Professional", "$29/mo", "PROFESSIONAL")),
        "enterprise" => Some(("Enterprise", "$59/mo", "ENTERPRISE")),
        _ => None,
    }
}

/// Server function: create a Stripe Checkout Session and return the URL
#[cfg_attr(feature = "ssr", allow(unused_imports))]
#[server(CreateCheckoutSession, "/api")]
pub async fn create_checkout_session(plan: String) -> Result<String, ServerFnError> {
    use crate::stripe::client::StripeClient;

    let (_, _, env_suffix) = plan_info(&plan)
        .ok_or_else(|| ServerFnError::new(format!("Unknown plan: {plan}")))?;

    let secret_key = std::env::var("STRIPE_SECRET_KEY")
        .map_err(|_| ServerFnError::new("STRIPE_SECRET_KEY not configured"))?;

    let price_env = format!("STRIPE_PRICE_{env_suffix}");
    let price_id = std::env::var(&price_env)
        .map_err(|_| ServerFnError::new(format!("{price_env} not configured")))?;

    let client = StripeClient::new(secret_key);
    let session = client
        .create_checkout_session(
            &price_id,
            "https://nexvigilant.com/checkout/success?session_id={CHECKOUT_SESSION_ID}",
            "https://nexvigilant.com/membership",
        )
        .await
        .map_err(ServerFnError::new)?;

    session
        .url
        .ok_or_else(|| ServerFnError::new("Stripe did not return a checkout URL"))
}

#[component]
pub fn CheckoutPage() -> impl IntoView {
    let params = leptos_router::hooks::use_query_map();
    let plan = move || {
        params.with(|p| p.get("plan").unwrap_or_default().to_string())
    };

    let checkout_action = ServerAction::<CreateCheckoutSession>::new();
    let action_value = checkout_action.value();
    let pending = checkout_action.pending();

    // Redirect on success
    Effect::new(move |_| {
        if let Some(Ok(url)) = action_value.get() {
            #[cfg(feature = "hydrate")]
            {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href(&url);
                }
            }
            let _ = url; // suppress unused warning in SSR
        }
    });

    let plan_display = move || {
        let p = plan();
        plan_info(&p)
            .map(|(name, price, _)| (name.to_string(), price.to_string()))
            .unwrap_or_else(|| ("Unknown".to_string(), "--".to_string()))
    };

    view! {
        <div class="mx-auto max-w-2xl px-4 py-16">
            <div class="mb-4">
                <a href="/membership" class="text-xs font-mono text-cyan-500 uppercase tracking-widest hover:text-cyan-400 transition-colors">
                    "< BACK TO PLANS"
                </a>
            </div>

            <h1 class="text-3xl font-black font-mono text-white uppercase tracking-tight">"CHECKOUT"</h1>

            // Order summary
            <div class="mt-8 rounded-xl border border-slate-800 bg-slate-900/50 p-8 backdrop-blur-sm">
                <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em]">"// ORDER SUMMARY"</h2>
                <div class="mt-4 space-y-3">
                    <div class="flex justify-between text-sm font-mono">
                        <span class="text-slate-400">{move || plan_display().0} " Membership"</span>
                        <span class="text-white font-bold">{move || plan_display().1}</span>
                    </div>
                    <div class="border-t border-slate-700 pt-3 flex justify-between font-medium font-mono">
                        <span class="text-white">"TOTAL"</span>
                        <span class="text-cyan-400 font-bold">{move || plan_display().1}</span>
                    </div>
                </div>
            </div>

            // Payment section
            <div class="mt-6 rounded-xl border border-slate-800 bg-slate-900/50 p-8 backdrop-blur-sm">
                <h2 class="text-xs font-mono font-bold text-slate-500 uppercase tracking-[0.3em]">"// PAYMENT"</h2>
                <p class="mt-3 text-sm text-slate-400 font-mono">
                    "Secure checkout powered by Stripe. You will be redirected to complete payment."
                </p>

                // Error display
                {move || {
                    action_value.get().and_then(|r| r.err()).map(|e| {
                        view! {
                            <div class="mt-4 rounded-lg border border-red-500/30 bg-red-500/10 p-3 text-sm text-red-400 font-mono">
                                {e.to_string()}
                            </div>
                        }
                    })
                }}

                // Success redirect notice (SSR fallback)
                {move || {
                    action_value.get().and_then(|r| r.ok()).map(|url| {
                        view! {
                            <div class="mt-4 rounded-lg border border-cyan-500/30 bg-cyan-500/10 p-3">
                                <p class="text-sm text-cyan-400 font-mono">"Redirecting to Stripe..."</p>
                                <a href={url} class="text-xs text-cyan-500 underline font-mono">"Click here if not redirected"</a>
                            </div>
                        }
                    })
                }}

                <ActionForm action=checkout_action attr:class="mt-6">
                    <input type="hidden" name="plan" value=plan/>
                    <button
                        type="submit"
                        disabled=pending
                        class="w-full group relative px-10 py-4 bg-cyan-600 text-white font-mono font-black text-sm uppercase tracking-widest rounded transition-all hover:bg-cyan-500 shadow-[0_0_20px_rgba(34,211,238,0.3)] disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        {move || if pending.get() { "PROCESSING..." } else { "SUBSCRIBE NOW" }}
                    </button>
                </ActionForm>

                <p class="mt-4 text-center text-[10px] text-slate-600 font-mono uppercase">
                    "Cancel anytime. Billed monthly."
                </p>
            </div>
        </div>
    }
}
