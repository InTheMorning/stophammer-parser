//! ADR 0052 sections 2 and 3: the parser exposes the channel
//! `itunes:new-feed-url` and `podcast:locked` as typed fields on
//! `IngestFeedData`.

use stophammer_parser::profile;

fn move_fields_xml() -> &'static str {
    include_str!("fixtures/move_fields.xml")
}

#[test]
fn new_feed_url_is_trimmed() {
    let parser = profile::stophammer();
    let feed = parser.parse(move_fields_xml()).unwrap();

    assert_eq!(
        feed.new_feed_url.as_deref(),
        Some("https://new.example/feed.xml")
    );
}

#[test]
fn empty_new_feed_url_gives_none() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
      <channel>
        <title>Empty New Feed Url</title>
        <podcast:guid>feed-guid</podcast:guid>
        <itunes:new-feed-url></itunes:new-feed-url>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.new_feed_url, None, "an empty element gives no value");
}

#[test]
fn item_level_new_feed_url_has_no_effect_on_the_feed_field() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
      <channel>
        <title>Item Level New Feed Url</title>
        <podcast:guid>feed-guid</podcast:guid>
        <item>
          <guid>track-1</guid>
          <title>Episode One</title>
          <itunes:new-feed-url>https://item.example/feed.xml</itunes:new-feed-url>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(
        feed.new_feed_url, None,
        "an item-level element must not set the feed field"
    );
}

#[test]
fn locked_yes_gives_true_with_owner() {
    let parser = profile::stophammer();
    let feed = parser.parse(move_fields_xml()).unwrap();

    assert_eq!(feed.locked, Some(true));
    assert_eq!(feed.locked_owner.as_deref(), Some("a@b.example"));
}

#[test]
fn locked_no_case_insensitive_gives_false_with_no_owner() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Locked No</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:locked>No</podcast:locked>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.locked, Some(false));
    assert_eq!(feed.locked_owner, None);
}

#[test]
fn locked_other_text_gives_none() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Locked Maybe</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:locked>maybe</podcast:locked>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.locked, None);
}

#[test]
fn feed_with_neither_element_gives_three_none_values() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>No Move Fields</title>
        <podcast:guid>feed-guid</podcast:guid>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.new_feed_url, None);
    assert_eq!(feed.locked, None);
    assert_eq!(feed.locked_owner, None);
}

#[cfg(feature = "serde")]
#[test]
fn feed_with_neither_element_has_no_move_field_keys_in_json() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>No Move Fields</title>
        <podcast:guid>feed-guid</podcast:guid>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    let value = serde_json::to_value(&feed).expect("feed serializes");
    let object = value.as_object().expect("feed serializes to a JSON object");

    assert!(
        !object.contains_key("new_feed_url"),
        "an absent new_feed_url must not appear as a JSON key"
    );
    assert!(
        !object.contains_key("locked"),
        "an absent locked must not appear as a JSON key"
    );
    assert!(
        !object.contains_key("locked_owner"),
        "an absent locked_owner must not appear as a JSON key"
    );
}
