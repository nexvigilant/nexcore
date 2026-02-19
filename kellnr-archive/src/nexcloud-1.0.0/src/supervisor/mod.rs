pub mod registry;

use crate::error::{NexCloudError, Result};
use crate::events::{CloudEvent, EventBus};
use crate::manifest::{CloudManifest, RestartPolicyKind};
use crate::process::{HealthChecker, HealthStatus, ProcessTask, RestartPolicy};
use crate::proxy::ReverseProxy;
use crate::proxy::router::RoutingTable;
use crate::proxy::tls::load_tls_config;
use registry::{ProcessState, ServiceRegistry};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub use registry::ServiceRegistry as CloudServiceRegistry;

/// The cloud supervisor: orchestrates processes, proxy, and health monitoring.
///
/// Tier: T3 (σ Sequence + ς State + μ Mapping + ∂ Boundary + ρ Recursion + ν Frequency + π Persistence)
/// Full domain orchestrator — the top-level type for the entire platform.
pub struct CloudSupervisor {
    manifest: CloudManifest,
    registry: Arc<ServiceRegistry>,
    event_bus: EventBus,
    processes: HashMap<String, ProcessTask>,
    restart_policies: HashMap<String, RestartPolicy>,
    health_checker: HealthChecker,
    log_dir: PathBuf,
}

impl CloudSupervisor {
    /// Create a new supervisor from a manifest.
    pub fn new(manifest: CloudManifest) -> Result<Self> {
        let log_dir = manifest.platform.log_dir.clone();

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&log_dir)?;

        let registry = Arc::new(ServiceRegistry::new());
        let event_bus = EventBus::new(512);

        // Register all services
        for svc in &manifest.services {
            registry.register(svc.name.clone(), svc.port);
        }

        // Build restart policies
        let restart_policies = manifest
            .services
            .iter()
            .map(|svc| {
                (
                    svc.name.clone(),
                    RestartPolicy::new(svc.max_restarts, svc.backoff_ms),
                )
            })
            .collect();

        // Build process tasks
        let processes = manifest
            .services
            .iter()
            .map(|svc| {
                (
                    svc.name.clone(),
                    ProcessTask::from_service_def(svc, &log_dir),
                )
            })
            .collect();

        Ok(Self {
            manifest,
            registry,
            event_bus,
            processes,
            restart_policies,
            health_checker: HealthChecker::default(),
            log_dir,
        })
    }

    /// Start all services in dependency order, then start the reverse proxy.
    ///
    /// Constitutional: Separation of powers — services (Executive), proxy (Legislative routing),
    /// health checks (Judicial review) all start independently.
    pub async fn start(&mut self) -> Result<()> {
        let order = self.manifest.topo_sort()?;
        let service_count = order.len();

        tracing::info!(
            platform = %self.manifest.platform.name,
            services = service_count,
            "starting platform"
        );

        // Spawn services in dependency order
        for name in &order {
            self.start_service(name).await?;
        }

        self.event_bus.emit(CloudEvent::PlatformStarted {
            name: self.manifest.platform.name.clone(),
            services: service_count,
            at: chrono::Utc::now(),
        });

        // Start health monitoring loop
        let registry = Arc::clone(&self.registry);
        let event_bus = self.event_bus.clone();
        let health_checker = HealthChecker::default();
        let manifest_services = self.manifest.services.clone();

        tokio::spawn(async move {
            health_monitor_loop(registry, event_bus, health_checker, &manifest_services).await;
        });

        // Build routing table
        let routing_table = RoutingTable::from_routes(&self.manifest.routes, |backend| {
            self.manifest.service_by_name(backend).map(|s| s.port)
        });

        // Build proxy with or without TLS (CAP-029 Homeland Security — boundary enforcement)
        let platform_name = self.manifest.platform.name.clone();
        let proxy = if let Some(ref tls_def) = self.manifest.proxy.tls {
            let tls_config = load_tls_config(tls_def)?;
            Arc::new(
                ReverseProxy::with_tls(
                    routing_table,
                    self.event_bus.clone(),
                    tls_config,
                    self.manifest.proxy.https_port,
                )
                .with_registry(Arc::clone(&self.registry), platform_name),
            )
        } else {
            Arc::new(
                ReverseProxy::new(routing_table, self.event_bus.clone())
                    .with_registry(Arc::clone(&self.registry), platform_name),
            )
        };

        let http_addr: SocketAddr = ([0, 0, 0, 0], self.manifest.proxy.http_port).into();

        // If TLS configured, spawn HTTPS listener on a background task and keep HTTP
        // for redirect. Otherwise, HTTP listener serves all traffic.
        if proxy.has_tls() {
            let https_addr: SocketAddr = ([0, 0, 0, 0], self.manifest.proxy.https_port).into();
            let https_proxy = Arc::clone(&proxy);

            // Spawn HTTPS listener in background
            tokio::spawn(async move {
                if let Err(e) = https_proxy.serve_https(https_addr).await {
                    tracing::error!("HTTPS listener failed: {e}");
                }
            });

            tracing::info!(
                http_port = self.manifest.proxy.http_port,
                https_port = self.manifest.proxy.https_port,
                "proxy started with TLS (HTTP redirects to HTTPS)"
            );
        }

        // HTTP listener runs forever (redirects if TLS, proxies if not)
        proxy.serve_http(http_addr).await?;

        Ok(())
    }

    /// Start a single service by name.
    async fn start_service(&mut self, name: &str) -> Result<()> {
        let process =
            self.processes
                .get_mut(name)
                .ok_or_else(|| NexCloudError::ServiceNotFound {
                    name: name.to_string(),
                })?;

        self.registry.update_state(name, ProcessState::Starting);

        let pid = process.spawn()?;
        self.registry.mark_started(name, pid);

        self.event_bus.emit(CloudEvent::ServiceStarted {
            name: name.to_string(),
            pid,
            at: chrono::Utc::now(),
        });

        // Wait briefly for the service to initialize
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Initial health check
        if let Some(svc) = self.manifest.service_by_name(name) {
            let status = self.health_checker.check(svc.port, &svc.health).await;
            match status {
                HealthStatus::Healthy => {
                    self.registry.record_healthy(name);
                    self.event_bus.emit(CloudEvent::ServiceHealthy {
                        name: name.to_string(),
                        at: chrono::Utc::now(),
                    });
                    tracing::info!(service = %name, pid = pid, "service healthy");
                }
                _ => {
                    // Not healthy yet, but that's OK — the health loop will handle it
                    tracing::debug!(service = %name, "initial health check pending");
                }
            }
        }

        Ok(())
    }

    /// Stop all services in reverse dependency order.
    pub async fn stop(&mut self) -> Result<()> {
        self.event_bus.emit(CloudEvent::PlatformStopping {
            at: chrono::Utc::now(),
        });

        let mut order = self.manifest.topo_sort()?;
        order.reverse(); // reverse order for shutdown

        tracing::info!("stopping {} services", order.len());

        for name in &order {
            self.stop_service(name).await?;
        }

        tracing::info!("all services stopped");
        Ok(())
    }

    /// Stop a single service.
    async fn stop_service(&mut self, name: &str) -> Result<()> {
        self.registry.update_state(name, ProcessState::Stopping);

        if let Some(process) = self.processes.get_mut(name) {
            process.stop(Duration::from_secs(10)).await?;
        }

        self.registry.update_state(name, ProcessState::Stopped);
        self.registry.update_pid(name, None);

        self.event_bus.emit(CloudEvent::ServiceStopped {
            name: name.to_string(),
            at: chrono::Utc::now(),
        });

        Ok(())
    }

    /// Stop a single service by name (public API for CLI restart).
    pub async fn stop_service_by_name(&mut self, name: &str) -> Result<()> {
        self.stop_service(name).await
    }

    /// Start a single service by name (public API for CLI restart).
    pub async fn start_service_by_name(&mut self, name: &str) -> Result<()> {
        self.start_service(name).await
    }

    /// Get a snapshot of all service states.
    pub fn status(&self) -> Vec<registry::ServiceRecord> {
        self.registry.snapshot()
    }

    /// Get the event bus for subscribing to events.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Get the service registry.
    pub fn registry(&self) -> &Arc<ServiceRegistry> {
        &self.registry
    }
}

