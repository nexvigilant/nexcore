# AI Guidance — nexcore-faers-etl

FDA Data Bridge and large-scale ETL pipeline.

## Use When
- Ingesting quarterly bulk data files from the FDA (ASCII/FAERS).
- Performing population-scale signal detection (millions of records).
- Stratifying safety signals by demographic or geographic factors.
- Normalizing drug or event names against large-scale reference sets.
- Generating Parquet outputs for high-performance downstream analysis.

## Grounding Patterns
- **DataFrame Priority**: Favor the `LazyFrame` API for all transformations to allow Polars to optimize the execution plan.
- **Batch Processing**: Use the `rayon`-enabled batch functions in the `signals` module for O(N) detection across large contingency sets.
- **T1 Primitives**:
  - `μ + Σ`: Root primitives for mapping and aggregating bulk data.
  - `σ + π`: Root primitives for sequential pipeline flow and durable storage.

## Maintenance SOPs
- **Parquet Compatibility**: Always use Snappy compression when writing Parquet files to maintain balance between speed and size.
- **Role Filtering**: Suspect drugs (Primary/Secondary) are the default; only include "Concomitant" drugs if `include_all_roles` is explicitly requested.
- **Memory Management**: When processing files >10GB, ensure the `FAERS_DATA_DIR` is on a fast SSD and monitor RSS usage during the aggregation phase.

## Key Entry Points
- `src/lib.rs`: The main `run_full_pipeline` coordinator.
- `src/analytics.rs`: Specialized algorithms (Velocity, Cascade, Polypharmacy).
- `src/api.rs`: Real-time bridge to the OpenFDA API.
