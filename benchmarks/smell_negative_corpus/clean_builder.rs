// Guards against: Long Method FP, Message Chains FP.
// Builder pattern with chained setters is idiomatic Rust — each method is trivial,
// and chaining is the intended usage, not a "message chain" smell.

/// Builder for configuring HTTP client connections.
pub struct HttpClientBuilder {
    base_url: String,
    timeout_ms: u64,
    max_retries: u32,
    user_agent: String,
    follow_redirects: bool,
    verify_ssl: bool,
    connect_timeout_ms: u64,
}

/// HTTP client built from a configured builder.
pub struct HttpClient {
    base_url: String,
    timeout_ms: u64,
    max_retries: u32,
    user_agent: String,
    follow_redirects: bool,
    verify_ssl: bool,
    connect_timeout_ms: u64,
}

impl HttpClientBuilder {
    /// Create a new builder targeting the given base URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            timeout_ms: 30_000,
            max_retries: 3,
            user_agent: String::from("http-client/1.0"),
            follow_redirects: true,
            verify_ssl: true,
            connect_timeout_ms: 5_000,
        }
    }

    /// Set the overall request timeout in milliseconds.
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set the maximum number of retry attempts.
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Override the default User-Agent header.
    pub fn user_agent(mut self, ua: &str) -> Self {
        self.user_agent = ua.to_string();
        self
    }

    /// Whether to follow HTTP redirects (3xx responses).
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Whether to verify TLS certificates.
    pub fn verify_ssl(mut self, verify: bool) -> Self {
        self.verify_ssl = verify;
        self
    }

    /// Set the connection-phase timeout in milliseconds.
    pub fn connect_timeout(mut self, ms: u64) -> Self {
        self.connect_timeout_ms = ms;
        self
    }

    /// Consume the builder and produce an HttpClient.
    pub fn build(self) -> HttpClient {
        HttpClient {
            base_url: self.base_url,
            timeout_ms: self.timeout_ms,
            max_retries: self.max_retries,
            user_agent: self.user_agent,
            follow_redirects: self.follow_redirects,
            verify_ssl: self.verify_ssl,
            connect_timeout_ms: self.connect_timeout_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_with_defaults() {
        let client = HttpClientBuilder::new("https://example.com").build();
        assert_eq!(client.timeout_ms, 30_000);
        assert!(client.follow_redirects);
    }

    #[test]
    fn builder_with_overrides() {
        let client = HttpClientBuilder::new("https://example.com")
            .timeout(10_000)
            .max_retries(5)
            .user_agent("test/0.1")
            .follow_redirects(false)
            .verify_ssl(false)
            .connect_timeout(2_000)
            .build();
        assert_eq!(client.timeout_ms, 10_000);
        assert_eq!(client.max_retries, 5);
        assert!(!client.follow_redirects);
    }
}
