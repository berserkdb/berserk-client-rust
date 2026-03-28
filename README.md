# berserk-client-rust

Rust client library for the [Berserk](https://berserk.dev) observability platform.

## Features

- **gRPC** (default) — Native gRPC streaming via tonic
- **HTTP** — ADX v2 REST endpoint compatible with Kusto tooling

## Quick Start

```toml
[dependencies]
berserk-client = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### gRPC (default)

```rust
use berserk_client::{Config, GrpcClient};

#[tokio::main]
async fn main() -> Result<(), berserk_client::Error> {
    let client = GrpcClient::new(Config::new("http://localhost:9510"));
    let response = client.query(
        "Logs | where severity == 'error' | take 10",
        None, None, "UTC",
    ).await?;

    for table in &response.tables {
        println!("Table: {} ({} rows)", table.name, table.rows.len());
    }
    Ok(())
}
```

### HTTP (ADX v2)

```toml
[dependencies]
berserk-client = { version = "0.1", default-features = false, features = ["http"] }
```

```rust
use berserk_client::{Config, HttpClient};

#[tokio::main]
async fn main() -> Result<(), berserk_client::Error> {
    let client = HttpClient::new(Config::new("http://localhost:9510"));
    let response = client.query("print v = 1").await?;
    println!("{:?}", response.tables);
    Ok(())
}
```

## License

Apache-2.0
