use std::time::Duration;

/// Client configuration for connecting to a Berserk query service.
#[derive(Debug, Clone)]
pub struct Config {
    /// Query service endpoint (e.g., "http://localhost:9510")
    pub endpoint: String,
    /// Username sent as x-bzrk-username header
    pub username: Option<String>,
    /// Maximum time for a complete request
    pub timeout: Duration,
    /// Maximum time between streaming frames
    pub alive_timeout: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Client name sent as x-bzrk-client-name header
    pub client_name: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:9510".to_string(),
            username: None,
            timeout: Duration::from_secs(30),
            alive_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            client_name: Some("berserk-client-rust".to_string()),
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

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
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

    pub fn with_client_name(mut self, name: impl Into<String>) -> Self {
        self.client_name = Some(name.into());
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
