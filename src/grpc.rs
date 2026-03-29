#[cfg(feature = "grpc")]
mod inner {
    use crate::config::Config;
    use crate::error::{Error, Result};
    use crate::types::*;
    use tokio::sync::OnceCell;
    use tonic::metadata::MetadataValue;
    use tonic::transport::Channel;

    /// Generated proto types for the query service.
    pub mod query_proto {
        tonic::include_proto!("query");
    }

    /// Generated proto types for dynamic values.
    pub mod berserk_proto {
        tonic::include_proto!("berserk");
    }

    use query_proto::query_service_client::QueryServiceClient as ProtoClient;

    /// gRPC client for the Berserk query service.
    pub struct GrpcClient {
        config: Config,
        channel: OnceCell<Channel>,
    }

    impl GrpcClient {
        pub fn new(config: Config) -> Self {
            Self {
                config,
                channel: OnceCell::new(),
            }
        }

        async fn channel(&self) -> Result<Channel> {
            self.channel
                .get_or_try_init(|| async {
                    let endpoint = self.config.normalized_endpoint();
                    Channel::from_shared(endpoint)
                        .map_err(|e| Error::GrpcConnection(e.to_string()))?
                        .connect_timeout(self.config.connect_timeout)
                        .timeout(self.config.timeout)
                        .connect()
                        .await
                        .map_err(|e| Error::GrpcConnection(e.to_string()))
                })
                .await
                .cloned()
        }

        /// Execute a query and collect all results into a QueryResponse.
        pub async fn query(
            &self,
            query: &str,
            since: Option<&str>,
            until: Option<&str>,
            timezone: &str,
        ) -> Result<QueryResponse> {
            let channel = self.channel().await?;
            let mut client = ProtoClient::new(channel);

            let request = query_proto::ExecuteQueryRequest {
                query: query.to_string(),
                since: since.unwrap_or_default().to_string(),
                until: until.unwrap_or_default().to_string(),
                timezone: timezone.to_string(),
            };

            let mut req = tonic::Request::new(request);
            req.set_timeout(self.config.timeout);

            if let Some(username) = &self.config.username {
                if let Ok(val) = username.parse::<MetadataValue<tonic::metadata::Ascii>>() {
                    req.metadata_mut().insert("x-bzrk-username", val);
                }
            }
            if let Some(client_name) = &self.config.client_name {
                if let Ok(val) = client_name.parse::<MetadataValue<tonic::metadata::Ascii>>() {
                    req.metadata_mut().insert("x-bzrk-client-name", val);
                }
            }

            let response = client
                .execute_query(req)
                .await
                .map_err(|s| Error::GrpcStatus(s.message().to_string()))?;

            let mut stream = response.into_inner();
            let mut tables: Vec<Table> = Vec::new();
            let mut current_schema: Option<(String, Vec<Column>)> = None;
            let mut current_rows: Vec<Vec<Value>> = Vec::new();
            let mut stats = None;
            let mut warnings = Vec::new();
            let mut partial_failures = Vec::new();
            let mut visualization = None;

            use tokio_stream::StreamExt;
            while let Some(frame) = stream.next().await {
                let frame = frame.map_err(|s| Error::GrpcStatus(s.message().to_string()))?;
                match frame.payload {
                    Some(query_proto::execute_query_result_frame::Payload::Schema(schema)) => {
                        // Flush previous table
                        if let Some((name, columns)) = current_schema.take() {
                            tables.push(Table {
                                name,
                                columns,
                                rows: std::mem::take(&mut current_rows),
                            });
                        }
                        let columns = schema
                            .columns
                            .into_iter()
                            .map(|c| Column {
                                name: c.name,
                                column_type: convert_column_type(c.r#type),
                            })
                            .collect();
                        current_schema = Some((schema.name, columns));
                    }
                    Some(query_proto::execute_query_result_frame::Payload::Batch(batch)) => {
                        for row in batch.rows {
                            let values = row.values.into_iter().map(convert_value).collect();
                            current_rows.push(values);
                        }
                    }
                    Some(query_proto::execute_query_result_frame::Payload::Progress(p)) => {
                        stats = Some(ExecutionStats {
                            rows_processed: p.rows_processed,
                            chunks_total: p.chunks_total,
                            chunks_scanned: p.chunks_scanned,
                            query_time_nanos: p.query_time_nanos,
                            chunk_scan_time_nanos: p.chunk_scan_time_nanos,
                        });
                    }
                    Some(query_proto::execute_query_result_frame::Payload::Error(e)) => {
                        return Err(Error::Query {
                            code: e.code,
                            title: e.title,
                            message: e.message,
                        });
                    }
                    Some(query_proto::execute_query_result_frame::Payload::Metadata(m)) => {
                        for pf in m.partial_failures {
                            partial_failures.push(PartialFailure {
                                segment_ids: pf.segment_ids,
                                message: pf.message,
                            });
                        }
                        for w in m.warnings {
                            warnings.push(QueryWarning {
                                kind: w.kind,
                                message: w.message,
                            });
                        }
                        if let Some(viz) = m.visualization {
                            if let Some(vt) = viz.visualization_type {
                                visualization = Some(VisualizationMetadata {
                                    visualization_type: vt,
                                    properties: viz.properties,
                                });
                            }
                        }
                    }
                    Some(query_proto::execute_query_result_frame::Payload::Done(_)) => break,
                    None => {}
                }
            }

            // Flush last table
            if let Some((name, columns)) = current_schema.take() {
                tables.push(Table {
                    name,
                    columns,
                    rows: current_rows,
                });
            }

            Ok(QueryResponse {
                tables,
                stats,
                warnings,
                partial_failures,
                visualization,
            })
        }
    }

