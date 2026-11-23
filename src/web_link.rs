//! The read rule for a web URL and for a link with no scheme.
//!
//! The parse output keeps each raw value. An app applies these functions when
//! it shows or writes a URL. `stophammer` ADR 0054 §4 and §4a own the rules.
//! musicindex ADR 0093 R-FEED-1 item 4 puts them in this crate, so each app
//! uses one copy.

/// Returns `value` when it parses as a URL with the scheme `http` or
/// `https`, and `None` otherwise.
///
/// `stophammer` ADR 0054 §4: a read route passes each URL field from RSS
/// through this function, so a client never receives a `javascript:`,
/// `data:` or other non-web URL. The function does not change the value.
#[must_use]
pub fn web_url_or_none(value: Option<&str>) -> Option<String> {
    let raw = value?;
    match url::Url::parse(raw) {
        Ok(parsed) if matches!(parsed.scheme(), "http" | "https") => Some(raw.to_string()),
        _ => None,
    }
}

/// Gives the read value of a link with no scheme. Also states if the
/// function added the scheme (`stophammer` ADR 0054 §4a).
///
/// The function trims the white space at the start and at the end of `raw`.
/// Call the result `trimmed`. The function runs two tests on `trimmed`:
///
/// 1. `trimmed` has no scheme. `trimmed` has no white space.
/// 2. The part of `trimmed` before the first `/`, `?` or `#` is a host
///    name. A host name has two or more labels, with a `.` between each
///    pair. Each label has 1 to 63 ASCII letters, digits or `-`. No label
///    starts or ends with `-`. The last label has two or more ASCII
///    letters, and no other character.
///
/// When both tests pass, the read value is `https://` and `trimmed`. The
/// function then gives `true`. The [`web_url_or_none`] rule applies to that
/// read value.
///
/// When a test fails, the function gives `web_url_or_none` of `trimmed`.
/// The function then gives `false`.
///
/// Use this function for a website link, a web page link and a live item
/// link. Do not use it for other URL fields.
#[must_use]
pub fn web_link_for_read(raw: &str) -> (Option<String>, bool) {
    let trimmed = raw.trim();
    let has_no_scheme_or_space =
        !starts_with_uri_scheme(trimmed) && !trimmed.chars().any(char::is_whitespace);
    if has_no_scheme_or_space {
        let host_candidate_end = trimmed.find(['/', '?', '#']).unwrap_or(trimmed.len());
        if is_host_name(&trimmed[..host_candidate_end]) {
            let with_scheme = format!("https://{trimmed}");
            return (web_url_or_none(Some(&with_scheme)), true);
        }
    }
    (web_url_or_none(Some(trimmed)), false)
}

/// True when `s` starts with a URI scheme: an ASCII letter, then zero or
/// more ASCII letters, digits, `+`, `-` or `.`, then a `:`.
fn starts_with_uri_scheme(s: &str) -> bool {
    let mut chars = s.chars();
    if !matches!(chars.next(), Some(c) if c.is_ascii_alphabetic()) {
        return false;
    }
    for c in chars {
        match c {
            ':' => return true,
            c if c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.') => {}
            _ => return false,
        }
    }
    false
}

/// True when `candidate` is a host name under `stophammer` ADR 0054 §4a.
/// `candidate` then has two or more labels, with a `.` between each pair.
/// Each label has 1 to 63 ASCII letters, digits or `-`. No label starts or
/// ends with `-`. The last label has two or more ASCII letters, and no
/// other character.
fn is_host_name(candidate: &str) -> bool {
    let labels: Vec<&str> = candidate.split('.').collect();
    let Some(last) = labels.last().copied() else {
        return false;
    };
    labels.len() >= 2
        && labels.iter().copied().all(is_host_name_label)
        && last.len() >= 2
        && last.chars().all(|c| c.is_ascii_alphabetic())
}

/// True when `label` has 1 to 63 ASCII letters, digits or `-`. `label` does
/// not start or end with `-`.
fn is_host_name_label(label: &str) -> bool {
    let len = label.chars().count();
    (1..=63).contains(&len)
        && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !label.starts_with('-')
        && !label.ends_with('-')
}
