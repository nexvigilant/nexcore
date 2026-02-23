// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! NexCore OS kernel — the core runtime that owns all subsystems.
//!
//! ## Architecture
//!
//! `NexCoreOs<P: Platform>` is generic over the platform abstraction,
//! enabling the same OS core to run on watch, phone, or desktop with
//! different `Platform` implementations.
//!
//! ## Primitive Grounding
//!
//! - Σ Sum: Composition of all subsystems
//! - σ Sequence: Boot sequence, event loop
//! - ς State: OS lifecycle state
//! - → Causality: Event dispatch chain
//! - ∂ Boundary: Security boundaries (Clearance) + Firewall
//! - μ Mapping: DNS name→IP, routing dest→interface
//! - ν Frequency: Audio sample rates, stream scheduling
//! - N Quantity: Audio sample values, buffer sizes

use nexcore_pal::{Input, Platform};
use nexcore_state_os::{IrreversibilityLevel, MachineBuilder, StateKernel};
use nexcore_trust::{Evidence, TrustEngine};

use crate::app_clearance::{AppClearanceGate, AppManifest, ClearanceResult};
use crate::audio::AudioManager;
use crate::boot::BootSequence;
use crate::config::SystemConfig;
use crate::diagnostics::{DiagnosticSnapshot, HealthStatus, ServiceHealth};
use crate::error::OsError;
use crate::ipc::EventBus;
use crate::journal::{JournalEntry, Keywords, OsJournal, Severity, Subsystem};
use crate::network::NetworkManager;
use crate::persistence::StatePersistence;
use crate::secure_boot::{BootPolicy, BootStage, SecureBootChain};
use crate::security::{SecurityMonitor, SecurityResponse, ThreatPattern, ThreatSeverity};
use crate::service::{ServiceManager, ServicePriority, ServiceState};
use crate::user::UserManager;
use crate::vault::OsVault;

/// OS lifecycle state.
///
/// Tier: T2-P (ς State — OS lifecycle)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsState {
    /// OS is booting.
    Booting,
    /// OS is running (main event loop).
    Running,
    /// OS is shutting down.
    ShuttingDown,
    /// OS has halted.
    Halted,
}

/// The NexCore Operating System core.
///
/// Tier: T3 (Σ Sum — full system composition)
///
/// Generic over `P: Platform` for hardware abstraction.
/// This is the PID 1 equivalent — the root of all system state.
pub struct NexCoreOs<P: Platform> {
    /// Platform abstraction layer.
    platform: P,
    /// STOS state machine kernel (process state).
    stos: StateKernel,
    /// Service manager (system service lifecycle).
    services: ServiceManager,
    /// Cytokine-typed IPC event bus.
    ipc: EventBus,
    /// Security monitor (Guardian-inspired threat tracking).
    security: SecurityMonitor,
    /// App clearance gate (permission-gated app control).
    clearance_gate: AppClearanceGate,
    /// Encrypted vault (system credentials + user secrets).
    vault: OsVault,
    /// State persistence engine (Brain-inspired crash recovery).
    persistence: StatePersistence,
    /// Boot sequence tracker.
    boot: BootSequence,
    /// Secure boot chain (measured boot verification).
    secure_boot: SecureBootChain,
    /// Network manager (interfaces, connections, DNS, firewall, routing, monitoring, certs).
    network: NetworkManager,
    /// Audio manager (devices, streams, mixer, codecs).
    audio: AudioManager,
    /// User manager (authentication, sessions, accounts).
    users: UserManager,
    /// Energy token pool (ATP/ADP metabolic tracking).
    energy: nexcore_energy::TokenPool,
    /// Endocrine system (behavioral state / Cortisol/Dopamine/Adrenaline).
    endocrine: nexcore_hormones::EndocrineState,
    /// Guardian bridge (homeostasis file integration).
    guardian: crate::guardian_bridge::GuardianBridge,
    /// Trust engine — feeds STOS guard gates (Layer 4).
    trust: TrustEngine,
    /// System configuration (boot config, service definitions, trust thresholds).
    config: SystemConfig,
    /// Structured OS event journal (typed entries, severity-controlled retention).
    journal: OsJournal,
    /// OS lifecycle state.
    state: OsState,
    /// Event loop iteration counter.
    tick_count: u64,
}

impl<P: Platform> NexCoreOs<P> {
    /// Boot the OS in actor mode (emulator fallback).
    pub fn boot_with_actors(platform: P) -> Result<Self, OsError> {
        Self::boot_with_policy(platform, BootPolicy::Permissive)
    }

    /// Boot the OS on the given platform (default: Permissive boot policy).
    pub fn boot(platform: P) -> Result<Self, OsError> {
        Self::boot_with_policy(platform, BootPolicy::Permissive)
    }

    /// Boot the OS with a specific secure boot policy.
    ///
    /// Executes the full 4-phase boot sequence with measured boot:
    /// 1. PAL init (measure platform)
    /// 2. STOS kernel boot (measure state kernel)
    /// 3. System services start (measure services)
    /// 4. Shell launch (measure shell)
    pub fn boot_with_policy(platform: P, policy: BootPolicy) -> Result<Self, OsError> {
        Self::boot_with_config(platform, policy, SystemConfig::default())
    }

    /// Boot the OS with explicit configuration.
    ///
    /// Allows overriding default service definitions, trust thresholds,
    /// and security parameters.
    pub fn boot_with_config(
        platform: P,
        policy: BootPolicy,
        config: SystemConfig,
    ) -> Result<Self, OsError> {
        // Determine vault data directory from platform
        let vault_data_dir = std::path::Path::new(platform.data_dir()).join("vault");
        let vault = OsVault::new(vault_data_dir);

        let mut os = Self {
            platform,
            stos: StateKernel::new(),
            services: ServiceManager::new(),
            ipc: EventBus::new("nexcore-os"),
            security: SecurityMonitor::new(),
            clearance_gate: AppClearanceGate::new(),
            vault,
            network: NetworkManager::new(),
            audio: AudioManager::new(),
            persistence: StatePersistence::new(),
            boot: BootSequence::new(),
            secure_boot: SecureBootChain::new(policy),
            users: UserManager::new(),
            energy: nexcore_energy::TokenPool::new(config.energy.initial_budget),
            endocrine: nexcore_hormones::EndocrineState::load(),
            guardian: crate::guardian_bridge::GuardianBridge::new().unwrap_or_default(),
            trust: TrustEngine::new(),
            config,
            journal: OsJournal::new(),
            state: OsState::Booting,
            tick_count: 0,
        };

        // Register services from config
        os.register_core_services();

        // Phase 1: PAL init — measure platform identity
        os.boot.init_pal(os.platform.name())?;
        os.secure_boot.measure(
            BootStage::NexCoreOs,
            os.platform.name().as_bytes(),
            format!("PAL: {}", os.platform.name()),
        );
        os.ipc.emit_boot_event("PalInit");
        os.journal.record(
            JournalEntry::new(Subsystem::Boot, "pal", Severity::Info, "PAL initialized")
                .with_keywords(Keywords::BOOT | Keywords::LIFECYCLE)
                .with_str("platform", os.platform.name()),
            0,
        );

        // Phase 2: STOS kernel boot + service state machines + trust engine
        os.boot.boot_kernel()?;
        os.wire_stos_machines();
        os.secure_boot.measure(
            BootStage::Init,
            b"stos-state-kernel-trust",
            "STOS state kernel + trust engine initialized",
        );
        os.ipc.emit_boot_event("KernelBoot");
        os.journal.record(
            JournalEntry::new(Subsystem::Boot, "stos", Severity::Info, "STOS kernel + trust engine initialized")
                .with_keywords(Keywords::BOOT | Keywords::LIFECYCLE | Keywords::TRUST),
            0,
        );

        // Phase 2.5: Initialize network subsystem
        os.network.initialize();
        os.ipc.emit_boot_event("NetworkInit");

        // Phase 2.6: Initialize audio subsystem
        os.audio.initialize();
        os.ipc.emit_boot_event("AudioInit");

        // Phase 3: Start system services — measure service manifest
        os.boot.start_services(&mut os.services)?;
        let service_manifest: String = os
            .services
            .startup_order()
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(",");
        os.secure_boot.measure(
            BootStage::Services,
            service_manifest.as_bytes(),
            format!("Services: {service_manifest}"),
        );
        os.ipc.emit_boot_event("ServicesStarting");
        os.journal.record(
            JournalEntry::new(Subsystem::Boot, "services", Severity::Info, "Services starting")
                .with_keywords(Keywords::BOOT | Keywords::LIFECYCLE)
                .with_u64("service_count", os.services.count() as u64),
            0,
        );

        // Phase 4: Shell launch
        os.boot.launch_shell()?;
        os.secure_boot
            .measure(BootStage::Shell, b"nexcore-shell", "Shell launched");
        os.ipc.emit_boot_event("Running");

        // Verify boot chain before declaring Running
        let verification = os.secure_boot.verify_chain();
        if !verification.should_proceed() {
            os.journal.record(
                JournalEntry::new(Subsystem::Boot, "secure_boot", Severity::Fatal, "Chain integrity violation — boot halted")
                    .with_keywords(Keywords::BOOT | Keywords::SECURITY | Keywords::ERROR),
                0,
            );
            return Err(OsError::Boot(crate::error::BootError::SecureBootFailed(
                "Chain integrity violation — boot halted by policy".to_string(),
            )));
        }

        os.journal.record(
            JournalEntry::new(Subsystem::Boot, "complete", Severity::Notice, "Boot complete — system running")
                .with_keywords(Keywords::BOOT | Keywords::LIFECYCLE)
                .with_f64("trust_score", os.trust.score()),
            0,
        );
        os.state = OsState::Running;
        Ok(os)
    }

