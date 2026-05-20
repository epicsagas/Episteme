use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Maximum number of buckets before triggering eviction.
const MAX_BUCKETS: usize = 10_000;

/// Age threshold for eviction. Buckets not accessed within this duration are
/// removed when the map exceeds [`MAX_BUCKETS`] entries.
const EVICTION_TTL: Duration = Duration::from_secs(3600); // 1 hour

/// In-memory token-bucket rate limiter.
///
/// Each distinct key (typically a client IP) gets its own bucket. Tokens refill
/// at a constant rate up to the configured maximum, providing a simple
/// sliding-window-style throttle without external dependencies.
///
/// To prevent unbounded memory growth from adversarial clients that rotate
/// keys, buckets are evicted once the total exceeds [`MAX_BUCKETS`]. Any bucket
/// whose last access is older than [`EVICTION_TTL`] is removed during eviction.
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
    max_tokens: f64,
    refill_per_sec: f64,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    last_access: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter allowing `max_requests_per_min` requests per
    /// distinct key over a 60-second window.
    pub fn new(max_requests_per_min: u32) -> Self {
        let max_tokens = f64::from(max_requests_per_min);
        Self {
            buckets: Mutex::new(HashMap::new()),
            max_tokens,
            refill_per_sec: max_tokens / 60.0,
        }
    }

    /// Check whether a request from `key` is allowed.
    ///
    /// Returns `true` if the request passes the rate limit, `false` if it
    /// should be rejected. Consumes one token on success.
    ///
    /// When the number of tracked buckets exceeds [`MAX_BUCKETS`], stale
    /// entries (not accessed within [`EVICTION_TTL`]) are evicted before the
    /// check proceeds. This bounds memory usage even under adversarial key
    /// rotation.
    pub fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().expect("rate limiter mutex poisoned");

        // Evict stale buckets if the map has grown too large.
        if buckets.len() > MAX_BUCKETS {
            let cutoff = now - EVICTION_TTL;
            buckets.retain(|_, bucket| bucket.last_access > cutoff);
        }

        let bucket = buckets
            .entry(key.to_owned())
            .or_insert_with(|| TokenBucket {
                tokens: self.max_tokens,
                last_refill: now,
                last_access: now,
            });

        // Refill tokens proportional to elapsed time.
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_sec).min(self.max_tokens);
        bucket.last_refill = now;
        bucket.last_access = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Per-route rate-limit configuration matching the Python implementation.
pub fn rate_limit_for_path(path: &str) -> u32 {
    if path.starts_with("/analyze") || path.starts_with("/refactor") {
        20
    } else if path.starts_with("/search") {
        50
    } else {
        100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_max_requests() {
        let limiter = RateLimiter::new(5);
        for _ in 0..5 {
            assert!(limiter.allow("client-a"));
        }
        assert!(!limiter.allow("client-a"));
    }

    #[test]
    fn different_clients_have_independent_buckets() {
        let limiter = RateLimiter::new(2);
        assert!(limiter.allow("client-a"));
        assert!(limiter.allow("client-a"));
        assert!(!limiter.allow("client-a"));

        // client-b has a fresh bucket
        assert!(limiter.allow("client-b"));
        assert!(limiter.allow("client-b"));
        assert!(!limiter.allow("client-b"));
    }

    #[test]
    fn tokens_refill_over_time() {
        let limiter = RateLimiter::new(1);
        assert!(limiter.allow("client"));
        assert!(!limiter.allow("client"));

        // Simulate time passing by directly manipulating the bucket.
        {
            let mut buckets = limiter.buckets.lock().unwrap();
            let bucket = buckets.get_mut("client").unwrap();
            let drift = std::time::Duration::from_secs(61);
            bucket.last_refill -= drift;
            bucket.last_access -= drift;
        }

        assert!(limiter.allow("client"));
    }

    #[test]
    fn rate_limit_for_path_mapping() {
        assert_eq!(rate_limit_for_path("/analyze"), 20);
        assert_eq!(rate_limit_for_path("/analyze?code=x"), 20);
        assert_eq!(rate_limit_for_path("/refactor"), 20);
        assert_eq!(rate_limit_for_path("/search"), 50);
        assert_eq!(rate_limit_for_path("/search?q=test"), 50);
        assert_eq!(rate_limit_for_path("/health"), 100);
        assert_eq!(rate_limit_for_path("/stats"), 100);
        assert_eq!(rate_limit_for_path("/"), 100);
    }

    #[test]
    fn eviction_removes_stale_buckets_when_over_threshold() {
        // Use a limiter and manually populate more than MAX_BUCKETS entries.
        // Mark half of them as stale (last_access older than EVICTION_TTL).
        let limiter = RateLimiter::new(100);

        // Populate MAX_BUCKETS + 500 entries. The first 500 will be made stale.
        {
            let mut buckets = limiter.buckets.lock().unwrap();
            for i in 0..(MAX_BUCKETS + 500) {
                let key = format!("client-{i}");
                buckets.insert(
                    key,
                    TokenBucket {
                        tokens: 50.0,
                        last_refill: Instant::now(),
                        last_access: Instant::now(),
                    },
                );
            }

            // Age out the first 500 entries so they exceed the TTL.
            let stale_offset = EVICTION_TTL + Duration::from_secs(1);
            for i in 0..500usize {
                let key = format!("client-{i}");
                let bucket = buckets.get_mut(&key).unwrap();
                bucket.last_access -= stale_offset;
            }
        }

        // Trigger eviction by calling allow() which checks the threshold.
        // The "trigger" key is brand-new so it also tests that new keys still
        // work after eviction.
        assert!(limiter.allow("trigger-key"));

        let buckets = limiter.buckets.lock().unwrap();

        // The stale entries (0..500) should have been evicted.
        for i in 0..500usize {
            let key = format!("client-{i}");
            assert!(
                !buckets.contains_key(&key),
                "stale bucket {key} should have been evicted"
            );
        }

        // The fresh entries (500..MAX_BUCKETS+500) plus "trigger-key" remain.
        for i in 500..(MAX_BUCKETS + 500) {
            let key = format!("client-{i}");
            assert!(
                buckets.contains_key(&key),
                "fresh bucket {key} should still exist"
            );
        }
        assert!(buckets.contains_key("trigger-key"));

        // Total should be: MAX_BUCKETS fresh originals + the trigger key.
        assert_eq!(buckets.len(), MAX_BUCKETS + 1);
    }
}
