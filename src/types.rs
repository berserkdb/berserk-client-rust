use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Complete query response with all result tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub tables: Vec<Table>,
    #[serde(default)]
    pub stats: Option<ExecutionStats>,
    #[serde(default)]
    pub warnings: Vec<QueryWarning>,
    #[serde(default)]
    pub partial_failures: Vec<PartialFailure>,
    #[serde(default)]
    pub visualization: Option<VisualizationMetadata>,
}

/// A result table with schema and rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Value>>,
}

/// Column definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: ColumnType,
}

/// Column data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColumnType {
    Bool,
    Int,
    Long,
    Real,
    String,
    Datetime,
    Timespan,
    Guid,
    Dynamic,
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Bool => write!(f, "bool"),
            ColumnType::Int => write!(f, "int"),
            ColumnType::Long => write!(f, "long"),
            ColumnType::Real => write!(f, "real"),
            ColumnType::String => write!(f, "string"),
            ColumnType::Datetime => write!(f, "datetime"),
            ColumnType::Timespan => write!(f, "timespan"),
            ColumnType::Guid => write!(f, "guid"),
            ColumnType::Dynamic => write!(f, "dynamic"),
        }
    }
}

/// A dynamic value from query results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i32),
    Long(i64),
    Real(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

/// Query execution statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub rows_processed: u64,
    pub chunks_total: u64,
    pub chunks_scanned: u64,
    pub query_time_nanos: u64,
    pub chunk_scan_time_nanos: u64,
}

/// A warning from query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryWarning {
    pub kind: String,
    pub message: String,
}

/// Partial failure info for segments that couldn't be read.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialFailure {
    pub segment_ids: Vec<String>,
    pub message: String,
}

/// Visualization metadata from the render operator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationMetadata {
    pub visualization_type: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}
