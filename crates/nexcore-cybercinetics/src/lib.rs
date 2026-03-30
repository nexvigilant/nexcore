#![doc = "Cyber-Cinetics: typed feedback controller encoding ∂(→(ν, ς, ρ))"]
#![doc = ""]
#![doc = "Maps the primitive composition ∂(→(ν, ς, ρ)) to a Rust type system:"]
#![doc = "- ν (frequency) — oscillation rate of the control loop"]
#![doc = "- ς (state) — current system snapshot"]
#![doc = "- ρ (recursion) — self-referential observation depth"]
#![doc = "- → (causality) — cause-effect chain linking input to output"]
#![doc = "- ∂ (boundary) — the controller envelope constraining all above"]
#![doc = ""]
#![doc = "Primary use: hook-binary feedback linking where hooks observe"]
#![doc = "system state and binaries act on it, creating a closed loop."]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// ν — Frequency of the control loop (Hz or iterations/sec).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Nu {
    pub rate: f64,
    pub floor: f64,
}

impl Nu {
    pub fn new(rate: f64, floor: f64) -> Self {
        Self { rate, floor }
    }

    pub fn is_decayed(&self) -> bool {
        self.rate < self.floor
    }

    pub fn health_ratio(&self) -> f64 {
        if self.floor <= 0.0 {
            return f64::INFINITY;
        }
        self.rate / self.floor
    }
}

/// ς — State snapshot of the controlled system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sigma<S: Clone + PartialEq> {
    pub value: S,
    pub tick: u64,
}

impl<S: Clone + PartialEq> Sigma<S> {
    pub fn new(value: S) -> Self {
        Self { value, tick: 0 }
    }

    pub fn transition(&mut self, next: S) {
        self.value = next;
        self.tick += 1;
    }

    pub fn has_transitioned(&self) -> bool {
        self.tick > 0
    }
}

/// ρ — Recursion depth for self-referential observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rho {
    pub depth: u8,
    pub ceiling: u8,
}

impl Rho {
    pub fn new(ceiling: u8) -> Self {
        Self { depth: 0, ceiling }
    }

    pub fn deepen(&mut self) -> bool {
        if self.depth < self.ceiling {
            self.depth += 1;
            true
        } else {
            false
        }
    }

    pub fn surface(&mut self) {
        self.depth = 0;
    }

    pub fn is_saturated(&self) -> bool {
        self.depth >= self.ceiling
    }
}

/// A single cause-effect link in the causal chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub cause: String,
    pub effect: String,
    pub fidelity: f64,
}

/// → — Causal chain from input to output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arrow {
    links: Vec<CausalLink>,
}

impl Arrow {
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    pub fn push(&mut self, cause: impl Into<String>, effect: impl Into<String>, fidelity: f64) {
        self.links.push(CausalLink {
            cause: cause.into(),
            effect: effect.into(),
            fidelity: fidelity.clamp(0.0, 1.0),
        });
    }

    /// F_total = Product(F_i) across all hops.
    pub fn f_total(&self) -> f64 {
        if self.links.is_empty() {
            return 0.0;
        }
        self.links.iter().map(|l| l.fidelity).product()
    }

    pub fn len(&self) -> usize {
        self.links.len()
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    pub fn weakest(&self) -> Option<&CausalLink> {
        self.links.iter().min_by(|a, b| {
            a.fidelity
                .partial_cmp(&b.fidelity)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

impl Default for Arrow {
    fn default() -> Self {
        Self::new()
    }
}

/// Controller verdict after one tick of the feedback loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Stable,
    FrequencyDecay,
    FidelityDegraded,
    RecursionSaturated,
    Compound(Vec<Verdict>),
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Stable => write!(f, "STABLE"),
            Verdict::FrequencyDecay => write!(f, "FREQ_DECAY"),
            Verdict::FidelityDegraded => write!(f, "FIDELITY_DEGRADED"),
            Verdict::RecursionSaturated => write!(f, "RECURSION_SATURATED"),
            Verdict::Compound(vs) => {
                let labels: Vec<String> = vs.iter().map(|v| v.to_string()).collect();
                write!(f, "COMPOUND({})", labels.join("+"))
            }
        }
    }
}

/// ∂(→(ν, ς, ρ)) — The feedback controller.
///
/// Parametric over the state type `S`. The boundary (∂) wraps
/// the causal chain (→) which links frequency (ν), state (ς),
/// and recursion (ρ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Controller<S: Clone + PartialEq> {
    pub nu: Nu,
    pub sigma: Sigma<S>,
    pub rho: Rho,
    pub arrow: Arrow,
    pub f_min: f64,
}

impl<S: Clone + PartialEq + fmt::Debug> Controller<S> {
    pub fn new(initial_state: S, nu_rate: f64, nu_floor: f64, rho_ceiling: u8, f_min: f64) -> Self {
        Self {
            nu: Nu::new(nu_rate, nu_floor),
            sigma: Sigma::new(initial_state),
            rho: Rho::new(rho_ceiling),
            arrow: Arrow::new(),
            f_min,
        }
    }