    fn convert_column_type(proto_type: i32) -> ColumnType {
        match query_proto::ColumnType::try_from(proto_type) {
            Ok(query_proto::ColumnType::Bool) => ColumnType::Bool,
            Ok(query_proto::ColumnType::Int) => ColumnType::Int,
            Ok(query_proto::ColumnType::Long) => ColumnType::Long,
            Ok(query_proto::ColumnType::Real) => ColumnType::Real,
            Ok(query_proto::ColumnType::String) => ColumnType::String,
            Ok(query_proto::ColumnType::Datetime) => ColumnType::Datetime,
            Ok(query_proto::ColumnType::Timespan) => ColumnType::Timespan,
            Ok(query_proto::ColumnType::Guid) => ColumnType::Guid,
            Ok(query_proto::ColumnType::Dynamic)
            | Ok(query_proto::ColumnType::Unspecified)
            | Err(_) => ColumnType::Dynamic,
        }
    }

    fn convert_value(dyn_val: berserk_proto::BqlValue) -> Value {
        match dyn_val.value {
            Some(berserk_proto::bql_value::Value::NullValue(_)) | None => Value::Null,
            Some(berserk_proto::bql_value::Value::BoolValue(b)) => Value::Bool(b),
            Some(berserk_proto::bql_value::Value::IntValue(i)) => Value::Int(i),
            Some(berserk_proto::bql_value::Value::LongValue(l)) => Value::Long(l),
            Some(berserk_proto::bql_value::Value::RealValue(d)) => Value::Real(d),
            Some(berserk_proto::bql_value::Value::StringValue(s)) => Value::String(s),
            Some(berserk_proto::bql_value::Value::DatetimeValue(t)) => Value::Long(t as i64),
            Some(berserk_proto::bql_value::Value::TimespanValue(t)) => Value::Long(t as i64),
            Some(berserk_proto::bql_value::Value::ArrayValue(arr)) => {
                Value::Array(arr.values.into_iter().map(convert_value).collect())
            }
            Some(berserk_proto::bql_value::Value::BagValue(bag)) => {
                let map = bag
                    .properties
                    .into_iter()
                    .map(|(k, v)| (k, convert_value(v)))
                    .collect();
                Value::Object(map)
            }
        }
    }
}

#[cfg(feature = "grpc")]
pub use inner::*;
