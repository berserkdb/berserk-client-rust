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
// The example uses the gRPC transport — compile it as a doc test only
// when that feature is on (an http-only build couldn't resolve
// `GrpcClient`, and a cfg inside the doctest would silently skip it).
#![cfg_attr(feature = "grpc", doc = "```rust,no_run")]
#![cfg_attr(not(feature = "grpc"), doc = "```rust,ignore")]
//! use berserk_client::{Config, GrpcClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), berserk_client::Error> {
//!     let config = Config::new("https://berserk.example.com")
//!         .with_token(std::env::var("BERSERK_TOKEN").unwrap());
//!     let client = GrpcClient::new(config);
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
