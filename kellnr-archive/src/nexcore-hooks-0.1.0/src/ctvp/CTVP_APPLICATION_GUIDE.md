# CTVP Framework Application Guide

## Quick Start: Applying CTVP to Any Capability

### Step 1: Define Your Capability

```rust
use nexcore_hooks::ctvp::*;

let capability = Capability::new("Your Feature Name")
    .with_description("What this capability does")
    .with_metric(MetricType::Rate)  // Rate, Count, Latency, or Boolean
    .with_threshold(0.80)           // Target threshold for Phase 2
    .with_alert_threshold(0.70)     // Alert when below this
    .with_min_observations(10);     // Min data before validating
```

### Step 2: Create a Tracker

```rust
let mut tracker = CapabilityTracker::new(capability);
```

### Step 3: Record Observations

```rust
// For rate metrics (success/failure)
tracker.record(achieved: true, value: 1.0);
tracker.record(achieved: false, value: 0.0);

// For latency metrics
tracker.record(true, latency_ms);
```

### Step 4: Check Validation Status

```rust
if tracker.has_sufficient_data() {
    if tracker.meets_threshold() {
        println!("✅ Phase 2: VALIDATED (CAR={:.1}%)", tracker.car() * 100.0);
    } else {
        println!("❌ Phase 2: Below threshold (CAR={:.1}%)", tracker.car() * 100.0);
    }
} else {
    println!("⏳ Need {} more observations", tracker.observations_needed());
}
```

### Step 5: Add Phase 4 Drift Detection

```rust
let mut drift_detector = DriftDetector::new(
    capability.threshold,      // target
    capability.alert_threshold // alert
);

// Check for drift periodically
if let Some(alert) = drift_detector.check_and_alert(tracker.car()) {
    eprintln!("🚨 DRIFT ALERT: {}", alert.message);
}
```

---

## Complete CTVP Implementation Checklist

### Phase 0: Preclinical (Mechanism Validity)
- [ ] Unit tests pass
- [ ] Property-based tests (if applicable)
- [ ] Type checking passes
- [ ] Static analysis clean
- [ ] Code coverage > 80%

**Evidence Quality: Strong if all pass**

### Phase 1: Safety (Failure Mode Validation)
- [ ] Fault injection tests exist
- [ ] Malformed input handling tested
- [ ] Missing dependency handling tested
- [ ] Resource exhaustion handling tested
- [ ] Error messages don't leak sensitive data

**Evidence Quality: Strong if all pass**

### Phase 2: Efficacy (Capability Achievement)
- [ ] Define capability with threshold
- [ ] Create tracker for observations
- [ ] Record all outcomes (success/failure)
- [ ] Wait for min_observations
- [ ] CAR >= threshold

**Evidence Quality: Strong when CAR >= threshold with sufficient data**

### Phase 3: Confirmation (Scale Validation)
- [ ] Canary rollout configured (rollout_percentage < 100)
- [ ] A/B comparison possible
- [ ] Metrics monitored during rollout
- [ ] No regressions detected
- [ ] Rollout to 100%

**Evidence Quality: Strong when full rollout with no issues**

### Phase 4: Surveillance (Ongoing Correctness)
- [ ] Drift detector configured
- [ ] Alert thresholds set
- [ ] Trend analysis active
- [ ] Alerting integrated (logs, notifications)
- [ ] Periodic health checks

**Evidence Quality: Strong when actively monitoring**

---

## Configuration Template

Create `~/.claude/your_feature_config.toml`:

```toml
[feature_flags]
enabled = true
rollout_percentage = 10  # Start at 10%, increase gradually

[thresholds]
car_target = 0.80        # Phase 2 validation target
min_sessions = 10        # Minimum data before validation
car_alert = 0.70         # Alert when CAR drops below

[cleanup]
retention_days = 30
auto_cleanup = true
```

---

## Five Problems Protocol

For every capability, explicitly identify:

1. **Safety Problem**: What failure modes aren't tested?
2. **Efficacy Problem**: Where does capability claim exceed evidence?
3. **Confirmation Problem**: What production conditions aren't validated?
4. **Structural Problem**: Where is architecture coupled to implementation?
5. **Functional Problem**: What hidden edge cases exist?

---

## Example: MCP Tool Suggester CTVP

```rust
// Capability Definition
let mcp_adoption = Capability::new("MCP Tool Adoption")
    .with_description("Claude uses suggested MCP tools")
    .with_metric(MetricType::Rate)
    .with_threshold(0.80)
    .with_alert_threshold(0.70)
    .with_min_observations(10);

// Phase 0: Unit tests in mcp_tool_suggester.rs
// Phase 1: Fault injection in hook_fault_injection.rs
// Phase 2: Tracking in mcp_efficacy.rs
// Phase 3: Canary via mcp_efficacy_config.toml
// Phase 4: Drift detection in mcp_efficacy_report CLI
```

---

## Validation Summary Template

```
╔══════════════════════════════════════════════════════╗
║  🔬 CTVP VALIDATION: [Capability Name]               ║
╠══════════════════════════════════════════════════════╣
║  ✅ P0 Mechanism    Strong                           ║
║  ✅ P1 Safety       Moderate                         ║
║  🔄 P2 Efficacy     InProgress (60%)                 ║
║  ⏳ P3 Confirm      NotStarted                       ║
║  ⏳ P4 Surveil      NotStarted                       ║
╠══════════════════════════════════════════════════════╣
║  Evidence stops at: Phase 1                          ║
║  Status: InProgress                                  ║
╚══════════════════════════════════════════════════════╝
```

---

## Key Metrics

| Metric | Formula | Target |
|--------|---------|--------|
| CAR | achieved / total | >= 80% |
| Drift Score | (baseline - current) / baseline | < 0.2 |
| Trend | recent_avg - older_avg | Stable or Improving |

---

## Framework Version

CTVP v1.0.0 - Validated 2026-01-29
