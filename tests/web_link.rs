// The read rules of stophammer ADR 0054 §4 and §4a. musicindex ADR 0086
// R-FEED-1 item 4 puts them in this crate.

use stophammer_parser::{web_link_for_read, web_url_or_none};

// ── web_url_or_none (ADR 0054 §4) ───────────────────────────────────────────

#[test]
fn web_url_or_none_keeps_an_https_value() {
    assert_eq!(
        web_url_or_none(Some("https://img.example.com/cover.jpg")),
        Some("https://img.example.com/cover.jpg".to_string())
    );
}

#[test]
fn web_url_or_none_keeps_an_http_value() {
    assert_eq!(
        web_url_or_none(Some("http://img.example.com/cover.jpg")),
        Some("http://img.example.com/cover.jpg".to_string())
    );
}

#[test]
fn web_url_or_none_rejects_javascript_and_data_schemes() {
    assert_eq!(web_url_or_none(Some("javascript:alert(1)")), None);
    assert_eq!(web_url_or_none(Some("data:audio/mp3;base64,AA")), None);
}

#[test]
fn web_url_or_none_rejects_an_unparseable_value() {
    assert_eq!(web_url_or_none(Some("not a url")), None);
}

#[test]
fn web_url_or_none_passes_through_none() {
    assert_eq!(web_url_or_none(None), None);
}

// ── web_link_for_read (ADR 0054 §4a) ────────────────────────────────────────

#[test]
fn web_link_for_read_adds_the_scheme_to_a_bare_host_name() {
    assert_eq!(
        web_link_for_read("www.pixelpalace.party"),
        (Some("https://www.pixelpalace.party".to_string()), true)
    );
}

#[test]
fn web_link_for_read_adds_the_scheme_to_a_bare_host_name_with_a_path() {
    assert_eq!(
        web_link_for_read("strangeloveofmusic.wixsite.com/strangelove"),
        (
            Some("https://strangeloveofmusic.wixsite.com/strangelove".to_string()),
            true
        )
    );
}

#[test]
fn web_link_for_read_adds_the_scheme_to_the_strange_love_channel_link() {
    assert_eq!(
        web_link_for_read("www.strangeloveofmusic.wixsite.com/strangelove"),
        (
            Some("https://www.strangeloveofmusic.wixsite.com/strangelove".to_string()),
            true
        )
    );
}

#[test]
fn web_link_for_read_keeps_an_https_value_unchanged() {
    assert_eq!(
        web_link_for_read("https://example.com/"),
        (Some("https://example.com/".to_string()), false)
    );
}

#[test]
fn web_link_for_read_gives_none_for_each_non_host_name_value() {
    for value in [
        "The Fraglebay",
        "localhost",
        "example",
        "example.c0m",
        "-a.com",
        "javascript:alert(1)",
    ] {
        assert_eq!(
            web_link_for_read(value),
            (None, false),
            "{value} should give (None, false)"
        );
    }
}

#[test]
fn web_link_for_read_trims_white_space_before_the_host_name_test() {
    assert_eq!(
        web_link_for_read("  www.TheSynthesatsers.com "),
        (Some("https://www.TheSynthesatsers.com".to_string()), true)
    );
}