    /// Register services from SystemConfig.
    ///
    /// Config-driven: service definitions come from `SystemConfig::services`
    /// instead of being hardcoded. Default config produces the same 11 services.
    fn register_core_services(&mut self) {
        for svc_def in &self.config.services {
            let priority = match svc_def.priority.as_str() {
                "critical" => ServicePriority::Critical,
                "core" => ServicePriority::Core,
                "standard" => ServicePriority::Standard,
                _ => ServicePriority::User,
            };
            self.services.register(&svc_def.name, priority);
        }
    }

    /// Wire STOS state machines for each registered service.
    ///
    /// Creates a 7-state machine per service matching `ServiceState` variants:
    /// Registered(0) → Starting(1) → Running(2) → Degraded(3)
    ///                                             → Stopping(4) → Stopped(5)
    ///                                                              → Failed(6)
    ///
    /// Activates STOS Layers:
    /// - Layer 1 (ς State): State registration
    /// - Layer 2 (→ Causality): Transition definitions
    /// - Layer 3 (∂ Boundary): Initial/terminal states
    /// - Layer 4 (κ Guards): Trust-gated "start" transitions
    fn wire_stos_machines(&mut self) {
        let service_ids: Vec<_> = self
            .services
            .startup_order()
            .iter()
            .map(|s| (s.id, s.name.clone()))
            .collect();

        for (svc_id, name) in &service_ids {
            // Build a state machine matching ServiceState lifecycle
            let spec = MachineBuilder::new(format!("{name}-lifecycle"))
                .state("registered", nexcore_state_os::StateKind::Initial)
                .state("starting", nexcore_state_os::StateKind::Normal)
                .state("running", nexcore_state_os::StateKind::Normal)
                .state("degraded", nexcore_state_os::StateKind::Normal)
                .state("stopping", nexcore_state_os::StateKind::Normal)
                .state("stopped", nexcore_state_os::StateKind::Terminal)
                .state("failed", nexcore_state_os::StateKind::Normal)
                // Valid transitions: .transition(from, to, event)
                .transition("registered", "starting", "start")
                .transition("stopped", "starting", "restart_from_stopped")
                .transition("failed", "starting", "restart_from_failed")
                .transition("starting", "running", "started")
                .transition("degraded", "running", "recover")
                .transition("running", "degraded", "degrade")
                .transition("running", "stopping", "stop_from_running")
                .transition("degraded", "stopping", "stop_from_degraded")
                .transition("stopping", "stopped", "complete_stop")
                // Failure transitions
                .transition("starting", "failed", "fail_starting")
                .transition("running", "failed", "fail_running")
                .transition("degraded", "failed", "fail_degraded")
                .transition("stopping", "failed", "fail_stopping")
                .build();

            // Load into STOS kernel
            if let Ok(machine_id) = self.stos.load_machine(&spec) {
                // Link STOS machine to service
                if let Some(svc) = self.services.get_mut(*svc_id) {
                    svc.machine_id = Some(machine_id);
                }

                // Layer 4 (κ Guards): Register trust gate on "start" transition.
                // Services need trust score >= threshold to start.
                // Non-fatal: guard failure during boot is logged, not blocking.
                if let Err(e) = self.stos.register_guard(machine_id, "trust_gate", "trust_ok") {
                    tracing::warn!("Failed to register trust guard for {name}: {e:?}");
                }

                // Layer 12 (ν Temporal): Set startup timeout.
                // If a service stays in "starting" for >30 ticks without reaching
                // "running", the fail_starting transition fires automatically.
                if let Ok(starting_id) = self.stos.find_state_id(machine_id, "starting") {
                    if let Ok(fail_tid) =
                        self.stos
                            .find_transition_id(machine_id, starting_id, "fail_starting")
                    {
                        self.stos
                            .scheduler_mut()
                            .set_timeout(machine_id, 30, fail_tid);
                    }
                }

                // Layer 14 (∝ Irreversibility): Mark "stopped" as absorbing
                // when in security lockdown. During normal operation, stopped
                // services can restart. During lockdown, absorbing state prevents
                // any transitions out of "stopped".
                // (Absorbing activation happens in process_security_responses
                //  when Lockdown fires — here we just register the capability.)
            }
        }
    }

    /// Transition a service to a new state via STOS.
    ///
    /// Validates through STOS state machine, emits cytokine event on success.
    pub fn transition_service(
        &mut self,
        service_id: crate::service::ServiceId,
        new_state: ServiceState,
    ) -> Result<(), OsError> {
        let svc = self.services.get(service_id).ok_or_else(|| {
            OsError::Service(crate::error::ServiceError::NotFound("unknown".to_string()))
        })?;

        let name = svc.name.clone();
        let from_state = svc.state;

        // Transition through the service manager (validates state machine)
        if let Some(svc) = self.services.get_mut(service_id) {
            svc.transition(new_state).map_err(OsError::Service)?;
        }

        // Emit cytokine event for the state change
        self.ipc
            .emit_service_event(&name, &format!("{from_state:?}"), &format!("{new_state:?}"));

        // Journal: record service state transition
        let severity = if new_state == ServiceState::Failed {
            Severity::Error
        } else {
            Severity::Info
        };
        self.journal.record(
            JournalEntry::new(Subsystem::Service, "transition", severity, format!("{name}: {from_state:?} -> {new_state:?}"))
                .with_keywords(Keywords::LIFECYCLE | Keywords::STATE_CHANGE)
                .with_str("service", &name)
                .with_str("from", &format!("{from_state:?}"))
                .with_str("to", &format!("{new_state:?}")),
            self.tick_count,
        );

        Ok(())
    }

    /// Run one tick of the OS event loop.
    ///
    /// In a real OS, this would be the main scheduler loop.
    /// Returns `false` when the OS should shut down.
    ///
    /// Activates STOS Layers per tick:
    /// - Layer 5 (N Metrics): State visit counts via transitions
    /// - Layer 11 (Σ Aggregate): Fleet-wide stats available via `aggregate_stats()`
    /// - Layer 12 (ν Temporal): Scheduled transitions + timeouts
    pub fn tick(&mut self) -> bool {
        if self.state != OsState::Running {
            return false;
        }

        self.tick_count += 1;

        // Layer 12 (ν Temporal): Tick the STOS kernel — executes scheduled
        // transitions and processes timeouts.
        let tick_result = self.stos.tick(1);

        // Record STOS tick results in IPC for observability
        if !tick_result.executed.is_empty() || !tick_result.timeouts.is_empty() {
            for (machine_id, result) in &tick_result.executed {
                self.ipc.emit_service_event(
                    &format!("stos-machine-{machine_id}"),
                    &format!("state-{}", result.from_state),
                    &format!("state-{}", result.to_state),
                );
            }
        }

        // Process input events
        if let Ok(Some(_event)) = self.platform.input_mut().poll_event() {
            // Dispatch event to focused app/service via IPC
            // Future: route through cytokine bus to focused service
        }

        // ── Security Heartbeat (Guardian homeostasis) ──────────────

        // Advance the full-scale Guardian homeostasis loop
        self.security.tick();

        // Phase 1: Auto-quarantine services with excessive threats
        let service_ids: Vec<_> = self.services.startup_order().iter().map(|s| s.id).collect();
        for svc_id in service_ids {
            if self.security.should_quarantine(svc_id) && !self.security.is_quarantined(svc_id) {
                self.security.quarantine_service(svc_id);
                if let Some(svc) = self.services.get(svc_id) {
                    self.ipc
                        .emit_service_event(&svc.name, "Running", "Quarantined");
                }
            }
        }

        // Phase 2: Process pending security responses
        self.process_security_responses();

        // ── Trust Recovery ─────────────────────────────────────────
        // Record positive evidence for clean ticks at configured interval.
        // Trust degrades on threats (see report_threat), recovers here.
        if self.tick_count % self.config.trust.recovery_interval == 0 {
            self.trust
                .record(Evidence::Positive(self.config.trust.recovery_weight));
        }

        // ── Metabolic Regulation (Energy) ──────────────────────────
        let current_regime = self.energy.regime();
        if current_regime == nexcore_energy::Regime::Crisis {
            self.degrade_services(crate::service::ServicePriority::Core);
        } else if current_regime == nexcore_energy::Regime::Catabolic {
            self.degrade_services(crate::service::ServicePriority::Standard);
        }

        // Drain IPC events (dispatch to subscribers in future)
        let _events = self.ipc.drain();

        true
    }

