# nexcore-openfda

Generalized REST API client for the OpenFDA platform. This crate provides typed access to all major FDA endpoints, including drugs (events, labels, recalls), devices, and food safety data.

## Intent
To provide a unified, async-first bridge to real-time FDA data. It enables AI agents and automated systems to query the most up-to-date safety and regulatory information without managing low-level HTTP or JSON-parsing overhead.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **μ (Mapping)**: The primary primitive for mapping REST API JSON responses to structured Rust types.
- **λ (Address)**: Manages the URL construction and endpoint addressing for the various FDA services.
- **→ (Causality)**: Implements the request-response lifecycle and concurrent fan-out searches.
- **∅ (Void)**: Handles the "No results found" (404) states common in search operations.

## Core Features
- **Concurrent Fan-out Search**: Query multiple FDA domains (Drugs, Devices, Food) simultaneously.
- **Full Endpoint Coverage**: Support for `/drug/event.json`, `/device/510k.json`, `/food/enforcement.json`, and more.
- **OpenFDA Enrichment**: Automatic extraction of manufacturer names, active ingredients, and NDCs from raw responses.
- **Caching-Aware**: Designed to be used behind a caching proxy or with local response persistence.

## SOPs for Use
### Fetching Drug Events
```rust
use nexcore_openfda::{OpenFdaClient, QueryParams};
use nexcore_openfda::endpoints::fetch_drug_events;

let client = OpenFdaClient::new()?;
let params = QueryParams::search("patient.drug.medicinalproduct:aspirin", 10);
let response = fetch_drug_events(&client, &params).await?;
```

### Concurrent Multi-Search
Use `search::fan_out_search` to perform a "global" query across all major FDA domains.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
