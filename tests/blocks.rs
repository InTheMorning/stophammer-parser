//! ADR 0057 §5: the parser reads channel-level `podcast:block` tags
//! as typed entries with raw values.

use stophammer_parser::profile;

#[test]
fn channel_with_two_blocks_gives_two_entries_in_order() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
      <channel>
        <title>Two Blocks</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:block>yes</podcast:block>
        <podcast:block id="musicindex">no</podcast:block>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(
        feed.blocks.len(),
        2,
        "ADR 0057 §5: two blocks must be parsed"
    );
    assert_eq!(
        feed.blocks[0].id, None,
        "ADR 0057 §5: first block has no id"
    );
    assert_eq!(
        feed.blocks[0].value, "yes",
        "ADR 0057 §5: first block value is yes"
    );
    assert_eq!(
        feed.blocks[1].id.as_deref(),
        Some("musicindex"),
        "ADR 0057 §5: second block id is musicindex"
    );
    assert_eq!(
        feed.blocks[1].value, "no",
        "ADR 0057 §5: second block value is no"
    );
}

#[test]
fn block_id_is_trimmed() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Block With Padding</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:block id=" podcastindex ">Yes</podcast:block>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(
        feed.blocks.len(),
        1,
        "ADR 0057 §5: padded id block must be parsed"
    );
    assert_eq!(
        feed.blocks[0].id.as_deref(),
        Some("podcastindex"),
        "ADR 0057 §5: id must be trimmed to podcastindex"
    );
    assert_eq!(
        feed.blocks[0].value, "Yes",
        "ADR 0057 §5: value case is preserved"
    );
}

#[test]
fn block_with_empty_id_gives_none() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Block With Empty Id</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:block id="">yes</podcast:block>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.blocks.len(), 1);
    assert_eq!(
        feed.blocks[0].id, None,
        "ADR 0057 §5: empty id attribute gives None"
    );
}

#[test]
fn block_with_empty_text_is_kept() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Block With Empty Text</title>
        <podcast:guid>feed-guid</podcast:guid>
        <podcast:block id="musicindex"></podcast:block>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(
        feed.blocks.len(),
        1,
        "ADR 0057 §5: empty text block must be kept"
    );
    assert_eq!(
        feed.blocks[0].value, "",
        "ADR 0057 §5: empty text is preserved as empty string"
    );
}

#[test]
fn item_level_block_gives_no_entry() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Item Level Block</title>
        <podcast:guid>feed-guid</podcast:guid>
        <item>
          <guid>track-1</guid>
          <title>Episode One</title>
          <podcast:block>yes</podcast:block>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(
        feed.blocks.len(),
        0,
        "ADR 0057 §5: item-level block must not be extracted"
    );
}

#[test]
fn itunes_block_gives_no_entry() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
         xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
      <channel>
        <title>iTunes Block</title>
        <podcast:guid>feed-guid</podcast:guid>
        <itunes:block>Yes</itunes:block>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(
        feed.blocks.len(),
        0,
        "ADR 0057 §5: itunes:block must not be extracted"
    );
}

#[test]
fn feed_with_no_blocks_serializes_with_no_blocks_key() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>No Blocks</title>
        <podcast:guid>feed-guid</podcast:guid>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(
        feed.blocks.len(),
        0,
        "ADR 0057 §5: feed with no blocks must have empty blocks field"
    );

    #[cfg(feature = "serde")]
    {
        let json = serde_json::to_string(&feed).expect("serialize to json");
        assert!(
            !json.contains("blocks"),
            "ADR 0057 §5: empty blocks field must not appear in JSON"
        );
    }
}
