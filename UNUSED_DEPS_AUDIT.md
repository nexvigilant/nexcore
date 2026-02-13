# Unused Workspace Dependencies Audit

**Date:** 2026-02-06
**Method:** Grep-based analysis across 101 Cargo.toml files
**Finding:** 36 workspace deps have zero references from active workspace members

## Candidates for Removal

### Compression/Serialization (5)
- `zip = "2.1.3"` — no active consumer
- `zstd = "0.13.1"` — no active consumer
- `csv` — no active consumer
- `ron` — no active consumer
- `yaml-rust` — no active consumer

### HTTP/TLS (6)
- `reqwest-middleware = "0.3.2"` — planned but unused
- `reqwest-retry = "0.5.0"` — planned but unused
- `http-body` — no active consumer
- `webpki-roots` — no active consumer
- `native-tls` — no active consumer
- `openssl` — no active consumer

### Macro/Utility (8)
- `lazy_static = "1.5.0"` — prefer `std::sync::LazyLock` (Rust 1.80+)
- `paste = "1.0.15"` — no active consumer
- `async-recursion = "1.1.1"` — no active consumer
- `itertools = "0.13.0"` — no active consumer
- `log = "0.4.22"` — workspace uses tracing, not log
- `phf_codegen = "0.11.2"` — no active consumer
- `pin-project` — no active consumer
- `pin-project-lite` — no active consumer

### Type/URL (5)
- `num-traits = "0.2.19"` — no active consumer
- `smallvec = "1.13.2"` — only archived crates
- `percent-encoding` — no active consumer
- `form_urlencoded` — no active consumer
- `typenum` — no active consumer

### MIME (2)
- `mime` — no active consumer
- `mime_guess` — no active consumer

### Tower/Axum (3)
- `axum-extra` — no active consumer
- `tower-layer` — no active consumer
- `tower-service` — no active consumer

### Database (1)
- `sqlx` — no active consumer (duckdb is used instead)

### Versioning (2)
- `semver` — no active consumer
- `version_check` — no active consumer

### Proc Macro (2)
- `inventory` — no active consumer
- `linkme` — no active consumer

### Stream (1)
- `tokio-stream` — no active consumer

### Internal (1)
- `nexcore-foundation` — package alias for nexcore-vigilance, only in archived crates

## Action Plan

1. **Safe to remove immediately** (clearly unused): zip, zstd, csv, ron, yaml-rust, lazy_static, log, sqlx, mime, mime_guess, phf_codegen, version_check, typenum, nexcore-foundation
2. **Verify first** (may be transitive): http-body, pin-project, pin-project-lite, percent-encoding, form_urlencoded
3. **Keep for now** (planned features): reqwest-middleware, reqwest-retry, axum-extra, tower-layer, tower-service
4. **Replace with std** (deprecated patterns): lazy_static → std::sync::LazyLock, log → tracing

## Impact

Removing unused deps reduces:
- Cargo.lock size (fewer transitive deps)
- Clean build time
- Supply chain attack surface
- Cognitive overhead when reviewing Cargo.toml
