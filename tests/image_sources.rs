//! The parser gives each image source of a feed as its own value, for
//! musicindex ADR 0096 R-IMAGE-1. The merged `image_url` stays as before.

use stophammer_parser::{IngestImage, IngestSrcsetEntry, profile};

fn feed(channel_body: &str, item_body: &str) -> String {
    format!(
        r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd" version="2.0">
  <channel>
    <title>Images</title>
    <podcast:guid>feed-images</podcast:guid>
    {channel_body}
    <item>
      <guid>item-1</guid>
      <title>Episode</title>
      <enclosure url="https://example.com/file-0.mp3" length="43200000" type="audio/mpeg"/>
      {item_body}
    </item>
  </channel>
</rss>"#
    )
}

#[test]
fn channel_gives_the_itunes_and_the_rss_image() {
    let xml = feed(
        r#"<itunes:image href=" https://example.com/itunes.jpg "/>
    <image><url> https://example.com/rss.jpg </url><title>Images</title></image>"#,
        "",
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    assert_eq!(
        feed.itunes_image_url.as_deref(),
        Some("https://example.com/itunes.jpg")
    );
    assert_eq!(
        feed.rss_image_url.as_deref(),
        Some("https://example.com/rss.jpg")
    );
    assert_eq!(
        feed.image_url.as_deref(),
        Some(" https://example.com/itunes.jpg "),
        "the merged value is not changed"
    );
}

#[test]
fn channel_image_does_not_fill_the_item_itunes_image() {
    let xml = feed(
        r#"<itunes:image href="https://example.com/itunes.jpg"/>"#,
        "",
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let track = &feed.tracks[0];
    assert_eq!(track.itunes_image_url, None);
    assert!(track.podcast_images.is_empty());
    assert!(track.podcast_images_srcset.is_empty());
}

#[test]
fn empty_image_values_give_none() {
    let xml = feed(
        r#"<itunes:image href="  "/>
    <image><url></url></image>"#,
        r#"<itunes:image href=""/>
      <podcast:image href=" "/>"#,
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    assert_eq!(feed.itunes_image_url, None);
    assert_eq!(feed.rss_image_url, None);
    assert_eq!(feed.tracks[0].itunes_image_url, None);
    assert!(
        feed.tracks[0].podcast_images.is_empty(),
        "a podcast:image with an empty href is skipped"
    );
}

#[test]
fn item_gives_each_image_source() {
    let xml = feed(
        "",
        r#"<itunes:image href="https://example.com/item-itunes.jpg"/>
      <podcast:image href="https://example.com/social.jpg" purpose="social" width="1200"/>
      <podcast:image href=" https://example.com/art.jpg " purpose=" artwork social "/>
      <podcast:image href="https://example.com/none.jpg"/>
      <podcast:images srcset="https://example.com/a.jpg 640w, https://example.com/b.jpg 3000w,https://example.com/c.jpg 2x, https://example.com/d.jpg"/>"#,
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let track = &feed.tracks[0];

    assert_eq!(
        track.itunes_image_url.as_deref(),
        Some("https://example.com/item-itunes.jpg")
    );
    assert_eq!(
        track.podcast_images,
        vec![
            IngestImage {
                href: "https://example.com/social.jpg".to_owned(),
                purpose: Some("social".to_owned()),
            },
            IngestImage {
                href: "https://example.com/art.jpg".to_owned(),
                purpose: Some("artwork social".to_owned()),
            },
            IngestImage {
                href: "https://example.com/none.jpg".to_owned(),
                purpose: None,
            },
        ],
        "each podcast:image in the sequence of the feed"
    );
    assert_eq!(
        track.podcast_images_srcset,
        vec![
            IngestSrcsetEntry {
                url: "https://example.com/a.jpg".to_owned(),
                descriptor: Some("640w".to_owned()),
                width: Some(640),
            },
            IngestSrcsetEntry {
                url: "https://example.com/b.jpg".to_owned(),
                descriptor: Some("3000w".to_owned()),
                width: Some(3000),
            },
            IngestSrcsetEntry {
                url: "https://example.com/c.jpg".to_owned(),
                descriptor: Some("2x".to_owned()),
                width: None,
            },
            IngestSrcsetEntry {
                url: "https://example.com/d.jpg".to_owned(),
                descriptor: None,
                width: None,
            },
        ],
        "each srcset candidate with its width"
    );
    assert_eq!(
        track.image_url.as_deref(),
        Some("https://example.com/item-itunes.jpg"),
        "the merged value is not changed"
    );
}

#[test]
fn read_gives_the_same_image_sources() {
    let xml = feed(
        r"<image><url>https://example.com/rss.jpg</url></image>",
        r#"<podcast:image href="https://example.com/art.jpg" purpose="artwork"/>"#,
    );
    let values = profile::stophammer().read(&xml).expect("feed reads");
    assert_eq!(
        values.rss_image_url.as_deref(),
        Some("https://example.com/rss.jpg")
    );
    assert_eq!(values.tracks[0].podcast_images.len(), 1);
}
