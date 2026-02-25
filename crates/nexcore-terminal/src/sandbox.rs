//! Process sandbox — Linux namespace isolation for terminal sessions.
//!
//! Uses `clone(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWNET | CLONE_NEWUSER)`
//! with seccomp BPF and cgroup v2 limits. No Docker dependency.
//! Stub for Phase 3 implementation.
