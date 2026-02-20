# AI Guidance — nexcore-api

HTTP gateway and multi-tenant interface for the NexCore kernel.

## Use When
- Developing web interfaces or external integrations for NexVigilant.
- Implementing new RESTful endpoints for PV or Skill operations.
- Managing multi-tenant isolation or security classification for API calls.

## Grounding Patterns
- **Addressing (λ)**: All routes must follow the `/v1/<domain>/<action>` pattern.
- **Mapping (μ)**: Use the `mcp_bridge` to re-use existing MCP tool logic within API handlers where possible.
- **T1 Primitives**:
  - `λ + μ`: Root primitives for routing.
  - `∂ + →`: Root primitives for middleware and security.

## Maintenance SOPs
- **Security**: Never add a public route without a corresponding `vr-tenant` check or `clearance` requirement.
- **Documentation**: All new routes MUST have `utoipa` macros. Failures to document will block the `crate_xray` audit.
- **Validation**: Run the `axum` compatibility test after modifying middleware.

## Key Entry Points
- `src/lib.rs`: The main router and OpenAPI configuration.
- `src/routes/`: Domain-specific request handlers.
- `src/mcp_bridge.rs`: In-process bridge to the MCP server logic.
