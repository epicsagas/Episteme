/// Validate a provided API key against a list of allowed keys.
///
/// Returns `true` when `provided` matches any entry in `allowed_keys`,
/// or when `allowed_keys` is empty (no auth required).
pub fn validate_api_key(provided: &str, allowed_keys: &[String]) -> bool {
    if allowed_keys.is_empty() {
        return true;
    }
    allowed_keys.iter().any(|k| k == provided)
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
}
