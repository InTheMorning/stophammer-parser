use stophammer_parser::{IngestValueBlock, RouteType, profile};

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

/// An item `podcast:value` block with no recipients gives one value block and
/// empty `payment_routes`. The reader then sees that the item has a block and
/// does not use the channel block (musicindex ADR 0093 R-TAG-6 item 1).
#[test]
fn item_block_with_no_recipients_gives_a_value_block() {
    let xml = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel>
      <title>Empty Item Block</title><podcast:guid>empty-item-block</podcast:guid>
      <podcast:value type="lightning" method="keysend">
        <podcast:valueRecipient name="Channel" type="node" address="channel-node" split="100" />
      </podcast:value>
      <item><guid>track</guid><title>Track</title>
        <podcast:value type="lightning" method="keysend"></podcast:value>
      </item>
      <podcast:liveItem status="live"><guid>live</guid><title>Live</title>
        <podcast:value type="lightning" method="keysend"></podcast:value>
      </podcast:liveItem>
    </channel></rss>"#;
    let feed = profile::stophammer().parse(xml).expect("feed parses");
    let empty_block = IngestValueBlock {
        value_type: "lightning".to_owned(),
        value_method: Some("keysend".to_owned()),
        block_index: 0,
    };

    let track = &feed.tracks[0];
    assert!(
        track.payment_routes.is_empty(),
        "the item block has no route"
    );
    assert_eq!(track.value_blocks, vec![empty_block.clone()]);

    let live_item = &feed.live_items[0];
    assert!(live_item.payment_routes.is_empty());
    assert_eq!(live_item.value_blocks, vec![empty_block.clone()]);

    assert_eq!(feed.feed_value_blocks, vec![empty_block]);
    assert_eq!(feed.feed_payment_routes.len(), 1);
}

/// The `block_index` of each value block is the `block_index` of the routes of
/// that block. A block with no recipients keeps its position in the list.
#[test]
fn value_block_index_matches_its_routes() {
    let feed = profile::stophammer()
        .parse(include_str!("fixtures/value_blocks.xml"))
        .expect("fixture parses");
    let blocks = &feed.feed_value_blocks;
    assert_eq!(blocks.len(), 2);
    for route in &feed.feed_payment_routes {
        let block = &blocks[route.block_index as usize];
        assert_eq!(block.block_index, route.block_index);
        assert_eq!(block.value_type, route.value_type);
        assert_eq!(block.value_method, route.value_method);
    }

    let xml = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0"><channel>
      <title>Two Blocks</title><podcast:guid>two-blocks</podcast:guid>
      <item><guid>track</guid><title>Track</title>
        <podcast:value type="usdc" method="eip3009"></podcast:value>
        <podcast:value type="lightning" method="keysend">
          <podcast:valueRecipient type="node" address="item-node" split="100" />
        </podcast:value>
      </item>
    </channel></rss>"#;
    let feed = profile::stophammer().parse(xml).expect("feed parses");
    let track = &feed.tracks[0];
    assert_eq!(track.value_blocks.len(), 2);
    assert_eq!(track.value_blocks[0].value_type, "usdc");
    assert_eq!(track.value_blocks[1].value_type, "lightning");
    assert_eq!(track.payment_routes.len(), 1);
    assert_eq!(track.payment_routes[0].block_index, 1);
    assert_eq!(track.value_blocks[1].block_index, 1);
}
