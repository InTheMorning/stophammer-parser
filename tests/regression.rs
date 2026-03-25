//! Regression tests for Sprint 5 bugs fixed across the TS crawlers.

use stophammer_parser::{RouteType, extract_podcast_namespace, profile};

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
    assert!(
        feed.podcast_namespace
            .as_ref()
            .is_some_and(|snapshot| snapshot.tags.iter().any(|tag| tag.tag == "podcast:value")),
        "legacy namespace URI should also populate the generic namespace snapshot"
    );
}

#[test]
fn podcast_namespace_snapshot_is_prefix_agnostic_and_preserves_nested_tags() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:pi="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Namespace Test</title>
        <pi:guid>feed-guid</pi:guid>
        <pi:medium>music</pi:medium>
        <pi:locked owner="owner@example.com">true</pi:locked>
        <pi:funding url="https://example.com/support">Support the show</pi:funding>
        <pi:publisher>Example Label</pi:publisher>
        <pi:updateFrequency complete="false" rrule="FREQ=WEEKLY">Weekly</pi:updateFrequency>
        <pi:podroll>
          <pi:remoteItem feedGuid="podroll-feed" feedUrl="https://example.com/podroll.xml" />
        </pi:podroll>
        <pi:liveItem status="live">
          <guid>live-guid</guid>
          <title>Live Set</title>
          <pi:socialInteract uri="https://example.com/social" protocol="activitypub" accountId="@artist@example.com" />
        </pi:liveItem>
        <item>
          <guid>track-guid</guid>
          <title>Track</title>
          <pi:transcript url="https://example.com/transcript.vtt" type="text/vtt" language="en" />
          <pi:chapters url="https://example.com/chapters.json" type="application/json+chapters" />
          <pi:soundbite startTime="12" duration="34" />
          <pi:location osm="relation:123">Montreal</pi:location>
          <pi:license url="https://example.com/license">CC-BY-4.0</pi:license>
          <pi:source uri="https://example.com/source.xml">Upstream feed</pi:source>
          <pi:integrity type="sha256" value="abc123" />
          <pi:chat server="irc.example.com" protocol="irc" />
          <pi:value type="lightning" method="keysend">
            <pi:valueRecipient name="Artist" address="pubkey" split="100" />
            <pi:valueTimeSplit startTime="0" split="100">
              <pi:remoteItem feedGuid="remote-feed" itemGuid="remote-item" />
            </pi:valueTimeSplit>
          </pi:value>
        </item>
      </channel>
    </rss>"#;

    let feed = profile::stophammer().parse(xml).expect("feed parses");
    let snapshot = feed.podcast_namespace.expect("namespace snapshot");

    assert!(
        snapshot
            .tags
            .iter()
            .all(|tag| tag.tag.starts_with("podcast:")),
        "snapshot tags should be canonicalized to the podcast: prefix"
    );
    assert!(
        snapshot.tags.iter().any(|tag| tag.tag == "podcast:locked"),
        "expected channel-level locked tag"
    );
    assert!(
        snapshot
            .tags
            .iter()
            .any(|tag| tag.tag == "podcast:publisher"),
        "expected channel-level publisher tag"
    );
    assert!(
        snapshot.tags.iter().any(|tag| {
            tag.tag == "podcast:socialInteract"
                && tag.entity_scope == "live_item"
                && tag.entity_guid.as_deref() == Some("live-guid")
        }),
        "expected live-item scoped socialInteract tag"
    );
    assert!(
        snapshot.tags.iter().any(|tag| {
            tag.tag == "podcast:location"
                && tag.entity_scope == "item"
                && tag.entity_guid.as_deref() == Some("track-guid")
                && tag.text.as_deref() == Some("Montreal")
        }),
        "expected item-scoped location tag with text preserved"
    );
    assert!(
        snapshot.tags.iter().any(|tag| {
            tag.tag == "podcast:valueRecipient"
                && tag.path == "rss.channel.item.podcast:value.podcast:valueRecipient"
                && tag.attributes.get("address").map(String::as_str) == Some("pubkey")
        }),
        "expected nested valueRecipient path and attributes"
    );
    assert!(
        snapshot.tags.iter().any(|tag| {
            tag.tag == "podcast:remoteItem"
                && tag.path
                    == "rss.channel.item.podcast:value.podcast:valueTimeSplit.podcast:remoteItem"
                && tag.attributes.get("itemGuid").map(String::as_str) == Some("remote-item")
        }),
        "expected nested valueTimeSplit remoteItem to be preserved"
    );
}