    pub fn tick(&mut self) -> Verdict {
        let mut issues = Vec::new();

        if self.nu.is_decayed() {
            issues.push(Verdict::FrequencyDecay);
        }

        if !self.arrow.is_empty() && self.arrow.f_total() < self.f_min {
            issues.push(Verdict::FidelityDegraded);
        }

        if self.rho.is_saturated() {
            issues.push(Verdict::RecursionSaturated);
        }

        match issues.len() {
            0 => Verdict::Stable,
            1 => issues.into_iter().next().unwrap_or(Verdict::Stable),
            _ => Verdict::Compound(issues),
        }
    }

    pub fn act(
        &mut self,
        next_state: S,
        cause: impl Into<String>,
        effect: impl Into<String>,
        fidelity: f64,
    ) {
        self.sigma.transition(next_state);
        self.arrow.push(cause, effect, fidelity);
    }

    pub fn observe(&mut self) -> bool {
        self.rho.deepen()
    }

    pub fn surface(&mut self) {
        self.rho.surface();
    }

    pub fn measure_frequency(&mut self, rate: f64) {
        self.nu.rate = rate;
    }
}

/// A hook-binary pair in the feedback loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookBinding {
    pub hook: String,
    pub binary: String,
    pub event: String,
    pub fidelity: f64,
}

impl HookBinding {
    pub fn new(
        hook: impl Into<String>,
        binary: impl Into<String>,
        event: impl Into<String>,
    ) -> Self {
        Self {
            hook: hook.into(),
            binary: binary.into(),
            event: event.into(),
            fidelity: 1.0,
        }
    }

    pub fn degrade(&mut self, factor: f64) {
        self.fidelity = (self.fidelity * factor).clamp(0.0, 1.0);
    }

    pub fn restore(&mut self) {
        self.fidelity = 1.0;
    }
}

/// Registry of hook-binary bindings governed by a controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingRegistry<S: Clone + PartialEq> {
    pub controller: Controller<S>,
    pub bindings: Vec<HookBinding>,
}

impl<S: Clone + PartialEq + fmt::Debug> BindingRegistry<S> {
    pub fn new(controller: Controller<S>) -> Self {
        Self {
            controller,
            bindings: Vec::new(),
        }
    }

    pub fn register(&mut self, binding: HookBinding) {
        self.bindings.push(binding);
    }

    pub fn aggregate_fidelity(&self) -> f64 {
        if self.bindings.is_empty() {
            return 0.0;
        }
        self.bindings.iter().map(|b| b.fidelity).product()
    }

    pub fn degraded_bindings(&self, threshold: f64) -> Vec<&HookBinding> {
        self.bindings
            .iter()
            .filter(|b| b.fidelity < threshold)
            .collect()
    }

