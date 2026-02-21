//! UpgradeShop component - Tier: T2-C

use leptos::prelude::*;

/// Upgrade definition. Tier: T2-P
#[derive(Clone, Copy)]
pub struct Upgrade {
    pub name: &'static str,
    pub cost: u64,
    pub cps: u64,
}

pub const UPGRADES: &[Upgrade] = &[
    Upgrade { name: "Auto-Clicker", cost: 10, cps: 1 },
    Upgrade { name: "Turbo Click", cost: 50, cps: 5 },
    Upgrade { name: "Mega Click", cost: 200, cps: 20 },
    Upgrade { name: "Quantum Click", cost: 1000, cps: 100 },
];

pub fn calc_cost(idx: usize, owned_count: u64) -> u64 {
    UPGRADES[idx].cost * (owned_count + 1)
}

#[component]
pub fn UpgradeShop<F: Fn(usize) + Clone + 'static>(
    score: ReadSignal<u64>,
    owned: ReadSignal<Vec<u64>>,
    on_buy: F,
) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-lg p-6" id="upgrades">
            <h2 class="text-2xl font-bold mb-4 text-yellow-400">"Upgrades"</h2>
            <UpgradeBtn idx=0 score owned on_buy=on_buy.clone() />
            <UpgradeBtn idx=1 score owned on_buy=on_buy.clone() />
            <UpgradeBtn idx=2 score owned on_buy=on_buy.clone() />
            <UpgradeBtn idx=3 score owned on_buy />
        </div>
    }
}

#[component]
fn UpgradeBtn<F: Fn(usize) + 'static>(
    idx: usize,
    score: ReadSignal<u64>,
    owned: ReadSignal<Vec<u64>>,
    on_buy: F,
) -> impl IntoView {
    let u = UPGRADES[idx];
    let cost_fn = move || calc_cost(idx, owned.get()[idx]);
    let can_buy = move || score.get() >= cost_fn();
    view! {
        <div class="flex justify-between items-center bg-gray-700 p-3 rounded mb-2">
            <span class="font-bold">{u.name} " (+" {u.cps} " CPS)"</span>
            <button on:click=move |_| on_buy(idx) disabled=move || !can_buy() id=format!("buy-{idx}")
                class="bg-green-600 px-4 py-1 rounded disabled:opacity-50">
                {move || format!("Buy ({}) - Owned: {}", cost_fn(), owned.get()[idx])}
            </button>
        </div>
    }
}
