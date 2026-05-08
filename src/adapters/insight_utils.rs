//! Shared utilities for insight operations (used by both CLI and MCP handlers).

use crate::adapters::user_graph_store::UserGraphStore;
use crate::ports::graph::MutableGraphRepository;

/// Produce an ISO-8601 UTC timestamp without depending on chrono.
pub fn format_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let (year, month, day) = days_to_ymd(days_since_epoch);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day).
pub fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

pub fn is_leap(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// Atomically generate the next TK-xxx ID using the SQLite sequence counter.
/// Falls back to scanning existing IDs if the atomic method is unavailable.
pub fn next_insight_id_atomic(store: &UserGraphStore) -> Result<String, String> {
    store.next_insight_id_atomic()
}

/// Compute the next TK-xxx ID by scanning existing user entity IDs (fallback).
pub fn next_insight_id(user_store: &dyn MutableGraphRepository) -> String {
    let ids = user_store.all_user_entity_ids();
    let max_num = ids
        .iter()
        .filter_map(|id| id.strip_prefix("TK-").and_then(|n| n.parse::<u32>().ok()))
        .max()
        .unwrap_or(0);
    format!("TK-{:03}", max_num + 1)
}

/// Parse a comma-separated string into a trimmed, non-empty Vec.
pub fn parse_comma_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Truncate text to `max_chars` Unicode characters, appending "..." if truncated.
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let truncated: String = text.chars().take(max_chars - 3).collect();
    format!("{truncated}...")
}

/// Derive a short title from insight text (first line, max 80 chars).
pub fn truncate_title(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or(text);
    truncate_text(first_line, 80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp_produces_iso8601() {
        let ts = format_timestamp();
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        let parts: Vec<&str> = ts.split('T').collect();
        assert_eq!(parts.len(), 2);
        let date_parts: Vec<&str> = parts[0].split('-').collect();
        assert_eq!(date_parts.len(), 3);
    }

    #[test]
    fn truncate_title_short_text_unchanged() {
        assert_eq!(truncate_title("Short text"), "Short text");
    }

    #[test]
    fn truncate_title_long_text_truncates() {
        let long = "a".repeat(100);
        let truncated = truncate_title(&long);
        assert!(truncated.len() <= 83); // 80 max but could be less with multi-byte ellipsis
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn truncate_title_multibyte_safe() {
        let korean = "한글".repeat(50);
        let truncated = truncate_title(&korean);
        // Should not panic and should end with "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn truncate_text_exact_length() {
        assert_eq!(truncate_text("hello", 5), "hello");
    }

    #[test]
    fn parse_comma_list_basic() {
        assert_eq!(
            parse_comma_list("a, b, c"),
            vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
        );
    }

    #[test]
    fn parse_comma_list_empty_entries() {
        assert_eq!(
            parse_comma_list("a,,b,"),
            vec!["a".to_owned(), "b".to_owned()]
        );
    }

    #[test]
    fn days_to_ymd_known_date() {
        // 2026-01-01 = 20454 days since epoch
        let (y, m, d) = days_to_ymd(20454);
        assert_eq!(y, 2026);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
    }
}
