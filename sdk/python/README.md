# NexCore Python SDK

Python client for [NexVigilant's](https://nexvigilant.com) NexCore pharmacovigilance platform.

Access 780+ MCP tools, PV signal detection (PRR/ROR/IC/EBGM), FAERS queries, and causality assessment through typed Python APIs backed by Rust computation.

## Install

```bash
pip install nexcore-sdk
# or from source:
pip install -e path/to/nexcore/sdk/python
```

## Quickstart

```python
from nexcore_sdk import NexCoreClient

client = NexCoreClient("http://localhost:3030")

# Multi-method signal detection with confidence intervals
result = client.signal_detect(a=100, b=500, c=50, d=10000)
print(f"PRR: {result.prr:.2f} [{result.prr_ci_lower:.2f}, {result.prr_ci_upper:.2f}]")
print(f"Signal: {result.signal_detected}")
```

## Features

| Method | Description | Endpoint |
|--------|-------------|----------|
| `signal_detect(a, b, c, d)` | Multi-method signal detection | `POST /api/v1/pv/signal/complete` |
| `signal_prr(a, b, c, d)` | PRR with 95% CI | `POST /api/v1/pv/signal/prr` |
| `signal_ror(a, b, c, d)` | ROR with 95% CI | `POST /api/v1/pv/signal/ror` |
| `naranjo(...)` | Naranjo causality assessment | `POST /api/v1/pv/naranjo` |
| `faers_search(query)` | Search FAERS reports | `GET /api/v1/faers/search` |
| `faers_drug_events(drug)` | Top adverse events for a drug | `GET /api/v1/faers/drug-events` |
| `faers_signal_check(drug, event)` | Drug-event signal check | `POST /api/v1/faers/signal-check` |
| `signal_batch(items)` | Batch signal detection | `POST /api/v1/signal/batch` |
| `mcp(tool, params)` | Any of 780+ MCP tools | `POST /api/v1/mcp/{tool}` |

## Measured Values

Every statistical output returns confidence intervals, not bare floats:

```python
prr = client.signal_prr(a=100, b=500, c=50, d=10000)
print(prr.value)      # 3.96
print(prr.ci_lower)   # 3.22
print(prr.ci_upper)   # 4.87
print(prr.signal)     # True
```

## Docker

```bash
docker run -p 3030:3030 nexvigilant/nexcore
# Then from host:
python -c "from nexcore_sdk import NexCoreClient; print(NexCoreClient().health())"
```

## Requirements

- Python 3.10+
- httpx >= 0.25
- pydantic >= 2.0
- NexCore API running (default: localhost:3030)