#[test]
fn podcast_namespace_can_be_extracted_even_when_full_feed_parse_fails() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <podcast:block>yes</podcast:block>
      </channel>
    </rss>"#;

    assert!(
        profile::stophammer().parse(xml).is_err(),
        "full parse should still reject feeds missing required fields"
    );

    let snapshot = extract_podcast_namespace(xml)
        .expect("namespace-only extraction should succeed")
        .expect("snapshot");
    assert_eq!(snapshot.tags.len(), 1);
    assert_eq!(snapshot.tags[0].tag, "podcast:block");
    assert_eq!(snapshot.tags[0].text.as_deref(), Some("yes"));
}

#[test]
fn feed_level_remote_item_is_extracted() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Remote Feed Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:remoteItem medium="publisher" feedGuid="artist-feed-guid" feedUrl="https://example.com/artist.xml" />
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.remote_items.len(), 1);
    assert_eq!(feed.remote_items[0].medium.as_deref(), Some("publisher"));
    assert_eq!(feed.remote_items[0].remote_feed_guid, "artist-feed-guid");
    assert_eq!(
        feed.remote_items[0].remote_feed_url.as_deref(),
        Some("https://example.com/artist.xml")
    );
}

#[test]
fn live_item_is_extracted_separately_from_tracks() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
      <channel>
        <title>Live Feed Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:medium>music</podcast:medium>
        <podcast:liveItem status="live" start="2026-03-17T00:00:00Z" end="2026-03-17T01:00:00Z">
          <guid>live-guid-1</guid>
          <title>Live Show</title>
          <description>Going live</description>
          <enclosure url="https://stream.example.com/live.mp3" length="123" type="audio/mpeg" />
          <itunes:duration>00:10:00</itunes:duration>
        </podcast:liveItem>
        <item>
          <guid>track-guid-1</guid>
          <title>Recorded Track</title>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.live_items.len(), 1);
    assert_eq!(feed.tracks.len(), 1);
    assert_eq!(feed.live_items[0].live_item_guid, "live-guid-1");
    assert_eq!(feed.live_items[0].title, "Live Show");
    assert_eq!(feed.live_items[0].status, "live");
    assert_eq!(feed.live_items[0].start_at, Some(1_773_705_600));
    assert_eq!(feed.live_items[0].end_at, Some(1_773_709_200));
    assert_eq!(
        feed.live_items[0].content_link.as_deref(),
        Some("https://stream.example.com/live.mp3")
    );
}

#[test]
fn podcast_persons_are_extracted_for_feed_track_and_live_item() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>People Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:person role="bandleader" group="music" href="https://example.com/artist" img="https://example.com/artist.jpg">Alice</podcast:person>
        <podcast:liveItem status="live">
          <guid>live-guid-1</guid>
          <title>Live Show</title>
          <podcast:person role="host" group="cast">MC</podcast:person>
        </podcast:liveItem>
        <item>
          <guid>track-guid-1</guid>
          <title>Song</title>
          <podcast:person role="guitarist" group="music">Bob</podcast:person>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.persons.len(), 1);
    assert_eq!(feed.persons[0].name, "Alice");
    assert_eq!(feed.persons[0].role.as_deref(), Some("bandleader"));
    assert_eq!(feed.persons[0].group_name.as_deref(), Some("music"));
    assert_eq!(
        feed.persons[0].href.as_deref(),
        Some("https://example.com/artist")
    );
    assert_eq!(
        feed.persons[0].img.as_deref(),
        Some("https://example.com/artist.jpg")
    );

    assert_eq!(feed.tracks[0].persons.len(), 1);
    assert_eq!(feed.tracks[0].persons[0].name, "Bob");
    assert_eq!(feed.tracks[0].persons[0].role.as_deref(), Some("guitarist"));

    assert_eq!(feed.live_items[0].persons.len(), 1);
    assert_eq!(feed.live_items[0].persons[0].name, "MC");
    assert_eq!(
        feed.live_items[0].persons[0].group_name.as_deref(),
        Some("cast")
    );
}

#[test]
fn podcast_txt_npub_is_extracted_as_entity_id_claim() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Identity Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:txt purpose="npub">npub1feedidentity</podcast:txt>
        <item>
          <guid>track-guid-1</guid>
          <title>Song</title>
          <podcast:txt purpose="npub">npub1trackidentity</podcast:txt>
        </item>
        <podcast:liveItem status="pending">
          <guid>live-guid-1</guid>
          <title>Live Show</title>
          <podcast:txt purpose="npub">npub1liveidentity</podcast:txt>
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.entity_ids.len(), 1);
    assert_eq!(feed.entity_ids[0].scheme, "nostr_npub");
    assert_eq!(feed.entity_ids[0].value, "npub1feedidentity");

    assert_eq!(feed.tracks[0].entity_ids.len(), 1);
    assert_eq!(feed.tracks[0].entity_ids[0].scheme, "nostr_npub");
    assert_eq!(feed.tracks[0].entity_ids[0].value, "npub1trackidentity");

    assert_eq!(feed.live_items[0].entity_ids.len(), 1);
    assert_eq!(feed.live_items[0].entity_ids[0].scheme, "nostr_npub");
    assert_eq!(feed.live_items[0].entity_ids[0].value, "npub1liveidentity");
}

