//! Date parser supporting RFC-2822 and ISO-8601 formats.
//!
//! Returns Unix timestamp in seconds.

use chrono::DateTime;

/// Parses a date string into Unix seconds.
///
/// Tries RFC-2822 first (standard RSS), then ISO-8601 / RFC-3339 as fallback.
/// Returns `None` for empty or unparseable strings.
///
/// # Examples
///
/// ```
/// use stophammer_parser::date::parse_date;
///
/// // RFC-2822
/// assert_eq!(parse_date("Mon, 01 Jan 2024 00:00:00 +0000"), Some(1_704_067_200));
///
/// // ISO-8601
/// assert_eq!(parse_date("2024-01-01T00:00:00Z"), Some(1_704_067_200));
///
/// // Empty
/// assert_eq!(parse_date(""), None);
/// ```
#[must_use]
pub fn parse_date(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try RFC-2822 first (standard RSS date format)
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return Some(dt.timestamp());
    }

    // Try RFC-3339 / ISO-8601
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp());
    }

    // Try bare Unix timestamp (some feeds emit raw seconds)
    s.parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc2822() {
        assert_eq!(
            parse_date("Mon, 01 Jan 2024 00:00:00 +0000"),
            Some(1_704_067_200)
        );
    }

    #[test]
    fn iso8601() {
        assert_eq!(
            parse_date("2024-01-01T00:00:00Z"),
            Some(1_704_067_200)
        );
    }

    #[test]
    fn unix_passthrough() {
        assert_eq!(parse_date("1704067200"), Some(1_704_067_200));
    }

    #[test]
    fn empty() {
        assert_eq!(parse_date(""), None);
    }

    #[test]
    fn invalid() {
        assert_eq!(parse_date("not a date"), None);
    }
}