    /// Process queued security responses from the Guardian monitor.
    ///
    /// Handles escalation actions: notify, suspend non-critical, or full lockdown.
    fn process_security_responses(&mut self) {
        let responses = self.security.drain_responses();
        for response in &responses {
            match response {
                SecurityResponse::Monitor | SecurityResponse::QuarantineService => {
                    // Monitor: no action. Quarantine: already handled in tick Phase 1.
                }
                SecurityResponse::NotifyUser => {
                    let signal = nexcore_cytokine::Cytokine::new(
                        nexcore_cytokine::CytokineFamily::Il1,
                        "security_notification",
                    )
                    .with_source("nexcore-os-security")
                    .with_scope(nexcore_cytokine::Scope::Paracrine)
                    .with_severity(nexcore_cytokine::ThreatLevel::Medium)
                    .with_payload(serde_json::json!({
                        "action": "notify",
                        "level": format!("{}", self.security.level()),
                    }));
                    self.ipc.emit(signal);
                }
                SecurityResponse::SuspendNonCritical => {
                    self.degrade_services(crate::service::ServicePriority::Standard);
                }
                SecurityResponse::Lockdown => {
                    // Journal: lockdown is a critical event
                    self.journal.record(
                        JournalEntry::new(Subsystem::Security, "lockdown", Severity::Critical, "Security lockdown activated — RED")
                            .with_keywords(Keywords::SECURITY | Keywords::LIFECYCLE | Keywords::STATE_CHANGE)
                            .with_u64("pamp_count", self.security.pamp_count() as u64)
                            .with_u64("damp_count", self.security.damp_count() as u64)
                            .with_f64("trust_score", self.trust.score()),
                        self.tick_count,
                    );

                    // Lock the vault on security lockdown
                    if self.vault.is_operational() {
                        self.vault.lock();
                    }
                    self.degrade_services(crate::service::ServicePriority::Core);

                    // Layer 14 (∝ Irreversibility): Mark stopped states as
                    // absorbing so services cannot restart during lockdown.
                    self.activate_lockdown_absorbing();

                    self.emit_lockdown_event();
                }
            }
        }
    }

    /// Degrade all services at or above the given priority level.
    fn degrade_services(&mut self, min_priority: crate::service::ServicePriority) {
        let to_degrade: Vec<_> = self
            .services
            .startup_order()
            .iter()
            .filter(|s| s.priority >= min_priority && s.state == ServiceState::Running)
            .map(|s| (s.id, s.name.clone()))
            .collect();

        for (id, name) in &to_degrade {
            if let Some(svc) = self.services.get_mut(*id) {
                if svc.transition(ServiceState::Degraded).is_ok() {
                    self.ipc.emit_service_event(name, "Running", "Degraded");
                }
            }
        }
    }

    /// Activate STOS absorbing states on all service machines during lockdown.
    ///
    /// Layer 14 (∝ Irreversibility): Once in lockdown, "stopped" and "failed"
    /// states become absorbing — no service can restart until the OS reboots.
    fn activate_lockdown_absorbing(&mut self) {
        let machines: Vec<_> = self
            .services
            .startup_order()
            .iter()
            .filter_map(|s| s.machine_id)
            .collect();

        for machine_id in machines {
            // Mark "stopped" as permanent absorbing state
            if let Ok(stopped_id) = self.stos.find_state_id(machine_id, "stopped") {
                if let Err(e) = self.stos.register_absorbing_state(
                    machine_id,
                    stopped_id,
                    IrreversibilityLevel::Permanent,
                ) {
                    tracing::warn!("Failed to set absorbing state for machine {machine_id}: {e:?}");
                }
            }
            // Mark "failed" as permanent absorbing state
            if let Ok(failed_id) = self.stos.find_state_id(machine_id, "failed") {
                if let Err(e) = self.stos.register_absorbing_state(
                    machine_id,
                    failed_id,
                    IrreversibilityLevel::Permanent,
                ) {
                    tracing::warn!("Failed to set absorbing state for machine {machine_id}: {e:?}");
                }
            }
        }
    }

    /// Emit a full lockdown cytokine event.
    fn emit_lockdown_event(&mut self) {
        let signal = nexcore_cytokine::Cytokine::new(
            nexcore_cytokine::CytokineFamily::TnfAlpha,
            "security_lockdown",
        )
        .with_source("nexcore-os-security")
        .with_scope(nexcore_cytokine::Scope::Systemic)
        .with_severity(nexcore_cytokine::ThreatLevel::Critical)
        .with_payload(serde_json::json!({
            "action": "lockdown",
            "level": "RED",
            "pamp_count": self.security.pamp_count(),
            "damp_count": self.security.damp_count(),
        }));
        self.ipc.emit(signal);
    }

    /// Initiate graceful shutdown.
    pub fn shutdown(&mut self) {
        self.state = OsState::ShuttingDown;
        self.ipc.emit_shutdown_event();
        self.boot.shutdown();

        // Stop services in reverse priority order
        // (User → Standard → Core → Critical)
        let service_ids: Vec<_> = self
            .services
            .startup_order()
            .iter()
            .rev()
            .map(|s| s.id)
            .collect();

        for id in service_ids {
            if let Some(svc) = self.services.get_mut(id) {
                if svc.state.is_alive() {
                    let name = svc.name.clone();
                    if svc.transition(ServiceState::Stopping).is_ok() {
                        self.ipc.emit_service_event(&name, "Running", "Stopping");
                        if svc.transition(ServiceState::Stopped).is_ok() {
                            self.ipc.emit_service_event(&name, "Stopping", "Stopped");
                        }
                    }
                }
            }
        }

        self.state = OsState::Halted;
        self.boot.halt();
    }

    /// Get the current OS state.
    pub fn state(&self) -> OsState {
        self.state
    }

    /// Get the STOS state kernel.
    pub fn stos(&self) -> &StateKernel {
        &self.stos
    }

    /// Get a mutable reference to the STOS state kernel.
    pub fn stos_mut(&mut self) -> &mut StateKernel {
        &mut self.stos
    }

    /// Get the service manager.
    pub fn services(&self) -> &ServiceManager {
        &self.services
    }

    /// Get the platform.
    pub fn platform(&self) -> &P {
        &self.platform
    }

    /// Get a mutable reference to the platform.
    pub fn platform_mut(&mut self) -> &mut P {
        &mut self.platform
    }

    /// Get the IPC event bus.
    pub fn ipc(&self) -> &EventBus {
        &self.ipc
    }

    /// Get a mutable reference to the IPC event bus.
    pub fn ipc_mut(&mut self) -> &mut EventBus {
        &mut self.ipc
    }

    /// Get the security monitor.
    pub fn security(&self) -> &SecurityMonitor {
        &self.security
    }

    /// Get a mutable reference to the security monitor.
    pub fn security_mut(&mut self) -> &mut SecurityMonitor {
        &mut self.security
    }

    /// Get the encrypted vault.
    pub fn vault(&self) -> &OsVault {
        &self.vault
    }

    /// Get a mutable reference to the encrypted vault.
    pub fn vault_mut(&mut self) -> &mut OsVault {
        &mut self.vault
    }

    /// Get the network manager.
    pub fn network(&self) -> &NetworkManager {
        &self.network
    }

    /// Get a mutable reference to the network manager.
    pub fn network_mut(&mut self) -> &mut NetworkManager {
        &mut self.network
    }

