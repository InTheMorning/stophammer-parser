//! Output types matching the stophammer ingest wire format.
//!
//! These types serialize to the same JSON shape as
//! `stophammer/src/ingest.rs`, keeping this crate self-contained.

/// Parsed feed data ready for ingestion.
///
/// Contains all feed-level metadata, payment routes, and tracks
/// extracted from an RSS/Podcast XML document.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestFeedData {
    /// Podcast GUID from `podcast:guid` or fallback.
    pub feed_guid: String,
    /// Feed title from `<title>`.
    pub title: String,
    /// Feed description, HTML-stripped.
    pub description: Option<String>,
    /// Image URL from `itunes:image` or `<image><url>`.
    pub image_url: Option<String>,
    /// Language code from `<language>`.
    pub language: Option<String>,
    /// Whether the feed is marked explicit.
    pub explicit: bool,
    /// iTunes podcast type (e.g. "episodic", "serial").
    pub itunes_type: Option<String>,
    /// Raw `podcast:medium` value.
    pub raw_medium: Option<String>,
    /// Author name from `itunes:author`.
    pub author_name: Option<String>,
    /// Owner name from `itunes:owner > itunes:name`.
    pub owner_name: Option<String>,
    /// Channel publication date as Unix seconds.
    pub pub_date: Option<i64>,
    /// Feed-level `podcast:remoteItem` references to artist/publisher feeds.
    pub remote_items: Vec<IngestRemoteFeedRef>,
    /// Feed-level payment recipients (fallback at play time).
    pub feed_payment_routes: Vec<IngestPaymentRoute>,
    /// Parsed live items that have not yet been promoted to permanent tracks.
    pub live_items: Vec<IngestLiveItemData>,
    /// Parsed tracks (items) from the feed.
    pub tracks: Vec<IngestTrackData>,
}

/// Parsed track (episode/item) data.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestTrackData {
    /// Track GUID from `<guid>`.
    pub track_guid: String,
    /// Track title.
    pub title: String,
    /// Publication date as Unix seconds.
    pub pub_date: Option<i64>,
    /// Duration in seconds.
    pub duration_secs: Option<i64>,
    /// Enclosure media URL.
    pub enclosure_url: Option<String>,
    /// Enclosure MIME type.
    pub enclosure_type: Option<String>,
    /// Enclosure size in bytes.
    pub enclosure_bytes: Option<i64>,
    /// Episode number.
    pub track_number: Option<i64>,
    /// Season number.
    pub season: Option<i64>,
    /// Whether the track is marked explicit.
    pub explicit: bool,
    /// Track description, HTML-stripped.
    pub description: Option<String>,
    /// Per-track author override.
    pub author_name: Option<String>,
    /// Payment recipients for this track.
    pub payment_routes: Vec<IngestPaymentRoute>,
    /// Value time splits for this track.
    pub value_time_splits: Vec<IngestValueTimeSplit>,
}

/// A channel-level `podcast:remoteItem` reference.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestRemoteFeedRef {
    /// Source order within the channel.
    pub position: i64,
    /// Optional relation hint, commonly `publisher`.
    pub medium: Option<String>,
    /// Referenced remote feed GUID.
    pub remote_feed_guid: String,
    /// Optional remote feed URL.
    pub remote_feed_url: Option<String>,
}

/// Parsed `podcast:liveItem` data.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestLiveItemData {
    /// Live item GUID from `<guid>`.
    pub live_item_guid: String,
    /// Live item title.
    pub title: String,
    /// Current live status (`pending`, `live`, `ended`).
    pub status: String,
    /// Scheduled or actual start time as Unix seconds.
    pub start_at: Option<i64>,
    /// Scheduled or actual end time as Unix seconds.
    pub end_at: Option<i64>,
    /// Content/live stream URL if present.
    pub content_link: Option<String>,
    /// Publication date as Unix seconds.
    pub pub_date: Option<i64>,
    /// Duration in seconds.
    pub duration_secs: Option<i64>,
    /// Enclosure media URL.
    pub enclosure_url: Option<String>,
    /// Enclosure MIME type.
    pub enclosure_type: Option<String>,
    /// Enclosure size in bytes.
    pub enclosure_bytes: Option<i64>,
    /// Episode number.
    pub track_number: Option<i64>,
    /// Season number.
    pub season: Option<i64>,
    /// Whether the live item is marked explicit.
    pub explicit: bool,
    /// Description, HTML-stripped.
    pub description: Option<String>,
    /// Per-item author override.
    pub author_name: Option<String>,
    /// Payment recipients attached to the live item.
    pub payment_routes: Vec<IngestPaymentRoute>,
    /// Value time splits attached to the live item.
    pub value_time_splits: Vec<IngestValueTimeSplit>,
}

/// A payment route recipient (wire format before DB row ID assignment).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestPaymentRoute {
    /// Display name of the recipient.
    pub recipient_name: Option<String>,
    /// Route type: node or lnaddress.
    pub route_type: RouteType,
    /// Lightning node pubkey or LN address.
    pub address: String,
    /// Custom TLV key.
    pub custom_key: Option<String>,
    /// Custom TLV value.
    pub custom_value: Option<String>,
    /// Split weight (integer).
    pub split: i64,
    /// When true, recipient is an app-fee destination.
    pub fee: bool,
}

/// A value time split referencing a remote item.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestValueTimeSplit {
    /// Start time in seconds from beginning of playback.
    pub start_time_secs: i64,
    /// Duration in seconds (None = until next split or end).
    pub duration_secs: Option<i64>,
    /// GUID of the remote feed to pay.
    pub remote_feed_guid: String,
    /// GUID of the remote item to pay.
    pub remote_item_guid: String,
    /// Split weight (integer).
    pub split: i64,
}

/// Lightning payment route type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum RouteType {
    /// Lightning network node (pubkey).
    Node,
    /// Lightning address (user@domain).
    Lnaddress,
}
