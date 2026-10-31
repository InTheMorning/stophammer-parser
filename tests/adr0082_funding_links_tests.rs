//! Tests for ADR 0082 task 001: each `podcast:funding` tag gives a link.

use stophammer_parser::profile;

#[test]
fn funding_tags_give_links_with_labels() {
    let long_label = "L".repeat(150);
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
<channel>
<title>Album Feed</title>
<link>http://example.com</link>
<description>Test</description>
<podcast:guid>funding-feed-guid</podcast:guid>
<podcast:funding url="  https://example.com/donate  ">  Support the show!  </podcast:funding>
<podcast:funding url="https://example.com/member">{long_label}</podcast:funding>
<podcast:funding>No URL</podcast:funding>
<podcast:funding url="">Empty URL</podcast:funding>
<item>
<guid>item-1</guid>
<title>Track</title>
<enclosure url="https://example.com/a.mp3" length="1" type="audio/mpeg"/>
<podcast:funding url="https://example.com/item"></podcast:funding>
</item>
<podcast:liveItem status="live">
<guid>live-1</guid>
<title>Live One</title>
<podcast:funding url="https://example.com/live">Live support</podcast:funding>
</podcast:liveItem>
</channel>
</rss>"#
    );

    let parsed = profile::stophammer().parse(&xml).expect("parse succeeded");

    let links = &parsed.links;
    assert_eq!(links.len(), 3, "website link and two funding links");
    assert_eq!(links[0].link_type, "website");
    assert_eq!(links[0].position, 0);
    assert_eq!(links[0].label, None, "website link has no label");

    assert_eq!(links[1].link_type, "funding");
    assert_eq!(links[1].position, 1);
    assert_eq!(links[1].url, "https://example.com/donate");
    assert_eq!(links[1].label.as_deref(), Some("Support the show!"));
    assert_eq!(links[1].extraction_path, "feed.podcast:funding");

    assert_eq!(links[2].link_type, "funding");
    assert_eq!(links[2].position, 2);
    assert_eq!(links[2].url, "https://example.com/member");
    assert_eq!(links[2].extraction_path, "feed.podcast:funding");
    let label = links[2].label.as_deref().expect("label kept");
    assert!(label.len() > 128, "label is not cut");
    assert_eq!(label, long_label, "label is kept whole");

    assert_eq!(parsed.tracks.len(), 1, "one item");
    let item_links = &parsed.tracks[0].links;
    assert_eq!(item_links.len(), 1, "one item funding link");
    assert_eq!(item_links[0].link_type, "funding");
    assert_eq!(item_links[0].url, "https://example.com/item");
    assert_eq!(item_links[0].label, None, "empty node value gives no label");
    assert_eq!(item_links[0].extraction_path, "entity.podcast:funding");

    assert_eq!(parsed.live_items.len(), 1, "one live item");
    let live_links = &parsed.live_items[0].links;
    assert_eq!(live_links.len(), 1, "one live item funding link");
    assert_eq!(live_links[0].link_type, "funding");
    assert_eq!(live_links[0].url, "https://example.com/live");
    assert_eq!(live_links[0].label.as_deref(), Some("Live support"));
    assert_eq!(live_links[0].extraction_path, "entity.podcast:funding");
}
