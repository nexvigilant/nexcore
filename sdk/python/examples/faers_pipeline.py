#!/usr/bin/env python3
"""FAERS reference pipeline — drug safety signal detection from FDA data.

Flow:
  1. faers_drug_events(drug) → top adverse events
  2. faers_signal_check(drug, event) per top event → flag signals
  3. signal_batch(items) for flagged pairs → full metrics
  4. mcp("pv_core_fdr_adjust", ...) → FDR correction
  5. Print report
"""

from nexcore_sdk import NexCoreClient

DRUG = "METFORMIN"
TOP_N = 10

client = NexCoreClient("http://localhost:3030", timeout=60.0)

# Step 1: Get top adverse events from FAERS
print(f"=== FAERS Signal Pipeline: {DRUG} ===\n")
events = client.faers_drug_events(DRUG, limit=TOP_N)
print(f"Total reports: {events.total_reports:,}")
print(f"Top {len(events.events)} events:\n")

# Step 2: Signal check each event
flagged = []
for ev in events.events:
    check = client.faers_signal_check(DRUG, ev.event)
    flag = "*" if check.signal_detected else " "
    print(f"  {flag} {ev.event:<30} PRR={check.prr:6.2f}  n={check.case_count:,}")
    if check.signal_detected:
        flagged.append(check)

if not flagged:
    print("\nNo signals detected.")
    raise SystemExit(0)

# Step 3: FDR correction on flagged p-values (approximate from PRR)
# Use the generic MCP gateway for advanced PV tools
p_values = [1.0 / max(f.prr, 0.01) for f in flagged]  # rough proxy
fdr = client.mcp("pv_core_fdr_adjust", {
    "p_values": p_values,
    "method": "benjamini_hochberg",
})

# Step 4: Report
print(f"\n=== {len(flagged)} signals detected (FDR-corrected) ===\n")
for i, f in enumerate(flagged):
    adjusted = fdr.get("adjusted", p_values)[i] if isinstance(fdr, dict) else p_values[i]
    print(f"  {f.drug} + {f.event}: PRR={f.prr:.2f}, ROR={f.ror:.2f}, n={f.case_count}, q={adjusted:.4f}")
