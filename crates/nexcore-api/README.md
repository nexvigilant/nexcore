# nexcore-api

High-performance REST gateway for the NexVigilant Core kernel. Built on Axum and Tower, it provides a secure, multi-tenant interface for pharmacovigilance, skill execution, and orchestrator control.

## Intent
To expose the full power of NexCore's Rust APIs to external systems, webhooks, and mobile clients via a standardized HTTP/OpenAPI interface.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **λ (Address)**: Defines the REST routing table and endpoint addresses.
- **μ (Mapping)**: Maps HTTP requests to internal library functions.
- **→ (Causality)**: Manages the request-response lifecycle and middleware cascade.
- **∂ (Boundary)**: Enforces multi-tenant isolation and security classification (5-level clearance).

## Core Features
- **Axum Framework**: Async-first routing with high concurrency.
- **OpenAPI/Scalar**: Automatic documentation generation (available at `/docs`).
- **Multi-Tenancy**: Built-in `vr-tenant` and `vr-core` integration.
- **MCP Bridge**: In-process dispatch to MCP tools for unified execution.
- **Audit Logging**: Comprehensive tracking of all kernel-altering requests.

## Accessing Documentation
Once running, the interactive documentation is available at:
- **Scalar UI**: `http://localhost:3030/docs`
- **OpenAPI Spec**: `http://localhost:3030/openapi.json`

## SOPs for Use
### Starting the Server
```bash
cargo build -p nexcore-api --release
./target/release/nexcore-api
```

### Adding a new Route
1. Define the handler in `src/routes/<domain>.rs`.
2. Add the route to the router in `src/lib.rs`.
3. Decorate with `#[utoipa::path]` for automatic documentation.
4. Ensure appropriate `SecurityRequirement` is applied for the tenant context.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
