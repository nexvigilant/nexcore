# nexcore-os

The core runtime and init system for NexCore OS. This crate provides the foundational infrastructure for booting the system, managing services, handling IPC events via cytokine-typed bus, and enforcing security boundaries using a Guardian-inspired monitor.

## Intent
To provide a secure, state-machine based operating system kernel. It abstracts hardware through the Platform Abstraction Layer (PAL) and uses the STOS (State Operating System) kernel to manage service lifecycles and transitions with high reliability and observability.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **σ (Sequence)**: Manages the 4-phase boot sequence and the main event loop.
- **ς (State)**: Represents the OS lifecycle state (Booting, Running, Halted) and service states.
- **→ (Causality)**: Orchestrates the event-driven IPC bus and causal responses to security threats.
- **∂ (Boundary)**: Enforces security clearances, app isolation, and firewall rules.
- **Σ (Sum)**: Represents the full composition of all system subsystems (Audio, Network, Vault, etc.).

## Core Subsystems
- **Boot**: Measured boot sequence with secure boot chain verification.
- **Service Manager**: Lifecycle coordination using STOS state machines.
- **Security Monitor**: Real-time threat tracking (PAMP/DAMP) and automated response.
- **Vault**: Encrypted storage for system credentials and user secrets.
- **IPC Bus**: Cytokine-typed event signaling for inter-service communication.
- **Network/Audio**: Comprehensive management of system-level I/O.

## SOPs for Use
### Booting the OS
```rust
use nexcore_os::kernel::NexCoreOs;
use nexcore_pal_linux::LinuxPlatform;

let platform = LinuxPlatform::virtual_platform(FormFactor::Desktop);
let mut os = NexCoreOs::boot(platform)?;
```

### Reporting a Security Threat
```rust
os.report_threat(ThreatSeverity::High, "Malicious activity detected", Some(service_id));
```

### Managing Services
```rust
os.transition_service(service_id, ServiceState::Running)?;
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
