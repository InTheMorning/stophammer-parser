use stophammer_parser::profile;

fn basic_xml() -> &'static str {
    include_str!("fixtures/basic.xml")
}

#[test]
fn parses_minimal_feed() {
    let parser = profile::stophammer();
    let feed = parser.parse(basic_xml()).unwrap();

    assert_eq!(feed.feed_guid, "feed-guid-123");
    assert_eq!(feed.title, "Test Podcast");
    assert_eq!(feed.language.as_deref(), Some("en"));
    assert_eq!(feed.itunes_type.as_deref(), Some("episodic"));
    assert_eq!(feed.raw_medium.as_deref(), Some("music"));
    assert_eq!(feed.author_name.as_deref(), Some("Test Author"));
    assert_eq!(feed.owner_name.as_deref(), Some("Owner Name"));
    assert_eq!(feed.image_url.as_deref(), Some("https://example.com/image.jpg"));
    assert!(!feed.explicit);
    assert_eq!(feed.pub_date, Some(1_704_067_200));
}

#[test]
fn parses_feed_description_with_entities() {
    let parser = profile::stophammer();
    let feed = parser.parse(basic_xml()).unwrap();

    assert_eq!(feed.description.as_deref(), Some("A test podcast & description"));
}

#[test]
fn parses_all_track_fields() {
    let parser = profile::stophammer();
    let feed = parser.parse(basic_xml()).unwrap();

    assert_eq!(feed.tracks.len(), 2);

    let track = &feed.tracks[0];
    assert_eq!(track.track_guid, "track-1");
    assert_eq!(track.title, "Episode One");
    assert_eq!(track.pub_date, Some(1_704_024_000));
    assert_eq!(track.duration_secs, Some(5025));
    assert_eq!(track.enclosure_url.as_deref(), Some("https://example.com/ep1.mp3"));
    assert_eq!(track.enclosure_type.as_deref(), Some("audio/mpeg"));
    assert_eq!(track.enclosure_bytes, Some(12_345_678));
    assert_eq!(track.track_number, Some(1));
    assert_eq!(track.season, Some(2));
    assert!(track.explicit);
    assert_eq!(track.description.as_deref(), Some("Episode one description"));
    assert_eq!(track.author_name.as_deref(), Some("Episode Author"));
}

#[test]
fn minimal_track_has_defaults() {
    let parser = profile::stophammer();
    let feed = parser.parse(basic_xml()).unwrap();

    let track = &feed.tracks[1];
    assert_eq!(track.track_guid, "track-2");
    assert_eq!(track.title, "Episode Two");
    assert_eq!(track.pub_date, None);
    assert_eq!(track.duration_secs, None);
    assert_eq!(track.enclosure_url, None);
    assert!(!track.explicit);
    assert!(track.payment_routes.is_empty());
    assert!(track.value_time_splits.is_empty());
}

#[test]
fn missing_title_errors() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <podcast:guid>guid</podcast:guid>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let err = parser.parse(xml).unwrap_err();
    assert!(err.is_missing_field());
}

#[test]
fn missing_guid_errors_without_fallback() {
    let xml = include_str!("fixtures/no_guid.xml");

    let parser = profile::stophammer();
    let err = parser.parse(xml).unwrap_err();
    assert!(err.is_missing_field());
}

#[test]
fn fallback_guid_works() {
    let xml = include_str!("fixtures/no_guid.xml");

    let parser = profile::stophammer_with_fallback("fallback-123".into());
    let feed = parser.parse(xml).unwrap();
    assert_eq!(feed.feed_guid, "fallback-123");
}

#[test]
fn malformed_xml_errors() {
    let parser = profile::stophammer();
    let err = parser.parse("<not xml").unwrap_err();
    assert!(err.is_xml());
}

#[test]
fn no_channel_errors() {
    let xml = r#"<?xml version="1.0"?><rss></rss>"#;
    let parser = profile::stophammer();
    let err = parser.parse(xml).unwrap_err();
    assert!(err.is_missing_field());
}

#[test]
fn skips_items_without_guid() {
    let xml = include_str!("fixtures/edge_cases.xml");
    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    // "No GUID Track" should be skipped
    assert_eq!(feed.tracks.len(), 2);
    assert!(feed.tracks.iter().all(|t| !t.title.contains("No GUID")));
}
