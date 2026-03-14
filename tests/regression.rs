//! Regression tests for Sprint 5 bugs fixed across the TS crawlers.

use stophammer_parser::profile;
use stophammer_parser::RouteType;

/// Sprint 5 Bug 1: `route_type` defaults to "node", not "lightning".
/// The old TS parsers defaulted unknown types to "lightning" which is
/// not a valid `RouteType` variant.
#[test]
fn route_type_defaults_to_node() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>guid</podcast:guid>
        <item>
          <guid>t1</guid>
          <title>Track</title>
          <podcast:value type="lightning" method="keysend">
            <podcast:valueRecipient name="Host" address="pubkey" split="100"/>
          </podcast:value>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();
    assert_eq!(feed.tracks[0].payment_routes[0].route_type, RouteType::Node);
}

/// Sprint 5 Bug 1b: unknown type values also default to node.
#[test]
fn unknown_route_type_defaults_to_node() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>guid</podcast:guid>
        <item>
          <guid>t1</guid>
          <title>Track</title>
          <podcast:value type="lightning" method="keysend">
            <podcast:valueRecipient name="Host" type="lightning" address="pubkey" split="100"/>
          </podcast:value>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();
    // "lightning" is not "lnaddress", so it defaults to Node
    assert_eq!(feed.tracks[0].payment_routes[0].route_type, RouteType::Node);
}

/// Sprint 5 Bug 2: VTS reads remote GUIDs from podcast:remoteItem child element,
/// not from attributes on the valueTimeSplit element itself.
#[test]
fn vts_reads_remote_item_child() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>guid</podcast:guid>
        <item>
          <guid>t1</guid>
          <title>Track</title>
          <podcast:value type="lightning" method="keysend">
            <podcast:valueRecipient name="Host" type="node" address="pubkey" split="100"/>
            <podcast:valueTimeSplit startTime="60" duration="120" split="50">
              <podcast:remoteItem feedGuid="correct-feed" itemGuid="correct-item"/>
            </podcast:valueTimeSplit>
          </podcast:value>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();
    let vts = &feed.tracks[0].value_time_splits[0];
    assert_eq!(vts.remote_feed_guid, "correct-feed");
    assert_eq!(vts.remote_item_guid, "correct-item");
}

/// Sprint 5 Bug 3: explicit accepts both "yes" and "true".
#[test]
fn explicit_yes_and_true() {
    let xml_yes = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
      <channel>
        <title>Test</title>
        <podcast:guid>guid</podcast:guid>
        <itunes:explicit>yes</itunes:explicit>
      </channel>
    </rss>"#;

    let xml_true = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
      <channel>
        <title>Test</title>
        <podcast:guid>guid2</podcast:guid>
        <itunes:explicit>true</itunes:explicit>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    assert!(parser.parse(xml_yes).unwrap().explicit);
    assert!(parser.parse(xml_true).unwrap().explicit);
}

/// Sprint 5 Bug 4: `feed_payment_routes` are extracted from channel-level value block.
#[test]
fn feed_payment_routes_extracted() {
    let xml = include_str!("fixtures/payment.xml");
    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.feed_payment_routes.len(), 2);
    assert_eq!(feed.feed_payment_routes[0].address, "feedpubkey123");
}

/// Sprint 5 Bug 5: `owner_name` parsed from `itunes:owner` > `itunes:name`.
#[test]
fn owner_name_from_nested_element() {
    let xml = include_str!("fixtures/basic.xml");
    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.owner_name.as_deref(), Some("Owner Name"));
}

/// Some feeds still use the older GitHub-doc Podcast Namespace URI instead of
/// the newer podcastindex.org URI. We should accept both.
#[test]
fn legacy_podcast_namespace_uri_is_accepted() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md">
      <channel>
        <title>Test</title>
        <podcast:guid>guid</podcast:guid>
        <podcast:medium>music</podcast:medium>
        <podcast:value type="lightning" method="keysend">
          <podcast:valueRecipient name="Host" address="pubkey" split="100"/>
        </podcast:value>
        <item>
          <guid>t1</guid>
          <title>Track</title>
          <podcast:episode>1</podcast:episode>
          <podcast:season>2</podcast:season>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.raw_medium.as_deref(), Some("music"));
    assert_eq!(feed.feed_payment_routes.len(), 1);
    assert_eq!(feed.tracks[0].track_number, Some(1));
    assert_eq!(feed.tracks[0].season, Some(2));
}