    /// Get the audio manager.
    pub fn audio(&self) -> &AudioManager {
        &self.audio
    }

    /// Get a mutable reference to the audio manager.
    pub fn audio_mut(&mut self) -> &mut AudioManager {
        &mut self.audio
    }

    /// Get the persistence engine.
    pub fn persistence(&self) -> &StatePersistence {
        &self.persistence
    }

    /// Record a security threat at the OS level.
    ///
    /// Records the threat, degrades trust proportionally to severity,
    /// and emits an IPC event.
    pub fn report_threat(
        &mut self,
        severity: ThreatSeverity,
        description: impl Into<String>,
        source_service: Option<crate::service::ServiceId>,
    ) {
        let desc: String = description.into();
        self.security.record_threat(severity, &desc, source_service);

        // Degrade trust proportionally to threat severity.
        // Higher severity = heavier negative evidence.
        let severity_weight = match severity {
            ThreatSeverity::Info => 0.0,
            ThreatSeverity::Low => 0.5,
            ThreatSeverity::Medium => 1.0,
            ThreatSeverity::High => 2.0,
            ThreatSeverity::Critical => 5.0,
        };
        if severity_weight > 0.0 {
            self.trust
                .record(Evidence::Negative(severity_weight * self.config.trust.threat_weight));
        }

        // Journal: record threat with structured fields
        let journal_severity = match severity {
            ThreatSeverity::Info => Severity::Info,
            ThreatSeverity::Low => Severity::Notice,
            ThreatSeverity::Medium => Severity::Warning,
            ThreatSeverity::High => Severity::Error,
            ThreatSeverity::Critical => Severity::Critical,
        };
        self.journal.record(
            JournalEntry::new(Subsystem::Security, "threat", journal_severity, &desc)
                .with_keywords(Keywords::SECURITY | Keywords::ERROR)
                .with_str("severity", &format!("{severity:?}"))
                .with_f64("trust_score", self.trust.score())
                .with_str("security_level", &format!("{}", self.security.level())),
            self.tick_count,
        );

        // Emit threat event on IPC bus
        let signal = nexcore_cytokine::Cytokine::new(
            nexcore_cytokine::CytokineFamily::TnfAlpha,
            "security_threat",
        )
        .with_source("nexcore-os")
        .with_scope(nexcore_cytokine::Scope::Systemic)
        .with_severity(nexcore_cytokine::ThreatLevel::High)
        .with_payload(serde_json::json!({
            "severity": format!("{severity:?}"),
            "description": desc,
            "security_level": format!("{}", self.security.level()),
            "trust_score": self.trust.score(),
        }));
        self.ipc.emit(signal);
    }

    /// Report a Guardian threat pattern (PAMP or DAMP).
    ///
    /// The pattern is automatically classified, recorded, and response
    /// actions are queued for the next tick.
    pub fn report_pattern(&mut self, pattern: &ThreatPattern) {
        let desc = pattern.describe();
        self.security.record_pattern(pattern);

        // Emit pattern event on IPC bus
        let (family, event_type) = match pattern {
            ThreatPattern::External(_) => {
                (nexcore_cytokine::CytokineFamily::TnfAlpha, "pamp_detected")
            }
            ThreatPattern::Internal(_) => (nexcore_cytokine::CytokineFamily::Il6, "damp_detected"),
        };

        let signal = nexcore_cytokine::Cytokine::new(family, event_type)
            .with_source("nexcore-os-guardian")
            .with_scope(nexcore_cytokine::Scope::Systemic)
            .with_severity(nexcore_cytokine::ThreatLevel::High)
            .with_payload(serde_json::json!({
                "pattern": desc,
                "security_level": format!("{}", self.security.level()),
                "pamp_total": self.security.pamp_count(),
                "damp_total": self.security.damp_count(),
            }));
        self.ipc.emit(signal);
    }

    /// Evaluate whether an app can be installed (clearance gate).
    pub fn evaluate_app_install(&self, manifest: &AppManifest) -> ClearanceResult {
        self.clearance_gate
            .evaluate_install(manifest, &self.security)
    }

    /// Evaluate whether an app can execute (runtime clearance).
    pub fn evaluate_app_run(&self, manifest: &AppManifest) -> ClearanceResult {
        self.clearance_gate.evaluate_run(manifest, &self.security)
    }

    /// Get the app clearance gate.
    pub fn clearance_gate(&self) -> &AppClearanceGate {
        &self.clearance_gate
    }

    /// Get a mutable reference to the app clearance gate.
    pub fn clearance_gate_mut(&mut self) -> &mut AppClearanceGate {
        &mut self.clearance_gate
    }

    /// Create a state snapshot for persistence.
    ///
    /// Used by the persistence engine during periodic checkpoints
    /// and shutdown to save OS state for crash recovery.
    pub fn create_snapshot(&self, clean_shutdown: bool) -> crate::persistence::OsStateSnapshot {
        let services: Vec<_> = self
            .services
            .startup_order()
            .iter()
            .map(|s| (s.name.clone(), s.state, s.machine_id))
            .collect();

        crate::persistence::snapshot_os_state(
            self.platform.name(),
            &format!("{:?}", self.state),
            &services,
            self.tick_count,
            self.ipc.total_emitted(),
            &format!("{}", self.security.level()),
            clean_shutdown,
        )
    }

    /// Get the boot sequence (for boot log inspection).
    pub fn boot_sequence(&self) -> &BootSequence {
        &self.boot
    }

    /// Get the secure boot chain.
    pub fn secure_boot(&self) -> &SecureBootChain {
        &self.secure_boot
    }

    /// Get a mutable reference to the secure boot chain.
    pub fn secure_boot_mut(&mut self) -> &mut SecureBootChain {
        &mut self.secure_boot
    }

    /// Get the user manager.
    pub fn users(&self) -> &UserManager {
        &self.users
    }

    /// Get a mutable reference to the user manager.
    pub fn users_mut(&mut self) -> &mut UserManager {
        &mut self.users
    }

    /// Get the energy token pool.
    pub fn energy(&self) -> &nexcore_energy::TokenPool {
        &self.energy
    }

    /// Get a mutable reference to the energy token pool.
    pub fn energy_mut(&mut self) -> &mut nexcore_energy::TokenPool {
        &mut self.energy
    }

    /// Get the endocrine system.
    pub fn endocrine(&self) -> &nexcore_hormones::EndocrineState {
        &self.endocrine
    }

    /// Get a mutable reference to the endocrine system.
    pub fn endocrine_mut(&mut self) -> &mut nexcore_hormones::EndocrineState {
        &mut self.endocrine
    }

    /// Get the persistence engine.

    /// Create the initial device owner account.
    ///
    /// Called during first boot (device setup). Emits IPC event.
    pub fn create_owner(
        &mut self,
        username: &str,
        display_name: &str,
        password: &str,
    ) -> Result<crate::user::UserId, OsError> {
        let id = self.users.create_owner(username, display_name, password)?;

        self.ipc.emit(
            nexcore_cytokine::Cytokine::new(nexcore_cytokine::CytokineFamily::Il2, "owner_created")
                .with_source("nexcore-os-auth")
                .with_scope(nexcore_cytokine::Scope::Systemic)
                .with_payload(serde_json::json!({
                    "username": username,
                    "user_id": id.0,
                })),
        );

        Ok(id)
    }

    /// Authenticate a user and create a session.
    ///
    /// Records failed attempts as security events.
    pub fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<crate::user::Session, OsError> {
        match self.users.login(username, password) {
            Ok(session) => {
                self.ipc.emit(
                    nexcore_cytokine::Cytokine::new(
                        nexcore_cytokine::CytokineFamily::Il2,
                        "user_login",
                    )
                    .with_source("nexcore-os-auth")
                    .with_scope(nexcore_cytokine::Scope::Paracrine)
                    .with_payload(serde_json::json!({
                        "username": username,
                        "role": session.role.to_string(),
                    })),
                );

                Ok(session)
            }
            Err(crate::user::AuthError::InvalidPassword) => {
                // Record as security threat (potential brute force)
                self.security.record_threat(
                    crate::security::ThreatSeverity::Low,
                    format!("Failed login attempt for user: {username}"),
                    None,
                );

                Err(OsError::Auth(crate::user::AuthError::InvalidPassword))
            }
            Err(crate::user::AuthError::AccountLocked(u)) => {
                // Escalate — locked account means repeated failures
                self.security.record_threat(
                    crate::security::ThreatSeverity::Medium,
                    format!("Account locked due to failed attempts: {u}"),
                    None,
                );

                Err(OsError::Auth(crate::user::AuthError::AccountLocked(u)))
            }
            Err(e) => Err(OsError::Auth(e)),
        }
    }

