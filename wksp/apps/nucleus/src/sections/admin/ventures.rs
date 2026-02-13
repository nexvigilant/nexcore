//! Ventures administration page — manage partnership inquiries

use leptos::prelude::*;
use crate::api_client::{PartnershipInquiry, UpdateInquiryStatusRequest};

/// Server function to list inquiries
#[server(ListInquiries, "/api")]
pub async fn list_inquiries_action() -> Result<Vec<PartnershipInquiry>, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    client.ventures_list_inquiries().await
        .map_err(ServerFnError::new)
}

/// Server function to update inquiry status
#[server(UpdateInquiryStatus, "/api")]
pub async fn update_status_action(id: String, status: String) -> Result<(), ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url = std::env::var("NEXCORE_API_URL").unwrap_or_else(|_| "http://localhost:3030".to_string());
    let api_key = std::env::var("NEXCORE_API_KEY").ok();
    let client = ApiClient::new(api_url, api_key);

    let req = UpdateInquiryStatusRequest { status };
    client.ventures_update_status(&id, &req).await
        .map(|_| ())
        .map_err(ServerFnError::new)
}

#[component]
pub fn VenturesAdminPage() -> impl IntoView {
    let inquiries = Resource::new(|| (), |_| list_inquiries_action());

    view! {
        <div class="mx-auto max-w-6xl px-4 py-12">
            <header class="mb-12">
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Venture Governance"</h1>
                <p class="mt-2 text-slate-400">"Review and manage strategic partnership inquiries"</p>
            </header>

            <div class="rounded-2xl border border-slate-800 bg-slate-900/50 overflow-hidden glass-panel">
                <Suspense fallback=|| view! { <div class="p-12 text-center animate-pulse text-slate-500">"SYNCHRONIZING..."</div> }>
                    {move || inquiries.get().map(|result| match result {
                        Ok(list) => view! { <InquiryTable list=list refresh=move || inquiries.refetch() /> }.into_any(),
                        Err(e) => view! { <div class="p-8 text-red-400 font-mono text-sm">{e.to_string()}</div> }.into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn InquiryTable<F>(list: Vec<PartnershipInquiry>, refresh: F) -> impl IntoView 
where F: Fn() + Copy + 'static
{
    if list.is_empty() {
        view! { <div class="p-12 text-center text-slate-500 italic font-mono">"NO INQUIRIES RECORDED IN SYSTEM."</div> }.into_any()
    } else {
        view! {
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="bg-slate-800/50 text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                            <th class="px-6 py-4">"Date"</th>
                            <th class="px-6 py-4">"Partner"</th>
                            <th class="px-6 py-4">"Interest"</th>
                            <th class="px-6 py-4">"Status"</th>
                            <th class="px-6 py-4 text-right">"Action"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-800/50">
                        {list.into_iter().rev().map(|inquiry| {
                            view! { <InquiryRow inquiry=inquiry refresh=refresh /> }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        }.into_any()
    }
}

#[component]
fn InquiryRow<F>(inquiry: PartnershipInquiry, refresh: F) -> impl IntoView 
where F: Fn() + Copy + 'static
{
    let update_action = ServerAction::<UpdateInquiryStatus>::new();
    let status = inquiry.status.clone();
    let id = inquiry.id.clone();

    // Effect to refresh when status is updated successfully
    Effect::new(move |_| {
        if let Some(Ok(_)) = update_action.value().get() {
            refresh();
        }
    });

    let status_color = match status.as_str() {
        "received" => "bg-slate-800 text-slate-400 border-slate-700",
        "under review" => "bg-amber-500/10 text-amber-400 border-amber-500/20",
        "accepted" => "bg-green-500/10 text-green-400 border-green-500/20",
        "declined" => "bg-red-500/10 text-red-400 border-red-500/20",
        _ => "bg-slate-800 text-slate-400 border-slate-700",
    };

    view! {
        <tr class="hover:bg-slate-800/30 transition-colors group text-sm">
            <td class="px-6 py-4 text-slate-500 font-mono text-xs">{inquiry.created_at.format("%Y-%m-%d").to_string()}</td>
            <td class="px-6 py-4">
                <p class="font-bold text-slate-200 uppercase tracking-tight">{inquiry.name.clone()}</p>
                <p class="text-[10px] text-slate-500 font-mono">{inquiry.organization.clone()}</p>
            </td>
            <td class="px-6 py-4">
                <span class="text-xs font-bold text-cyan-500 font-mono tracking-wider">{inquiry.interest.to_uppercase()}</span>
            </td>
            <td class="px-6 py-4">
                <span class=format!("text-[10px] px-2 py-0.5 rounded border font-bold uppercase {}", status_color)>
                    {status}
                </span>
            </td>
            <td class="px-6 py-4 text-right">
                <div class="flex justify-end gap-2">
                    {move || if update_action.pending().get() {
                        view! { <span class="text-[10px] font-mono text-slate-600">"UPDATING..."</span> }.into_any()
                    } else {
                        let iid = id.clone();
                        view! {
                            <select 
                                on:change=move |ev| {
                                    let new_status = event_target_value(&ev);
                                    update_action.dispatch(UpdateInquiryStatus { id: iid.clone(), status: new_status });
                                }
                                class="bg-slate-950 border border-slate-800 text-[10px] font-mono text-slate-400 rounded px-2 py-1 focus:outline-none focus:border-cyan-500 transition-all"
                            >
                                <option value="" disabled selected>"CHANGE STATUS"</option>
                                <option value="received">"RECEIVED"</option>
                                <option value="under review">"REVIEW"</option>
                                <option value="accepted">"ACCEPT"</option>
                                <option value="declined">"DECLINE"</option>
                            </select>
                        }.into_any()
                    }}
                </div>
            </td>
        </tr>
    }
}