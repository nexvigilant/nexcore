//! Admin: Billing management — subscriptions, plans, revenue

use leptos::prelude::*;

#[component]
pub fn BillingAdminPage() -> impl IntoView {
    view! {
        <div class="mx-auto max-w-6xl px-4 py-8 space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-white font-mono uppercase tracking-tight">"Billing Admin"</h1>
                <p class="mt-1 text-slate-400">"Manage subscription plans, billing cycles, and revenue tracking"</p>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                <Stat label="MRR" value="$0" color="text-emerald-400"/>
                <Stat label="Subscribers" value="0" color="text-cyan-400"/>
                <Stat label="Churn Rate" value="0%" color="text-amber-400"/>
                <Stat label="LTV" value="$0" color="text-violet-400"/>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Subscription Plans"</h2>
                <div class="grid gap-4 lg:grid-cols-3">
                    <PlanCard
                        name="Starter"
                        price="Free"
                        features="Community access, 2 courses, basic assessments"
                        subscribers=0
                    />
                    <PlanCard
                        name="Professional"
                        price="$49/mo"
                        features="Full academy, all assessments, signal detection, mentoring"
                        subscribers=0
                    />
                    <PlanCard
                        name="Enterprise"
                        price="Custom"
                        features="Everything + consulting, API access, custom integrations, QPPV support"
                        subscribers=0
                    />
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Recent Transactions"</h2>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-6 text-center">
                    <p class="text-slate-400">"No transactions yet. Revenue tracking will activate when the first subscription is processed."</p>
                </div>
            </div>

            <div>
                <h2 class="text-sm font-bold uppercase tracking-widest text-slate-500 mb-4">"Payment Configuration"</h2>
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5 space-y-3">
                    <ConfigRow label="Payment Provider" value="Stripe"/>
                    <ConfigRow label="Currency" value="USD"/>
                    <ConfigRow label="Tax Handling" value="Stripe Tax"/>
                    <ConfigRow label="Invoice Generation" value="Automatic"/>
                    <ConfigRow label="Webhook Status" value="Configured"/>
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
fn PlanCard(
    name: &'static str,
    price: &'static str,
    features: &'static str,
    subscribers: u32,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <h3 class="text-lg font-bold text-white font-mono">{name}</h3>
            <p class="mt-1 text-2xl font-bold text-cyan-400">{price}</p>
            <p class="mt-3 text-sm text-slate-400 leading-relaxed">{features}</p>
            <p class="mt-3 text-xs text-slate-500 font-mono">{format!("{} subscribers", subscribers)}</p>
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
