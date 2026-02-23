// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! System configuration for NexCore OS boot and runtime.
//!
//! ## Primitive Grounding
//!
//! - π Persistence: Configuration survives reboots
//! - μ Mapping: Config key→value mappings
//! - σ Sequence: Service startup ordering via priority
//! - ∂ Boundary: Security thresholds define operational boundaries

/// Energy subsystem configuration.
///
/// Tier: T2-C (N Quantity — energy budgets)
#[derive(Debug, Clone)]
pub struct EnergyConfig {
    /// Initial token budget.
    pub initial_budget: u64,
}

impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            initial_budget: 1_000_000,
        }
    }
}

/// System configuration loaded from defaults (or future TOML).
///
/// Tier: T2-C (π Persistence + μ Mapping — persistent config mapping)
#[derive(Debug, Clone)]
pub struct SystemConfig {
    /// System hostname.
    pub hostname: String,
    /// Service definitions (determines what gets registered at boot).
    pub services: Vec<ServiceDef>,
    /// Security subsystem configuration.
    pub security: SecurityConfig,
    /// Trust engine configuration.
    pub trust: TrustOsConfig,
    /// Energy subsystem configuration.
    pub energy: EnergyConfig,
}

/// A service definition for boot-time registration.
///
/// Tier: T2-P (ς State — service identity)
#[derive(Debug, Clone)]
pub struct ServiceDef {
    /// Service name (must be unique).
    pub name: String,
    /// Priority level: "critical", "core", "standard", "user".
    pub priority: String,
    /// Whether to auto-start during boot.
    pub auto_start: bool,
    /// Maximum restart attempts before marking as failed.
    pub max_restarts: u32,
}

/// Security subsystem configuration.
///
/// Tier: T2-C (∂ Boundary + ν Frequency — security thresholds)
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// How many active threats before auto-escalation.
    pub escalation_threshold: usize,
    /// Whether to lock vault on Red lockdown.
    pub lock_vault_on_lockdown: bool,
}

/// Trust engine configuration for OS-level trust gating.
///
/// Tier: T2-C (κ Comparison + ∂ Boundary — trust thresholds)
#[derive(Debug, Clone)]
pub struct TrustOsConfig {
    /// Trust score needed to start/restart a service (0.0-1.0).
    pub start_threshold: f64,
    /// Weight of negative evidence from threats (higher = trust degrades faster).
    pub threat_weight: f64,
    /// Weight of positive evidence from clean ticks.
    pub recovery_weight: f64,
    /// How often (in ticks) to record positive evidence for clean operation.
    pub recovery_interval: u64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            hostname: "nexcore".to_string(),
            services: default_services(),
            security: SecurityConfig::default(),
            trust: TrustOsConfig::default(),
            energy: EnergyConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            escalation_threshold: 5,
            lock_vault_on_lockdown: true,
        }
    }
}

impl Default for TrustOsConfig {
    fn default() -> Self {
        Self {
            start_threshold: 0.3,
            threat_weight: 1.0,
            recovery_weight: 0.5,
            recovery_interval: 10,
        }
    }
}

/// Produce the default set of 11 services matching current hardcoded registration.
fn default_services() -> Vec<ServiceDef> {
    vec![
        // Critical (boot first)
        ServiceDef {
            name: "stos-runtime".to_string(),
            priority: "critical".to_string(),
            auto_start: true,
            max_restarts: 3,
        },
        ServiceDef {
            name: "clearance".to_string(),
            priority: "critical".to_string(),
            auto_start: true,
            max_restarts: 3,
        },
        ServiceDef {
            name: "vault".to_string(),
            priority: "critical".to_string(),
            auto_start: true,
            max_restarts: 3,
        },
        ServiceDef {
            name: "user-auth".to_string(),
            priority: "critical".to_string(),
            auto_start: true,
            max_restarts: 3,
        },
        // Core (boot second)
        ServiceDef {
            name: "guardian".to_string(),
            priority: "core".to_string(),
            auto_start: true,
            max_restarts: 5,
        },
        ServiceDef {
            name: "network".to_string(),
            priority: "core".to_string(),
            auto_start: true,
            max_restarts: 5,
        },
        ServiceDef {
            name: "audio".to_string(),
            priority: "core".to_string(),
            auto_start: true,
            max_restarts: 5,
        },
        ServiceDef {
            name: "energy".to_string(),
            priority: "core".to_string(),
            auto_start: true,
            max_restarts: 5,
        },
        // Standard (boot third)
        ServiceDef {
            name: "brain".to_string(),
            priority: "standard".to_string(),
            auto_start: true,
            max_restarts: 5,
        },
        ServiceDef {
            name: "cytokine-bus".to_string(),
            priority: "standard".to_string(),
            auto_start: true,
            max_restarts: 5,
        },
        // User (boot last)
        ServiceDef {
            name: "shell".to_string(),
            priority: "user".to_string(),
            auto_start: true,
            max_restarts: 10,
        },
    ]
}

