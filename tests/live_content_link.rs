//! Tests for `podcast:contentLink` on a `podcast:liveItem` (stophammer ADR 0093).

use stophammer_parser::profile;

/// The bitpunk.fm live feed, captured on 2026-11-09 in musicindex-research
/// `sources/captures/2026-11-09-live-item-stream/feeds/bitpunk-live.xml`.
const BITPUNK_LIVE: &str = include_str!("fixtures/bitpunk_live.xml");

const STREAM: &str = "https://customer-t35yra9mwp9qe3jn.cloudflarestream.com/0bb6fd3739e0b81e8f7641d393f6401b/manifest/video.m3u8";

#[test]
fn a_captured_live_item_gives_its_content_link_child_element() {
    let feed = profile::stophammer().parse(BITPUNK_LIVE).unwrap();

    assert_eq!(feed.live_items.len(), 1);
    let item = &feed.live_items[0];
    assert_eq!(
        item.content_link.as_deref(),
        Some("https://zine.bitpunk.fm/live"),
        "ADR 0093 §1: content_link is the href of the podcast:contentLink child element"
    );
    assert_eq!(
        item.enclosure_url.as_deref(),
        Some(STREAM),
        "ADR 0093 §3: the stream stays in enclosure_url"
    );

    let content_links: Vec<_> = item
        .links
        .iter()
        .filter(|link| link.link_type == "content_link")
        .collect();
    assert_eq!(content_links.len(), 1);
    assert_eq!(content_links[0].url, "https://zine.bitpunk.fm/live");
    assert_eq!(content_links[0].label.as_deref(), Some("Listen Live!"));
    assert_eq!(
        content_links[0].extraction_path,
        "live_item.podcast:contentLink"
    );
    assert!(
        item.links
            .iter()
            .all(|link| link.link_type != "content_stream"),
        "ADR 0093 §2: the link type content_stream is removed"
    );
}

#[test]
fn a_live_item_with_no_content_link_gives_none_and_not_the_stream() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
          <enclosure url="https://stream.example/live.mp3" type="audio/mpeg" length="0"/>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert_eq!(feed.live_items.len(), 1);
    let item = &feed.live_items[0];
    assert_eq!(
        item.content_link, None,
        "ADR 0093 §1: the enclosure URL never fills content_link"
    );
    assert_eq!(
        item.enclosure_url.as_deref(),
        Some("https://stream.example/live.mp3")
    );
    assert!(
        item.links
            .iter()
            .all(|link| link.link_type != "content_link")
    );
}

#[test]
fn the_first_content_link_with_an_href_gives_the_value_and_each_one_gives_a_link() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="pending" contentLink="https://attribute.example/">
          <guid>live-1</guid>
          <title>Live One</title>
          <enclosure url="https://stream.example/live.mp3" type="audio/mpeg" length="0"/>
          <podcast:contentLink href="  ">No href</podcast:contentLink>
          <podcast:contentLink href=" https://chat.example/room ">  Chat  </podcast:contentLink>
          <podcast:contentLink href="https://player.example/"/>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    let item = &feed.live_items[0];
    assert_eq!(
        item.content_link.as_deref(),
        Some("https://chat.example/room"),
        "ADR 0093 §1: the first non-empty href, trimmed; the attribute is not read"
    );

    let content_links: Vec<_> = item
        .links
        .iter()
        .filter(|link| link.link_type == "content_link")
        .map(|link| (link.url.as_str(), link.label.as_deref()))
        .collect();
    assert_eq!(
        content_links,
        vec![
            ("https://chat.example/room", Some("Chat")),
            ("https://player.example/", None),
        ],
        "ADR 0093 §2: each element with an href gives a link, and an empty text gives no label"
    );
}

#[test]
fn a_content_link_on_an_item_gives_no_link() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <item>
          <guid>track-1</guid>
          <title>Track One</title>
          <enclosure url="https://media.example/a.mp3" type="audio/mpeg" length="0"/>
          <podcast:contentLink href="https://page.example/">Page</podcast:contentLink>
        </item>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert!(
        feed.tracks[0]
            .links
            .iter()
            .all(|link| link.link_type != "content_link"),
        "ADR 0093 §2 covers a live item only"
    );
}
