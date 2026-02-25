//! Sovereign DataFrame MCP tools — Phase 5, Directive 006A.
//!
//! 9 stateless tools exposing nexcore-dataframe operations.
//! Every tool accepts inline JSON data or file path, returns JSON result.
//!
//! Primitive composition: μ(Mapping) + Σ(Sum) + ∂(Boundary) + N(Quantity)

use crate::params::{
    DataframeAggregateParams, DataframeColumnStatsParams, DataframeConstructParams,
    DataframeCounterParams, DataframeDescribeParams, DataframeJoinParams, DataframeQueryParams,
    DataframeSaveParams, DataframeTransformParams,
};
use nexcore_dataframe::{Agg, Column, Counter, DataFrame, JoinType, Scalar};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::cmp::Ordering;
use std::path::Path;

// =============================================================================
// Helpers
// =============================================================================

fn text_result(value: &serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()),
    )])
}

fn load_df(
    data: Option<Vec<serde_json::Value>>,
    path: Option<String>,
) -> Result<DataFrame, McpError> {
    match (data, path) {
        (Some(rows), _) => {
            let json_str = serde_json::to_string(&rows)
                .map_err(|e| McpError::invalid_params(format!("Invalid JSON data: {e}"), None))?;
            DataFrame::from_json(&json_str)
                .map_err(|e| McpError::invalid_params(format!("DataFrame parse error: {e}"), None))
        }
        (_, Some(p)) => {
            let content = std::fs::read_to_string(&p)
                .map_err(|e| McpError::invalid_params(format!("File read error: {e}"), None))?;
            DataFrame::from_json(&content)
                .map_err(|e| McpError::invalid_params(format!("DataFrame parse error: {e}"), None))
        }
        (None, None) => Err(McpError::invalid_params(
            "Either 'data' or 'path' must be provided",
            None,
        )),
    }
}

fn df_to_json_rows(df: &DataFrame) -> Vec<serde_json::Value> {
    let names = df.column_names();
    (0..df.height())
        .map(|i| {
            let mut obj = serde_json::Map::new();
            for &name in &names {
                let val = df
                    .column(name)
                    .ok()
                    .and_then(|c| c.get(i))
                    .map(scalar_to_json)
                    .unwrap_or(serde_json::Value::Null);
                obj.insert(name.to_string(), val);
            }
            serde_json::Value::Object(obj)
        })
        .collect()
}

fn scalar_to_json(s: Scalar) -> serde_json::Value {
    match s {
        Scalar::Null => serde_json::Value::Null,
        Scalar::Bool(b) => json!(b),
        Scalar::Int64(n) => json!(n),
        Scalar::UInt64(n) => json!(n),
        Scalar::Float64(f) => {
            if f.is_nan() || f.is_infinite() {
                serde_json::Value::Null
            } else {
                json!(f)
            }
        }
        Scalar::String(s) => json!(s),
        _ => serde_json::Value::Null,
    }
}

fn json_to_scalar(v: &serde_json::Value) -> Scalar {
    match v {
        serde_json::Value::Null => Scalar::Null,
        serde_json::Value::Bool(b) => Scalar::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Scalar::Int64(i)
            } else if let Some(u) = n.as_u64() {
                Scalar::UInt64(u)
            } else if let Some(f) = n.as_f64() {
                Scalar::Float64(f)
            } else {
                Scalar::Null
            }
        }
        serde_json::Value::String(s) => Scalar::String(s.clone()),
        _ => Scalar::Null,
    }
}

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

// =============================================================================
// 1. dataframe_describe
// =============================================================================

/// Schema + row count + per-column statistics + sample rows.
pub fn describe(params: DataframeDescribeParams) -> Result<CallToolResult, McpError> {
    let df = load_df(params.data, params.path)?;
    let sample_n = params.sample_rows.unwrap_or(5).min(df.height());

    let schema: Vec<serde_json::Value> = df
        .column_names()
        .iter()
        .filter_map(|&name| {
            let col = df.column(name).ok()?;
            Some(json!({
                "name": name,
                "dtype": format!("{:?}", col.dtype()),
                "non_null": col.non_null_count(),
                "null": col.null_count(),
                "n_unique": col.n_unique(),
            }))
        })
        .collect();

    let sample = df_to_json_rows(&df.head(sample_n));

    Ok(text_result(&json!({
        "height": df.height(),
        "width": df.width(),
        "columns": schema,
        "sample_rows": sample,
    })))
}

// =============================================================================
// 2. dataframe_query
// =============================================================================

