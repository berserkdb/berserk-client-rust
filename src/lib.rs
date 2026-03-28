//! Berserk query client for Rust.
//!
//! Provides gRPC and HTTP transports for querying the Berserk observability platform.
//!
//! # Features
//!
//! - `grpc` (default) — gRPC client using tonic
//! - `http` — HTTP client using the ADX v2 REST endpoint
//!
//! # Example
//!
//! ```rust,no_run
//! use berserk_client::{Config, GrpcClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), berserk_client::Error> {
//!     let client = GrpcClient::new(Config::new("http://localhost:9510"));
//!     let response = client.query("print v = 1", None, None, "UTC").await?;
//!     println!("{:?}", response.tables);
//!     Ok(())
//! }
//! ```

mod config;
mod error;
pub mod types;

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "http")]
pub mod http;

pub use config::Config;
pub use error::{Error, Result};
pub use types::*;

#[cfg(feature = "grpc")]
pub use grpc::GrpcClient;

#[cfg(feature = "http")]
pub use http::HttpClient;
