use stophammer_parser::{RouteType, profile};

#[test]
fn parses_route_type_and_block_metadata() {
    let feed = profile::stophammer()
        .parse(include_str!("fixtures/value_blocks.xml"))
        .expect("fixture parses");

    assert_eq!(feed.feed_payment_routes.len(), 2);
    let lightning = &feed.feed_payment_routes[0];
    assert_eq!(lightning.block_index, 0);
    assert_eq!(lightning.value_type, "lightning");
    assert_eq!(lightning.value_method, None);
    assert_eq!(lightning.route_type, RouteType::Node);

    let usdc = &feed.feed_payment_routes[1];
    assert_eq!(usdc.block_index, 1);
    assert_eq!(usdc.value_type, "usdc");
    assert_eq!(usdc.value_method.as_deref(), Some("eip3009"));
    assert_eq!(usdc.route_type, RouteType::Other("address".to_owned()));
    assert_eq!(usdc.address, "eip155:8453:0x1234567890abcdef");

    let track = &feed.tracks[0];
    assert_eq!(track.payment_routes[0].value_type, "usdc");
    assert_eq!(
        track.payment_routes[0].value_method.as_deref(),
        Some("eip3009")
    );
    assert_eq!(track.payment_routes[0].block_index, 0);
    let split = &track.value_time_splits[0];
    assert_eq!(split.value_type, "usdc");
    assert_eq!(split.value_method.as_deref(), Some("eip3009"));
    assert_eq!(split.block_index, 0);
    assert_eq!(split.recipients[0].value_type, split.value_type);
    assert_eq!(split.recipients[0].value_method, split.value_method);
    assert_eq!(split.recipients[0].block_index, split.block_index);
}

#[test]
fn normalizes_value_block_attributes() {
    let xml = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel>
      <title>Defaults</title><podcast:guid>defaults</podcast:guid>
      <podcast:value type="   "><podcast:valueRecipient address="node" /></podcast:value>
      <item><guid>track</guid><title>Track</title>
        <podcast:value type=" USDC "><podcast:valueRecipient address="node" /></podcast:value>
      </item>
    </channel></rss>"#;
    let feed = profile::stophammer().parse(xml).expect("feed parses");
    assert_eq!(feed.feed_payment_routes[0].value_type, "lightning");
    assert_eq!(feed.feed_payment_routes[0].route_type, RouteType::Node);
    assert_eq!(feed.tracks[0].payment_routes[0].value_type, "usdc");
    assert_eq!(feed.tracks[0].payment_routes[0].value_method, None);
}

#[cfg(feature = "serde")]
#[test]
fn route_type_uses_plain_json_strings() {
    let other = RouteType::Other("address".to_owned());
    let json = serde_json::to_string(&other).expect("serializes");
    assert_eq!(json, r#""address""#);
    assert_eq!(
        serde_json::from_str::<RouteType>(&json).expect("reads"),
        other
    );
    assert_eq!(
        serde_json::from_str::<RouteType>(r#""node""#).expect("reads known type"),
        RouteType::Node
    );
}