    /// Batch degrade all bindings that missed their last execution window.
    ///
    /// `factor` is the degradation multiplier (0.0–1.0) applied to each
    /// binding whose fidelity is above `floor`. Bindings already at or
    /// below `floor` are left unchanged.
    ///
    /// Returns the count of bindings that were degraded.
    pub fn decay_all(&mut self, factor: f64, floor: f64) -> usize {
        let factor = factor.clamp(0.0, 1.0);
        let floor = floor.clamp(0.0, 1.0);
        let mut count = 0;
        for binding in &mut self.bindings {
            if binding.fidelity > floor {
                binding.fidelity = (binding.fidelity * factor).max(floor);
                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nu_health_ratio() {
        let nu = Nu::new(10.0, 5.0);
        assert!(!nu.is_decayed());
        assert!((nu.health_ratio() - 2.0).abs() < f64::EPSILON);

        let decayed = Nu::new(3.0, 5.0);
        assert!(decayed.is_decayed());
    }

    #[test]
    fn sigma_transitions() {
        let mut sigma = Sigma::new("idle");
        assert!(!sigma.has_transitioned());
        sigma.transition("active");
        assert!(sigma.has_transitioned());
        assert_eq!(sigma.tick, 1);
        assert_eq!(sigma.value, "active");
    }

    #[test]
    fn rho_depth_ceiling() {
        let mut rho = Rho::new(2);
        assert!(!rho.is_saturated());
        assert!(rho.deepen());
        assert!(rho.deepen());
        assert!(rho.is_saturated());
        assert!(!rho.deepen());
        rho.surface();
        assert_eq!(rho.depth, 0);
    }

    #[test]
    fn arrow_fidelity_composition() {
        let mut arrow = Arrow::new();
        arrow.push("hook", "binary", 0.95);
        arrow.push("binary", "state", 0.90);
        assert!((arrow.f_total() - 0.855).abs() < 1e-9);
        assert_eq!(arrow.len(), 2);
    }

    #[test]
    fn arrow_weakest_link() {
        let mut arrow = Arrow::new();
        arrow.push("a", "b", 0.95);
        arrow.push("b", "c", 0.70);
        arrow.push("c", "d", 0.85);
        let weakest = arrow.weakest();
        assert!(weakest.is_some());
        assert!((weakest.map(|w| w.fidelity).unwrap_or(0.0) - 0.70).abs() < f64::EPSILON);
    }

    #[test]
    fn controller_stable_verdict() {
        let mut ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        assert_eq!(ctrl.tick(), Verdict::Stable);
    }

    #[test]
    fn controller_frequency_decay() {
        let mut ctrl: Controller<&str> = Controller::new("idle", 3.0, 5.0, 3, 0.80);
        assert_eq!(ctrl.tick(), Verdict::FrequencyDecay);
    }

    #[test]
    fn controller_fidelity_degraded() {
        let mut ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        ctrl.arrow.push("a", "b", 0.5);
        ctrl.arrow.push("b", "c", 0.5);
        assert_eq!(ctrl.tick(), Verdict::FidelityDegraded);
    }

    #[test]
    fn controller_compound_verdict() {
        let mut ctrl: Controller<&str> = Controller::new("idle", 3.0, 5.0, 1, 0.80);
        ctrl.arrow.push("a", "b", 0.3);
        assert!(ctrl.observe());
        match ctrl.tick() {
            Verdict::Compound(vs) => assert_eq!(vs.len(), 3),
            other => {
                let _ = other;
                assert!(false, "expected Compound");
            }
        }
    }

    #[test]
    fn controller_act_transitions() {
        let mut ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        ctrl.act("active", "user_input", "state_change", 0.95);
        assert_eq!(ctrl.sigma.value, "active");
        assert_eq!(ctrl.sigma.tick, 1);
        assert_eq!(ctrl.arrow.len(), 1);
    }

    #[test]
    fn hook_binding_degrade_restore() {
        let mut binding = HookBinding::new("exhale.sh", "brain-cli", "Stop");
        assert!((binding.fidelity - 1.0).abs() < f64::EPSILON);
        binding.degrade(0.8);
        assert!((binding.fidelity - 0.8).abs() < f64::EPSILON);
        binding.restore();
        assert!((binding.fidelity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn registry_aggregate_fidelity() {
        let ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        let mut reg = BindingRegistry::new(ctrl);
        let mut b1 = HookBinding::new("a.sh", "bin-a", "Start");
        let b2 = HookBinding::new("b.sh", "bin-b", "Stop");
        b1.degrade(0.9);
        reg.register(b1);
        reg.register(b2);
        assert!((reg.aggregate_fidelity() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn registry_degraded_bindings() {
        let ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        let mut reg = BindingRegistry::new(ctrl);
        let mut b1 = HookBinding::new("a.sh", "bin-a", "Start");
        b1.degrade(0.5);
        reg.register(b1);
        reg.register(HookBinding::new("b.sh", "bin-b", "Stop"));
        let degraded = reg.degraded_bindings(0.80);
        assert_eq!(degraded.len(), 1);
        assert_eq!(degraded[0].hook, "a.sh");
    }

    #[test]
    fn verdict_display() {
        assert_eq!(Verdict::Stable.to_string(), "STABLE");
        assert_eq!(Verdict::FrequencyDecay.to_string(), "FREQ_DECAY");
        let compound = Verdict::Compound(vec![Verdict::FrequencyDecay, Verdict::FidelityDegraded]);
        assert_eq!(
            compound.to_string(),
            "COMPOUND(FREQ_DECAY+FIDELITY_DEGRADED)"
        );
    }

    #[test]
    fn empty_arrow_f_total_is_zero() {
        let arrow = Arrow::new();
        assert!((arrow.f_total() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn nu_zero_floor() {
        let nu = Nu::new(5.0, 0.0);
        assert!(!nu.is_decayed());
        assert!(nu.health_ratio().is_infinite());
    }

    #[test]
    fn decay_all_degrades_above_floor() {
        let ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        let mut reg = BindingRegistry::new(ctrl);
        reg.register(HookBinding::new("a.sh", "bin-a", "Start"));
        reg.register(HookBinding::new("b.sh", "bin-b", "Stop"));
        let count = reg.decay_all(0.9, 0.5);
        assert_eq!(count, 2);
        assert!((reg.bindings[0].fidelity - 0.9).abs() < f64::EPSILON);
        assert!((reg.bindings[1].fidelity - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn decay_all_respects_floor() {
        let ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        let mut reg = BindingRegistry::new(ctrl);
        let mut b1 = HookBinding::new("a.sh", "bin-a", "Start");
        b1.degrade(0.3); // already below floor of 0.5
        reg.register(b1);
        reg.register(HookBinding::new("b.sh", "bin-b", "Stop"));
        let count = reg.decay_all(0.8, 0.5);
        assert_eq!(count, 1); // only b2 was above floor
        assert!((reg.bindings[0].fidelity - 0.3).abs() < f64::EPSILON); // unchanged
        assert!((reg.bindings[1].fidelity - 0.8).abs() < f64::EPSILON); // degraded
    }

    #[test]
    fn decay_all_empty_registry() {
        let ctrl: Controller<&str> = Controller::new("idle", 10.0, 5.0, 3, 0.80);
        let mut reg = BindingRegistry::new(ctrl);
        assert_eq!(reg.decay_all(0.9, 0.1), 0);
    }
}
