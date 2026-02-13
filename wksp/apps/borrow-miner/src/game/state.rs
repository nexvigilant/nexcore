//! Game state - Leptos signals and actions
//!
//! Uses T2-P primitive wrappers for type safety.

use leptos::prelude::*;
use std::collections::VecDeque;

use crate::{MAX_OWNED, PHOSPHOR_GREEN};
use super::types::{
    Combo, Depth, FloatingScore, Lifetime, OreType, Particle, ParticleId, Points, Score, random,
};

/// Game state container (reactive signals)
/// Tier: T2-C (composed of T2-P wrapped signals)
#[derive(Clone)]
pub struct GameState {
    pub score: RwSignal<Score>,
    pub combo: RwSignal<Combo>,
    pub depth: RwSignal<Depth>,
    pub owned_ores: RwSignal<VecDeque<OreType>>,
    pub dropped_count: RwSignal<usize>,
    pub mining_cooldown: RwSignal<bool>,
    pub particles: RwSignal<Vec<Particle>>,
    pub floating_scores: RwSignal<Vec<FloatingScore>>,
    pub shake: RwSignal<bool>,
    pub last_ore: RwSignal<Option<OreType>>,
    particle_id: RwSignal<ParticleId>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            score: RwSignal::new(Score::ZERO),
            combo: RwSignal::new(Combo::ZERO),
            depth: RwSignal::new(Depth::default()),
            owned_ores: RwSignal::new(VecDeque::new()),
            dropped_count: RwSignal::new(0),
            mining_cooldown: RwSignal::new(false),
            particles: RwSignal::new(Vec::new()),
            floating_scores: RwSignal::new(Vec::new()),
            shake: RwSignal::new(false),
            last_ore: RwSignal::new(None),
            particle_id: RwSignal::new(ParticleId::default()),
        }
    }

    pub fn mine(&self) {
        if self.mining_cooldown.get() { return; }
        if self.owned_ores.get().len() >= MAX_OWNED { return; }

        self.set_cooldown();
        let ore = OreType::roll();
        self.last_ore.set(Some(ore));
        self.owned_ores.update(|o| o.push_back(ore));

        let points = Points::from_ore(&ore, self.combo.get(), self.depth.get());
        self.score.update(|s| *s += points);
        self.combo.update(|c| *c = c.increment());
        self.depth.update(|d| *d = d.increase(0.01));

        if ore.is_rare() { self.do_shake(); }
        self.add_mine_particles(ore);
        self.add_score_popup(points);
    }

    pub fn drop_ore(&self) {
        let ore = self.owned_ores.get().front().copied();
        let Some(ore) = ore else { return };

        self.owned_ores.update(|o| { o.pop_front(); });
        let bonus = Points(ore.base_value() / 2);
        self.score.update(|s| *s += bonus);
        self.dropped_count.update(|c| *c += 1);
        self.add_drop_particles();
    }

    pub fn tick(&self, dt: f64) {
        self.update_particles(dt);
        self.update_scores(dt);
    }

    pub fn decay_combo(&self) {
        self.combo.update(|c| *c = c.decrement());
    }
}

// Private helpers
impl GameState {
    fn set_cooldown(&self) {
        self.mining_cooldown.set(true);
        schedule_timeout(self.mining_cooldown, false, 200);
    }

    fn do_shake(&self) {
        self.shake.set(true);
        schedule_timeout(self.shake, false, 150);
    }

    fn next_particle_id(&self) -> ParticleId {
        let id = self.particle_id.get();
        self.particle_id.set(id.next_batch(10));
        id
    }

    fn add_mine_particles(&self, ore: OreType) {
        let base_id = self.next_particle_id();
        let particles = make_mine_particles(base_id, ore);
        self.particles.update(|p| p.extend(particles));
    }

    fn add_drop_particles(&self) {
        let base_id = self.next_particle_id();
        let particles = make_drop_particles(base_id);
        self.particles.update(|p| p.extend(particles));
    }

    fn add_score_popup(&self, points: Points) {
        let id = ParticleId(self.particle_id.get().0 + 100);
        let score = make_floating_score(id, points);
        self.floating_scores.update(|s| s.push(score));
    }

    fn update_particles(&self, dt: f64) {
        self.particles.update(|particles| {
            particles.iter_mut().for_each(|p| tick_particle(p, dt));
            particles.retain(|p| !p.life.is_expired());
        });
    }

    fn update_scores(&self, dt: f64) {
        self.floating_scores.update(|scores| {
            scores.iter_mut().for_each(|s| tick_score(s, dt));
            scores.retain(|s| !s.life.is_expired());
        });
    }
}

// Pure functions
fn tick_particle(p: &mut Particle, dt: f64) {
    p.x += p.vx * dt;
    p.y += p.vy * dt;
    p.vy += 200.0 * dt;
    p.life = p.life.decay(dt / 0.5);
}

fn tick_score(s: &mut FloatingScore, dt: f64) {
    s.y -= 30.0 * dt;
    s.life = s.life.decay(dt);
}

fn make_mine_particles(base_id: ParticleId, ore: OreType) -> Vec<Particle> {
    (0..8).map(|i| {
        let angle = (i as f64) * std::f64::consts::PI / 4.0 + random() * 0.5;
        let speed = 50.0 + random() * 100.0;
        Particle {
            id: ParticleId(base_id.0 + i),
            x: 50.0, y: 50.0,
            vx: angle.cos() * speed, vy: angle.sin() * speed,
            color: ore.color().to_string(),
            life: Lifetime::default(),
            size: 4.0 + random() * 6.0,
        }
    }).collect()
}

fn make_drop_particles(base_id: ParticleId) -> Vec<Particle> {
    (0..4).map(|i| Particle {
        id: ParticleId(base_id.0 + i),
        x: 50.0, y: 85.0,
        vx: (random() - 0.5) * 60.0, vy: -30.0 - random() * 40.0,
        color: PHOSPHOR_GREEN.to_string(),
        life: Lifetime(0.8),
        size: 3.0 + random() * 4.0,
    }).collect()
}

fn make_floating_score(id: ParticleId, points: Points) -> FloatingScore {
    FloatingScore {
        id, value: points,
        life: Lifetime::default(),
        x: 50.0 + random() * 20.0 - 10.0,
        y: 40.0,
    }
}

fn schedule_timeout<T: Clone + Send + Sync + 'static>(signal: RwSignal<T>, value: T, _ms: i32) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::prelude::*;
        let cb = Closure::once(move || signal.set(value));
        if let Some(window) = leptos::web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(), _ms);
        }
        cb.forget();
    }
    #[cfg(not(target_arch = "wasm32"))]
    { signal.set(value); }
}