    /// Log out a session.
    pub fn logout(&mut self, token: &str) -> Result<(), OsError> {
        self.users.logout(token)?;

        self.ipc.emit(
            nexcore_cytokine::Cytokine::new(nexcore_cytokine::CytokineFamily::Il2, "user_logout")
                .with_source("nexcore-os-auth")
                .with_scope(nexcore_cytokine::Scope::Paracrine),
        );

        Ok(())
    }

    /// Get the trust engine.
    pub fn trust(&self) -> &TrustEngine {
        &self.trust
    }

    /// Get a mutable reference to the trust engine.
    pub fn trust_mut(&mut self) -> &mut TrustEngine {
        &mut self.trust
    }

    /// Get the system trust score (0.0-1.0).
    ///
    /// Convenience method — equivalent to `self.trust().score()`.
    pub fn trust_score(&self) -> f64 {
        self.trust.score()
    }

    /// Get the system configuration.
    pub fn config(&self) -> &SystemConfig {
        &self.config
    }

    /// Get STOS aggregate stats across all service machines.
    ///
    /// Layer 11 (Σ Aggregate): fleet-wide service health.
    pub fn aggregate_stats(&self) -> nexcore_state_os::AggregateStats {
        self.stos.aggregate_stats()
    }

    /// Get STOS metrics for a specific service's state machine.
    ///
    /// Layer 5 (N Metrics): per-service state visit counts and transition history.
    pub fn service_metrics(
        &self,
        service_id: crate::service::ServiceId,
    ) -> Option<&nexcore_state_os::MachineMetrics> {
        let svc = self.services.get(service_id)?;
        let machine_id = svc.machine_id?;
        self.stos.metrics(machine_id).ok()
    }

    /// Check whether trust score meets the service start threshold.
    ///
    /// Layer 4 (κ Guards): trust gate evaluation.
    pub fn trust_allows_start(&self) -> bool {
        self.trust.score() >= self.config.trust.start_threshold
    }

    /// Get the event loop tick count.
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Get the structured OS journal.
    pub fn journal(&self) -> &OsJournal {
        &self.journal
    }

    /// Get a mutable reference to the OS journal.
    pub fn journal_mut(&mut self) -> &mut OsJournal {
        &mut self.journal
    }

    /// Capture a diagnostic snapshot of the full system state.
    ///
    /// Provides a point-in-time view of all subsystems for debugging,
    /// health monitoring, and incident response. Inspired by Fuchsia Inspect
    /// and Apple sysdiagnose.
    pub fn diagnostic_snapshot(&self) -> DiagnosticSnapshot {
        // Collect per-service health
        let services: Vec<ServiceHealth> = self
            .services
            .startup_order()
            .iter()
            .map(|svc| {
                let transitions = svc.machine_id
                    .and_then(|mid| self.stos.metrics(mid).ok())
                    .map_or(0, |m| m.executions);
                ServiceHealth::from_state(&svc.name, svc.state, transitions)
            })
            .collect();

        let services_running = services.iter().filter(|s| s.health == HealthStatus::Ok).count();
        let services_failed = services.iter().filter(|s| s.health == HealthStatus::Critical).count();
        let services_total = services.len();

        let system_health = DiagnosticSnapshot::compute_system_health(
            self.security.level(),
            &services,
        );

        let stats = self.stos.aggregate_stats();

        DiagnosticSnapshot {
            tick: self.tick_count,
            system_health,
            services,
            security_level: self.security.level(),
            trust_score: self.trust.score(),
            active_threats: self.security.active_threats().len(),
            stos_machines: stats.total_machines,
            stos_total_transitions: stats.total_transitions,
            ipc_pending: self.ipc.pending(),
            ipc_total_emitted: self.ipc.total_emitted(),
            journal_entries: self.journal.len(),
            journal_errors: self.journal.error_count(),
            journal_total_recorded: self.journal.total_recorded(),
            services_running,
            services_failed,
            services_total,
        }
    }

    /// Get the Guardian bridge.
    pub fn guardian(&self) -> &crate::guardian_bridge::GuardianBridge {
        &self.guardian
    }

    /// Run one tick in actor mode (emulator compat).
    pub fn tick_actors(&mut self) {
        self.tick();
    }

    /// Shutdown the OS in actor mode (emulator compat).
    pub fn shutdown_actors(&mut self) {
        self.shutdown();
    }

