//! Member profile page — view professional details and connections

use crate::api_client::CommunityMember;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// Server function to fetch a specific member profile
#[server(GetMemberProfile, "/api")]
pub async fn get_member_profile(user_id: String) -> Result<CommunityMember, ServerFnError> {
    use crate::api_client::server::ApiClient;

    let api_url =
        crate::runtime_config::nexcore_api_url();
    let api_key = crate::runtime_config::nexcore_api_key();
    let client = ApiClient::new(api_url, api_key);

    client
        .user_get_profile(&user_id)
        .await
        .map(|p| CommunityMember {
            id: p.uid,
            name: p.display_name.unwrap_or_else(|| "Unknown".to_string()),
            role: p.role,
            organization: p.organization.unwrap_or_else(|| "Independent".to_string()),
            domains: vec![], // ProfileData doesn't have domains yet in this mock
            avatar_url: Some(p.photo_url),
            joined_at: chrono::Utc::now(),
            post_count: 0,
        })
        .map_err(ServerFnError::new)
}

#[component]
pub fn MemberProfilePage() -> impl IntoView {
    let params = use_params_map();
    let user_id = move || params.with(|p| p.get("userId").unwrap_or_default());

    let profile = Resource::new(move || user_id(), move |id| get_member_profile(id));

    view! {
        <div class="mx-auto max-w-4xl px-4 py-8">
            <Suspense fallback=|| view! { <div class="animate-pulse h-64 bg-slate-900/50 rounded-2xl border border-slate-800"></div> }>
                {move || profile.get().map(|result| match result {
                    Ok(member) => view! { <ProfileView member=member /> }.into_any(),
                    Err(e) => view! {
                        <div class="rounded-xl border border-red-500/20 bg-red-500/5 p-8 text-center">
                            <p class="text-red-400 font-mono">"Failed to load profile: "{e.to_string()}</p>
                            <a href="/community/members" class="mt-4 inline-block text-sm text-slate-500 hover:text-white underline">"Back to members"</a>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn ProfileView(member: CommunityMember) -> impl IntoView {
    let initial = member
        .name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    view! {
        <div class="space-y-6">
            /* Header Card */
            <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8">
                <div class="flex flex-col md:flex-row items-center md:items-start gap-6">
                    {if let Some(avatar) = member.avatar_url {
                        view! { <img src=avatar alt=member.name.clone() class="h-24 w-24 rounded-full border-2 border-slate-800 object-cover" /> }.into_any()
                    } else {
                        view! {
                            <div class="h-24 w-24 rounded-full bg-slate-800 border-2 border-slate-700 flex items-center justify-center text-3xl font-bold text-cyan-400 font-mono">
                                {initial}
                            </div>
                        }.into_any()
                    }}

                    <div class="flex-1 text-center md:text-left">
                        <h1 class="text-3xl font-bold text-white">{member.name}</h1>
                        <p class="text-lg text-cyan-400 font-medium">{member.role}</p>
                        <p class="text-slate-400">{member.organization}</p>

                        <div class="mt-4 flex flex-wrap justify-center md:justify-start gap-2">
                            {member.domains.into_iter().map(|d| view! {
                                <span class="rounded bg-slate-800 px-3 py-1 text-xs font-bold font-mono uppercase tracking-wider text-slate-300 border border-slate-700">
                                    {d}
                                </span>
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="flex gap-3">
                        <button class="rounded-lg bg-cyan-500 px-6 py-2.5 text-sm font-bold text-slate-950 hover:bg-cyan-400 transition-colors">
                            "Connect"
                        </button>
                        <button class="rounded-lg border border-slate-700 bg-slate-800 px-4 py-2.5 text-sm font-bold text-white hover:bg-slate-700 transition-colors">
                            "Message"
                        </button>
                    </div>
                </div>
            </div>

            /* Activity / About Grid */
            <div class="grid gap-6 md:grid-cols-3">
                <div class="md:col-span-2 space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                        <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-4">"About"</h2>
                        <p class="text-slate-300 leading-relaxed">
                            "Pharmacovigilance professional focused on risk management and signal detection. Interested in the intersection of AI and drug safety."
                        </p>
                    </div>

                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                        <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-4">"Recent Activity"</h2>
                        <div class="text-center py-12 text-slate-600 italic">
                            "No public activity to show yet."
                        </div>
                    </div>
                </div>

                <div class="space-y-6">
                    <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                        <h2 class="text-sm font-bold text-slate-500 uppercase tracking-widest mb-4">"Stats"</h2>
                        <div class="space-y-4">
                            <StatRow label="Connections" value="124" />
                            <StatRow label="Posts" value="12" />
                            <StatRow label="Certificates" value="4" />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatRow(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="flex justify-between items-center">
            <span class="text-sm text-slate-500">{label}</span>
            <span class="text-sm font-bold text-white font-mono">{value}</span>
        </div>
    }
}
