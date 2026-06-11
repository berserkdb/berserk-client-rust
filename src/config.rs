use std::time::Duration;

/// Client configuration for connecting to a Berserk gateway.
#[derive(Debug, Clone)]
pub struct Config {
    /// Gateway endpoint (e.g., "https://berserk.example.com" or
    /// "http://localhost:9500").
    pub endpoint: String,
    /// Bearer token sent as `authorization` on every call — a CLI
    /// access token or service-principal token minted by the gateway.
    /// Unauthenticated calls are rejected by the gateway.
    pub token: Option<String>,
    /// Path prefix the gateway mounts the gRPC surface under. Defaults
    /// to "/api/grpc". Set to "" when connecting directly to a query
    /// service (in-cluster / dev).
    pub grpc_path_prefix: String,
    /// Maximum time for a complete request
    pub timeout: Duration,
    /// Maximum time between streaming frames
    pub alive_timeout: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Database to resolve unqualified table names against. Sent on every
    /// ExecuteQueryRequest as `database.name`. Defaults to "default".
    pub database: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9500".to_string(),
            token: None,
            grpc_path_prefix: "/api/grpc".to_string(),
            timeout: Duration::from_secs(30),
            alive_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            database: "default".to_string(),
        }
    }
}

impl Config {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..Default::default()
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn with_grpc_path_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.grpc_path_prefix = prefix.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_alive_timeout(mut self, alive_timeout: Duration) -> Self {
        self.alive_timeout = alive_timeout;
        self
    }

    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = database.into();
        self
    }

    /// Normalize endpoint — ensure it has a scheme prefix.
    pub(crate) fn normalized_endpoint(&self) -> String {
        if self.endpoint.starts_with("http://") || self.endpoint.starts_with("https://") {
            self.endpoint.clone()
        } else {
            format!("http://{}", self.endpoint)
        }
    }
}
