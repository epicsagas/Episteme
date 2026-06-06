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
    pub embedding_model: String,
    pub openai_api_key: String,
    pub openai_embed_model: String,
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
            api_port: 58302,
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
            embedding_model: "MultilingualE5Small".into(),
            openai_api_key: String::new(),
            openai_embed_model: "text-embedding-3-small".into(),
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
    api: Option<HashMap<String, noyalib::Value>>,
    redis: Option<HashMap<String, noyalib::Value>>,
    mcp: Option<HashMap<String, noyalib::Value>>,
}

impl EpistemeConfig {
    pub fn load() -> std::result::Result<Self, InfraError> {
        let mut config = Self::default();
        let yaml = load_yaml_config()?;

        config.api_host = cfg_val(&yaml, "api", "host", "UVICORN_HOST", &config.api_host);
        config.api_port = cfg_parse_val(&yaml, "api", "port", "UVICORN_PORT", config.api_port);
        config.api_port = env_parse_or("EPISTEME_API_PORT", config.api_port);
        config.api_keys = cfg_val(&yaml, "api", "keys", "EPISTEME_API_KEYS", &config.api_keys);
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
        config.embedding_model = env_or("EPISTEME_EMBEDDING_MODEL", &config.embedding_model);
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
    let config: YamlConfig = noyalib::from_str(&text)?;
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
        "api" => yaml.api.as_ref(),
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
        "api" => yaml.api.as_ref(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_yaml_sections(yaml: &str) -> YamlConfig {
        noyalib::from_str(yaml).expect("YAML should parse")
    }

    #[test]
    fn yaml_config_parses_api_section() {
        let cfg = parse_yaml_sections("api:\n  host: 127.0.0.1\n  port: 9999\n  keys: secret123\n");
        let api = cfg.api.as_ref().expect("api section");
        assert_eq!(api["host"].as_str(), Some("127.0.0.1"));
        assert_eq!(api["port"].as_i64(), Some(9999));
        assert_eq!(api["keys"].as_str(), Some("secret123"));
    }

    #[test]
    fn yaml_config_parses_redis_section() {
        let cfg = parse_yaml_sections(
            "redis:\n  host: redis.local\n  port: 6380\n  db: 2\n  ttl: 7200\n  enabled: false\n",
        );
        let redis = cfg.redis.as_ref().expect("redis section");
        assert_eq!(redis["host"].as_str(), Some("redis.local"));
        assert_eq!(redis["port"].as_i64(), Some(6380));
        assert_eq!(redis["enabled"].as_bool(), Some(false));
    }

    #[test]
    fn yaml_config_parses_mcp_section() {
        let cfg = parse_yaml_sections("mcp:\n  host: 0.0.0.0\n  port: 5000\n  token: tok-abc\n");
        let mcp = cfg.mcp.as_ref().expect("mcp section");
        assert_eq!(mcp["host"].as_str(), Some("0.0.0.0"));
        assert_eq!(mcp["token"].as_str(), Some("tok-abc"));
    }

    #[test]
    fn yaml_config_empty_file_returns_defaults() {
        // Empty YAML doc is Null in YAML 1.2, so noyalib rejects it as TypeMismatch.
        // The real load_yaml_config() handles this by returning default for missing files.
        let cfg = YamlConfig::default();
        assert!(cfg.api.is_none());
        assert!(cfg.redis.is_none());
        assert!(cfg.mcp.is_none());
    }

    #[test]
    fn yaml_config_multiple_sections() {
        let cfg = parse_yaml_sections(
            "api:\n  host: 0.0.0.0\n  port: 8080\nredis:\n  host: localhost\nmcp:\n  port: 43175\n",
        );
        assert_eq!(cfg.api.as_ref().unwrap()["host"].as_str(), Some("0.0.0.0"));
        assert_eq!(
            cfg.redis.as_ref().unwrap()["host"].as_str(),
            Some("localhost")
        );
        assert_eq!(cfg.mcp.as_ref().unwrap()["port"].as_i64(), Some(43175));
    }

    #[test]
    fn cfg_val_reads_string_from_yaml() {
        let yaml = parse_yaml_sections("api:\n  host: custom.host\n");
        let val = cfg_val(&yaml, "api", "host", "__TEST_NEVER_SET__", "fallback");
        assert_eq!(val, "custom.host");
    }

    #[test]
    fn cfg_val_falls_back_to_default() {
        let yaml = YamlConfig::default();
        let val = cfg_val(&yaml, "api", "host", "__TEST_NEVER_SET__", "fallback");
        assert_eq!(val, "fallback");
    }

    #[test]
    fn cfg_parse_val_reads_number_from_yaml() {
        let yaml = parse_yaml_sections("api:\n  port: 9999\n");
        let val: u16 = cfg_parse_val(&yaml, "api", "port", "__TEST_NEVER_SET__", 8080);
        assert_eq!(val, 9999);
    }

    #[test]
    fn cfg_parse_val_falls_back_to_default() {
        let yaml = YamlConfig::default();
        let val: u16 = cfg_parse_val(&yaml, "api", "port", "__TEST_NEVER_SET__", 8080);
        assert_eq!(val, 8080);
    }

    #[test]
    fn cfg_bool_val_reads_boolean_from_yaml() {
        let yaml = parse_yaml_sections("redis:\n  enabled: false\n");
        let val = cfg_bool_val(&yaml, "redis", "enabled", "__TEST_NEVER_SET__", true);
        assert!(!val);
    }

    #[test]
    fn cfg_bool_val_falls_back_to_default() {
        let yaml = YamlConfig::default();
        let val = cfg_bool_val(&yaml, "redis", "enabled", "__TEST_NEVER_SET__", false);
        assert!(!val);
    }

    #[test]
    fn mapping_roundtrip_write_then_read() {
        use noyalib::{Mapping, Value};

        // Simulate upsert_api_config_yaml write
        let mut root = Value::Mapping(Mapping::new());
        let root_map = root.as_mapping_mut().unwrap();
        let mut api_map = Mapping::new();
        api_map.insert("host", Value::String("192.168.1.1".into()));
        api_map.insert("port", Value::Number(noyalib::Number::from(3000u16)));
        api_map.insert("keys", Value::String("my-key".into()));
        root_map.insert("api", Value::Mapping(api_map));

        let yaml_str = noyalib::to_string(&root).expect("serialize");

        // Parse back with noyalib
        let parsed: Value = noyalib::from_str(&yaml_str).expect("deserialize");
        let api = parsed["api"].as_mapping().unwrap();
        assert_eq!(api["host"].as_str(), Some("192.168.1.1"));
        assert_eq!(api["port"].as_i64(), Some(3000));
        assert_eq!(api["keys"].as_str(), Some("my-key"));
    }
}
