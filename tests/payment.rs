use stophammer_parser::RouteType;
use stophammer_parser::profile;

fn payment_xml() -> &'static str {
    include_str!("fixtures/payment.xml")
}

#[test]
fn extracts_feed_payment_routes() {
    let parser = profile::stophammer();
    let feed = parser.parse(payment_xml()).unwrap();

    assert_eq!(feed.feed_payment_routes.len(), 2);

    let host = &feed.feed_payment_routes[0];
    assert_eq!(host.recipient_name.as_deref(), Some("Feed Host"));
    assert_eq!(host.route_type, RouteType::Node);
    assert_eq!(host.address, "feedpubkey123");
    assert_eq!(host.split, 90);
    assert_eq!(host.custom_key.as_deref(), Some("7629169"));
    assert_eq!(host.custom_value.as_deref(), Some("feedid123"));
    assert!(!host.fee);

    let app = &feed.feed_payment_routes[1];
    assert_eq!(app.recipient_name.as_deref(), Some("App Fee"));
    assert!(app.fee);
    assert_eq!(app.split, 10);
}

#[test]
fn extracts_track_payment_routes() {
    let parser = profile::stophammer();
    let feed = parser.parse(payment_xml()).unwrap();

    let track = &feed.tracks[0];
    assert_eq!(track.payment_routes.len(), 2);

    let artist = &track.payment_routes[0];
    assert_eq!(artist.route_type, RouteType::Node);
    assert_eq!(artist.address, "artistpubkey789");
    assert_eq!(artist.split, 80);

    let ln = &track.payment_routes[1];
    assert_eq!(ln.route_type, RouteType::Lnaddress);
    assert_eq!(ln.address, "artist@getalby.com");
    assert_eq!(ln.split, 20);
}

#[test]
fn extracts_value_time_splits() {
    let parser = profile::stophammer();
    let feed = parser.parse(payment_xml()).unwrap();

    let track = &feed.tracks[0];
    assert_eq!(track.value_time_splits.len(), 2);

    let vts1 = &track.value_time_splits[0];
    assert_eq!(vts1.start_time_secs, 60);
    assert_eq!(vts1.duration_secs, Some(120));
    assert_eq!(vts1.remote_feed_guid, "remote-feed-guid");
    assert_eq!(vts1.remote_item_guid, "remote-item-guid");
    assert_eq!(vts1.split, 50);

    let vts2 = &track.value_time_splits[1];
    assert_eq!(vts2.start_time_secs, 300);
    assert_eq!(vts2.duration_secs, None);
    assert_eq!(vts2.remote_feed_guid, "remote-feed-2");
    assert_eq!(vts2.remote_item_guid, "remote-item-2");
}

#[test]
fn skips_vts_with_remote_percentage() {
    let parser = profile::stophammer();
    let feed = parser.parse(payment_xml()).unwrap();

    let track = &feed.tracks[1];
    assert!(track.value_time_splits.is_empty());
}

#[test]
fn skips_recipients_without_address() {
    let xml = r#"<?xml version="1.0"?>
    <rss xmlns:podcast="https://podcastindex.org/namespace/1.0">
      <channel>
        <title>Test</title>
        <podcast:guid>guid</podcast:guid>
        <item>
          <guid>t1</guid>
          <title>Track</title>
          <podcast:value type="lightning" method="keysend">
            <podcast:valueRecipient name="No Addr" split="100"/>
            <podcast:valueRecipient name="Empty Addr" address="" split="50"/>
            <podcast:valueRecipient name="Valid" address="pubkey123" split="100"/>
          </podcast:value>
        </item>
      </channel>
    </rss>"#;

    let parser = profile::stophammer();
    let feed = parser.parse(xml).unwrap();

    assert_eq!(feed.tracks[0].payment_routes.len(), 1);
    assert_eq!(feed.tracks[0].payment_routes[0].address, "pubkey123");
}
