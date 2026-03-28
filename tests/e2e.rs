//! End-to-end tests against a live Berserk cluster.
//!
//! Set BERSERK_ENDPOINT to run (e.g., BERSERK_ENDPOINT=http://localhost:9510).
//! Skipped when the env var is absent.

fn endpoint() -> Option<String> {
    std::env::var("BERSERK_ENDPOINT").ok()
}

macro_rules! require_endpoint {
    () => {
        match endpoint() {
            Some(ep) => ep,
            None => {
                eprintln!("BERSERK_ENDPOINT not set, skipping");
                return Ok(());
            }
        }
    };
}

#[cfg(feature = "grpc")]
mod grpc {
    use super::*;
    use berserk_client::{ColumnType, Config, Error, GrpcClient, Value};

    #[tokio::test]
    async fn simple_query() -> Result<(), Box<dyn std::error::Error>> {
        let ep = require_endpoint!();
        let client = GrpcClient::new(Config::new(&ep));
        let resp = client.query("print v = 1", None, None, "UTC").await?;

        assert_eq!(resp.tables.len(), 1);
        let table = &resp.tables[0];
        assert_eq!(table.columns.len(), 1);
        assert_eq!(table.columns[0].name, "v");
        assert_eq!(table.columns[0].column_type, ColumnType::Long);
        assert_eq!(table.rows.len(), 1);
        assert!(matches!(table.rows[0][0], Value::Long(1)));
        Ok(())
    }

    #[tokio::test]
    async fn invalid_query() -> Result<(), Box<dyn std::error::Error>> {
        let ep = require_endpoint!();
        let client = GrpcClient::new(Config::new(&ep));
        let result = client.query("this is not valid kql!!!", None, None, "UTC").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::Query { .. }), "expected Query error, got: {err}");
        Ok(())
    }

    #[tokio::test]
    async fn multi_column() -> Result<(), Box<dyn std::error::Error>> {
        let ep = require_endpoint!();
        let client = GrpcClient::new(Config::new(&ep));
        let resp = client.query(r#"print a = 1, b = "hello", c = true"#, None, None, "UTC").await?;

        let table = &resp.tables[0];
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.columns[0].name, "a");
        assert_eq!(table.columns[1].name, "b");
        assert_eq!(table.columns[2].name, "c");
        assert_eq!(table.rows.len(), 1);
        assert!(matches!(table.rows[0][0], Value::Long(1)));
        assert!(matches!(&table.rows[0][1], Value::String(s) if s == "hello"));
        assert!(matches!(table.rows[0][2], Value::Bool(true)));
        Ok(())
    }
}

#[cfg(feature = "http")]
mod http {
    use super::*;
    use berserk_client::{Config, HttpClient};

    #[tokio::test]
    async fn simple_query() -> Result<(), Box<dyn std::error::Error>> {
        let ep = require_endpoint!();
        let client = HttpClient::new(Config::new(&ep));
        let resp = client.query("print v = 1").await?;

        assert_eq!(resp.tables.len(), 1);
        assert_eq!(resp.tables[0].rows.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn invalid_query() -> Result<(), Box<dyn std::error::Error>> {
        let ep = require_endpoint!();
        let client = HttpClient::new(Config::new(&ep));
        let result = client.query("this is not valid kql!!!").await;

        assert!(result.is_err());
        Ok(())
    }
}
