//! iTunes duration string parser.
//!
//! Supports `HH:MM:SS`, `MM:SS`, bare seconds, and fractional seconds.

/// Parses an iTunes duration string into whole seconds.
///
/// Accepted formats:
/// - `"3661"` or `"3661.5"` — bare seconds (fractional truncated)
/// - `"1:02:03"` — hours:minutes:seconds
/// - `"62:03"` — minutes:seconds
/// - Empty or whitespace — returns `None`
///
/// # Examples
///
/// ```
/// use stophammer_parser::duration::parse_duration;
///
/// assert_eq!(parse_duration("1:23:45"), Some(5025));
/// assert_eq!(parse_duration("62:03"), Some(3723));
/// assert_eq!(parse_duration("300"), Some(300));
/// assert_eq!(parse_duration("300.7"), Some(300));
/// assert_eq!(parse_duration(""), None);
/// ```
#[must_use]
pub fn parse_duration(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        1 => {
            // Bare seconds, possibly fractional
            parse_seconds_part(parts[0])
        }
        2 => {
            // MM:SS
            let minutes = parts[0].parse::<i64>().ok()?;
            let seconds = parse_seconds_part(parts[1])?;
            Some(minutes * 60 + seconds)
        }
        3 => {
            // HH:MM:SS
            let hours = parts[0].parse::<i64>().ok()?;
            let minutes = parts[1].parse::<i64>().ok()?;
            let seconds = parse_seconds_part(parts[2])?;
            Some(hours * 3600 + minutes * 60 + seconds)
        }
        _ => None,
    }
}

/// Parses a seconds component, truncating any fractional part.
fn parse_seconds_part(s: &str) -> Option<i64> {
    if let Some(dot_pos) = s.find('.') {
        s[..dot_pos].parse::<i64>().ok()
    } else {
        s.parse::<i64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hh_mm_ss() {
        assert_eq!(parse_duration("1:23:45"), Some(5025));
    }

    #[test]
    fn mm_ss() {
        assert_eq!(parse_duration("62:03"), Some(3723));
    }

    #[test]
    fn bare_seconds() {
        assert_eq!(parse_duration("300"), Some(300));
    }

    #[test]
    fn fractional_seconds() {
        assert_eq!(parse_duration("300.7"), Some(300));
    }

    #[test]
    fn empty_string() {
        assert_eq!(parse_duration(""), None);
    }

    #[test]
    fn invalid_string() {
        assert_eq!(parse_duration("abc"), None);
    }

    #[test]
    fn zero() {
        assert_eq!(parse_duration("0"), Some(0));
        assert_eq!(parse_duration("0:00"), Some(0));
        assert_eq!(parse_duration("0:00:00"), Some(0));
    }

    #[test]
    fn whitespace_trimmed() {
        assert_eq!(parse_duration("  300  "), Some(300));
    }
}