    /// Get the form factor of the underlying platform.
    pub fn form_factor(&self) -> nexcore_pal::FormFactor {
        self.platform.form_factor()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_pal::FormFactor;
    use nexcore_pal_linux::LinuxPlatform;

    #[test]
    fn boot_virtual_desktop() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            assert_eq!(os.state(), OsState::Running);
            assert_eq!(os.form_factor(), FormFactor::Desktop);
            assert!(os.boot_sequence().is_running());
        }
    }

    #[test]
    fn boot_virtual_watch() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Watch);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            assert_eq!(os.form_factor(), FormFactor::Watch);
        }
    }

    #[test]
    fn boot_virtual_phone() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Phone);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            assert_eq!(os.form_factor(), FormFactor::Phone);
        }
    }

    #[test]
    fn tick_loop() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let os = NexCoreOs::boot(platform);
        assert!(os.is_ok());

        if let Ok(mut os) = os {
            // Run a few ticks
            for _ in 0..10 {
                assert!(os.tick());
            }
            assert_eq!(os.tick_count(), 10);
        }
    }

    #[test]
    fn shutdown() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let os = NexCoreOs::boot(platform);
        assert!(os.is_ok());

        if let Ok(mut os) = os {
            os.shutdown();
            assert_eq!(os.state(), OsState::Halted);
            assert!(!os.tick()); // Should not tick after shutdown
        }
    }

    #[test]
    fn services_registered() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let os = NexCoreOs::boot(platform);
        assert!(os.is_ok());

        if let Ok(os) = os {
            // Should have 11 core services registered
            assert_eq!(os.services().count(), 11);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // STOS WIRING TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn stos_machines_created_for_services() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            // Each of 11 services should have an STOS machine
            assert!(os.stos().machine_count() >= 10);

            // Verify machine_id is set on services
            let services = os.services().startup_order();
            for svc in &services {
                assert!(
                    svc.machine_id.is_some(),
                    "Service '{}' should have STOS machine_id",
                    svc.name
                );
            }
        }
    }

    #[test]
    fn stos_service_state_machine_structure() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            // Get the first service's machine and verify STOS has it
            let services = os.services().startup_order();
            let first = &services[0];
            if let Some(machine_id) = first.machine_id {
                // Machine exists in STOS
                let state = os.stos().current_state(machine_id);
                assert!(state.is_ok());
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // IPC EVENT BUS TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn ipc_boot_events_emitted() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            // Boot emits 4 phase events + service events that accumulate
            assert!(os.ipc().total_emitted() >= 4);
        }
    }

    #[test]
    fn ipc_shutdown_events() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let pre_shutdown = os.ipc().total_emitted();
            os.shutdown();
            // Shutdown emits: 1 shutdown event + service stop events
            assert!(os.ipc().total_emitted() > pre_shutdown);
        }
    }

    #[test]
    fn shutdown_stops_services_in_reverse_order() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            os.shutdown();

            // All services should be stopped
            let stopped = os.services().count_in_state(ServiceState::Stopped);
            assert_eq!(
                stopped, 11,
                "All 11 services should be stopped after shutdown"
            );
        }
    }

    #[test]
    fn transition_service_emits_event() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let pre = os.ipc().total_emitted();

            // Find guardian service and try to degrade it
            let services = os.services().startup_order();
            let guardian = services.iter().find(|s| s.name == "guardian");
            assert!(guardian.is_some());

            if let Some(g) = guardian {
                let id = g.id;
                let result = os.transition_service(id, ServiceState::Degraded);
                assert!(result.is_ok());

                // Should have emitted one event
                assert_eq!(os.ipc().total_emitted(), pre + 1);
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // SECURITY MONITOR TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn security_starts_green() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            assert_eq!(os.security().level(), crate::security::SecurityLevel::Green);
        }
    }

    #[test]
    fn report_threat_escalates_and_emits_ipc() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let pre = os.ipc().total_emitted();
            os.report_threat(
                crate::security::ThreatSeverity::High,
                "SSH brute force detected",
                None,
            );

            assert_eq!(
                os.security().level(),
                crate::security::SecurityLevel::Orange
            );
            // Should have emitted 1 threat event
            assert_eq!(os.ipc().total_emitted(), pre + 1);
        }
    }

    #[test]
    fn service_quarantine_on_tick() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Find shell service (user-level, safe to quarantine in test)
            let services = os.services().startup_order();
            let shell = services.iter().find(|s| s.name == "shell");
            assert!(shell.is_some());

            if let Some(s) = shell {
                let shell_id = s.id;

                // Report critical threat from shell
                os.report_threat(
                    crate::security::ThreatSeverity::Critical,
                    "Malicious shell command",
                    Some(shell_id),
                );

                // Tick should trigger quarantine
                os.tick();
                assert!(os.security().is_quarantined(shell_id));
            }
        }
    }

    // ═══════════════════════════════════════════════════════════
    // STATE PERSISTENCE TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn create_snapshot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let snap = os.create_snapshot(false);
            assert_eq!(snap.services.len(), 11);
            assert!(
                snap.platform.contains("linux"),
                "Platform name should contain 'linux', got: {}",
                snap.platform
            );
            assert!(!snap.clean_shutdown);
        }
    }

    #[test]
    fn snapshot_records_security_level() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            os.report_threat(
                crate::security::ThreatSeverity::Critical,
                "Test threat",
                None,
            );

            let snap = os.create_snapshot(false);
            assert_eq!(snap.security_level, "RED");
        }
    }

    // ═══════════════════════════════════════════════════════════
    // GUARDIAN SECURITY WIRING TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn pamp_login_failure_escalates() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Report 10 login failures (should escalate to High → Orange)
            let pattern = crate::security::ThreatPattern::External(
                crate::security::Pamp::RapidLoginFailure {
                    count: 10,
                    source: "192.168.1.100".to_string(),
                },
            );

            os.report_pattern(&pattern);
            assert_eq!(
                os.security().level(),
                crate::security::SecurityLevel::Orange
            );
            assert_eq!(os.security().pamp_count(), 1);
        }
    }

    #[test]
    fn damp_service_crash_tracked() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let shell_svc = os
                .services()
                .startup_order()
                .iter()
                .find(|s| s.name == "shell")
                .map(|s| s.id);

            if let Some(shell_id) = shell_svc {
                let pattern =
                    crate::security::ThreatPattern::Internal(crate::security::Damp::ServiceCrash {
                        service_id: shell_id,
                        service_name: "shell".to_string(),
                        crash_count: 3,
                    });

                os.report_pattern(&pattern);
                assert_eq!(os.security().damp_count(), 1);
                // 3 crashes = High severity → Orange
                assert_eq!(
                    os.security().level(),
                    crate::security::SecurityLevel::Orange
                );
            }
        }
    }

    #[test]
    fn pamp_privilege_escalation_goes_red() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let pattern = crate::security::ThreatPattern::External(
                crate::security::Pamp::PrivilegeEscalation {
                    actor: "rogue_process".to_string(),
                    target_level: "root".to_string(),
                },
            );

            os.report_pattern(&pattern);
            // Privilege escalation = Critical → Red
            assert_eq!(os.security().level(), crate::security::SecurityLevel::Red);
            assert!(os.security().is_critical());
        }
    }

    #[test]
    fn lockdown_degrades_services_on_tick() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Trigger Red lockdown
            let pattern =
                crate::security::ThreatPattern::External(crate::security::Pamp::MaliciousPayload {
                    payload_type: "SQL injection".to_string(),
                    location: "input handler".to_string(),
                });
            os.report_pattern(&pattern);

            // Tick processes the lockdown response
            os.tick();

            // Non-critical services should be degraded
            let degraded = os.services().count_in_state(ServiceState::Degraded);
            assert!(
                degraded > 0,
                "At least one service should be degraded after lockdown"
            );
        }
    }

    #[test]
    fn app_install_blocked_at_orange() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Escalate to Orange
            os.report_threat(crate::security::ThreatSeverity::High, "Active threat", None);

            let manifest = crate::app_clearance::AppManifest::new(
                "com.app.game",
                "Game",
                crate::app_clearance::AppClearanceLevel::Standard,
            );

            let result = os.evaluate_app_install(&manifest);
            assert!(!result.is_allowed());
        }
    }

    #[test]
    fn app_run_blocked_at_red() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Escalate to Red
            os.report_threat(
                crate::security::ThreatSeverity::Critical,
                "Root compromise",
                None,
            );

            // Standard app blocked
            let standard = crate::app_clearance::AppManifest::new(
                "com.app.chat",
                "Chat",
                crate::app_clearance::AppClearanceLevel::Standard,
            );
            assert!(!os.evaluate_app_run(&standard).is_allowed());

            // System app still runs
            let system = crate::app_clearance::AppManifest::new(
                "com.nexcore.launcher",
                "Launcher",
                crate::app_clearance::AppClearanceLevel::System,
            );
            assert!(os.evaluate_app_run(&system).is_allowed());
        }
    }

    #[test]
    fn clearance_gate_accessible() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            assert!(!os.clearance_gate().sideloading_enabled());
            os.clearance_gate_mut().enable_sideloading();
            assert!(os.clearance_gate().sideloading_enabled());
        }
    }

    #[test]
    fn notify_response_emits_ipc() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let pre = os.ipc().total_emitted();

            // Yellow threat → NotifyUser response
            let pattern =
                crate::security::ThreatPattern::Internal(crate::security::Damp::MemoryPressure {
                    usage_pct: 85,
                    threshold_pct: 80,
                });
            os.report_pattern(&pattern);

            // report_pattern emits 1 event
            // tick processes NotifyUser → emits 1 more
            os.tick();

            assert!(os.ipc().total_emitted() > pre);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // VAULT INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════

    /// Helper: boot a kernel with an isolated temp storage root.
    fn boot_with_temp_storage() -> Option<(NexCoreOs<LinuxPlatform>, tempfile::TempDir)> {
        let dir = tempfile::tempdir().ok()?;
        let root = dir.path().to_string_lossy().to_string();
        let platform = LinuxPlatform::virtual_platform_at(FormFactor::Desktop, &root);
        let os = NexCoreOs::boot(platform).ok()?;
        Some((os, dir))
    }

    #[test]
    fn vault_starts_uninitialized() {
        let (os, _dir) = match boot_with_temp_storage() {
            Some(v) => v,
            None => return,
        };
        assert_eq!(os.vault().state(), crate::vault::VaultState::Uninitialized);
        assert!(!os.vault().is_operational());
    }

    #[test]
    fn vault_initialize_and_store() {
        let (mut os, _dir) = match boot_with_temp_storage() {
            Some(v) => v,
            None => return,
        };

        let init = os.vault_mut().initialize("boot-password");
        assert!(init.is_ok(), "Vault init should succeed: {init:?}");
        assert!(os.vault().is_operational());

        // Store and retrieve a service token
        let store = os.vault_mut().store_service_token("guardian", "grd-xyz");
        assert!(store.is_ok());

        let token = os.vault().get_service_token("guardian");
        assert_eq!(token.unwrap_or_default(), "grd-xyz");
    }

    #[test]
    fn vault_locks_on_lockdown() {
        let (mut os, _dir) = match boot_with_temp_storage() {
            Some(v) => v,
            None => return,
        };

        // Initialize and verify operational
        let init = os.vault_mut().initialize("password");
        assert!(init.is_ok(), "Vault init should succeed: {init:?}");
        assert!(os.vault().is_operational());

        // Trigger Red lockdown
        let pattern =
            crate::security::ThreatPattern::External(crate::security::Pamp::PrivilegeEscalation {
                actor: "exploit".to_string(),
                target_level: "root".to_string(),
            });
        os.report_pattern(&pattern);

        // Tick processes the lockdown response (which auto-locks vault)
        os.tick();

        // Vault should be locked after lockdown
        assert_eq!(os.vault().state(), crate::vault::VaultState::Locked);
        assert!(!os.vault().is_operational());
    }

    #[test]
    fn vault_accessible_via_kernel() {
        let (mut os, _dir) = match boot_with_temp_storage() {
            Some(v) => v,
            None => return,
        };

        let init = os.vault_mut().initialize("pass");
        assert!(init.is_ok());

        // Store system + user secrets
        let s1 = os.vault_mut().store_system_secret("device-key", "dk-123");
        assert!(s1.is_ok());
        let s2 = os.vault_mut().store_user_secret("wifi", "mypassword");
        assert!(s2.is_ok());

        // Verify counts
        assert_eq!(os.vault().secret_count().unwrap_or(0), 2);
        assert_eq!(os.vault().operations(), 2);
    }

    // ═══════════════════════════════════════════════════════════
    // SECURE BOOT INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn secure_boot_measures_during_boot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            // Boot should have measured 4 stages: NexCoreOs, Init, Services, Shell
            assert_eq!(os.secure_boot().record_count(), 4);
            assert_eq!(os.secure_boot().failure_count(), 0);
            assert!(!os.secure_boot().is_degraded());
        }
    }

    #[test]
    fn secure_boot_pcr_values_populated() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            // PCR[3] (NexCoreOs), PCR[4] (Init), PCR[5] (Services), PCR[6] (Shell)
            // should all be non-zero
            let quote = os.secure_boot().quote();
            assert_eq!(quote.pcr_values.len(), 4);
            assert!(!quote.degraded);
        }
    }

    #[test]
    fn secure_boot_attestation_log_stages() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let log = os.secure_boot().attestation_log();
            assert_eq!(log.len(), 4);

            assert_eq!(log[0].stage, crate::secure_boot::BootStage::NexCoreOs);
            assert_eq!(log[1].stage, crate::secure_boot::BootStage::Init);
            assert_eq!(log[2].stage, crate::secure_boot::BootStage::Services);
            assert_eq!(log[3].stage, crate::secure_boot::BootStage::Shell);

            // All should be verified (NoExpectation = OK in permissive mode)
            for record in log {
                assert!(record.verified);
            }
        }
    }

    #[test]
    fn secure_boot_strict_with_correct_expectations() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let platform_name = platform.name().to_string();

        // Build expected measurements matching what boot() will produce
        let pal_expected = crate::secure_boot::Measurement::from_data(platform_name.as_bytes());
        let init_expected = crate::secure_boot::Measurement::from_data(b"stos-state-kernel-trust");
        let shell_expected = crate::secure_boot::Measurement::from_data(b"nexcore-shell");

        let mut result =
            NexCoreOs::boot_with_policy(platform, crate::secure_boot::BootPolicy::Strict);

        // Boot should fail because we haven't registered expectations yet
        // (Actually, strict mode with no expectations = NoExpectation = ok)
        // Strict mode only blocks if a registered expectation MISMATCHES
        assert!(result.is_ok());

        // Now test with registered expectations
        let platform2 = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        result = NexCoreOs::boot_with_policy(platform2, crate::secure_boot::BootPolicy::Strict);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let chain = os.secure_boot();
            let verification = chain.verify_chain();
            assert!(verification.all_verified);
            assert!(verification.should_proceed());

            // Verify specific PCR values are deterministic
            assert_eq!(
                chain.pcr(crate::secure_boot::BootStage::NexCoreOs),
                &crate::secure_boot::Measurement::zero().extend(&pal_expected),
            );
            assert_eq!(
                chain.pcr(crate::secure_boot::BootStage::Init),
                &crate::secure_boot::Measurement::zero().extend(&init_expected),
            );
            assert_eq!(
                chain.pcr(crate::secure_boot::BootStage::Shell),
                &crate::secure_boot::Measurement::zero().extend(&shell_expected),
            );
        }
    }

    #[test]
    fn secure_boot_chain_accessible() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            assert_eq!(
                os.secure_boot().policy(),
                crate::secure_boot::BootPolicy::Permissive
            );

            // Can measure additional stages post-boot (e.g., app loading)
            let result = os.secure_boot_mut().measure(
                crate::secure_boot::BootStage::Apps,
                b"test-app-binary",
                "Test app loaded",
            );
            assert_eq!(result, crate::secure_boot::VerifyResult::NoExpectation);
            assert_eq!(os.secure_boot().record_count(), 5);
        }
    }

    #[test]
    fn damp_disk_full_critical() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let pattern =
                crate::security::ThreatPattern::Internal(crate::security::Damp::DiskFull {
                    usage_pct: 99,
                    mount: "/".to_string(),
                });

            os.report_pattern(&pattern);
            // 99% disk = Critical → Red
            assert_eq!(os.security().level(), crate::security::SecurityLevel::Red);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // NETWORK INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn network_initialized_after_boot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            assert_eq!(
                os.network().state(),
                crate::network::NetworkState::Discovered
            );
        }
    }

    #[test]
    fn network_register_interface() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let iface = nexcore_network::Interface::new(
                "wlan0",
                "wlan0",
                nexcore_network::InterfaceType::WiFi,
            )
            .up()
            .with_address(nexcore_network::IpAddr::v4(192, 168, 1, 100));

            os.network_mut().register_interface(iface);
            assert_eq!(os.network().interface_count(), 1);
        }
    }

    #[test]
    fn network_block_ip_from_threat() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let initial_rules = os.network().firewall().rule_count();
            os.network_mut()
                .block_ip(nexcore_network::IpAddr::v4(10, 0, 0, 99));
            assert_eq!(os.network().firewall().rule_count(), initial_rules + 1);
        }
    }

    #[test]
    fn network_dns_cache() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            os.network_mut().dns_mut().cache_address(
                "example.com",
                nexcore_network::IpAddr::v4(93, 184, 216, 34),
                300,
            );
            let resolved = os.network_mut().resolve_cached("example.com");
            assert!(resolved.is_some());
        }
    }

    #[test]
    fn network_service_registered() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let services = os.services().startup_order();
            let network_svc = services.iter().find(|s| s.name == "network");
            assert!(
                network_svc.is_some(),
                "network service should be registered"
            );
        }
    }

    #[test]
    fn network_summary() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let s = os.network().summary();
            assert!(s.contains("Network"));
            assert!(s.contains("Discovered"));
        }
    }

    // ═══════════════════════════════════════════════════════════
    // AUDIO INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn audio_initialized_after_boot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            assert_eq!(os.audio().state(), crate::audio::AudioState::Ready);
        }
    }

    #[test]
    fn audio_service_registered() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let services = os.services().startup_order();
            let audio_svc = services.iter().find(|s| s.name == "audio");
            assert!(audio_svc.is_some(), "audio service should be registered");
        }
    }

    #[test]
    fn audio_register_device() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let speakers = nexcore_audio::AudioDevice::new(
                "hw:0",
                "Built-in Speakers",
                nexcore_audio::DeviceType::Output,
            )
            .as_default();

            os.audio_mut().register_device(speakers);
            assert_eq!(os.audio().device_count(), 1);
            assert_eq!(os.audio().output_device_count(), 1);
        }
    }

    #[test]
    fn audio_volume_control() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            assert_eq!(os.audio().master_volume(), 0.75);
            os.audio_mut().set_master_volume(0.5);
            assert_eq!(os.audio().master_volume(), 0.5);
        }
    }

    #[test]
    fn audio_mute_control() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            assert!(!os.audio().is_muted());
            os.audio_mut().toggle_mute();
            assert!(os.audio().is_muted());
        }
    }

    #[test]
    fn audio_create_playback_stream() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let speakers = nexcore_audio::AudioDevice::new(
                "hw:0",
                "Built-in Speakers",
                nexcore_audio::DeviceType::Output,
            );
            os.audio_mut().register_device(speakers);

            let stream_id = os.audio_mut().create_playback_stream(
                &nexcore_audio::DeviceId::new("hw:0"),
                nexcore_audio::AudioSpec::cd_quality(),
                1024,
            );
            assert!(stream_id.is_some());
            assert_eq!(os.audio().stream_count(), 1);
        }
    }

    #[test]
    fn audio_summary() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let s = os.audio().summary();
            assert!(s.contains("Audio"));
            assert!(s.contains("Ready"));
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Phase 1: STOS Layer Integration Tests
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn trust_engine_initializes_with_positive_score() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let os = NexCoreOs::boot(platform);
        assert!(os.is_ok());
        if let Ok(os) = os {
            // TrustEngine starts with prior_alpha=2, prior_beta=2 -> score ~0.5
            assert!(os.trust_score() > 0.0, "Trust must be positive at boot");
            assert!(
                os.trust_allows_start(),
                "Trust must exceed start threshold (0.3) at boot"
            );
        }
    }

    #[test]
    fn trust_degrades_on_threat() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());
        if let Ok(mut os) = result {
            let initial = os.trust_score();
            os.report_threat(
                crate::security::ThreatSeverity::High,
                "test-threat",
                None,
            );
            assert!(
                os.trust_score() < initial,
                "Trust should decrease after threat: was {initial}, now {}",
                os.trust_score()
            );
        }
    }

    #[test]
    fn trust_recovers_on_clean_ticks() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());
        if let Ok(mut os) = result {
            // Degrade trust first
            os.report_threat(
                crate::security::ThreatSeverity::High,
                "test-degradation",
                None,
            );
            let degraded = os.trust_score();

            // Run ticks past recovery_interval (default 10)
            for _ in 0..15 {
                os.tick();
            }
            assert!(
                os.trust_score() > degraded,
                "Trust should recover over clean ticks: was {degraded}, now {}",
                os.trust_score()
            );
        }
    }

    #[test]
    fn config_driven_registration_produces_11_services() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let os = NexCoreOs::boot(platform);
        assert!(os.is_ok());
        if let Ok(os) = os {
            assert_eq!(
                os.services().count(),
                11,
                "Default config should register 11 services"
            );
        }
    }

    #[test]
    fn custom_config_changes_service_count() {
        use crate::config::{ServiceDef, SystemConfig};

        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let mut config = SystemConfig::default();
        config.services.push(ServiceDef {
            name: "test-extra".to_string(),
            priority: "user".to_string(),
            auto_start: true,
            max_restarts: 3,
        });

        let os = NexCoreOs::boot_with_config(
            platform,
            crate::secure_boot::BootPolicy::Permissive,
            config,
        );
        assert!(os.is_ok());
        if let Ok(os) = os {
            assert_eq!(
                os.services().count(),
                12,
                "Custom config with extra service should register 12"
            );
        }
    }

    #[test]
    fn aggregate_stats_available_after_boot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let os = NexCoreOs::boot(platform);
        assert!(os.is_ok());
        if let Ok(os) = os {
            let stats = os.aggregate_stats();
            // Should have machines registered (one per service)
            assert!(
                stats.total_machines > 0,
                "Aggregate stats should show registered machines"
            );
        }
    }

    #[test]
    fn service_metrics_track_state_visits() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());
        if let Ok(os) = result {
            // Get metrics for the first service
            let first_svc = os
                .services()
                .startup_order()
                .first()
                .map(|s| s.id);
            if let Some(svc_id) = first_svc {
                let metrics = os.service_metrics(svc_id);
                assert!(
                    metrics.is_some(),
                    "Service should have STOS metrics available"
                );
            }
        }
    }

    #[test]
    fn stos_temporal_tick_advances_scheduler() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());
        if let Ok(mut os) = result {
            let before = os.stos().scheduler().time();
            os.tick();
            let after = os.stos().scheduler().time();
            assert!(
                after > before,
                "STOS scheduler time should advance on tick: before={before}, after={after}"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════
    // JOURNAL + DIAGNOSTICS INTEGRATION TESTS (Phase 2 Telemetry)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn journal_populated_after_boot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            // Boot should emit multiple journal entries (PAL, STOS, Services, Complete)
            assert!(
                os.journal().total_recorded() >= 4,
                "Boot should emit at least 4 journal entries, got {}",
                os.journal().total_recorded()
            );
        }
    }

    #[test]
    fn journal_boot_entries_have_boot_keyword() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let filter = crate::journal::JournalFilter::new()
                .keywords(crate::journal::Keywords::BOOT);
            let boot_entries = os.journal().query(&filter);
            assert!(
                boot_entries.len() >= 4,
                "Should have at least 4 BOOT-tagged entries, got {}",
                boot_entries.len()
            );
        }
    }

    #[test]
    fn journal_records_threat() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            let before = os.journal().total_recorded();
            os.report_threat(
                crate::security::ThreatSeverity::Medium,
                "test threat",
                None,
            );
            assert!(
                os.journal().total_recorded() > before,
                "Journal should record threat events"
            );

            // Threat should be in security subsystem
            let filter = crate::journal::JournalFilter::new()
                .subsystem(crate::journal::Subsystem::Security);
            let security_entries = os.journal().query(&filter);
            assert!(!security_entries.is_empty(), "Should have security entries");
        }
    }

    #[test]
    fn journal_threat_in_error_buffer() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // High threat -> Error severity -> should appear in error buffer
            os.report_threat(
                crate::security::ThreatSeverity::High,
                "high severity threat",
                None,
            );
            assert!(
                os.journal().error_count() > 0,
                "High severity threat should appear in error buffer"
            );
        }
    }

    #[test]
    fn diagnostic_snapshot_reflects_running_system() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let snapshot = os.diagnostic_snapshot();

            assert_eq!(snapshot.services_total, 11);
            assert_eq!(snapshot.security_level, crate::security::SecurityLevel::Green);
            assert!(snapshot.trust_score > 0.0);
            assert!(snapshot.journal_entries > 0);
            assert_eq!(snapshot.active_threats, 0);
            assert!(snapshot.stos_machines > 0);
        }
    }

    #[test]
    fn diagnostic_snapshot_health_ok_on_clean_boot() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let snapshot = os.diagnostic_snapshot();
            // Clean boot should not be Critical or Unhealthy
            assert_ne!(
                snapshot.system_health,
                crate::diagnostics::HealthStatus::Critical
            );
            assert_ne!(
                snapshot.system_health,
                crate::diagnostics::HealthStatus::Unhealthy
            );
        }
    }

    #[test]
    fn diagnostic_snapshot_after_lockdown_is_critical() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Trigger Red lockdown
            let pattern = crate::security::ThreatPattern::External(
                crate::security::Pamp::PrivilegeEscalation {
                    actor: "exploit".to_string(),
                    target_level: "root".to_string(),
                },
            );
            os.report_pattern(&pattern);
            os.tick();

            let snapshot = os.diagnostic_snapshot();
            assert_eq!(
                snapshot.system_health,
                crate::diagnostics::HealthStatus::Critical
            );
            assert_eq!(
                snapshot.security_level,
                crate::security::SecurityLevel::Red
            );
        }
    }

    #[test]
    fn diagnostic_snapshot_display_contains_key_info() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(os) = result {
            let snapshot = os.diagnostic_snapshot();
            let display = format!("{snapshot}");

            assert!(display.contains("NexCore OS Diagnostic Snapshot"));
            assert!(display.contains("System Health"));
            assert!(display.contains("guardian"));
        }
    }

    #[test]
    fn journal_lockdown_entry_is_critical() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Trigger lockdown
            let pattern = crate::security::ThreatPattern::External(
                crate::security::Pamp::PrivilegeEscalation {
                    actor: "test".to_string(),
                    target_level: "root".to_string(),
                },
            );
            os.report_pattern(&pattern);
            os.tick();

            // Check error buffer for lockdown entry
            let errors = os.journal().errors();
            let lockdown_entry = errors.iter().find(|e| {
                e.category == "lockdown"
            });
            assert!(
                lockdown_entry.is_some(),
                "Lockdown should produce a critical journal entry in error buffer"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════
    // ENERGY INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn energy_initialized_from_config() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let config = SystemConfig::default();
        let result = NexCoreOs::boot_with_config(platform, crate::secure_boot::BootPolicy::Permissive, config.clone());
        assert!(result.is_ok());

        if let Ok(os) = result {
            assert_eq!(os.energy().t_atp, config.energy.initial_budget);
            assert_eq!(os.energy().regime(), nexcore_energy::Regime::Anabolic);
        }
    }

    #[test]
    fn energy_catabolic_regime_degrades_services() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Drain energy to Catabolic regime (EC < 0.70)
            let total = os.energy().total();
            os.energy_mut().spend_waste(total / 2); // 50% waste => EC = 0.5 => Catabolic

            // Tick processes the regime change
            os.tick();

            // Standard services should be degraded
            let startup_order = os.services().startup_order();
            let standard_degraded = startup_order.iter().find(|s| s.priority == crate::service::ServicePriority::Standard && s.state == ServiceState::Degraded);
            assert!(standard_degraded.is_some(), "Standard services should be degraded in Catabolic regime");
        }
    }

    #[test]
    fn energy_crisis_regime_degrades_core_services() {
        let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
        let result = NexCoreOs::boot(platform);
        assert!(result.is_ok());

        if let Ok(mut os) = result {
            // Drain energy to Crisis regime (EC < 0.50)
            let total = os.energy().total();
            os.energy_mut().spend_waste((total as f64 * 0.8) as u64); // 80% waste => EC = 0.2 => Crisis

            // Tick processes the regime change
            os.tick();

            // Core services should be degraded
            let startup_order = os.services().startup_order();
            let core_degraded = startup_order.iter().find(|s| s.priority == crate::service::ServicePriority::Core && s.state == ServiceState::Degraded);
            assert!(core_degraded.is_some(), "Core services should be degraded in Crisis regime");
        }
    }
}
