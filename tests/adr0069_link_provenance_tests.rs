//! Tests for ADR 0069 task 001: each link says how the album names it.
//!
//! The parser marks the feed-level `remoteItem` that is the publisher of the album.

use stophammer_parser::profile;

#[test]
fn album_with_podcast_publisher_at_feed_level_marks_true() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
<channel>
<title>Album Feed</title>
<link>http://example.com</link>
<description>Test</description>
<podcast:guid>album-feed-guid</podcast:guid>
<podcast:remoteItem feedGuid="label-guid" medium="publisher"/>
<podcast:publisher>
<podcast:remoteItem feedGuid="published-label-guid" medium="publisher"/>
</podcast:publisher>
</channel>
</rss>"#;

    let parser = profile::stophammer();
    let parsed = parser.parse(xml).expect("parse succeeded");
    assert_eq!(parsed.remote_items.len(), 2, "two feed-level remote items");

    // The bare item (first in source order at channel level) gets false.
    assert_eq!(parsed.remote_items[0].remote_feed_guid, "label-guid");
    assert!(
        !parsed.remote_items[0].publisher_reference,
        "bare feed-level item is not publisher"
    );

    // The item inside podcast:publisher gets true.
    assert_eq!(
        parsed.remote_items[1].remote_feed_guid,
        "published-label-guid"
    );
    assert!(
        parsed.remote_items[1].publisher_reference,
        "nested item inside publisher wrapper is publisher"
    );
}

#[test]
fn podcast_publisher_wrapper_takes_precedence_over_bare_items() {
    // The bare item comes first in the channel, before the wrapper.
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
<channel>
<title>Album Feed</title>
<link>http://example.com</link>
<description>Test</description>
<podcast:guid>album-feed-guid</podcast:guid>
<podcast:remoteItem feedGuid="credit-guid" medium="publisher"/>
<podcast:publisher>
<podcast:remoteItem feedGuid="published-label-guid" medium="publisher"/>
</podcast:publisher>
</channel>
</rss>"#;

    let parser = profile::stophammer();
    let parsed = parser.parse(xml).expect("parse succeeded");
    assert_eq!(parsed.remote_items.len(), 2, "two feed-level remote items");

    let mark = |guid: &str| {
        parsed
            .remote_items
            .iter()
            .find(|item| item.remote_feed_guid == guid)
            .unwrap_or_else(|| panic!("no remote item {guid}"))
            .publisher_reference
    };
    assert!(
        mark("published-label-guid"),
        "the item inside podcast:publisher is the publisher, also when a bare item comes first"
    );
    assert!(
        !mark("credit-guid"),
        "a bare item is a credit when podcast:publisher exists"
    );
}

#[test]
fn first_bare_publisher_item_is_publisher_when_no_wrapper() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
<channel>
<title>Album Feed</title>
<link>http://example.com</link>
<description>Test</description>
<podcast:guid>album-feed-guid</podcast:guid>
<podcast:remoteItem feedGuid="first-publisher-guid" medium="publisher"/>
<podcast:remoteItem feedGuid="credit-guid" medium="publisher"/>
</channel>
</rss>"#;

    let parser = profile::stophammer();
    let parsed = parser.parse(xml).expect("parse succeeded");
    assert_eq!(parsed.remote_items.len(), 2, "two feed-level remote items");

    // First bare publisher item gets true (no podcast:publisher wrapper exists).
    assert_eq!(
        parsed.remote_items[0].remote_feed_guid,
        "first-publisher-guid"
    );
    assert!(
        parsed.remote_items[0].publisher_reference,
        "first bare publisher item is publisher"
    );

    // Second bare publisher item gets false (already found publisher).
    assert_eq!(parsed.remote_items[1].remote_feed_guid, "credit-guid");
    assert!(
        !parsed.remote_items[1].publisher_reference,
        "second bare publisher item is credit"
    );
}

#[test]
fn feed_medium_items_get_false() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
<channel>
<title>Publisher Feed</title>
<link>http://example.com</link>
<description>Test</description>
<podcast:guid>publisher-feed-guid</podcast:guid>
<podcast:remoteItem feedGuid="album-guid-1" medium="music"/>
<podcast:remoteItem feedGuid="album-guid-2" medium="music"/>
</channel>
</rss>"#;

    let parser = profile::stophammer();
    let parsed = parser.parse(xml).expect("parse succeeded");
    assert_eq!(parsed.remote_items.len(), 2, "two feed-level remote items");

    // Music items always get false (they are not publishers).
    for item in &parsed.remote_items {
        assert!(!item.publisher_reference, "music item is never publisher");
    }
}
