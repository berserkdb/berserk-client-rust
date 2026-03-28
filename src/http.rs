#[cfg(feature = "http")]
mod inner {
    use crate::config::Config;
    use crate::error::{Error, Result};
    use crate::types::*;
    use serde::{Deserialize, Serialize};

    /// HTTP client for the Berserk ADX v2 REST endpoint.
    pub struct HttpClient {
        config: Config,
        client: reqwest::Client,
    }

    #[derive(Serialize)]
    struct KustoV2Request {
        csl: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        db: Option<String>,
    }

    /// A frame in the v2 response array.
    #[derive(Deserialize)]
    #[serde(tag = "FrameType")]
    enum V2Frame {
        DataSetHeader {},
        DataTable {
            #[serde(rename = "TableKind")]
            table_kind: String,
            #[serde(rename = "TableName")]
            table_name: String,
            #[serde(rename = "Columns")]
            columns: Vec<V2Column>,
            #[serde(rename = "Rows")]
            rows: Vec<Vec<serde_json::Value>>,
        },
        DataSetCompletion {
            #[serde(rename = "HasErrors")]
            has_errors: bool,
        },
        #[serde(other)]
        Unknown,
    }

    #[derive(Deserialize)]
    struct V2Column {
        #[serde(rename = "ColumnName")]
        column_name: String,
        #[serde(rename = "ColumnType")]
        column_type: String,
    }

    impl HttpClient {
        pub fn new(config: Config) -> Self {
            let client = reqwest::Client::builder()
                .timeout(config.timeout)
                .connect_timeout(config.connect_timeout)
                .build()
                .unwrap_or_default();
            Self { config, client }
        }

        /// Execute a query via the ADX v2 REST endpoint.
        pub async fn query(&self, query: &str) -> Result<QueryResponse> {
            let url = format!("{}/v2/rest/query", self.config.normalized_endpoint());

            let body = KustoV2Request {
                csl: query.to_string(),
                db: None,
            };

            let mut req = self.client.post(&url).json(&body);

            if let Some(username) = &self.config.username {
                req = req.header("x-bzrk-username", username);
            }
            if let Some(client_name) = &self.config.client_name {
                req = req.header("x-bzrk-client-name", client_name);
            }

            let resp = req.send().await.map_err(|e| Error::Http(e.to_string()))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(Error::Http(format!("HTTP {}: {}", status, body)));
            }

            let frames: Vec<V2Frame> = resp
                .json()
                .await
                .map_err(|e| Error::Parse(e.to_string()))?;

            let mut tables = Vec::new();
            let mut has_errors = false;

            for frame in frames {
                match frame {
                    V2Frame::DataTable {
                        table_kind,
                        table_name,
                        columns,
                        rows,
                    } if table_kind == "PrimaryResult" => {
                        let cols: Vec<Column> = columns
                            .into_iter()
                            .map(|c| Column {
                                name: c.column_name,
                                column_type: parse_column_type(&c.column_type),
                            })
                            .collect();
                        let converted_rows: Vec<Vec<Value>> = rows
                            .into_iter()
                            .map(|row| row.into_iter().map(json_to_value).collect())
                            .collect();
                        tables.push(Table {
                            name: table_name,
                            columns: cols,
                            rows: converted_rows,
                        });
                    }
                    V2Frame::DataSetCompletion { has_errors: he } => {
                        has_errors = he;
                    }
                    _ => {}
                }
            }

            if has_errors {
                return Err(Error::Query {
                    code: "ServerError".to_string(),
                    title: "Query completed with errors".to_string(),
                    message: "The server reported errors in the result set".to_string(),
                });
            }

            Ok(QueryResponse {
                tables,
                stats: None,
                warnings: Vec::new(),
                partial_failures: Vec::new(),
                visualization: None,
            })
        }
    }

    fn parse_column_type(s: &str) -> ColumnType {
        match s {
            "bool" => ColumnType::Bool,
            "int" => ColumnType::Int,
            "long" => ColumnType::Long,
            "real" | "double" => ColumnType::Real,
            "string" => ColumnType::String,
            "datetime" => ColumnType::Datetime,
            "timespan" => ColumnType::Timespan,
            "guid" | "uuid" => ColumnType::Guid,
            _ => ColumnType::Dynamic,
        }
    }

    fn json_to_value(v: serde_json::Value) -> Value {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        Value::Int(i as i32)
                    } else {
                        Value::Long(i)
                    }
                } else {
                    Value::Real(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(json_to_value).collect())
            }
            serde_json::Value::Object(obj) => {
                let map = obj
                    .into_iter()
                    .map(|(k, v)| (k, json_to_value(v)))
                    .collect();
                Value::Object(map)
            }
        }
    }
}

#[cfg(feature = "http")]
pub use inner::*;
