// Guards against: Long Method FP.
// A constructor that reads many environment variables is inherently linear and
// cannot be meaningfully broken into smaller helpers — each line is one field.

use std::env;

/// Application-level configuration loaded from environment variables.
pub struct AppConfig {
    pub app_name: String,
    pub environment: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_pool_size: u32,
    pub redis_url: String,
    pub log_level: String,
    pub session_secret: String,
    pub session_ttl_secs: u64,
    pub max_upload_bytes: u64,
    pub enable_cors: bool,
}

impl AppConfig {
    /// Load configuration from environment variables, with sensible defaults.
    ///
    /// Each field maps to one env var — the linear structure is intentional
    /// and cannot be simplified without obscuring which var maps where.
    pub fn from_env() -> Self {
        Self {
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "my-app".into()),
            environment: env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("APP_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/myapp".into()),
            database_pool_size: env::var("DB_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            session_secret: env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".into()),
            session_ttl_secs: env::var("SESSION_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            max_upload_bytes: env::var("MAX_UPLOAD_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10 * 1024 * 1024),
            enable_cors: env::var("ENABLE_CORS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_returns_defaults() {
        // Clear potentially set vars for test isolation
        env::remove_var("APP_NAME");
        let config = AppConfig::from_env();
        assert_eq!(config.app_name, "my-app");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
    }
}
