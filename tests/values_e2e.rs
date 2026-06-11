//! Validates that every `BqlValue` oneof arm in proto/dynamic_value.proto
//! decodes through the real `GrpcClient` against a live cluster — one
//! `print` query produces a column per value type, and each decoded cell
//! is asserted.
//!
//! Set BERSERK_ENDPOINT to a *direct* query-service endpoint (e.g.
//! query.bzrk.svc.cluster.local:9510). The client has no gateway auth
//! support, so the authenticated edge won't work here.

#![cfg(feature = "grpc")]

use std::collections::HashMap;

use berserk_client::{ColumnType, Config, GrpcClient, Value};

const GUID: &str = "74be27de-1e4e-49d9-b579-fe0b331d3642";
// 2024-01-15T10:30:00Z. The server emits datetimes as nanoseconds since
// the Unix epoch (NOTE: the proto comment claims ticks since
// 0001-01-01 — the wire disagrees). The client surfaces both datetime
// and timespan as Value::Long.
const DT_UNIX_NANOS: i64 = 1_705_314_600 * 1_000_000_000;
// Timespans ARE emitted as 100ns ticks: 1h = 3600s * 1e7.
const TS_1H_TICKS: i64 = 3600 * 10_000_000;

#[tokio::test]
async fn all_value_types_decode() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(ep) = std::env::var("BERSERK_ENDPOINT") else {
        eprintln!("BERSERK_ENDPOINT not set, skipping");
        return Ok(());
    };

    // One column per BqlValue oneof arm, plus in-oneof default values
    // (false / 0 / "") which proto3 oneof presence must keep
    // distinguishable from null.
    let query = format!(
        r#"print b = true,
  f = false,
  i = toint(42),
  l = tolong(1234567890123),
  z = tolong(0),
  r = 3.14,
  s = "hello",
  es = "",
  dt = todatetime("2024-01-15T10:30:00Z"),
  ts = 1h,
  g = toguid("{GUID}"),
  arr = dynamic([1, "two", true]),
  bag = dynamic({{"a": 1, "nested": {{"c": false}}}}),
  n = toint("not-a-number")"#
    );

    let client = GrpcClient::new(Config::new(&ep));
    let resp = client.query(&query, None, None, "UTC").await?;

    let table = resp
        .tables
        .iter()
        .find(|t| t.name == "PrimaryResult")
        .or(resp.tables.first())
        .expect("no result table");
    assert_eq!(table.rows.len(), 1, "expected exactly one row");
    let row = &table.rows[0];

    let expected: Vec<(&str, ColumnType, Value)> = vec![
        ("b", ColumnType::Bool, Value::Bool(true)),
        ("f", ColumnType::Bool, Value::Bool(false)),
        ("i", ColumnType::Int, Value::Int(42)),
        ("l", ColumnType::Long, Value::Long(1_234_567_890_123)),
        ("z", ColumnType::Long, Value::Long(0)),
        ("r", ColumnType::Real, Value::Real(3.14)),
        ("s", ColumnType::String, Value::String("hello".into())),
        ("es", ColumnType::String, Value::String("".into())),
        ("dt", ColumnType::Datetime, Value::Long(DT_UNIX_NANOS)),
        ("ts", ColumnType::Timespan, Value::Long(TS_1H_TICKS)),
        // The proto enum has COLUMN_TYPE_GUID, but the engine reports
        // guid-typed expressions as string columns (values arrive on
        // the string_value arm). If the server ever starts emitting
        // GUID, this expectation should flip to ColumnType::Guid.
        ("g", ColumnType::String, Value::String(GUID.into())),
        (
            "arr",
            ColumnType::Dynamic,
            Value::Array(vec![
                Value::Long(1),
                Value::String("two".into()),
                Value::Bool(true),
            ]),
        ),
        (
            "bag",
            ColumnType::Dynamic,
            Value::Object(HashMap::from([
                ("a".to_string(), Value::Long(1)),
                (
                    "nested".to_string(),
                    Value::Object(HashMap::from([("c".to_string(), Value::Bool(false))])),
                ),
            ])),
        ),
        ("n", ColumnType::Int, Value::Null),
    ];

    let index: HashMap<&str, usize> = table
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| (c.name.as_str(), i))
        .collect();

    for (name, expected_type, expected_value) in expected {
        let i = *index
            .get(name)
            .unwrap_or_else(|| panic!("column {name} missing from schema"));
        assert_eq!(
            table.columns[i].column_type, expected_type,
            "column type for {name}"
        );
        assert_eq!(row[i], expected_value, "decoded value for {name}");
    }

    Ok(())
}