/// Filter + sort + limit rows from a DataFrame.
pub fn query(params: DataframeQueryParams) -> Result<CallToolResult, McpError> {
    let mut df = load_df(params.data, params.path)?;

    // Apply filters
    if let Some(filters) = &params.filters {
        for f in filters {
            let target = json_to_scalar(&f.value);
            let op = f.op.clone();
            let col_name = f.column.clone();

            df = df
                .filter_by(&col_name, |v| apply_comparison(v, &target, &op))
                .map_err(|e| McpError::invalid_params(format!("Filter error: {e}"), None))?;
        }
    }

    // Sort
    if let Some(sort_col) = &params.sort_by {
        let desc = params.descending.unwrap_or(false);
        df = df
            .sort(sort_col, desc)
            .map_err(|e| McpError::invalid_params(format!("Sort error: {e}"), None))?;
    }

    // Limit
    if let Some(limit) = params.limit {
        df = df.head(limit);
    }

    // Select columns
    if let Some(cols) = &params.columns {
        let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
        df = df
            .select(&col_refs)
            .map_err(|e| McpError::invalid_params(format!("Select error: {e}"), None))?;
    }

    let rows = df_to_json_rows(&df);

    Ok(text_result(&json!({
        "height": df.height(),
        "width": df.width(),
        "rows": rows,
    })))
}

fn apply_comparison(v: &Scalar, target: &Scalar, op: &str) -> bool {
    match op {
        "eq" => v.compare(target) == Ordering::Equal,
        "ne" => v.compare(target) != Ordering::Equal,
        "gt" => v.compare(target) == Ordering::Greater,
        "ge" => v.compare(target) != Ordering::Less,
        "lt" => v.compare(target) == Ordering::Less,
        "le" => v.compare(target) != Ordering::Greater,
        "contains" => {
            if let (Some(haystack), Some(needle)) = (v.as_str(), target.as_str()) {
                haystack.contains(needle)
            } else {
                false
            }
        }
        _ => false,
    }
}

// =============================================================================
// 3. dataframe_aggregate
// =============================================================================

/// Group by columns and apply aggregation functions.
pub fn aggregate(params: DataframeAggregateParams) -> Result<CallToolResult, McpError> {
    let df = load_df(params.data, params.path)?;

    if params.group_by.is_empty() {
        return Err(McpError::invalid_params(
            "group_by must have at least one column",
            None,
        ));
    }
    if params.aggs.is_empty() {
        return Err(McpError::invalid_params(
            "aggs must have at least one aggregation",
            None,
        ));
    }

    let group_cols: Vec<&str> = params.group_by.iter().map(|s| s.as_str()).collect();
    let agg_specs: Vec<Agg> = params
        .aggs
        .iter()
        .map(|a| match a.func.as_str() {
            "sum" => Agg::Sum(a.column.clone().unwrap_or_default()),
            "mean" => Agg::Mean(a.column.clone().unwrap_or_default()),
            "min" => Agg::Min(a.column.clone().unwrap_or_default()),
            "max" => Agg::Max(a.column.clone().unwrap_or_default()),
            "count" => Agg::Count,
            "first" => Agg::First(a.column.clone().unwrap_or_default()),
            "last" => Agg::Last(a.column.clone().unwrap_or_default()),
            "n_unique" => Agg::NUnique(a.column.clone().unwrap_or_default()),
            _ => Agg::Count,
        })
        .collect();

    let grouped = df
        .group_by(&group_cols)
        .map_err(|e| McpError::invalid_params(format!("GroupBy error: {e}"), None))?;

    let result = grouped
        .agg(&agg_specs)
        .map_err(|e| McpError::invalid_params(format!("Aggregation error: {e}"), None))?;

    let rows = df_to_json_rows(&result);

    Ok(text_result(&json!({
        "groups": result.height(),
        "columns": result.column_names(),
        "rows": rows,
    })))
}

// =============================================================================
// 4. dataframe_counter
// =============================================================================

