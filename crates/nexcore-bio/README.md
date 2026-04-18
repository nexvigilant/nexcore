# nexcore-bio

Biological systems aggregator — unified Rust re-exports of all NexCore
biological crates, with optional PyO3 bindings that expose them as a single
Python module `nexcore_bio`.

## Rust usage

```rust
use nexcore_bio::{dna, cytokine, immunity, metabolite};
```

Every biological NexCore crate is reachable from this umbrella; see
`AGGREGATED_CRATES` for the canonical list.

## Python usage

Requires [maturin](https://www.maturin.rs):

```bash
cd ~/Projects/Active/nucleus/workspaces/nexcore/crates/nexcore-bio
maturin develop --release -F python   # installs into active virtualenv
```

Then:

```python
import nexcore_bio
print(nexcore_bio.aggregated_crates())
print(nexcore_bio.metabolite_predict("CCO"))    # ethanol metabolites
```

## Scope

This crate is the seam between the Rust workspace and the Python ecosystem.
The default Rust build (`cargo build -p nexcore-bio`) does **not** pull in
PyO3 — Python support is gated behind the `python` feature, kept optional so
the nexcore workspace remains pure Rust by default.

## License

All Rights Reserved — proprietary. Contact matthew@camp-corp.com for licensing.
