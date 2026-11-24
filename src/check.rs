//! Required-field checks that an app selects (musicindex ADR 0093 R-FEED-2).
//!
//! [`FeedParser::read`](crate::FeedParser::read) gives the values of a feed
//! with no check. A check in this module makes the required fields of one
//! app into plain values.

use crate::error::{ErrorKind, ParseError};
use crate::types::{FeedValues, IngestFeedData, IngestLiveItemData, IngestTrackData};

/// The stophammer required-field check (musicindex ADR 0093 R-FEED-2 item 2).
///
/// - The feed needs a title and a feed GUID. If one is missing, the check
///   gives an error. The title is examined first.
/// - An item needs an item GUID and a title. The check drops an item with a
///   missing field.
/// - A live item needs a GUID, a title and a status. The check drops a live
///   item with a missing field.
///
/// # Errors
///
/// Returns `ParseError` when the feed has no title or no feed GUID. The error
/// keeps the raw `podcast:medium` (stophammer ADR 0074 §3a).
pub fn required_fields(values: FeedValues) -> Result<IngestFeedData, ParseError> {
    let Some(title) = values.title else {
        return Err(ParseError {
            kind: ErrorKind::NoTitle {
                raw_medium: values.raw_medium,
            },
        });
    };
    let Some(feed_guid) = values.feed_guid else {
        return Err(ParseError {
            kind: ErrorKind::NoGuid {
                raw_medium: values.raw_medium,
            },
        });
    };

    Ok(IngestFeedData {
        feed_guid,
        title,
        description: values.description,
        image_url: values.image_url,
        itunes_image_url: values.itunes_image_url,
        rss_image_url: values.rss_image_url,
        language: values.language,
        explicit: values.explicit,
        itunes_type: values.itunes_type,
        raw_medium: values.raw_medium,
        author_name: values.author_name,
        owner_name: values.owner_name,
        pub_date: values.pub_date,
        last_build_date: values.last_build_date,
        new_feed_url: values.new_feed_url,
        locked: values.locked,
        locked_owner: values.locked_owner,
        remote_items: values.remote_items,
        persons: values.persons,
        entity_ids: values.entity_ids,
        links: values.links,
        blocks: values.blocks,
        podcast_namespace: values.podcast_namespace,
        feed_payment_routes: values.feed_payment_routes,
        feed_value_blocks: values.feed_value_blocks,
        live_items: values
            .live_items
            .into_iter()
            .filter_map(required_live_item_fields)
            .collect(),
        tracks: values
            .tracks
            .into_iter()
            .filter_map(required_track_fields)
            .collect(),
    })
}

fn required_track_fields(track: IngestTrackData<Option<String>>) -> Option<IngestTrackData> {
    Some(IngestTrackData {
        track_guid: track.track_guid?,
        title: track.title?,
        pub_date: track.pub_date,
        duration_secs: track.duration_secs,
        image_url: track.image_url,
        itunes_image_url: track.itunes_image_url,
        podcast_images: track.podcast_images,
        podcast_images_srcset: track.podcast_images_srcset,
        language: track.language,
        enclosure_url: track.enclosure_url,
        enclosure_type: track.enclosure_type,
        enclosure_bytes: track.enclosure_bytes,
        alternate_enclosures: track.alternate_enclosures,
        track_number: track.track_number,
        season: track.season,
        explicit: track.explicit,
        description: track.description,
        author_name: track.author_name,
        persons: track.persons,
        entity_ids: track.entity_ids,
        links: track.links,
        payment_routes: track.payment_routes,
        value_blocks: track.value_blocks,
        value_time_splits: track.value_time_splits,
        transcripts: track.transcripts,
        remote_items: track.remote_items,
    })
}

fn required_live_item_fields(
    live_item: IngestLiveItemData<Option<String>>,
) -> Option<IngestLiveItemData> {
    Some(IngestLiveItemData {
        live_item_guid: live_item.live_item_guid?,
        title: live_item.title?,
        status: live_item.status?,
        start_at: live_item.start_at,
        end_at: live_item.end_at,
        content_link: live_item.content_link,
        pub_date: live_item.pub_date,
        duration_secs: live_item.duration_secs,
        image_url: live_item.image_url,
        language: live_item.language,
        enclosure_url: live_item.enclosure_url,
        enclosure_type: live_item.enclosure_type,
        enclosure_bytes: live_item.enclosure_bytes,
        alternate_enclosures: live_item.alternate_enclosures,
        track_number: live_item.track_number,
        season: live_item.season,
        explicit: live_item.explicit,
        description: live_item.description,
        author_name: live_item.author_name,
        persons: live_item.persons,
        entity_ids: live_item.entity_ids,
        links: live_item.links,
        payment_routes: live_item.payment_routes,
        value_blocks: live_item.value_blocks,
        value_time_splits: live_item.value_time_splits,
        transcripts: live_item.transcripts,
        live_value_uri: live_item.live_value_uri,
        live_value_protocol: live_item.live_value_protocol,
    })
}