/// Count occurrences by key columns using the optimized Counter type.
/// Counter is semantically superior to group_by+count for counting operations.
pub fn counter(params: DataframeCounterParams) -> Result<CallToolResult, McpError> {
    let df = load_df(params.data, params.path)?;

    if params.key_columns.is_empty() {
        return Err(McpError::invalid_params(
            "key_columns must have at least one column",
            None,
        ));
    }

    let key_cols: Vec<&str> = params.key_columns.iter().map(|s| s.as_str()).collect();

    let mut ctr = Counter::from_dataframe(&df, &key_cols)
        .map_err(|e| McpError::invalid_params(format!("Counter error: {e}"), None))?;

    // Apply min_count filter
    if let Some(min) = params.min_count {
        if min > 1 {
            ctr = ctr.filter_min_count(min);
        }
    }

    let total = ctr.total();
    let n_groups = ctr.len();

    // Convert to DataFrame for output
    let result_df = ctr
        .into_dataframe()
        .map_err(|e| McpError::invalid_params(format!("Counter→DataFrame error: {e}"), None))?;

    let rows = df_to_json_rows(&result_df);

    Ok(text_result(&json!({
        "unique_groups": n_groups,
        "total_count": total,
        "key_columns": params.key_columns,
        "rows": rows,
    })))
}

// =============================================================================
// 5. dataframe_column_stats
// =============================================================================

/// Detailed statistics for a single column.
pub fn column_stats(params: DataframeColumnStatsParams) -> Result<CallToolResult, McpError> {
    let df = load_df(params.data, params.path)?;

    let col = df
        .column(&params.column)
        .map_err(|e| McpError::invalid_params(format!("Column error: {e}"), None))?;

    let mut stats = json!({
        "name": col.name(),
        "dtype": format!("{:?}", col.dtype()),
        "length": col.len(),
        "non_null": col.non_null_count(),
        "null_count": col.null_count(),
        "n_unique": col.n_unique(),
    });

    // Numeric stats
    let min = col.min();
    let max = col.max();
    let mean = col.mean();
    let median = col.median();
    let std_dev = col.std_dev();
    let sum = col.sum();

    if !min.is_null() {
        stats["min"] = scalar_to_json(min);
        stats["max"] = scalar_to_json(max);
        stats["mean"] = scalar_to_json(mean);
        stats["median"] = scalar_to_json(median);
        stats["std_dev"] = scalar_to_json(std_dev);
        stats["sum"] = scalar_to_json(sum);

        // Quantiles
        if let Ok(p25) = col.quantile(0.25) {
            stats["p25"] = scalar_to_json(p25);
        }
        if let Ok(p50) = col.quantile(0.50) {
            stats["p50"] = scalar_to_json(p50);
        }
        if let Ok(p75) = col.quantile(0.75) {
            stats["p75"] = scalar_to_json(p75);
        }
        if let Ok(p90) = col.quantile(0.90) {
            stats["p90"] = scalar_to_json(p90);
        }
        if let Ok(p95) = col.quantile(0.95) {
            stats["p95"] = scalar_to_json(p95);
        }
        if let Ok(p99) = col.quantile(0.99) {
            stats["p99"] = scalar_to_json(p99);
        }
    }

    // First/last values
    stats["first"] = scalar_to_json(col.first());
    stats["last"] = scalar_to_json(col.last());

    Ok(text_result(&stats))
}

// =============================================================================
// 6. dataframe_construct
// =============================================================================

/// Build a DataFrame from typed column definitions.
pub fn construct(params: DataframeConstructParams) -> Result<CallToolResult, McpError> {
    if params.columns.is_empty() {
        return Err(McpError::invalid_params(
            "columns must have at least one definition",
            None,
        ));
    }

    let columns: Vec<Column> = params
        .columns
        .iter()
        .map(|def| build_column(def))
        .collect::<Result<Vec<_>, _>>()?;

    let df = DataFrame::new(columns).map_err(|e| {
        McpError::invalid_params(format!("DataFrame construction error: {e}"), None)
    })?;

    let rows = df_to_json_rows(&df);

    Ok(text_result(&json!({
        "height": df.height(),
        "width": df.width(),
        "columns": df.column_names(),
        "data": rows,
    })))
}

fn build_column(def: &crate::params::DataframeColumnDef) -> Result<Column, McpError> {
    match def.dtype.as_str() {
        "bool" => {
            let vals: Vec<bool> = def
                .values
                .iter()
                .map(|v| v.as_bool().unwrap_or(false))
                .collect();
            Ok(Column::from_bools(&def.name, vals))
        }
        "i64" => {
            let vals: Vec<i64> = def.values.iter().map(|v| v.as_i64().unwrap_or(0)).collect();
            Ok(Column::from_i64s(&def.name, vals))
        }
        "u64" => {
            let vals: Vec<u64> = def.values.iter().map(|v| v.as_u64().unwrap_or(0)).collect();
            Ok(Column::from_u64s(&def.name, vals))
        }
        "f64" => {
            let vals: Vec<f64> = def
                .values
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect();
            Ok(Column::from_f64s(&def.name, vals))
        }
        "string" => {
            let vals: Vec<String> = def
                .values
                .iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect();
            Ok(Column::from_strings(&def.name, vals))
        }
        other => Err(McpError::invalid_params(
            format!("Unknown dtype '{other}'. Use: bool, i64, u64, f64, string"),
            None,
        )),
    }
}

