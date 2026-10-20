//! `podcast:valueTimeSplit` in both forms the specification permits
//! (stophammer ADR 0073 §1).
//!
//! Each XML body below is a specification example from
//! `podcast-namespace/docs/tags/value-time-split.md` (commit d4036dc), with
//! one change: a `podcast:guid` and an item `guid` are added so that the item
//! parses. These tests are adapted from the research request
//! `claim_value_time_split.rs` to the model of ADR 0073.

use stophammer_parser::{ValueTimeSplitForm, profile};

const SPEC_REMOTE: &str = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
   <channel>
      <title>Metal Showcase</title>
      <podcast:guid>claim-remote</podcast:guid>
      <podcast:medium>music</podcast:medium>
      <item>
         <guid>i1</guid>
         <title>Special interview with Torcon VII</title>
         <podcast:value type="lightning" method="keysend">
            <podcast:valueRecipient name="Alice (Podcaster)" type="node" address="02d5c1bf8b940dc9cadca86d1b0a3c37fbe39cee4c7e839e33bef9174531d27f52" split="95" />
            <podcast:valueRecipient name="Hosting Provider" type="node" address="03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a" split="5" fee="true" />
            <podcast:valueTimeSplit startTime="60" duration="237" remotePercentage="95">
               <podcast:remoteItem itemGuid="https://podcastindex.org/podcast/4148683#1" feedGuid="a94f5cc9-8c58-55fc-91fe-a324087a655b" medium="music" />
            </podcast:valueTimeSplit>
            <podcast:valueTimeSplit startTime="330" duration="53" remoteStartTime="174" remotePercentage="95">
               <podcast:remoteItem itemGuid="https://podcastindex.org/podcast/4148683#3" feedGuid="a94f5cc9-8c58-55fc-91fe-a324087a655b" medium="music" />
            </podcast:valueTimeSplit>
         </podcast:value>
      </item>
   </channel>
</rss>"#;

const SPEC_LOCAL: &str = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
   <channel>
      <title>Cool Pod</title>
      <podcast:guid>claim-local</podcast:guid>
      <podcast:medium>podcast</podcast:medium>
      <item>
         <guid>i1</guid>
         <title>Adam Hates the word "pod" (and I do, too)</title>
         <podcast:value type="lightning" method="keysend">
            <podcast:valueRecipient name="Alice (Podcaster)" type="node" address="02d5c1bf8b940dc9cadca86d1b0a3c37fbe39cee4c7e839e33bef9174531d27f52" split="95" />
            <podcast:valueRecipient name="Hosting Provider" type="node" address="03ae9f91a0cb8ff43840e3c322c4c61f019d8c1c3cea15a25cfc425ac605e61a4a" split="5" fee="true" />
            <podcast:valueTimeSplit startTime="63" duration="388">
               <podcast:valueRecipient name="Alice (Podcaster)" type="node" address="02d5c1bf8b940dc9cadca86d1b0a3c37fbe39cee4c7e839e33bef9174531d27f52" split="85" />
               <podcast:valueRecipient name="Jimbob (Guest)" type="node" address="02dd306e68c46681aa21d88a436fb35355a8579dd30201581cefa17cb179fc4c15" split="10" />
            </podcast:valueTimeSplit>
            <podcast:valueTimeSplit startTime="367" duration="246">
               <podcast:valueRecipient name="Alice (Podcaster)" type="node" address="02d5c1bf8b940dc9cadca86d1b0a3c37fbe39cee4c7e839e33bef9174531d27f52" split="85" />
               <podcast:valueRecipient name="Bobjim (Guest)" type="node" address="032f4ffbbafffbe51726ad3c164a3d0d37ec27bc67b29a159b0f49ae8ac21b8508" split="10" />
            </podcast:valueTimeSplit>
         </podcast:value>
      </item>
   </channel>
</rss>"#;

fn split_count(xml: &str) -> usize {
    let feed = profile::stophammer().parse(xml).expect("feed parses");
    feed.tracks.iter().map(|t| t.value_time_splits.len()).sum()
}

/// The remote-item example of the specification keeps both splits, each with
/// the stated `remotePercentage`. The second split also keeps its stated
/// `remoteStartTime`.
#[test]
fn spec_remote_example_keeps_both_splits() {
    let feed = profile::stophammer()
        .parse(SPEC_REMOTE)
        .expect("feed parses");
    let splits = &feed.tracks[0].value_time_splits;
    assert_eq!(
        splits.len(),
        2,
        "both splits of the remote example are kept"
    );
    assert!(
        splits.iter().all(|s| s.form == ValueTimeSplitForm::Remote),
        "both splits are in the remote form"
    );
    assert!(
        splits.iter().all(|s| s.remote_percentage == Some(95.0)),
        "stated remotePercentage: {:?}",
        splits
            .iter()
            .map(|s| s.remote_percentage)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        splits[1].remote_start_time_secs,
        Some(174.0),
        "the second split keeps its stated remoteStartTime"
    );
}

/// The locally specified example of the specification keeps both splits,
/// each with its two recipients.
#[test]
fn spec_local_example_keeps_both_splits() {
    let feed = profile::stophammer()
        .parse(SPEC_LOCAL)
        .expect("feed parses");
    let splits = &feed.tracks[0].value_time_splits;
    assert_eq!(splits.len(), 2, "both local splits are kept");
    for split in splits {
        assert_eq!(split.form, ValueTimeSplitForm::Local);
        assert_eq!(
            split.recipients.len(),
            2,
            "each local split has two recipients"
        );
        assert_eq!(split.recipients[0].split, 85);
        assert_eq!(split.recipients[1].split, 10);
    }
}