/// Hill-curve-based restart backoff.
///
/// As failure count increases, backoff time approaches `max_backoff_ms`
/// asymptotically. Models homeostasis: proportional response, not linear.
///
/// ```text
/// response = max * count^n / (k^n + count^n)
/// ```
///
/// - n=2: steepness (sigmoidal)
/// - k=3: half-max at 3 failures
///
/// Tier: T2-C (N Quantity + → Causality — failure count → backoff duration)
#[allow(
    clippy::cast_precision_loss, // max_backoff_ms fits comfortably in f64 mantissa for practical values
    clippy::cast_sign_loss       // ratio is always in [0,1], product is always non-negative
)]
pub fn hill_curve_backoff_ms(failure_count: u32, max_backoff_ms: u64) -> u64 {
    if failure_count == 0 {
        return 0;
    }
    let n = 2.0_f64;
    let k = 3.0_f64;
    let count = f64::from(failure_count);
    let ratio = count.powf(n) / (k.powf(n) + count.powf(n));
    (max_backoff_ms as f64 * ratio) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_11_services() {
        let config = SystemConfig::default();
        assert_eq!(config.services.len(), 11);
    }

    #[test]
    fn default_config_service_names_match_current() {
        let config = SystemConfig::default();
        let names: Vec<&str> = config.services.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"stos-runtime"));
        assert!(names.contains(&"clearance"));
        assert!(names.contains(&"vault"));
        assert!(names.contains(&"user-auth"));
        assert!(names.contains(&"guardian"));
        assert!(names.contains(&"network"));
        assert!(names.contains(&"audio"));
        assert!(names.contains(&"energy"));
        assert!(names.contains(&"brain"));
        assert!(names.contains(&"cytokine-bus"));
        assert!(names.contains(&"shell"));
    }

    #[test]
    fn default_config_priority_ordering() {
        let config = SystemConfig::default();
        let critical: Vec<_> = config
            .services
            .iter()
            .filter(|s| s.priority == "critical")
            .collect();
        assert_eq!(critical.len(), 4);
    }

    #[test]
    fn hill_curve_backoff_zero_failures() {
        assert_eq!(hill_curve_backoff_ms(0, 60_000), 0);
    }

    #[test]
    fn hill_curve_backoff_increases_monotonically() {
        let max = 60_000_u64;
        let mut prev = 0;
        for i in 1..=10 {
            let backoff = hill_curve_backoff_ms(i, max);
            assert!(
                backoff >= prev,
                "Backoff should increase: f={i} got {backoff} < prev {prev}"
            );
            prev = backoff;
        }
    }

    #[test]
    fn hill_curve_backoff_half_max_at_three() {
        let max = 60_000_u64;
        let at_three = hill_curve_backoff_ms(3, max);
        // Hill curve: at k=3, response = max * 9/(9+9) = max/2
        assert_eq!(at_three, 30_000);
    }

    #[test]
    fn hill_curve_backoff_approaches_max() {
        let max = 60_000_u64;
        let at_ten = hill_curve_backoff_ms(10, max);
        // At 10 failures: 100/(9+100) ≈ 0.917 → ~55,000ms
        assert!(at_ten > 50_000, "Should approach max: got {at_ten}");
        assert!(at_ten <= max, "Should not exceed max: got {at_ten}");
    }

    #[test]
    fn trust_config_defaults() {
        let config = TrustOsConfig::default();
        assert!((config.start_threshold - 0.3).abs() < f64::EPSILON);
        assert!(config.recovery_interval == 10);
    }
}
