use stophammer_parser::phase::Phase;
use stophammer_parser::profile;

fn basic_xml() -> &'static str {
    include_str!("fixtures/basic.xml")
}

#[test]
fn rss2_only_excludes_itunes_fields() {
    let parser = profile::minimal();
    let feed = parser.parse(basic_xml()).unwrap();

    // RSS2 core + Phase1 only: no itunes fields
    assert_eq!(feed.title, "Test Podcast");
    assert_eq!(feed.feed_guid, "feed-guid-123");
    assert_eq!(feed.language.as_deref(), Some("en"));

    // iTunes fields should be None (not extracted)
    assert_eq!(feed.itunes_type, None);
    assert_eq!(feed.author_name, None);
    assert_eq!(feed.owner_name, None);
    assert_eq!(feed.image_url, None); // itunes:image is Itunes phase

    // Track itunes fields
    let track = &feed.tracks[0];
    assert_eq!(track.duration_secs, None); // itunes:duration
    assert_eq!(track.track_number, None); // itunes:episode, but Phase1 has podcast:episode fallback
}

#[test]
fn phase2_enables_payments() {
    let parser = profile::stophammer_phases_only(&[
        Phase::Rss2Core,
        Phase::Phase1,
        Phase::Phase2,
    ]);
    let xml = include_str!("fixtures/payment.xml");
    let feed = parser.parse(xml).unwrap();

    assert!(!feed.feed_payment_routes.is_empty());
    assert!(!feed.tracks[0].payment_routes.is_empty());
    // Phase3 not enabled, so no VTS
    assert!(feed.tracks[0].value_time_splits.is_empty());
}

#[test]
fn all_phases_extracts_everything() {
    let parser = profile::stophammer();
    let xml = include_str!("fixtures/payment.xml");
    let feed = parser.parse(xml).unwrap();

    assert!(!feed.feed_payment_routes.is_empty());
    assert!(!feed.tracks[0].payment_routes.is_empty());
    assert!(!feed.tracks[0].value_time_splits.is_empty());
}
