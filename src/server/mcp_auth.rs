/// Constant-time string comparison to prevent timing attacks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        // Still do a dummy comparison to avoid leaking length via timing.
        let mut _dummy: u8 = 0;
        for b in a.bytes() {
            _dummy |= b ^ b;
        }
        return false;
    }
    let mut result: u8 = 0;
    for (xa, xb) in a.bytes().zip(b.bytes()) {
        result |= xa ^ xb;
    }
    result == 0
}

/// Validate a provided API key against a list of allowed keys.
///
/// Returns `true` when `provided` matches any entry in `allowed_keys`,
/// or when `allowed_keys` is empty (no auth required).
pub fn validate_api_key(provided: &str, allowed_keys: &[String]) -> bool {
    if allowed_keys.is_empty() {
        return true;
    }
    allowed_keys.iter().any(|k| constant_time_eq(provided, k))
}

/// Parse a comma-separated `EPISTEME_API_KEYS` env value into a vec of trimmed keys.
///
/// Empty segments are discarded.
pub fn parse_api_keys(env_val: &str) -> Vec<String> {
    env_val
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Generate a bearer token in the format `epis-` + 32 hex characters.
pub fn generate_token() -> String {
    use rand::Rng;
    let bytes: [u8; 16] = rand::rng().random();
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("epis-{hex}")
}

/// Check whether a host address is a loopback / local-only address.
pub fn is_localhost(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allowed_means_no_auth() {
        assert!(validate_api_key("anything", &[]));
    }

    #[test]
    fn valid_key_accepted() {
        let keys: Vec<String> = vec!["abc123".into(), "def456".into()];
        assert!(validate_api_key("abc123", &keys));
    }

    #[test]
    fn invalid_key_rejected() {
        let keys: Vec<String> = vec!["abc123".into()];
        assert!(!validate_api_key("wrong", &keys));
    }

    #[test]
    fn parse_keys_splits_and_trims() {
        let parsed = parse_api_keys(" key1 , key2 ,,  key3  ");
        assert_eq!(parsed, vec!["key1", "key2", "key3"]);
    }

    #[test]
    fn parse_keys_empty_string() {
        let parsed = parse_api_keys("");
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_keys_only_commas() {
        let parsed = parse_api_keys(",,,");
        assert!(parsed.is_empty());
    }

    #[test]
    fn generate_token_format() {
        let token = generate_token();
        assert!(token.starts_with("epis-"));
        assert_eq!(token.len(), 5 + 32); // "epis-" + 32 hex chars
        let hex_part = &token[5..];
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_tokens_are_unique() {
        let a = generate_token();
        let b = generate_token();
        assert_ne!(a, b);
    }

    #[test]
    fn constant_time_eq_matches() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("short", "muchlonger"));
    }

    #[test]
    fn is_localhost_checks() {
        assert!(is_localhost("127.0.0.1"));
        assert!(is_localhost("::1"));
        assert!(is_localhost("localhost"));
        assert!(!is_localhost("0.0.0.0"));
        assert!(!is_localhost("192.168.1.1"));
    }
}
