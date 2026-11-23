//! musicindex ADR 0093 R-FEED-2, R-FEED-3 and R-FEED-4.

use std::fmt::Write as _;

use stophammer_parser::{FeeReading, RouteType, check, profile};

const NS_CURRENT: &str = "https://podcastindex.org/namespace/1.0";
const NS_GITHUB: &str =
    "https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md";

/// A feed that uses the prefix `pi` for the Podcast Namespace.
fn feed_with_prefix_pi(namespace: &str) -> String {
    format!(
        r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:pi="{namespace}">
  <channel>
    <title>Prefix Feed</title>
    <pi:guid>feed-guid-pi</pi:guid>
    <pi:medium>music</pi:medium>
    <pi:remoteItem feedGuid="publisher-guid" medium="publisher"/>
    <pi:value type="lightning" method="keysend">
      <pi:valueRecipient name="Artist" type="node" address="pubkey-1" split="100"/>
    </pi:value>
    <item>
      <guid>item-1</guid>
      <title>Song</title>
      <pi:value type="lightning" method="keysend">
        <pi:valueRecipient name="Item Artist" type="node" address="pubkey-2" split="95"/>
        <pi:valueTimeSplit startTime="10" duration="30">
          <pi:remoteItem feedGuid="remote-feed"/>
        </pi:valueTimeSplit>
      </pi:value>
    </item>
    <pi:liveItem status="live">
      <guid>live-1</guid>
      <title>Live Show</title>
    </pi:liveItem>
  </channel>
</rss>"#
    )
}

fn assert_reads_prefix_pi(namespace: &str) {
    let feed = profile::stophammer()
        .parse(&feed_with_prefix_pi(namespace))
        .unwrap();

    assert_eq!(feed.feed_guid, "feed-guid-pi");
    assert_eq!(feed.raw_medium.as_deref(), Some("music"));
    assert_eq!(feed.remote_items.len(), 1);
    assert_eq!(feed.remote_items[0].remote_feed_guid, "publisher-guid");
    assert_eq!(feed.feed_payment_routes.len(), 1);
    assert_eq!(feed.feed_payment_routes[0].address, "pubkey-1");

    assert_eq!(feed.tracks.len(), 1);
    let track = &feed.tracks[0];
    assert_eq!(track.payment_routes.len(), 1);
    assert_eq!(track.payment_routes[0].address, "pubkey-2");
    assert_eq!(track.value_time_splits.len(), 1);
    assert_eq!(
        track.value_time_splits[0].remote_feed_guid.as_deref(),
        Some("remote-feed")
    );

    assert_eq!(feed.live_items.len(), 1);
    assert_eq!(feed.live_items[0].status, "live");

    let snapshot = feed.podcast_namespace.expect("namespace snapshot");
    assert!(snapshot.tags.iter().any(|tag| tag.tag == "podcast:guid"));
}

#[test]
fn r_feed_3_reads_the_current_namespace_with_another_prefix() {
    assert_reads_prefix_pi(NS_CURRENT);
}

#[test]
fn r_feed_3_reads_the_github_namespace_with_another_prefix() {
    assert_reads_prefix_pi(NS_GITHUB);
}

const MISSING_FIELDS: &str = r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
  <channel>
    <podcast:medium>music</podcast:medium>
    <item>
      <title>No GUID</title>
    </item>
    <item>
      <guid>no-title</guid>
    </item>
    <item>
      <guid>full</guid>
      <title>Full</title>
    </item>
    <podcast:liveItem>
      <guid>live-no-status</guid>
      <title>Live</title>
    </podcast:liveItem>
  </channel>
</rss>"#;

#[test]
fn r_feed_2_read_gives_the_values_with_no_required_field() {
    let values = profile::stophammer().read(MISSING_FIELDS).unwrap();

    assert_eq!(values.feed_guid, None);
    assert_eq!(values.title, None);
    assert_eq!(values.raw_medium.as_deref(), Some("music"));

    assert_eq!(values.tracks.len(), 3);
    assert_eq!(values.tracks[0].track_guid, None);
    assert_eq!(values.tracks[0].title.as_deref(), Some("No GUID"));
    assert_eq!(values.tracks[1].track_guid.as_deref(), Some("no-title"));
    assert_eq!(values.tracks[1].title, None);

    assert_eq!(values.live_items.len(), 1);
    assert_eq!(values.live_items[0].status, None);
}

#[test]
fn r_feed_2_the_stophammer_check_keeps_its_required_fields() {
    let error = profile::stophammer().parse(MISSING_FIELDS).unwrap_err();
    assert!(error.is_missing_field());
    assert_eq!(error.raw_medium(), Some("music"));

    let with_feed_fields = MISSING_FIELDS.replace(
        "<podcast:medium>",
        "<title>Feed</title><podcast:guid>feed-guid</podcast:guid><podcast:medium>",
    );
    let values = profile::stophammer().read(&with_feed_fields).unwrap();
    let feed = check::required_fields(values).unwrap();

    assert_eq!(feed.feed_guid, "feed-guid");
    assert_eq!(feed.tracks.len(), 1);
    assert_eq!(feed.tracks[0].track_guid, "full");
    assert!(feed.live_items.is_empty());
    assert_eq!(
        profile::stophammer().parse(&with_feed_fields).unwrap(),
        feed
    );
}

