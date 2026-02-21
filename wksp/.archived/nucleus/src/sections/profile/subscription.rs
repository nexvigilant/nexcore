//! Subscription management page — plan details, billing, upgrade/downgrade

use leptos::prelude::*;

#[component]
pub fn SubscriptionPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto space-y-6">
            <div>
                <a href="/profile" class="text-cyan-400 hover:text-cyan-300 text-sm">"Back to Profile"</a>
                <h1 class="text-2xl font-bold text-white mt-2">"Subscription"</h1>
                <p class="text-slate-400 mt-1">"Manage your plan, billing, and payment methods"</p>
            </div>

            /* Current plan */
            <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
                <div class="flex items-center justify-between">
                    <div>
                        <span class="text-xs text-slate-500 uppercase font-mono">"Current Plan"</span>
                        <h2 class="text-xl font-bold text-white mt-1">"Starter (Free)"</h2>
                        <p class="text-slate-400 text-sm mt-1">"Community access, 2 courses, basic assessments"</p>
                    </div>
                    <button class="px-6 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">
                        "Upgrade"
                    </button>
                </div>
            </div>

            /* Plan comparison */
            <div>
                <h2 class="text-lg font-semibold text-white mb-4">"Available Plans"</h2>
                <div class="grid gap-4 lg:grid-cols-3">
                    <PlanCard
                        name="Starter"
                        price="Free"
                        current=true
                        features=vec!["Community access", "2 courses", "Basic assessments", "Public circles"]
                    />
                    <PlanCard
                        name="Professional"
                        price="$49/mo"
                        current=false
                        features=vec!["Full academy access", "All 14 assessments", "Signal detection tools", "Mentoring", "Private circles", "PVDSL access"]
                    />
                    <PlanCard
                        name="Enterprise"
                        price="Custom"
                        current=false
                        features=vec!["Everything in Professional", "Consulting services", "API access", "Custom integrations", "QPPV support", "Dedicated account manager"]
                    />
                </div>
            </div>

            /* Billing history */
            <div>
                <h2 class="text-lg font-semibold text-white mb-4">"Billing History"</h2>
                <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6 text-center">
                    <p class="text-slate-400">"No billing history. Upgrade to a paid plan to see invoices here."</p>
                </div>
            </div>

            /* Payment method */
            <div>
                <h2 class="text-lg font-semibold text-white mb-4">"Payment Method"</h2>
                <div class="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6 text-center">
                    <p class="text-slate-400">"No payment method on file"</p>
                    <button class="mt-3 px-4 py-2 border border-slate-700 text-slate-400 hover:text-white rounded-lg text-sm font-medium transition-colors">
                        "Add Payment Method"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PlanCard(
    #[prop(into)] name: String,
    #[prop(into)] price: String,
    current: bool,
    features: Vec<&'static str>,
) -> impl IntoView {
    let border = if current {
        "border-cyan-500/30"
    } else {
        "border-slate-700/50"
    };
    view! {
        <div class={format!("bg-slate-800/50 border {} rounded-xl p-6", border)}>
            {if current { Some(view! {
                <span class="text-xs text-cyan-400 font-mono uppercase">"Current Plan"</span>
            }) } else { None }}
            <h3 class="text-lg font-bold text-white mt-1">{name}</h3>
            <p class="text-2xl font-bold text-cyan-400 mt-1">{price}</p>
            <ul class="mt-4 space-y-2">
                {features.into_iter().map(|f| view! {
                    <li class="text-sm text-slate-300 flex items-center gap-2">
                        <span class="text-cyan-400">"+"</span>
                        {f}
                    </li>
                }).collect::<Vec<_>>()}
            </ul>
            {if !current { Some(view! {
                <button class="w-full mt-4 px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors">
                    "Select Plan"
                </button>
            }) } else { None }}
        </div>
    }
}
