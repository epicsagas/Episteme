//! Redis-backed response cache for the Episteme API.
//!
//! When the `redis-cache` feature is enabled, this module provides a
//! [`CacheManager`] that connects to Redis and caches expensive API responses
//! (analyze, search) with configurable TTL. When the feature is disabled, all
//! operations are no-ops.

#[cfg(feature = "redis-cache")]
use deadpool_redis::{Config as RedisPoolConfig, Pool, Runtime};
#[cfg(feature = "redis-cache")]
use redis::AsyncCommands;
#[cfg(feature = "redis-cache")]
use std::sync::Arc;
#[cfg(feature = "redis-cache")]
use tokio::sync::RwLock;

use md5::{Digest, Md5};
use serde_json::Value;

// ---------------------------------------------------------------------------
// CacheStats (always available)
// ---------------------------------------------------------------------------

/// Summary of the Redis cache state.
#[derive(Debug, serde::Serialize, Clone)]
pub struct CacheStats {
    pub enabled: bool,
    pub connected: bool,
    pub keys: u64,
    pub total_keys: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub memory: String,
}

// ---------------------------------------------------------------------------
// Cache key generation (always available)
// ---------------------------------------------------------------------------

/// Generate a deterministic cache key from a prefix and variadic string args.
///
/// The arguments are joined with `:` and then MD5-hashed so that arbitrarily
/// long inputs produce fixed-width keys.
pub fn cache_key(prefix: &str, args: &[&str]) -> String {
    let combined = args.join(":");
    let digest = Md5::digest(combined.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest.as_slice() {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{:02x}", b);
    }
    format!("{prefix}:{hex}")
}

// ===========================================================================
// Feature-gated implementation (redis-cache enabled)
// ===========================================================================

#[cfg(feature = "redis-cache")]
#[derive(Clone)]
pub struct CacheManager {
    pool: Arc<RwLock<Option<Pool>>>,
    enabled: bool,
    default_ttl: u64,
    prefix: String,
}

#[cfg(feature = "redis-cache")]
impl CacheManager {
    /// Create a new cache manager. When `enabled` is false, all operations
    /// become cheap no-ops (no Redis connection is ever opened).
    pub fn new(enabled: bool, default_ttl: u64) -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            enabled,
            default_ttl,
            prefix: "episteme".to_owned(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn is_connected(&self) -> bool {
        if !self.enabled {
            return false;
        }
        let pool = { self.pool.read().await.clone() };
        let Some(pool) = pool else {
            return false;
        };
        let mut conn = match pool.get().await {
            Ok(c) => c,
            Err(_) => return false,
        };
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .is_ok()
    }

    /// Connect to Redis at `redis://{host}:{port}/{db}`.
    ///
    /// Returns `Ok(())` silently when the cache is disabled.
    pub async fn connect(&self, host: &str, port: u16, db: u64) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        if self.pool.read().await.is_some() {
            return Ok(());
        }
        let url = format!("redis://{host}:{port}/{db}");
        let cfg = RedisPoolConfig::from_url(url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| e.to_string())?;
        let mut conn = pool.get().await.map_err(|e| e.to_string())?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        *self.pool.write().await = Some(pool);
        Ok(())
    }

    /// Close the Redis connection (sets the internal handle to `None`).
    pub async fn disconnect(&self) {
        *self.pool.write().await = None;
    }

    /// Retrieve a cached JSON value by key. Returns `None` when the cache is
    /// disabled, not connected, or the key does not exist.
    pub async fn get(&self, key: &str) -> Option<Value> {
        if !self.enabled {
            return None;
        }
        let pool = { self.pool.read().await.clone() }?;
        let full_key = format!("{}:{key}", self.prefix);
        let mut conn = pool.get().await.ok()?;
        let result: Option<String> = conn.get(&full_key).await.ok()?;
        result.and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Store a JSON value in the cache with an optional per-key TTL.
    /// Falls back to `self.default_ttl` when `ttl` is `None`.
    pub async fn set(&self, key: &str, value: &Value, ttl: Option<u64>) {
        if !self.enabled {
            return;
        }
        let full_key = format!("{}:{key}", self.prefix);
        let content = match serde_json::to_string(value) {
            Ok(s) => s,
            Err(_) => return,
        };
        let ttl_secs = ttl.unwrap_or(self.default_ttl);
        let Some(pool) = self.pool.read().await.clone() else {
            return;
        };
        if let Ok(mut conn) = pool.get().await {
            let _: Result<(), _> = conn.set_ex(&full_key, &content, ttl_secs).await;
        };
    }

    /// Delete all keys matching a prefix pattern using `SCAN` + `DEL`.
    pub async fn delete(&self, pattern: &str) {
        if !self.enabled {
            return;
        }
        let Some(pool) = self.pool.read().await.clone() else {
            return;
        };
        let full_pattern = format!("{}:{pattern}*", self.prefix);
        let Ok(mut conn) = pool.get().await else {
            return;
        };

        let mut keys = Vec::new();
        let mut cursor: u64 = 0;
        loop {
            let result: (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&full_pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .unwrap_or((0, Vec::new()));
            cursor = result.0;
            keys.extend(result.1);
            if cursor == 0 {
                break;
            }
        }
        if !keys.is_empty() {
            let _: Result<(), _> = conn.del(&keys).await;
        }
    }

    /// Return operational stats from the Redis instance.
    pub async fn get_stats(&self) -> CacheStats {
        if !self.enabled {
            return CacheStats {
                enabled: false,
                connected: false,
                keys: 0,
                total_keys: 0,
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                memory: String::new(),
            };
        }
        let Some(pool) = self.pool.read().await.clone() else {
            return CacheStats {
                enabled: false,
                connected: false,
                keys: 0,
                total_keys: 0,
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                memory: "disconnected".to_owned(),
            };
        };
        let Ok(mut conn) = pool.get().await else {
            return CacheStats {
                enabled: false,
                connected: false,
                keys: 0,
                total_keys: 0,
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                memory: "disconnected".to_owned(),
            };
        };

        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        let total_keys: u64 = redis::cmd("DBSIZE")
            .query_async(&mut conn)
            .await
            .unwrap_or(0);
        let episteme_keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("{}:*", self.prefix))
            .query_async(&mut conn)
            .await
            .unwrap_or_default();
        let keys = episteme_keys.len() as u64;
        let hits = info
            .lines()
            .find(|l| l.starts_with("keyspace_hits:"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let misses = info
            .lines()
            .find(|l| l.starts_with("keyspace_misses:"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let total = hits + misses;
        let hit_rate = if total == 0 {
            0.0
        } else {
            ((hits as f64 / total as f64) * 10000.0).round() / 100.0
        };

        let memory = info
            .lines()
            .find(|l| l.starts_with("used_memory_human:"))
            .and_then(|l| l.split(':').nth(1))
            .unwrap_or("unknown")
            .trim()
            .to_owned();

        CacheStats {
            enabled: true,
            connected: true,
            keys,
            total_keys,
            hits,
            misses,
            hit_rate,
            memory,
        }
    }
}

// ===========================================================================
// No-op fallback (redis-cache disabled)
// ===========================================================================

#[cfg(not(feature = "redis-cache"))]
/// Stub cache manager that does nothing (used when the `redis-cache` feature
/// is not enabled at compile time).
#[derive(Clone)]
#[allow(dead_code)]
pub struct CacheManager {
    enabled: bool,
}

#[cfg(not(feature = "redis-cache"))]
impl CacheManager {
    pub fn new(_enabled: bool, _default_ttl: u64) -> Self {
        Self { enabled: false }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    pub async fn is_connected(&self) -> bool {
        false
    }
    pub async fn connect(&self, _host: &str, _port: u16, _db: u64) -> Result<(), String> {
        Ok(())
    }
    pub async fn disconnect(&self) {}
    pub async fn get(&self, _key: &str) -> Option<Value> {
        None
    }
    pub async fn set(&self, _key: &str, _value: &Value, _ttl: Option<u64>) {}
    pub async fn delete(&self, _pattern: &str) {}
    pub async fn get_stats(&self) -> CacheStats {
        CacheStats {
            enabled: false,
            connected: false,
            keys: 0,
            total_keys: 0,
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
            memory: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_deterministic() {
        let k1 = cache_key("analyze", &["fn main() {}", "rust"]);
        let k2 = cache_key("analyze", &["fn main() {}", "rust"]);
        assert_eq!(k1, k2, "identical inputs must produce identical keys");
    }

    #[test]
    fn cache_key_differs_for_different_inputs() {
        let k1 = cache_key("analyze", &["fn main() {}", "rust"]);
        let k2 = cache_key("analyze", &["fn main() {}", "python"]);
        assert_ne!(k1, k2, "different inputs must produce different keys");
    }

    #[test]
    fn cache_key_includes_prefix() {
        let k = cache_key("search", &["god class"]);
        assert!(
            k.starts_with("search:"),
            "key should start with the prefix: got {k}"
        );
    }

    #[tokio::test]
    async fn disabled_cache_returns_none() {
        let cm = CacheManager::new(false, 3600);
        assert!(cm.get("any-key").await.is_none());
    }

    #[tokio::test]
    async fn disabled_cache_stats_report_disabled() {
        let cm = CacheManager::new(false, 3600);
        let stats = cm.get_stats().await;
        assert!(!stats.enabled);
        assert_eq!(stats.keys, 0);
    }

    #[tokio::test]
    async fn disabled_cache_set_is_noop() {
        let cm = CacheManager::new(false, 3600);
        cm.set("k", &serde_json::json!({"a": 1}), None).await;
        // Should not panic or block.
    }
}