/// One value recipient for each `fee` form, in this sequence.
const FEE_FORMS: &[(Option<&str>, FeeReading)] = &[
    (Some("true"), FeeReading::Fee),
    (Some("TRUE"), FeeReading::Fee),
    (Some("1"), FeeReading::Fee),
    (Some("yes"), FeeReading::Fee),
    (Some(" Yes "), FeeReading::Fee),
    (Some("false"), FeeReading::NotFee),
    (Some("0"), FeeReading::NotFee),
    (Some("no"), FeeReading::NotFee),
    (Some("NO"), FeeReading::NotFee),
    (Some(""), FeeReading::NotFee),
    (None, FeeReading::NotFee),
    (Some("maybe"), FeeReading::Unknown),
    (Some("2"), FeeReading::Unknown),
];

fn fee_feed() -> String {
    let mut recipients = String::new();
    for (index, (fee, _)) in FEE_FORMS.iter().enumerate() {
        let fee = fee.map_or_else(String::new, |fee| format!(r#" fee="{fee}""#));
        write!(
            recipients,
            r#"<podcast:valueRecipient address="addr-{index}" split="1"{fee}/>"#
        )
        .unwrap();
    }
    format!(
        r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
  <channel>
    <title>Fee Feed</title>
    <podcast:guid>fee-feed</podcast:guid>
    <podcast:value type="lightning" method="keysend">{recipients}</podcast:value>
  </channel>
</rss>"#
    )
}

#[test]
fn r_feed_4_reads_each_fee_form_and_keeps_each_recipient() {
    let feed = profile::stophammer().parse(&fee_feed()).unwrap();
    let routes = &feed.feed_payment_routes;

    assert_eq!(routes.len(), FEE_FORMS.len());
    for (route, (text, reading)) in routes.iter().zip(FEE_FORMS) {
        assert_eq!(route.fee_text.as_deref(), *text, "{text:?}");
        assert_eq!(route.fee, reading.is_fee(), "{text:?}");
        assert_eq!(
            route.fee_warning(),
            *reading == FeeReading::Unknown,
            "{text:?}"
        );
    }
}

#[test]
fn r_feed_4_gives_the_text_and_the_parsed_form_of_type_and_split() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:podcast="https://podcastindex.org/namespace/1.0">
  <channel>
    <title>Text Feed</title>
    <podcast:guid>text-feed</podcast:guid>
    <item>
      <guid>item-1</guid>
      <title>Song</title>
      <podcast:value type="lightning" method="keysend">
        <podcast:valueRecipient address="a" split="60" fee=""/>
        <podcast:valueRecipient address="b@example.com" type=" LNAddress " split="many"/>
        <podcast:valueRecipient address="c" type="Wallet" split=" 5 "/>
        <podcast:valueTimeSplit startTime="0">
          <podcast:valueRecipient address="d" type="node" split="40" fee="Yes"/>
        </podcast:valueTimeSplit>
      </podcast:value>
    </item>
  </channel>
</rss>"#;
    let feed = profile::stophammer().parse(xml).unwrap();
    let routes = &feed.tracks[0].payment_routes;
    assert_eq!(routes.len(), 3);

    // A missing `type` gives the default in the parsed form only.
    assert_eq!(routes[0].type_text, None);
    assert_eq!(routes[0].route_type, RouteType::Node);
    assert_eq!(routes[0].split_text.as_deref(), Some("60"));
    assert_eq!(routes[0].split, 60);
    assert_eq!(routes[0].fee_text.as_deref(), Some(""));
    assert!(!routes[0].fee);
    assert!(!routes[0].fee_warning());

    assert_eq!(routes[1].type_text.as_deref(), Some(" LNAddress "));
    assert_eq!(routes[1].route_type, RouteType::Lnaddress);
    assert_eq!(routes[1].split_text.as_deref(), Some("many"));
    assert_eq!(routes[1].split, 0);
    assert_eq!(routes[1].fee_text, None);

    assert_eq!(routes[2].type_text.as_deref(), Some("Wallet"));
    assert_eq!(routes[2].route_type, RouteType::Other("wallet".to_owned()));
    assert_eq!(routes[2].split_text.as_deref(), Some(" 5 "));
    assert_eq!(routes[2].split, 0);

    let local = &feed.tracks[0].value_time_splits[0].recipients[0];
    assert_eq!(local.type_text.as_deref(), Some("node"));
    assert_eq!(local.split_text.as_deref(), Some("40"));
    assert_eq!(local.fee_text.as_deref(), Some("Yes"));
    assert!(local.fee);
}
