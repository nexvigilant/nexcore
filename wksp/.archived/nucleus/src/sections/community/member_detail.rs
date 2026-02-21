//! Member detail — public member profile

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn MemberDetailPage() -> impl IntoView {
    let params = use_params_map();
    let _user_id = move || params.get().get("userId").unwrap_or_default();

    let connected = RwSignal::new(false);
    let (active_tab, set_active_tab) = signal("activity");

    view! {
        <div class="mx-auto max-w-4xl px-4 py-12">
            /* Back navigation */
            <a href="/community/members" class="inline-flex items-center gap-2 text-xs font-bold text-slate-500 font-mono uppercase tracking-widest hover:text-white transition-colors mb-8">
                "<-" " MEMBERS"
            </a>

            /* Profile header */
            <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-8 mb-8 relative overflow-hidden">
                <div class="absolute top-0 right-0 w-64 h-64 bg-gradient-to-bl from-cyan-500/5 to-transparent rounded-full -translate-y-1/2 translate-x-1/2" />

                <div class="relative flex items-start gap-6">
                    <div class="h-20 w-20 rounded-full bg-slate-800 border-2 border-slate-700 flex items-center justify-center text-2xl font-bold text-cyan-400 font-mono shrink-0">"SC"</div>

                    <div class="flex-1">
                        <div class="flex items-start justify-between">
                            <div>
                                <h1 class="text-2xl font-bold text-white">"Dr. Sarah Chen"</h1>
                                <p class="text-sm text-slate-400 mt-1">"Signal Detection Lead at Major Pharma Co"</p>
                            </div>
                            <button
                                class=move || if connected.get() {
                                    "shrink-0 rounded-lg bg-emerald-500/10 border border-emerald-500/20 px-6 py-2.5 text-xs font-bold text-emerald-400 uppercase tracking-widest"
                                } else {
                                    "shrink-0 rounded-lg bg-cyan-600 px-6 py-2.5 text-xs font-bold text-white hover:bg-cyan-500 transition-all uppercase tracking-widest shadow-lg shadow-cyan-900/20"
                                }
                                on:click=move |_| connected.set(!connected.get())
                            >
                                {move || if connected.get() { "CONNECTED" } else { "CONNECT" }}
                            </button>
                        </div>

                        <p class="mt-4 text-sm text-slate-300 leading-relaxed">
                            "Pharmacovigilance professional with 12 years of experience in signal detection, disproportionality analysis, and quantitative safety evaluation. Passionate about applying Bayesian methods to improve drug safety surveillance."
                        </p>

                        <div class="mt-6 flex items-center gap-8 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest">
                            <span>"47 posts"</span>
                            <span>"89 connections"</span>
                            <span>"3 circles"</span>
                            <span>"Joined Dec 2025"</span>
                        </div>
                    </div>
                </div>
            </div>

            /* PV domains and badges */
            <div class="grid gap-6 md:grid-cols-2 mb-8">
                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                    <h3 class="text-sm font-bold text-slate-400 font-mono uppercase tracking-widest mb-4">"PV DOMAINS"</h3>
                    <div class="flex flex-wrap gap-2">
                        <DomainBadge code="D02" name="Signal Detection" />
                        <DomainBadge code="D03" name="Disproportionality" />
                        <DomainBadge code="D07" name="Benefit-Risk" />
                        <DomainBadge code="D08" name="PSUR/PBRER" />
                        <DomainBadge code="D10" name="Risk Management" />
                    </div>
                </div>

                <div class="rounded-2xl border border-slate-800 bg-slate-900/50 p-6">
                    <h3 class="text-sm font-bold text-slate-400 font-mono uppercase tracking-widest mb-4">"BADGES"</h3>
                    <div class="flex flex-wrap gap-3">
                        <Badge icon="*" label="Early Adopter" color="text-amber-400 bg-amber-500/10 border-amber-500/20" />
                        <Badge icon="!" label="Top Contributor" color="text-cyan-400 bg-cyan-500/10 border-cyan-500/20" />
                        <Badge icon="+" label="Signal Spotter" color="text-emerald-400 bg-emerald-500/10 border-emerald-500/20" />
                        <Badge icon="#" label="Mentor" color="text-violet-400 bg-violet-500/10 border-violet-500/20" />
                    </div>
                </div>
            </div>

            /* Activity tabs */
            <div class="flex gap-6 border-b border-slate-800 pb-4 mb-8 font-mono text-[11px] font-bold uppercase tracking-widest text-slate-500">
                <ProfileTab label="activity" display="Recent Activity" active=active_tab set_active=set_active_tab />
                <ProfileTab label="posts" display="Posts" active=active_tab set_active=set_active_tab />
                <ProfileTab label="circles" display="Circles" active=active_tab set_active=set_active_tab />
            </div>

            <Show when=move || active_tab.get() == "activity">
                <div class="space-y-3">
                    <ActivityItem
                        action="posted in"
                        target="Signal Detection"
                        time="2h ago"
                        preview="Has anyone compared PRR vs ROR for pediatric safety databases?"
                    />
                    <ActivityItem
                        action="replied to"
                        target="James Wilson"
                        time="5h ago"
                        preview="Agreed — the BCPNN approach handles sparse data much better in our experience."
                    />
                    <ActivityItem
                        action="joined"
                        target="AI & Automation in PV"
                        time="1d ago"
                        preview=""
                    />
                    <ActivityItem
                        action="liked a post by"
                        target="Maria Santos"
                        time="2d ago"
                        preview="Important update: EMA has released new guidance on signal management..."
                    />
                </div>
            </Show>

            <Show when=move || active_tab.get() == "posts">
                <div class="space-y-4">
                    <MemberPost
                        title="PRR vs ROR in Pediatric Safety Databases"
                        time="2h ago"
                        likes=24
                        replies=3
                    />
                    <MemberPost
                        title="Bayesian Approaches to Small Cell Counts"
                        time="3d ago"
                        likes=31
                        replies=8
                    />
                    <MemberPost
                        title="New IC025 Threshold Recommendations"
                        time="1w ago"
                        likes=45
                        replies=12
                    />
                </div>
            </Show>

            <Show when=move || active_tab.get() == "circles">
                <div class="grid gap-4 md:grid-cols-2">
                    <MemberCircle name="Signal Detection Practitioners" members=89 role_label="Admin" />
                    <MemberCircle name="AI & Automation in PV" members=67 role_label="Member" />
                    <MemberCircle name="Regulatory Affairs Hub" members=72 role_label="Member" />
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ProfileTab(
    label: &'static str,
    display: &'static str,
    active: ReadSignal<&'static str>,
    set_active: WriteSignal<&'static str>,
) -> impl IntoView {
    view! {
        <button
            class=move || if active.get() == label {
                "text-cyan-400 border-b-2 border-cyan-400 pb-4 -mb-[18px] transition-colors"
            } else {
                "hover:text-white transition-colors"
            }
            on:click=move |_| set_active.set(label)
        >
            {display}
        </button>
    }
}

#[component]
fn DomainBadge(code: &'static str, name: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 rounded-lg bg-slate-800/50 border border-slate-700 px-3 py-1.5">
            <span class="text-[10px] font-bold text-cyan-400 font-mono">{code}</span>
            <span class="text-xs text-slate-300">{name}</span>
        </div>
    }
}

#[component]
fn Badge(icon: &'static str, label: &'static str, color: &'static str) -> impl IntoView {
    view! {
        <div class=format!("flex items-center gap-1.5 rounded-full border px-3 py-1 {color}")>
            <span class="text-xs font-bold">{icon}</span>
            <span class="text-xs font-medium">{label}</span>
        </div>
    }
}

#[component]
fn ActivityItem(
    action: &'static str,
    target: &'static str,
    time: &'static str,
    preview: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/30 p-4 hover:border-slate-700 transition-all">
            <div class="flex items-center justify-between">
                <p class="text-sm text-slate-300">
                    <span class="text-slate-500">{action}" "</span>
                    <span class="font-bold text-white">{target}</span>
                </p>
                <span class="text-[10px] text-slate-600 font-mono">{time}</span>
            </div>
            {(!preview.is_empty()).then(|| view! {
                <p class="mt-2 text-xs text-slate-500 italic truncate">{preview}</p>
            })}
        </div>
    }
}

#[component]
fn MemberPost(title: &'static str, time: &'static str, likes: u32, replies: u32) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors cursor-pointer">
            <h4 class="text-sm font-bold text-white hover:text-cyan-400 transition-colors">{title}</h4>
            <div class="mt-2 flex items-center gap-4 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest">
                <span>{time}</span>
                <span>{likes}" likes"</span>
                <span>{replies}" replies"</span>
            </div>
        </div>
    }
}

#[component]
fn MemberCircle(name: &'static str, members: u32, role_label: &'static str) -> impl IntoView {
    let role_color = if role_label == "Admin" {
        "text-amber-400 bg-amber-500/10"
    } else {
        "text-slate-400 bg-slate-800"
    };

    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 hover:border-slate-700 transition-colors">
            <div class="flex items-center justify-between">
                <h4 class="text-sm font-bold text-white">{name}</h4>
                <span class=format!("rounded-full px-2.5 py-0.5 text-xs font-medium {role_color}")>{role_label}</span>
            </div>
            <p class="mt-2 text-[10px] font-bold text-slate-500 font-mono uppercase tracking-widest">{members}" members"</p>
        </div>
    }
}
