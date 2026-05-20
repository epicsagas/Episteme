use crate::adapters::error::InfraError;
use crate::adapters::paths::episteme_home;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemeConfig {
    pub api_host: String,
    pub api_port: u16,
    pub api_keys: String,
    pub log_level: String,
    pub enable_json_logging: bool,
    pub enable_debug_endpoints: bool,
    pub mcp_host: String,
    pub mcp_port: u16,
    pub mcp_token: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_db: u16,
    pub redis_ttl: u64,
    pub redis_enabled: bool,
    pub embedding_provider: String,
    pub openai_api_key: String,
    pub openai_embed_model: String,
    pub openai_embed_dim: usize,
    pub telemetry_enabled: bool,
    pub posthog_api_key: String,
    pub posthog_host: String,
    pub sentry_dsn: String,
    pub cors_origins: String,
    pub web_port: u16,
}

impl Default for EpistemeConfig {
    fn default() -> Self {
        Self {
            api_host: "0.0.0.0".into(),
            api_port: 8000,
            api_keys: String::new(),
            log_level: "INFO".into(),
            enable_json_logging: true,
            enable_debug_endpoints: false,
            mcp_host: "127.0.0.1".into(),
            mcp_port: 43175,
            mcp_token: String::new(),
            redis_host: "localhost".into(),
            redis_port: 6379,
            redis_db: 0,
            redis_ttl: 3600,
            redis_enabled: true,
            embedding_provider: "local".into(),
            openai_api_key: String::new(),
            openai_embed_model: "text-embedding-3-small".into(),
            openai_embed_dim: 1536,
            telemetry_enabled: true,
            posthog_api_key: String::new(),
            posthog_host: "https://app.posthog.com".into(),
            sentry_dsn: String::new(),
            cors_origins: String::new(),
            web_port: 8080,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct YamlConfig {
    redis: Option<HashMap<String, serde_yaml::Value>>,
    mcp: Option<HashMap<String, serde_yaml::Value>>,
}

impl EpistemeConfig {
    pub fn load() -> std::result::Result<Self, InfraError> {
        let mut config = Self::default();
        let yaml = load_yaml_config()?;

        config.api_host = env_or("UVICORN_HOST", &config.api_host);
        config.api_port = env_parse_or("UVICORN_PORT", config.api_port);
        config.api_keys = env_or("EPISTEME_API_KEYS", &config.api_keys);
        config.log_level = env_or("LOG_LEVEL", &config.log_level);
        config.enable_json_logging = env_bool_or("ENABLE_JSON_LOGGING", config.enable_json_logging);
        config.enable_debug_endpoints =
            env_bool_or("ENABLE_DEBUG_ENDPOINTS", config.enable_debug_endpoints);
        config.mcp_host = cfg_val(&yaml, "mcp", "host", "EPISTEME_MCP_HOST", &config.mcp_host);
        config.mcp_port = cfg_parse_val(&yaml, "mcp", "port", "EPISTEME_MCP_PORT", config.mcp_port);
        config.mcp_token = cfg_val(
            &yaml,
            "mcp",
            "token",
            "EPISTEME_MCP_TOKEN",
            &config.mcp_token,
        );

        config.redis_host = cfg_val(
            &yaml,
            "redis",
            "host",
            "EPISTEME_REDIS_HOST",
            &config.redis_host,
        );
        config.redis_port = cfg_parse_val(
            &yaml,
            "redis",
            "port",
            "EPISTEME_REDIS_PORT",
            config.redis_port,
        );
        config.redis_db = cfg_parse_val(&yaml, "redis", "db", "EPISTEME_REDIS_DB", config.redis_db);
        config.redis_ttl = cfg_parse_val(
            &yaml,
            "redis",
            "ttl",
            "EPISTEME_REDIS_TTL",
            config.redis_ttl,
        );
        config.redis_enabled = cfg_bool_val(
            &yaml,
            "redis",
            "enabled",
            "EPISTEME_REDIS_ENABLED",
            config.redis_enabled,
        );

        config.embedding_provider =
            env_or("EPISTEME_EMBEDDING_PROVIDER", &config.embedding_provider);
        config.openai_api_key = env_or("OPENAI_API_KEY", &config.openai_api_key);
        config.openai_embed_model =
            env_or("EPISTEME_OPENAI_EMBED_MODEL", &config.openai_embed_model);
        config.telemetry_enabled =
            env_bool_or("EPISTEME_TELEMETRY_ENABLED", config.telemetry_enabled);
        config.posthog_api_key = env_or("EPISTEME_POSTHOG_API_KEY", &config.posthog_api_key);
        config.posthog_host = env_or("EPISTEME_POSTHOG_HOST", &config.posthog_host);
        config.sentry_dsn = env_or("EPISTEME_SENTRY_DSN", &config.sentry_dsn);
        config.cors_origins = env_or("EPISTEME_CORS_ORIGINS", &config.cors_origins);
        config.web_port = env_parse_or("EPISTEME_WEB_PORT", config.web_port);

        Ok(config)
    }
}

fn load_yaml_config() -> std::result::Result<YamlConfig, InfraError> {
    let path = episteme_home().join("config.yaml");
    if !path.exists() {
        return Ok(YamlConfig::default());
    }
    let text = std::fs::read_to_string(&path)?;
    let config: YamlConfig = serde_yaml::from_str(&text)?;
    Ok(config)
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn env_parse_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_bool_or(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(default)
}

fn cfg_val(yaml: &YamlConfig, section: &str, key: &str, env_var: &str, default: &str) -> String {
    if let Ok(v) = std::env::var(env_var) {
        return v;
    }
    let map = match section {
        "redis" => yaml.redis.as_ref(),
        "mcp" => yaml.mcp.as_ref(),
        _ => None,
    };
    map.and_then(|m| m.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| default.to_owned())
}

fn cfg_parse_val<T: std::str::FromStr>(
    yaml: &YamlConfig,
    section: &str,
    key: &str,
    env_var: &str,
    default: T,
) -> T {
    if let Ok(v) = std::env::var(env_var)
        && let Ok(parsed) = v.parse()
    {
        return parsed;
    }
    let map = match section {
        "redis" => yaml.redis.as_ref(),
        "mcp" => yaml.mcp.as_ref(),
        _ => None,
    };
    map.and_then(|m| m.get(key))
        .and_then(|v| {
            if let Some(i) = v.as_i64() {
                i.to_string().parse().ok()
            } else {
                v.as_str().and_then(|s| s.parse().ok())
            }
        })
        .unwrap_or(default)
}

fn cfg_bool_val(
    yaml: &YamlConfig,
    _section: &str,
    key: &str,
    env_var: &str,
    default: bool,
) -> bool {
    if let Ok(v) = std::env::var(env_var) {
        return v.to_lowercase() == "true";
    }
    yaml.redis
        .as_ref()
        .and_then(|m| m.get(key))
        .map(|v| {
            v.as_bool().unwrap_or_else(|| {
                v.as_str()
                    .map(|s| s.to_lowercase() == "true")
                    .unwrap_or(default)
            })
        })
        .unwrap_or(default)
}