#[test]
fn feed_track_and_live_item_links_are_extracted() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:atom="http://www.w3.org/2005/Atom">
      <channel>
        <title>Link Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <link>https://example.com/artist</link>
        <atom:link rel="alternate" href="https://example.com/artist-home" type="text/html" />
        <atom:link rel="self" href="https://example.com/feed.xml" type="application/rss+xml" />
        <item>
          <guid>track-guid-1</guid>
          <title>Song</title>
          <link>https://example.com/song</link>
          <atom:link rel="alternate" href="https://example.com/song-home" type="text/html" />
        </item>
        <podcast:liveItem status="live" contentLink="https://stream.example.com/live.mp3">
          <guid>live-guid-1</guid>
          <title>Live Show</title>
          <atom:link rel="alternate" href="https://example.com/live-show" type="text/html" />
        </podcast:liveItem>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.links.len(), 3);
    assert_eq!(feed.links[0].link_type, "website");
    assert_eq!(feed.links[0].url, "https://example.com/artist");
    assert_eq!(feed.links[0].extraction_path, "feed.link");
    assert_eq!(feed.links[1].link_type, "website");
    assert_eq!(feed.links[1].url, "https://example.com/artist-home");
    assert_eq!(
        feed.links[1].extraction_path,
        "feed.atom:link[@rel='alternate']"
    );
    assert_eq!(feed.links[2].link_type, "self_feed");
    assert_eq!(feed.links[2].extraction_path, "feed.atom:link[@rel='self']");

    assert_eq!(feed.tracks[0].links.len(), 2);
    assert_eq!(feed.tracks[0].links[0].link_type, "web_page");
    assert_eq!(feed.tracks[0].links[0].url, "https://example.com/song");
    assert_eq!(feed.tracks[0].links[0].extraction_path, "entity.link");
    assert_eq!(feed.tracks[0].links[1].link_type, "web_page");
    assert_eq!(feed.tracks[0].links[1].url, "https://example.com/song-home");
    assert_eq!(
        feed.tracks[0].links[1].extraction_path,
        "entity.atom:link[@rel='alternate']"
    );

    assert_eq!(feed.live_items[0].links.len(), 2);
    assert_eq!(feed.live_items[0].links[0].link_type, "web_page");
    assert_eq!(
        feed.live_items[0].links[0].url,
        "https://example.com/live-show"
    );
    assert_eq!(
        feed.live_items[0].links[0].extraction_path,
        "entity.atom:link[@rel='alternate']"
    );
    assert_eq!(feed.live_items[0].links[1].link_type, "content_stream");
    assert_eq!(
        feed.live_items[0].links[1].url,
        "https://stream.example.com/live.mp3"
    );
    assert_eq!(
        feed.live_items[0].links[1].extraction_path,
        "live_item.@contentLink"
    );
}

#[test]
fn alternate_enclosures_are_extracted_for_tracks_and_live_items() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Alt Enclosure Test</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:liveItem status="live">
          <guid>live-guid-1</guid>
          <title>Live Show</title>
          <enclosure url="https://cdn.example.com/live.mp3" length="123" type="audio/mpeg" />
          <podcast:alternateEnclosure url="https://cdn.example.com/live.opus" type="audio/ogg" length="456" rel="stream" title="Opus Stream" />
        </podcast:liveItem>
        <item>
          <guid>track-guid-1</guid>
          <title>Song</title>
          <enclosure url="https://cdn.example.com/song.mp3" length="111" type="audio/mpeg" />
          <podcast:alternateEnclosure url="https://cdn.example.com/song.flac" type="audio/flac" length="222" title="Lossless" />
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.tracks[0].alternate_enclosures.len(), 1);
    assert_eq!(
        feed.tracks[0].alternate_enclosures[0].url,
        "https://cdn.example.com/song.flac"
    );
    assert_eq!(
        feed.tracks[0].alternate_enclosures[0].mime_type.as_deref(),
        Some("audio/flac")
    );
    assert_eq!(feed.tracks[0].alternate_enclosures[0].bytes, Some(222));
    assert_eq!(
        feed.tracks[0].alternate_enclosures[0].title.as_deref(),
        Some("Lossless")
    );

    assert_eq!(feed.live_items[0].alternate_enclosures.len(), 1);
    assert_eq!(
        feed.live_items[0].alternate_enclosures[0].url,
        "https://cdn.example.com/live.opus"
    );
    assert_eq!(
        feed.live_items[0].alternate_enclosures[0].rel.as_deref(),
        Some("stream")
    );
}
