#!/usr/bin/env python3
"""NexCore SDK quickstart — 10 lines to your first signal detection."""

from nexcore_sdk import NexCoreClient

client = NexCoreClient("http://localhost:3030")

# Run multi-method signal detection on a 2x2 contingency table
result = client.signal_detect(a=100, b=500, c=50, d=10000)

print(f"PRR:  {result.prr:.2f} [{result.prr_ci_lower:.2f}, {result.prr_ci_upper:.2f}]")
print(f"ROR:  {result.ror:.2f} [{result.ror_ci_lower:.2f}, {result.ror_ci_upper:.2f}]")
print(f"EBGM: {result.ebgm:.2f} (EB05: {result.eb05:.2f})")
print(f"Signal detected: {result.signal_detected}")