/// `startTime` is a number of seconds, so a fraction is a valid value.
#[test]
#[expect(
    clippy::float_cmp,
    reason = "the source states a short decimal, so the parsed value is exact"
)]
fn fractional_start_time_is_kept() {
    let xml = SPEC_REMOTE
        .replace(r#"remotePercentage="95""#, r#"split="50""#)
        .replace(r#"startTime="60""#, r#"startTime="12.5""#);
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let splits = &feed.tracks[0].value_time_splits;
    assert_eq!(
        splits.len(),
        2,
        "both splits are kept with a fractional startTime"
    );
    assert!(
        splits.iter().any(|s| s.start_time_secs == 12.5),
        "stated start times: {:?}",
        splits.iter().map(|s| s.start_time_secs).collect::<Vec<_>>()
    );
}

/// With no stated `remotePercentage`, the split gives `None`. The response
/// of task 002 applies the default of 100.
#[test]
fn absent_remote_percentage_is_none() {
    let xml = SPEC_REMOTE.replace(r#" remotePercentage="95""#, "");
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let splits = &feed.tracks[0].value_time_splits;
    assert_eq!(
        splits.len(),
        2,
        "both splits are kept without remotePercentage"
    );
    assert!(
        splits.iter().all(|s| s.remote_percentage.is_none()),
        "stated: {:?}",
        splits
            .iter()
            .map(|s| s.remote_percentage)
            .collect::<Vec<_>>()
    );
}

/// A live feed. Its XML holds three `valueTimeSplit` elements, each with the
/// same `remotePercentage`.
#[test]
fn podhome_feed_keeps_its_three_splits() {
    let xml = include_str!("fixtures/podhome_satellite_skirmish.xml");
    assert_eq!(xml.matches("<podcast:valueTimeSplit ").count(), 3);
    let feed = profile::stophammer().parse(xml).expect("feed parses");
    let splits = &feed.tracks[0].value_time_splits;
    assert_eq!(splits.len(), 3);
    assert!(
        splits.iter().all(|s| s.remote_percentage == Some(95.0)),
        "stated: {:?}",
        splits
            .iter()
            .map(|s| s.remote_percentage)
            .collect::<Vec<_>>()
    );
}

/// The non-spec `split` attribute has no meaning for the parser. The parser
/// ignores it and keeps the split.
#[test]
fn non_spec_split_attribute_is_ignored() {
    let xml = SPEC_REMOTE.replace(r#"remotePercentage="95""#, r#"split="50""#);
    assert_eq!(split_count(&xml), 2, "the split stays");
    let feed = profile::stophammer().parse(&xml).expect("feed parses");
    let splits = &feed.tracks[0].value_time_splits;
    assert!(
        splits.iter().all(|s| s.remote_percentage.is_none()),
        "the ignored split attribute leaves remote_percentage unset"
    );
}

/// A split with a `remoteItem` child and a `valueRecipient` child mixes the
/// two forms. The specification permits neither, so the parser drops it.
#[test]
fn mixed_remote_and_local_children_is_dropped() {
    let xml = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
   <channel>
      <title>Test</title>
      <podcast:guid>mixed-children</podcast:guid>
      <item>
         <guid>i1</guid>
         <title>Mixed children split</title>
         <podcast:value type="lightning" method="keysend">
            <podcast:valueTimeSplit startTime="10" duration="20">
               <podcast:remoteItem feedGuid="feed-guid" itemGuid="item-guid" />
               <podcast:valueRecipient name="Alice" address="pubkey" split="100" />
            </podcast:valueTimeSplit>
         </podcast:value>
      </item>
   </channel>
</rss>"#;
    assert_eq!(
        split_count(xml),
        0,
        "a split with children of both kinds is dropped"
    );
}

/// A `remotePercentage` that does not parse as a number drops the whole
/// split.
#[test]
fn unparseable_remote_percentage_is_dropped() {
    let xml = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
   <channel>
      <title>Test</title>
      <podcast:guid>bad-percentage</podcast:guid>
      <item>
         <guid>i1</guid>
         <title>Bad remotePercentage</title>
         <podcast:value type="lightning" method="keysend">
            <podcast:valueTimeSplit startTime="10" duration="20" remotePercentage="abc">
               <podcast:remoteItem feedGuid="feed-guid" itemGuid="item-guid" />
            </podcast:valueTimeSplit>
         </podcast:value>
      </item>
   </channel>
</rss>"#;
    assert_eq!(
        split_count(xml),
        0,
        "an unreadable remotePercentage drops the split"
    );
}

/// `itemGuid` is optional in the remote form. A remote split with no
/// `itemGuid` is kept, with `remote_item_guid` `None`.
#[test]
fn remote_split_with_no_item_guid_is_kept() {
    let xml = r#"<rss xmlns:podcast="https://podcastindex.org/namespace/1.0" version="2.0">
   <channel>
      <title>Test</title>
      <podcast:guid>no-item-guid</podcast:guid>
      <item>
         <guid>i1</guid>
         <title>No itemGuid</title>
         <podcast:value type="lightning" method="keysend">
            <podcast:valueTimeSplit startTime="10" duration="20">
               <podcast:remoteItem feedGuid="feed-guid" />
            </podcast:valueTimeSplit>
         </podcast:value>
      </item>
   </channel>
</rss>"#;
    let feed = profile::stophammer().parse(xml).expect("feed parses");
    let splits = &feed.tracks[0].value_time_splits;
    assert_eq!(splits.len(), 1, "the split is kept with no itemGuid");
    assert_eq!(splits[0].remote_item_guid, None);
    assert_eq!(splits[0].remote_feed_guid.as_deref(), Some("feed-guid"));
}