/// Background loop that monitors service health and triggers restarts.
async fn health_monitor_loop(
    registry: Arc<ServiceRegistry>,
    event_bus: EventBus,
    checker: HealthChecker,
    services: &[crate::manifest::ServiceDef],
) {
    let check_interval = Duration::from_secs(10);
    let mut restart_policies: HashMap<String, RestartPolicy> = services
        .iter()
        .map(|s| {
            (
                s.name.clone(),
                RestartPolicy::new(s.max_restarts, s.backoff_ms),
            )
        })
        .collect();

    loop {
        tokio::time::sleep(check_interval).await;

        for svc in services {
            let record = match registry.get(&svc.name) {
                Some(r) => r,
                None => continue,
            };

            // Only check services that should be running
            match record.state {
                ProcessState::Healthy | ProcessState::Starting | ProcessState::Unhealthy => {}
                _ => continue,
            }

            let status = checker.check(svc.port, &svc.health).await;

            match status {
                HealthStatus::Healthy => {
                    if record.state != ProcessState::Healthy {
                        registry.record_healthy(&svc.name);
                        event_bus.emit(CloudEvent::ServiceHealthy {
                            name: svc.name.clone(),
                            at: chrono::Utc::now(),
                        });
                    }
                    // Reset restart policy after sustained health
                    if let Some(policy) = restart_policies.get_mut(&svc.name) {
                        policy.reset();
                    }
                }
                HealthStatus::Unhealthy(reason) | HealthStatus::Unreachable(reason) => {
                    registry.update_state(&svc.name, ProcessState::Unhealthy);
                    event_bus.emit(CloudEvent::HealthCheckFailed {
                        name: svc.name.clone(),
                        reason: reason.clone(),
                        at: chrono::Utc::now(),
                    });

                    tracing::warn!(
                        service = %svc.name,
                        reason = %reason,
                        "health check failed"
                    );

                    // Check restart policy
                    if svc.restart == RestartPolicyKind::Never {
                        continue;
                    }

                    if let Some(policy) = restart_policies.get_mut(&svc.name) {
                        if policy.should_retry() {
                            let backoff = policy.next_backoff();
                            let attempt = policy.attempts();

                            registry.update_state(&svc.name, ProcessState::Restarting);
                            registry.increment_restarts(&svc.name);

                            event_bus.emit(CloudEvent::RestartScheduled {
                                name: svc.name.clone(),
                                attempt,
                                backoff,
                                at: chrono::Utc::now(),
                            });

                            tracing::info!(
                                service = %svc.name,
                                attempt = attempt,
                                backoff_ms = backoff.as_millis() as u64,
                                "restart scheduled"
                            );

                            // Note: actual restart would re-spawn the process here.
                            // In Phase 1, we just track state. Full restart loop comes
                            // when the supervisor owns the process handles directly.
                        } else {
                            registry.update_state(&svc.name, ProcessState::Failed);
                            tracing::error!(
                                service = %svc.name,
                                "max restarts exceeded — service marked as FAILED"
                            );
                        }
                    }
                }
            }
        }
    }
}
