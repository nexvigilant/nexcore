//! Code Generator — AI-assisted boilerplate and logic generation

use crate::api_client::BrainThinkRequest;
use leptos::prelude::*;

#[server(GenerateCode, "/api")]
pub async fn generate_code_action(prompt: String) -> Result<String, ServerFnError> {
    use crate::api_client::server::ApiClient;
    let api_url =
        std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = BrainThinkRequest {
        prompt: format!(
            "Act as an expert Rust engineer. Generate clean, documented code based on this request: {prompt}"
        ),
        session_id: None,
    };

    client
        .brain_think(&req)
        .await
        .map(|res| res.content)
        .map_err(ServerFnError::new)
}

#[component]
pub fn CodeGenPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-5xl px-4 py-12">
            <header class="mb-12">
                <div class="flex items-center gap-4 mb-4">
                    <a href="/tools" class="text-slate-500 hover:text-white transition-colors">{"\u{2190}"}</a>
                    <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Code Generator"</h1>
                </div>
                <p class="text-slate-400">"Automated generation of Rust code, DTOs, and system logic."</p>
            </header>

            <CodeGenStudio/>
        </div>
    }
}

#[component]
pub fn CodeGenStudio() -> impl IntoView {
    let prompt = RwSignal::new(String::new());
    let generate_action = ServerAction::<GenerateCode>::new();
    let result = generate_action.value();

    view! {
        <div class="grid gap-8 lg:grid-cols-2">
            /* ---- Input Section ---- */
            <section class="space-y-6">
                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                    <label class="block text-xs font-bold text-slate-500 uppercase tracking-widest mb-4 font-mono">
                        "Generation Prompt"
                    </label>
                    <textarea
                        prop:value=move || prompt.get()
                        on:input=move |ev| prompt.set(event_target_value(&ev))
                        class="w-full h-48 bg-slate-950 border border-slate-800 rounded-xl p-4 text-sm text-white focus:border-cyan-500 focus:outline-none transition-all font-mono leading-relaxed"
                        placeholder="e.g., Create a Leptos component for a user dashboard card with a progress bar..."
                    ></textarea>

                    <button
                        on:click=move |_| {
                            generate_action.dispatch(GenerateCode { prompt: prompt.get() });
                        }
                        disabled=generate_action.pending()
                        class="w-full mt-6 py-4 rounded-xl bg-cyan-600 text-white font-black font-mono uppercase tracking-widest hover:bg-cyan-500 transition-all disabled:opacity-50 shadow-lg shadow-cyan-900/20"
                    >
                        {move || if generate_action.pending().get() { "GENERATING..." } else { "EXECUTE GENERATION" }}
                    </button>
                </div>

                <div class="p-6 rounded-2xl border border-dashed border-slate-800">
                    <h4 class="text-[10px] font-bold text-slate-600 uppercase tracking-[0.2em] mb-3 font-mono">"Quick Templates"</h4>
                    <div class="flex flex-wrap gap-2">
                        <TemplateBtn label="Leptos Component" prompt=prompt />
                        <TemplateBtn label="API DTO" prompt=prompt />
                        <TemplateBtn label="Server Function" prompt=prompt />
                        <TemplateBtn label="PVDSL Logic" prompt=prompt />
                    </div>
                </div>
            </section>

            /* ---- Output Section ---- */
            <section class="rounded-2xl border border-slate-800 bg-slate-950/50 overflow-hidden flex flex-col h-[600px]">
                <div class="bg-slate-900/80 px-4 py-2 border-b border-slate-800 flex justify-between items-center">
                    <span class="text-[10px] font-bold text-slate-500 uppercase tracking-widest font-mono">"Generated Artifact"</span>
                    <div class="flex gap-2">
                        <span class="h-2 w-2 rounded-full bg-slate-800"></span>
                        <span class="h-2 w-2 rounded-full bg-slate-800"></span>
                    </div>
                </div>
                <div class="flex-1 p-6 overflow-y-auto font-mono text-sm">
                    <Suspense fallback=|| view! { <p class="text-slate-700 animate-pulse">"AWAITING INPUT..."</p> }>
                        {move || result.get().map(|res| match res {
                            Ok(code) => view! {
                                <pre class="text-slate-300 whitespace-pre-wrap animate-in fade-in duration-700">{code}</pre>
                            }.into_any(),
                            Err(e) => view! {
                                <p class="text-red-500">"ERROR: " {e.to_string()}</p>
                            }.into_any()
                        })}
                    </Suspense>
                </div>
            </section>
        </div>
    }
}

#[component]
fn TemplateBtn(label: &'static str, prompt: RwSignal<String>) -> impl IntoView {
    let p = match label {
        "Leptos Component" => {
            "Generate a Leptos 0.7 component named 'StatCard' that displays a label and a value..."
        }
        "API DTO" => {
            "Generate a Rust struct with Serde attributes for an API response representing a safety signal..."
        }
        "Server Function" => {
            "Generate a Leptos server function that fetches a list of users from an API endpoint..."
        }
        "PVDSL Logic" => {
            "Generate a PVDSL script that calculates a disproportionality signal for a drug-event pair..."
        }
        _ => "",
    };

    view! {
        <button
            on:click=move |_| prompt.set(p.to_string())
            class="px-3 py-1 rounded-lg border border-slate-800 bg-slate-900/50 text-[10px] font-bold text-slate-500 hover:text-cyan-400 hover:border-cyan-500/30 transition-all uppercase tracking-widest font-mono"
        >
            {label}
        </button>
    }
}
