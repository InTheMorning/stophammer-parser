//! The parser gives the channel `podcast:podroll` as its own field, for
//! stophammer ADR 0107 §1. The entries are not in `remote_items`.

use stophammer_parser::{IngestRemoteFeedRef, profile};

fn feed(channel_body: &str, item_body: &str) -> String {
    format!(
        r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
  <channel>
    <title>Podroll</title>
    <podcast:guid>feed-podroll</podcast:guid>
    <podcast:medium>music</podcast:medium>
    {channel_body}
    <item>
      <guid>item-1</guid>
      <title>Track</title>
      <enclosure url="https://example.com/file-0.mp3" length="43200000" type="audio/mpeg"/>
      {item_body}
    </item>
  </channel>
</rss>"#
    )
}

fn entry(
    position: i64,
    guid: &str,
    url: Option<&str>,
    medium: Option<&str>,
) -> IngestRemoteFeedRef {
    IngestRemoteFeedRef {
        position,
        medium: medium.map(str::to_owned),
        remote_feed_guid: guid.to_owned(),
        remote_feed_url: url.map(str::to_owned),
        rel: None,
        item_guid: None,
        item_title: None,
        publisher_reference: false,
    }
}

#[test]
fn channel_podroll_gives_its_entries() {
    let xml = feed(
        r#"<podcast:remoteItem feedGuid="artist-guid" medium="publisher"/>
    <podcast:podroll>
      <podcast:remoteItem feedGuid=" roll-1 " feedUrl="https://example.com/roll-1.xml" medium="music"/>
      <podcast:remoteItem feedUrl="https://example.com/no-guid.xml"/>
      <podcast:remoteItem feedGuid="roll-2"/>
    </podcast:podroll>"#,
        "",
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");

    assert_eq!(
        feed.remote_items.len(),
        1,
        "the podroll is not a remote item"
    );
    assert_eq!(feed.remote_items[0].remote_feed_guid, "artist-guid");
    assert_eq!(
        feed.podroll,
        vec![
            entry(
                0,
                "roll-1",
                Some("https://example.com/roll-1.xml"),
                Some("music")
            ),
            entry(1, "roll-2", None, None),
        ],
        "each podroll entry with a feedGuid, with its own positions"
    );
}

#[test]
fn item_podroll_gives_no_entry() {
    let xml = feed(
        "",
        r#"<podcast:podroll><podcast:remoteItem feedGuid="roll-1"/></podcast:podroll>"#,
    );
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    assert!(feed.podroll.is_empty());
}

#[test]
fn read_gives_the_podroll() {
    let xml = feed(
        r#"<podcast:podroll><podcast:remoteItem feedGuid="roll-1" medium="music"/></podcast:podroll>"#,
        "",
    );
    let values = profile::stophammer().read(&xml).expect("feed reads");
    assert_eq!(
        values.podroll,
        vec![entry(0, "roll-1", None, Some("music"))]
    );
}
