//! Certificate verification page

use crate::api_client::CertificateVerificationResult;
use leptos::prelude::*;

/// Verify certificate code against backend certificate records.
#[server(VerifyCertificateCode, "/api")]
pub async fn verify_certificate_code(
    code: String,
) -> Result<CertificateVerificationResult, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let normalized = code.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(ServerFnError::new("Please enter a verification code."));
    }

    let api_url =
        std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client
        .academy_verify_certificate_code(&normalized)
        .await
        .map_err(ServerFnError::new)
}

#[component]
pub fn VerifyPage() -> impl IntoView {
    let code = RwSignal::new(String::new());
    let validation_error = RwSignal::new(Option::<String>::None);
    let verify_action = Action::new(|code: &String| {
        let code = code.clone();
        async move { verify_certificate_code(code).await }
    });

    view! {
        <div class="mx-auto max-w-xl px-4 py-16">
            <h1 class="text-4xl font-bold text-white">"Verify Certificate"</h1>
            <p class="mt-3 text-slate-400">
                "Enter a certificate verification code to confirm its authenticity."
            </p>

            <div class="mt-8 space-y-4">
                <input
                    type="text"
                    placeholder="Enter verification code (e.g., NV-CERT-2026-XXXX)"
                    class="w-full rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-white placeholder-slate-500 focus:border-cyan-500 focus:outline-none"
                    on:input=move |ev| code.set(event_target_value(&ev))
                />
                <button
                    class="w-full rounded-lg bg-cyan-500 px-6 py-3 font-medium text-white hover:bg-cyan-400 transition-colors"
                    on:click=move |_| {
                        let entered = code.get().trim().to_string();
                        if entered.is_empty() {
                            validation_error.set(Some("Please enter a verification code.".to_string()));
                            return;
                        }
                        validation_error.set(None);
                        verify_action.dispatch(entered);
                    }
                    disabled=verify_action.pending()
                >
                    {move || if verify_action.pending().get() { "Verifying..." } else { "Verify" }}
                </button>
            </div>

            {move || validation_error.get().map(|msg| view! {
                <div class="mt-6 rounded-lg border border-amber-500/30 bg-amber-500/10 p-4 text-sm text-amber-300">
                    {msg}
                </div>
            })}

            {move || verify_action.value().get().map(|res| match res {
                Ok(v) if v.valid => view! {
                    <div class="mt-6 rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-4 text-sm text-emerald-200 space-y-1">
                        <p class="font-semibold">"Valid certificate"</p>
                        <p><span class="text-emerald-400">"Code: "</span>{v.code}</p>
                        <p><span class="text-emerald-400">"Title: "</span>{v.title.unwrap_or_else(|| "Unknown".to_string())}</p>
                        <p><span class="text-emerald-400">"Issued: "</span>{v.issued_at.unwrap_or_else(|| "Unknown".to_string())}</p>
                    </div>
                }.into_any(),
                Ok(v) => view! {
                    <div class="mt-6 rounded-lg border border-red-500/30 bg-red-500/10 p-4 text-sm text-red-200">
                        {v.message}
                    </div>
                }.into_any(),
                Err(e) => view! {
                    <div class="mt-6 rounded-lg border border-red-500/30 bg-red-500/10 p-4 text-sm text-red-200">
                        {"Verification service error: "}{e.to_string()}
                    </div>
                }.into_any(),
            })}

            <section class="mt-12 rounded-xl border border-slate-800 bg-slate-900/40 p-6">
                <h2 class="text-lg font-bold text-white">"Enterprise Trust + Validation Package"</h2>
                <p class="mt-2 text-sm text-slate-400">
                    "For pharmaceutical adoption reviews, we provide the package below to reduce procurement and QA friction."
                </p>

                <ul class="mt-4 space-y-2 text-sm text-slate-300">
                    <li class="flex gap-2"><span class="text-cyan-400">"•"</span><span>"Computerized system validation artifacts (URS, risk matrix, IQ/OQ/PQ templates)"</span></li>
                    <li class="flex gap-2"><span class="text-cyan-400">"•"</span><span>"Audit trail and traceability expectations mapped to regulated workflows"</span></li>
                    <li class="flex gap-2"><span class="text-cyan-400">"•"</span><span>"Security boundary and access-control overview for technical due diligence"</span></li>
                    <li class="flex gap-2"><span class="text-cyan-400">"•"</span><span>"Integration playbook for staged rollout with existing PV systems"</span></li>
                    <li class="flex gap-2"><span class="text-cyan-400">"•"</span><span>"Adoption scorecard for value realization and governance tracking"</span></li>
                </ul>

                <div class="mt-6 flex flex-col gap-3 sm:flex-row">
                    <a
                        href="/enterprise-readiness"
                        class="rounded-lg border border-white/20 bg-white/5 px-5 py-3 text-center text-xs font-bold uppercase tracking-widest text-white hover:bg-white/10 transition-colors"
                    >
                        "Open Enterprise Readiness"
                    </a>
                    <a
                        href="/services"
                        class="rounded-lg bg-cyan-600 px-5 py-3 text-center text-xs font-bold uppercase tracking-widest text-white hover:bg-cyan-500 transition-colors"
                    >
                        "View Adoption Risk Map"
                    </a>
                    <a
                        href="/contact"
                        class="rounded-lg border border-slate-700 px-5 py-3 text-center text-xs font-bold uppercase tracking-widest text-slate-300 hover:bg-slate-800 transition-colors"
                    >
                        "Request Validation Package"
                    </a>
                </div>
            </section>
        </div>
    }
}
