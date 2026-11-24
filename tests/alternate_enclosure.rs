//! `podcast:alternateEnclosure` gives its URIs in `podcast:source` child
//! elements. The parser keeps each source, and `url` is the first source
//! with the scheme `http` or `https`.

use stophammer_parser::profile;

fn item_feed(item_body: &str) -> String {
    format!(
        r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
  <channel>
    <title>Alternates</title>
    <podcast:guid>feed-alternates</podcast:guid>
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

/// The form of the real feed `themnshow.xml` (partytime fixture, line 93):
/// two alternates, each with one `podcast:source` and one
/// `podcast:integrity`, and no `url` attribute.
#[test]
fn source_children_give_the_alternate_url() {
    let xml = item_feed(
        r#"<podcast:alternateEnclosure type="audio/mpeg" length="56113920" bitrate="128000" lang="EN" default="true" title="Standard">
        <podcast:source uri="https://op3.dev/e,pg=7a2d292c-8656-5fcf-88d2-31b10e54d7c7/mikeneumann.show/assets/static/themnshow054-20240914-FINAL.mp3"/>
        <podcast:integrity type="sri" value="sha384-8uYM8wvgwX5fUJOX3TMwSMIuVxtgxEzfO39iMjBe/l+R7sr1KRn+h8zMpilupYDR"/>
      </podcast:alternateEnclosure>
      <podcast:alternateEnclosure type="audio/mpeg" length="28089720" bitrate="64000" lang="EN" default="false" title="mono">
        <podcast:source uri="https://op3.dev/e,pg=7a2d292c-8656-5fcf-88d2-31b10e54d7c7/mikeneumann.show/assets/static/themnshow054-20240914-FINAL-mono64.mp3"/>
        <podcast:integrity type="sri" value="sha384-7M2z1g2eMpc8v8TRyk24JXcJTQdKpWkp5+9+L2NUkNzcGHIWeKnMCaEfIhqep+Ka"/>
      </podcast:alternateEnclosure>"#,
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let alternates = &feed.tracks[0].alternate_enclosures;
    assert_eq!(alternates.len(), 2, "each alternate with a source is kept");

    let standard = "https://op3.dev/e,pg=7a2d292c-8656-5fcf-88d2-31b10e54d7c7/mikeneumann.show/assets/static/themnshow054-20240914-FINAL.mp3";
    let mono = "https://op3.dev/e,pg=7a2d292c-8656-5fcf-88d2-31b10e54d7c7/mikeneumann.show/assets/static/themnshow054-20240914-FINAL-mono64.mp3";

    assert_eq!(alternates[0].position, 0);
    assert_eq!(alternates[0].url, standard);
    assert_eq!(alternates[0].sources, vec![standard.to_owned()]);
    assert_eq!(alternates[0].mime_type.as_deref(), Some("audio/mpeg"));
    assert_eq!(alternates[0].bytes, Some(56_113_920));
    assert_eq!(alternates[0].title.as_deref(), Some("Standard"));

    assert_eq!(alternates[1].position, 1);
    assert_eq!(alternates[1].url, mono);
    assert_eq!(alternates[1].sources, vec![mono.to_owned()]);
    assert_eq!(alternates[1].mime_type.as_deref(), Some("audio/mpeg"));
    assert_eq!(alternates[1].bytes, Some(28_089_720));
    assert_eq!(alternates[1].title.as_deref(), Some("mono"));
}

/// An `ipfs://` first source stays in `sources`. `url` is the first `https`
/// source. An empty `uri` is not a source, and a source `uri` is trimmed.
#[test]
fn url_is_the_first_web_source() {
    let xml = item_feed(
        r#"<podcast:alternateEnclosure type="audio/mpeg" length="43200000">
        <podcast:source uri="ipfs://someRandomMpegFile"/>
        <podcast:source uri="  "/>
        <podcast:source uri=" https://example.com/file-0.mp3 "/>
        <podcast:source uri="https://mirror.example.com/file-0.mp3"/>
      </podcast:alternateEnclosure>"#,
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let alternates = &feed.tracks[0].alternate_enclosures;
    assert_eq!(alternates.len(), 1);
    assert_eq!(alternates[0].url, "https://example.com/file-0.mp3");
    assert_eq!(
        alternates[0].sources,
        vec![
            "ipfs://someRandomMpegFile".to_owned(),
            "https://example.com/file-0.mp3".to_owned(),
            "https://mirror.example.com/file-0.mp3".to_owned(),
        ]
    );
}

/// An element with only a `url` attribute keeps the earlier behavior. An
/// element with only a non-web source uses the `url` attribute. An element
/// with no web source and no `url` attribute is skipped, and the position
/// counts only the kept elements.
#[test]
fn url_attribute_is_the_fallback() {
    let xml = item_feed(
        r#"<podcast:alternateEnclosure url="https://cdn.example.com/song.flac" type="audio/flac" length="222" title="Lossless"/>
      <podcast:alternateEnclosure type="audio/mpeg">
        <podcast:source uri="ipfs://onlyIpfs"/>
      </podcast:alternateEnclosure>
      <podcast:alternateEnclosure url="https://cdn.example.com/song.opus" type="audio/ogg">
        <podcast:source uri="ipfs://opusOnIpfs"/>
      </podcast:alternateEnclosure>"#,
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let alternates = &feed.tracks[0].alternate_enclosures;
    assert_eq!(alternates.len(), 2, "the element with no URL is skipped");

    assert_eq!(alternates[0].position, 0);
    assert_eq!(alternates[0].url, "https://cdn.example.com/song.flac");
    assert!(alternates[0].sources.is_empty());
    assert_eq!(alternates[0].mime_type.as_deref(), Some("audio/flac"));

    assert_eq!(alternates[1].position, 1);
    assert_eq!(alternates[1].url, "https://cdn.example.com/song.opus");
    assert_eq!(alternates[1].sources, vec!["ipfs://opusOnIpfs".to_owned()]);
}
