//! Ventures / partnership page

use leptos::prelude::*;
use crate::api_client::PartnershipRequest;

/// Server function to submit a partnership inquiry
#[server(SubmitInquiryAction, "/api")]
pub async fn submit_inquiry_action(
    name: String,
    email: String,
    organization: String,
    interest: String,
    message: String,
) -> Result<(), ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = PartnershipRequest {
        name,
        email,
        organization,
        interest,
        message,
    };

    client.ventures_submit_inquiry(&req).await
        .map(|_| ())
        .map_err(ServerFnError::new)
}

#[component]
pub fn VenturesPage() -> impl IntoView {
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let org = RwSignal::new(String::new());
    let interest = RwSignal::new(String::from("Technology"));
    let message = RwSignal::new(String::new());
    
    let submit_action = ServerAction::<SubmitInquiryAction>::new();
    let result = submit_action.value();

    view! {
        <div class="mx-auto max-w-6xl px-4 py-16">
            <div class="text-center mb-16">
                <h1 class="text-4xl font-bold text-white font-mono uppercase tracking-tight">"VENTURES"</h1>
                <p class="mt-4 max-w-2xl mx-auto text-lg text-slate-400">
                    "Strategic partnerships and collaborative opportunities in pharmacovigilance innovation."
                </p>
            </div>

            <div class="grid gap-12 lg:grid-cols-3">
                <div class="lg:col-span-2 grid gap-6 md:grid-cols-2">
                    <ValueProp 
                        title="Technology Partners" 
                        desc="Integrate NexVigilant signal detection and intelligence into your safety platform." 
                    />
                    <ValueProp 
                        title="Academic Institutions" 
                        desc="Partner on PV curriculum development, research collaborations, and student training." 
                    />
                    <ValueProp 
                        title="Industry Advisory" 
                        desc="Independent PV consulting services for pharmaceutical companies and CROs." 
                    />
                    <ValueProp 
                        title="Content Creators" 
                        desc="Contribute educational content, case studies, and expert commentary." 
                    />
                </div>

                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                    <h3 class="text-xl font-bold text-white font-mono uppercase mb-6 tracking-tight">"Inquire"</h3>
                    
                    <div class="space-y-4">
                        <InquiryInput label="Full Name" signal=name placeholder="Your name" />
                        <InquiryInput label="Email Address" signal=email placeholder="you@example.com" />
                        <InquiryInput label="Organization" signal=org placeholder="Company/Institution" />
                        
                        <div>
                            <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-1.5 ml-1">"Interest"</label>
                            <select 
                                on:change=move |ev| interest.set(event_target_value(&ev))
                                class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-cyan-500 focus:outline-none transition-all font-mono"
                            >
                                <option value="Technology">"Technology"</option>
                                <option value="Academic">"Academic"</option>
                                <option value="Industry">"Industry"</option>
                                <option value="Content">"Content"</option>
                            </select>
                        </div>

                        <div>
                            <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-1.5 ml-1">"Message"</label>
                            <textarea 
                                rows="4"
                                prop:value=move || message.get()
                                on:input=move |ev| message.set(event_target_value(&ev))
                                class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-cyan-500 focus:outline-none transition-all font-mono"
                                placeholder="How can we collaborate?"
                            ></textarea>
                        </div>

                        <button 
                            on:click=move |_| {
                                submit_action.dispatch(SubmitInquiryAction {
                                    name: name.get(),
                                    email: email.get(),
                                    organization: org.get(),
                                    interest: interest.get(),
                                    message: message.get(),
                                });
                            }
                            disabled=submit_action.pending()
                            class="w-full rounded-lg bg-cyan-600 py-3 text-sm font-bold text-white hover:bg-cyan-500 transition-all shadow-lg shadow-cyan-900/20 disabled:opacity-50 mt-2"
                        >
                            {move || if submit_action.pending().get() { "SUBMITTING..." } else { "SUBMIT INQUIRY" }}
                        </button>
                    </div>

                    {move || result.get().map(|res| match res {
                        Ok(_) => view! { <p class="mt-4 text-green-400 text-xs font-bold text-center">"THANK YOU. RECEIVED."</p> }.into_any(),
                        Err(e) => view! { <p class="mt-4 text-red-400 text-xs font-mono">{e.to_string()}</p> }.into_any(),
                    })}
                </div>
            </div>
        </div>
    }
}

#[component]
fn ValueProp(title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/30 p-8 hover:bg-slate-900/50 transition-colors group">
            <h3 class="text-xl font-bold text-cyan-400 font-mono tracking-tight group-hover:text-cyan-300 transition-colors">{title}</h3>
            <p class="mt-3 text-slate-400 leading-relaxed text-sm">{desc}</p>
        </div>
    }
}

#[component]
fn InquiryInput(label: &'static str, signal: RwSignal<String>, placeholder: &'static str) -> impl IntoView {
    view! {
        <div>
            <label class="block text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-1.5 ml-1">{label}</label>
            <input type="text"
                prop:value=move || signal.get()
                on:input=move |ev| signal.set(event_target_value(&ev))
                class="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white focus:border-cyan-500 focus:outline-none transition-all font-mono"
                placeholder=placeholder
            />
        </div>
    }
}