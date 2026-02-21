# Archived Crates — Leptos ON HOLD

These crates were archived on 2026-02-21 per SOP-DEV-003 Phase 1 triage.

**Reason:** All depend on Leptos (0.7 or 0.8), which is ON HOLD per NexVigilant policy.
Frontend development has migrated to Next.js 16 (Studio at `~/nexcore/studio/studio/`).

**Policy:** These crates are preserved for future reference. They are NOT deleted.
If Leptos development resumes, they can be migrated to `crates/` using SOP-DEV-003 Phase 2.

## Archived Crates

| Crate | Source | LOC | Category |
|-------|--------|-----|----------|
| `nucleus` | `wksp/apps/nucleus/` | 35,906 | Leptos PWA (production) |
| `adventure-hud` | `wksp/apps/adventure-hud/` | 954 | Leptos game HUD |
| `borrow-miner` | `wksp/apps/borrow-miner/` | 1,997 | Leptos mining game |
| `education-machine` | `wksp/apps/education-machine/` | 1,483 | Leptos educational |
| `ferro-clicker` | `wksp/apps/ferro-clicker/` | 366 | Leptos clicker |
| `ferro-explore` | `wksp/apps/ferro-explore/` | 185 | Leptos explorer |
| `wksp-api-client` | `wksp/crates/wksp-api-client/` | 781 | Leptos API client |
| `wksp-components` | `wksp/crates/wksp-components/` | 1,532 | Leptos component lib |
| `wksp-core` | `wksp/crates/wksp-core/` | 42 | Leptos core types |
| `integrity-calc` | `wksp/labs/integrity-calc/` | 1,655 | Leptos lab |
| `sos` | `wksp/sos/` | 1,855 | Leptos 0.8 CSR app |

**Total archived:** 44,756 LOC across 11 crates.

## Recovery

To restore any crate:
```bash
cp -a .archived/<name>/ ~/nexcore/crates/<name>/
# Then follow SOP-DEV-003 Phase 2 steps 2.3-2.13
```
