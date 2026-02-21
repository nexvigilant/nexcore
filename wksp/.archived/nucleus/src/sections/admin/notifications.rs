//! Admin: Notifications management — templates, channels, delivery config

use leptos::prelude::*;

#[component]
pub fn NotificationsAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Notifications Admin"</h1>
                <p class="mt-1 text-slate-400">"Manage notification templates, delivery channels, and user preferences"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="Templates" value="12" color="text-cyan-400"/>
                <Stat label="Channels" value="3" color="text-violet-400"/>
                <Stat label="Sent Today" value="0" color="text-emerald-400"/>
                <Stat label="Delivery Rate" value="—" color="text-amber-400"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Delivery Channels"</h2>
                <div class="space-y-2">
                    <ChannelRow name="In-App" status="Active" desc="Push notifications within Nucleus"/>
                    <ChannelRow name="Email" status="Active" desc="Transactional and marketing emails via SendGrid"/>
                    <ChannelRow name="Browser Push" status="Active" desc="Web push notifications for desktop/mobile"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Notification Templates"</h2>
                <div class="space-y-2">
                    <TemplateRow name="Welcome" channels="Email, In-App" category="Onboarding"/>
                    <TemplateRow name="New Post Reply" channels="In-App, Push" category="Community"/>
                    <TemplateRow name="Course Completed" channels="Email, In-App" category="Academy"/>
                    <TemplateRow name="Signal Detected" channels="Email, In-App, Push" category="Vigilance"/>
                    <TemplateRow name="Mentoring Match" channels="Email, In-App" category="Careers"/>
                    <TemplateRow name="Badge Earned" channels="In-App" category="Community"/>
                    <TemplateRow name="Assessment Due" channels="Email, Push" category="Academy"/>
                    <TemplateRow name="New Connection" channels="In-App" category="Community"/>
                    <TemplateRow name="Subscription Renewal" channels="Email" category="Billing"/>
                    <TemplateRow name="Guardian Alert" channels="Email, In-App, Push" category="Vigilance"/>
                    <TemplateRow name="Weekly Digest" channels="Email" category="System"/>
                    <TemplateRow name="Password Reset" channels="Email" category="Auth"/>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Delivery Settings"</h2>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 space-y-3">
                    <ConfigRow label="Batch Window" value="5 minutes"/>
                    <ConfigRow label="Rate Limit" value="10/hour per user"/>
                    <ConfigRow label="Quiet Hours" value="22:00 - 08:00 (user local)"/>
                    <ConfigRow label="Retry Policy" value="3 attempts, exponential backoff"/>
                    <ConfigRow label="Unsubscribe" value="One-click in all emails"/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: &'static str, color: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">{label}</p>
            <p class=format!("mt-2 text-3xl font-bold font-mono {color}")>{value}</p>
        </div>
    }
}

#[component]
fn ChannelRow(name: &'static str, status: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <div class="h-2 w-2 rounded-full bg-emerald-400"></div>
                <div>
                    <span class="text-sm text-white font-medium">{name}</span>
                    <p class="text-xs text-slate-500">{desc}</p>
                </div>
            </div>
            <span class="rounded bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-400 font-mono uppercase">{status}</span>
        </div>
    }
}

#[component]
fn TemplateRow(
    name: &'static str,
    channels: &'static str,
    category: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between rounded-lg border border-slate-800/50 bg-slate-900/30 p-4">
            <div class="flex items-center gap-3">
                <span class="text-sm text-white font-medium">{name}</span>
                <span class="text-xs text-slate-500">{channels}</span>
            </div>
            <span class="rounded bg-slate-800 px-2 py-0.5 text-[10px] text-slate-500 font-mono uppercase">{category}</span>
        </div>
    }
}

#[component]
fn ConfigRow(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-2 border-b border-slate-800/30 last:border-0">
            <span class="text-sm text-slate-400">{label}</span>
            <span class="text-sm text-white font-mono">{value}</span>
        </div>
    }
}
