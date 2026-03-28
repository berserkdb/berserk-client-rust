/// Errors from the Berserk client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// gRPC transport/connection error
    #[error("gRPC connection error: {0}")]
    GrpcConnection(String),

    /// gRPC status error from server
    #[error("gRPC error: {0}")]
    GrpcStatus(String),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(String),

    /// Query execution error returned by the server
    #[error("query error [{code}]: {message}")]
    Query {
        code: String,
        title: String,
        message: String,
    },

    /// Response parsing error
    #[error("parse error: {0}")]
    Parse(String),

    /// Timeout
    #[error("timeout: {0}")]
    Timeout(String),
}

pub type Result<T> = std::result::Result<T, Error>;
