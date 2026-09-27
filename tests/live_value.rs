//! Tests for `podcast:liveValue` on a `podcast:liveItem` (ADR 0064 §3).

use stophammer_parser::profile;

/// Legacy Podcast Namespace URI, still emitted by some feeds.
const PODCAST_NS_LEGACY: &str =
    "https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md";

#[test]
fn live_value_with_uri_and_protocol_is_read() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
          <podcast:liveValue uri="https://relay.example/event?event_id=a" protocol="socket.io"/>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert_eq!(feed.live_items.len(), 1);
    assert_eq!(
        feed.live_items[0].live_value_uri.as_deref(),
        Some("https://relay.example/event?event_id=a")
    );
    assert_eq!(
        feed.live_items[0].live_value_protocol.as_deref(),
        Some("socket.io")
    );
}

#[test]
fn live_item_without_live_value_gives_none_for_both_fields() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert_eq!(feed.live_items.len(), 1);
    assert_eq!(feed.live_items[0].live_value_uri, None);
    assert_eq!(feed.live_items[0].live_value_protocol, None);
}

#[test]
fn identifier_only_uri_stays_as_is() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
          <podcast:liveValue uri="event-one" protocol="socket.io"/>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert_eq!(
        feed.live_items[0].live_value_uri.as_deref(),
        Some("event-one")
    );
}

#[test]
fn empty_uri_gives_none_for_both_fields() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
          <podcast:liveValue uri="" protocol="socket.io"/>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert_eq!(feed.live_items[0].live_value_uri, None);
    assert_eq!(feed.live_items[0].live_value_protocol, None);
}

#[test]
fn whitespace_around_attributes_is_trimmed() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
          <podcast:liveValue uri="  https://relay.example/trimmed  " protocol="  socket.io  "/>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert_eq!(
        feed.live_items[0].live_value_uri.as_deref(),
        Some("https://relay.example/trimmed")
    );
    assert_eq!(
        feed.live_items[0].live_value_protocol.as_deref(),
        Some("socket.io")
    );
}

#[test]
fn legacy_namespace_live_value_is_read() {
    let xml = format!(
        r#"<?xml version="1.0"?>
    <rss xmlns:pi="{PODCAST_NS_LEGACY}">
      <channel>
        <title>Test</title>
        <pi:guid>feed-guid</pi:guid>
        <pi:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
          <pi:liveValue uri="https://relay.example/legacy" protocol="socket.io"/>
        </pi:liveItem>
      </channel>
    </rss>"#
    );

    let feed = profile::stophammer().parse(&xml).unwrap();

    assert_eq!(
        feed.live_items[0].live_value_uri.as_deref(),
        Some("https://relay.example/legacy")
    );
}

#[test]
fn channel_and_item_level_live_value_does_not_fill_a_live_item() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveValue uri="https://relay.example/channel-leak" protocol="socket.io"/>
        <item>
          <guid>track-1</guid>
          <title>Track One</title>
          <podcast:liveValue uri="https://relay.example/item-leak" protocol="socket.io"/>
        </item>
        <podcast:liveItem status="live">
          <guid>live-1</guid>
          <title>Live One</title>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).unwrap();

    assert_eq!(feed.tracks.len(), 1);
    assert_eq!(feed.live_items.len(), 1);
    assert_eq!(feed.live_items[0].live_value_uri, None);
    assert_eq!(feed.live_items[0].live_value_protocol, None);
}
