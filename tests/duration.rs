use stophammer_parser::duration::parse_duration;

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
fn fractional_truncated() {
    assert_eq!(parse_duration("300.7"), Some(300));
}

#[test]
fn empty_returns_none() {
    assert_eq!(parse_duration(""), None);
}

#[test]
fn invalid_returns_none() {
    assert_eq!(parse_duration("abc"), None);
}

#[test]
fn zero_formats() {
    assert_eq!(parse_duration("0"), Some(0));
    assert_eq!(parse_duration("0:00"), Some(0));
    assert_eq!(parse_duration("0:00:00"), Some(0));
}