// =============================================================================
// 7. dataframe_transform
// =============================================================================

/// Select, drop, or rename columns.
pub fn transform(params: DataframeTransformParams) -> Result<CallToolResult, McpError> {
    let mut df = load_df(params.data, params.path)?;

    // Rename first (before select/drop reference new names)
    if let Some(renames) = &params.rename {
        for r in renames {
            df = df
                .rename_column(&r.from, &r.to)
                .map_err(|e| McpError::invalid_params(format!("Rename error: {e}"), None))?;
        }
    }

    // Select (keep only these columns)
    if let Some(cols) = &params.select {
        let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
        df = df
            .select(&col_refs)
            .map_err(|e| McpError::invalid_params(format!("Select error: {e}"), None))?;
    }

    // Drop
    if let Some(cols) = &params.drop {
        let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
        df = df.drop_columns(&col_refs);
    }

    let rows = df_to_json_rows(&df);

    Ok(text_result(&json!({
        "height": df.height(),
        "width": df.width(),
        "columns": df.column_names(),
        "data": rows,
    })))
}

// =============================================================================
// 8. dataframe_save
// =============================================================================

/// Write DataFrame to a JSON file.
pub fn save(params: DataframeSaveParams) -> Result<CallToolResult, McpError> {
    let df = load_df(params.data, params.source_path)?;
    let path = Path::new(&params.output_path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| McpError::invalid_params(format!("Cannot create directory: {e}"), None))?;
    }

    df.to_json_file(path)
        .map_err(|e| McpError::invalid_params(format!("Write error: {e}"), None))?;

    Ok(text_result(&json!({
        "rows_written": df.height(),
        "columns": df.width(),
        "path": params.output_path,
    })))
}

// =============================================================================
// 9. dataframe_join
// =============================================================================

/// Join two DataFrames on key columns.
/// Supports inner, left, right, outer, semi, and anti joins.
pub fn join(params: DataframeJoinParams) -> Result<CallToolResult, McpError> {
    let left_df = load_df(params.left_data, params.left_path)?;
    let right_df = load_df(params.right_data, params.right_path)?;

    // Parse join type
    let how = match params.how.as_deref().unwrap_or("inner") {
        "inner" => JoinType::Inner,
        "left" => JoinType::Left,
        "right" => JoinType::Right,
        "outer" | "full" => JoinType::Outer,
        "semi" => JoinType::Semi,
        "anti" => JoinType::Anti,
        other => {
            return Err(McpError::invalid_params(
                format!("Unknown join type '{other}'. Use: inner, left, right, outer, semi, anti"),
                None,
            ));
        }
    };

    // Determine key columns — either shared `on` or asymmetric `left_on`/`right_on`
    let result = match (params.on, params.left_on, params.right_on) {
        (Some(on), None, None) => {
            if on.is_empty() {
                return Err(McpError::invalid_params(
                    "'on' must have at least one column name",
                    None,
                ));
            }
            let on_refs: Vec<&str> = on.iter().map(|s| s.as_str()).collect();
            left_df.join(&right_df, &on_refs, how)
        }
        (None, Some(l_on), Some(r_on)) => {
            if l_on.is_empty() || r_on.is_empty() {
                return Err(McpError::invalid_params(
                    "'left_on' and 'right_on' must have at least one column name",
                    None,
                ));
            }
            let l_refs: Vec<&str> = l_on.iter().map(|s| s.as_str()).collect();
            let r_refs: Vec<&str> = r_on.iter().map(|s| s.as_str()).collect();
            left_df.join_on(&right_df, &l_refs, &r_refs, how)
        }
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            return Err(McpError::invalid_params(
                "Use either 'on' OR 'left_on'/'right_on', not both",
                None,
            ));
        }
        _ => {
            return Err(McpError::invalid_params(
                "Must provide either 'on' or both 'left_on' and 'right_on'",
                None,
            ));
        }
    };

    let df = result.map_err(|e| McpError::invalid_params(format!("Join error: {e}"), None))?;
    let rows = df_to_json_rows(&df);

    Ok(text_result(&json!({
        "height": df.height(),
        "width": df.width(),
        "columns": df.column_names(),
        "join_type": format!("{how:?}"),
        "rows": rows,
    })))
}
