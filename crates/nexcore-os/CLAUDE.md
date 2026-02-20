# AI Guidance — nexcore-os

OS core runtime, service manager, and boot coordinator.

## Use When
- Implementing new system-level services or drivers.
- Extending the secure boot or attestation logic.
- Adding new security threat patterns (PAMP/DAMP) for the monitor.
- Interfacing with the encrypted vault or user authentication system.

## Grounding Patterns
- **Boot Sequence (σ)**: All boot stages MUST be measured in the `SecureBootChain` to maintain integrity.
- **Service Transitions (ς)**: Never update `ServiceState` directly; always use `transition_service()` to ensure STOS validation and IPC emission.
- **T1 Primitives**:
  - `σ + ς`: Root primitives for the sequential lifecycle of the system.
  - `∂ + Σ`: Root primitives for security boundaries and subsystem composition.

## Maintenance SOPs
- **Security Escalation**: Responses like `Lockdown` are irreversible within a single session. Ensure critical threats are verified before escalating to Red.
- **Vault Safety**: The `vault` should be initialized as early as possible after boot to enable dependent services.
- **IPC Reliability**: Cytokine signals on the `EventBus` are fire-and-forget. For critical coordination, use STOS state transitions.

## Key Entry Points
- `src/kernel.rs`: The main `NexCoreOs` struct and event loop.
- `src/boot.rs`: The 4-phase boot sequence implementation.
- `src/security.rs`: Threat monitor and response logic.
- `src/service.rs`: Service registry and priority definitions.
