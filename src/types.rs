//! Output types matching the stophammer ingest wire format.
//!
//! These types serialize to the same JSON shape as
//! `stophammer/src/ingest.rs`, keeping this crate self-contained. The parser
//! also preserves the full Podcast Namespace 1.0 surface in a generic
//! `podcast_namespace` snapshot so callers do not lose spec-defined tags that
//! are not yet normalized into dedicated ingest fields.

use std::collections::HashMap;

/// Parsed feed data ready for ingestion.
///
/// Contains all feed-level metadata, payment routes, and tracks
/// extracted from an RSS/Podcast XML document.
///
/// `R` is the type of each required field (musicindex ADR 0086 R-FEED-2).
/// [`FeedParser::read`](crate::FeedParser::read) gives
/// [`FeedValues`], where `R` is `Option<String>` and no field is required.
/// [`check::required_fields`](crate::check::required_fields) gives the
/// default form, where `R` is `String`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestFeedData<R = String> {
    /// Podcast GUID from `podcast:guid`. A required field.
    pub feed_guid: R,
    /// Feed title from `<title>`. A required field.
    pub title: R,
    /// Feed description, HTML-stripped.
    pub description: Option<String>,
    /// Image URL from `itunes:image`, `podcast:images`, or `<image><url>`.
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
    /// Channel `lastBuildDate` as Unix seconds.
    ///
    /// This is the time the feed file was generated. It is not a publication
    /// date, and a generator can rewrite it on every fetch.
    pub last_build_date: Option<i64>,
    /// The channel `itunes:new-feed-url`, trimmed.
    ///
    /// Stophammer ADR 0052 owns this field. An empty value gives `None`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub new_feed_url: Option<String>,
    /// The channel `podcast:locked` text, as a flag.
    ///
    /// Stophammer ADR 0052 owns this field. `yes` gives `true`. `no` gives
    /// `false`. Any other text gives `None`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub locked: Option<bool>,
    /// The `owner` attribute of the channel `podcast:locked`, trimmed.
    ///
    /// Stophammer ADR 0052 owns this field. An empty value gives `None`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub locked_owner: Option<String>,
    /// Feed-level `podcast:remoteItem` references to artist/publisher feeds.
    pub remote_items: Vec<IngestRemoteFeedRef>,
    /// Feed-level contributor claims from `podcast:person`.
    pub persons: Vec<IngestPerson>,
    /// Feed-level identity claims from typed metadata such as `podcast:txt`.
    pub entity_ids: Vec<IngestEntityId>,
    /// Feed-level typed links such as websites or self-feed URLs.
    pub links: Vec<IngestLink>,
    /// Channel-level `podcast:block` tags (ADR 0057).
    ///
    /// The node evaluates these to determine whether to ingest the feed.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub blocks: Vec<IngestBlockTag>,
    /// Full Podcast Namespace 1.0 snapshot captured from channel/item/liveItem XML.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub podcast_namespace: Option<IngestPodcastNamespaceSnapshot>,
    /// Feed-level payment recipients (fallback at play time).
    pub feed_payment_routes: Vec<IngestPaymentRoute>,
    /// Parsed live items that have not yet been promoted to permanent tracks.
    pub live_items: Vec<IngestLiveItemData<R>>,
    /// Parsed tracks (items) from the feed.
    pub tracks: Vec<IngestTrackData<R>>,
}

/// The values of a feed before a required-field check (musicindex ADR 0086
/// R-FEED-2). A missing required field is `None`.
pub type FeedValues = IngestFeedData<Option<String>>;

/// A channel-level `podcast:block` tag.
///
/// Stophammer ADR 0057 owns this field.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestBlockTag {
    /// The `id` attribute of the tag, trimmed. An empty attribute gives `None`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub id: Option<String>,
    /// The text content of the tag, trimmed. The case is preserved.
    pub value: String,
}

/// Parsed track (episode/item) data.
///
/// `R` is the type of each required field, as in [`IngestFeedData`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestTrackData<R = String> {
    /// Track GUID from `<guid>`. A required field.
    pub track_guid: R,
    /// Track title. A required field.
    pub title: R,
    /// Publication date as Unix seconds.
    pub pub_date: Option<i64>,
    /// Duration in seconds.
    pub duration_secs: Option<i64>,
    /// Track-level artwork URL from `itunes:image href` or `podcast:images`.
    pub image_url: Option<String>,
    /// Language code override for this track.
    pub language: Option<String>,
    /// Enclosure media URL.
    pub enclosure_url: Option<String>,
    /// Enclosure MIME type.
    pub enclosure_type: Option<String>,
    /// Enclosure size in bytes.
    pub enclosure_bytes: Option<i64>,
    /// Alternate enclosure variants published for this item.
    pub alternate_enclosures: Vec<IngestAlternateEnclosure>,
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
    /// Contributor claims from per-item `podcast:person`.
    pub persons: Vec<IngestPerson>,
    /// Identity claims from per-item typed metadata such as `podcast:txt`.
    pub entity_ids: Vec<IngestEntityId>,
    /// Typed links attached to the item.
    pub links: Vec<IngestLink>,
    /// Payment recipients for this track.
    pub payment_routes: Vec<IngestPaymentRoute>,
    /// Value time splits for this track.
    pub value_time_splits: Vec<IngestValueTimeSplit>,
    /// Transcript files published for this track via `podcast:transcript`.
    pub transcripts: Vec<IngestTranscript>,
    /// Item-level `podcast:remoteItem` references (not inside valueTimeSplit).
    pub remote_items: Vec<IngestRemoteFeedRef>,
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
    /// Raw `rel` attribute of the `podcast:remoteItem` element.
    ///
    /// The Podcast Namespace does not specify `rel` on `podcast:remoteItem`.
    /// The value is non-standard. A missing or empty attribute gives `None`.
    /// The parser keeps the value unchanged.
    #[cfg_attr(feature = "serde", serde(default))]
    pub rel: Option<String>,
    /// The `itemGuid` attribute of the `podcast:remoteItem` element, trimmed.
    ///
    /// Stophammer ADR 0060 owns this field. An empty value gives `None`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub item_guid: Option<String>,
    /// The `title` attribute of the `podcast:remoteItem` element, trimmed.
    ///
    /// Stophammer ADR 0060 owns this field. An empty value gives `None`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub item_title: Option<String>,
    /// True when this is the publisher of the album, false for credits.
    ///
    /// Stophammer ADR 0069 §3 owns this field. The publisher is the item
    /// inside `<podcast:publisher>`, or the first bare channel
    /// `remoteItem` with `medium="publisher"` when there is no
    /// `<podcast:publisher>`. A credit is each other bare item. The parser
    /// gives false for music items and for each other item kind.
    #[cfg_attr(feature = "serde", serde(default))]
    pub publisher_reference: bool,
}

/// A `podcast:person` contributor claim.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestPerson {
    /// Source order within the enclosing feed/item/liveItem.
    pub position: i64,
    /// Display name published by the feed.
    pub name: String,
    /// Role attribute, when present.
    pub role: Option<String>,
    /// Group attribute, when present.
    pub group_name: Option<String>,
    /// Href attribute, when present.
    pub href: Option<String>,
    /// Img attribute, when present.
    pub img: Option<String>,
    /// Npub attribute, when present.
    pub npub: Option<String>,
}

/// A staged external identity claim extracted from typed feed metadata.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestEntityId {
    /// Source order within the enclosing feed/item/liveItem.
    pub position: i64,
    /// Canonical scheme name for the extracted ID.
    pub scheme: String,
    /// Raw value published by the feed.
    pub value: String,
}

/// A typed source link extracted from the feed.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestLink {
    /// Source order within the enclosing feed/item/liveItem.
    pub position: i64,
    /// Normalized link type.
    pub link_type: String,
    /// Raw URL value.
    pub url: String,
    /// Display text of the link, from the node value (ADR 0082 §2).
    /// It is `None` for each link type except `funding`.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub label: Option<String>,
    /// Source extraction path within the feed.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extraction_path: String,
}

/// An alternate media enclosure published for a track or live item.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestAlternateEnclosure {
    /// Source order within the enclosing item/live item.
    pub position: i64,
    /// Alternate enclosure URL.
    pub url: String,
    /// MIME type when published.
    pub mime_type: Option<String>,
    /// Byte length when published.
    pub bytes: Option<i64>,
    /// Relation hint such as `stream` when published.
    pub rel: Option<String>,
    /// Human-readable title/label when published.
    pub title: Option<String>,
    /// Source extraction path within the feed.
    #[cfg_attr(feature = "serde", serde(default))]
    pub extraction_path: String,
}

/// A transcript file published for a track or live item via `podcast:transcript`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestTranscript {
    /// Source order within the enclosing item/live item.
    pub position: i64,
    /// Transcript URL.
    pub url: String,
    /// MIME type when published (e.g. `text/vtt`, `application/srt`).
    pub mime_type: Option<String>,
    /// Language code when published.
    pub language: Option<String>,
    /// Relation hint when published (e.g. `captions`).
    pub rel: Option<String>,
}

/// Parsed `podcast:liveItem` data.
///
/// `R` is the type of each required field, as in [`IngestFeedData`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestLiveItemData<R = String> {
    /// Live item GUID from `<guid>`. A required field.
    pub live_item_guid: R,
    /// Live item title. A required field.
    pub title: R,
    /// Current live status (`pending`, `live`, `ended`), trimmed and in
    /// lower case. A required field. An empty attribute is missing.
    pub status: R,
    /// Scheduled or actual start time as Unix seconds.
    pub start_at: Option<i64>,
    /// Scheduled or actual end time as Unix seconds.
    pub end_at: Option<i64>,
    /// The `href` of the first `podcast:contentLink` child element (ADR 0093).
    /// It is not the enclosure URL.
    pub content_link: Option<String>,
    /// Publication date as Unix seconds.
    pub pub_date: Option<i64>,
    /// Duration in seconds.
    pub duration_secs: Option<i64>,
    /// Live item artwork URL from `itunes:image href`.
    pub image_url: Option<String>,
    /// Language code override for this live item.
    pub language: Option<String>,
    /// Enclosure media URL.
    pub enclosure_url: Option<String>,
    /// Enclosure MIME type.
    pub enclosure_type: Option<String>,
    /// Enclosure size in bytes.
    pub enclosure_bytes: Option<i64>,
    /// Alternate enclosure variants published for this live item.
    pub alternate_enclosures: Vec<IngestAlternateEnclosure>,
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
    /// Contributor claims from `podcast:person`.
    pub persons: Vec<IngestPerson>,
    /// Identity claims from typed metadata such as `podcast:txt`.
    pub entity_ids: Vec<IngestEntityId>,
    /// Typed links attached to the live item.
    pub links: Vec<IngestLink>,
    /// Payment recipients attached to the live item.
    pub payment_routes: Vec<IngestPaymentRoute>,
    /// Value time splits attached to the live item.
    pub value_time_splits: Vec<IngestValueTimeSplit>,
    /// Transcript files published for this live item via `podcast:transcript`.
    pub transcripts: Vec<IngestTranscript>,
    /// Relay URI from the first direct `podcast:liveValue` child.
    pub live_value_uri: Option<String>,
    /// Relay protocol from the first direct `podcast:liveValue` child.
    pub live_value_protocol: Option<String>,
}

/// Full Podcast Namespace 1.0 snapshot for a parsed feed.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestPodcastNamespaceSnapshot {
    /// All Podcast Namespace elements seen in source order.
    pub tags: Vec<IngestPodcastNamespaceTag>,
}

/// A raw Podcast Namespace element preserved from the source XML.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestPodcastNamespaceTag {
    /// Source order across the feed document.
    pub position: i64,
    /// `channel`, `item`, or `live_item`.
    pub entity_scope: String,
    /// Enclosing item/live-item GUID when available.
    pub entity_guid: Option<String>,
    /// Canonicalized element path such as `rss.channel.podcast:value.podcast:valueRecipient`.
    pub path: String,
    /// Canonicalized tag name such as `podcast:valueRecipient`.
    pub tag: String,
    /// Raw attributes exactly as published.
    pub attributes: HashMap<String, String>,
    /// Trimmed direct text content, when present.
    pub text: Option<String>,
}

/// A payment route recipient (wire format before DB row ID assignment).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestPaymentRoute {
    /// Type of the `podcast:value` block.
    pub value_type: String,
    /// Method of the `podcast:value` block.
    pub value_method: Option<String>,
    /// Position of the `podcast:value` block in its parent.
    pub block_index: u32,
    /// Display name of the recipient.
    pub recipient_name: Option<String>,
    /// The parsed form of `type`. A missing or empty `type` gives
    /// [`RouteType::Node`].
    pub route_type: RouteType,
    /// The `type` attribute as the feed gives it (musicindex ADR 0086
    /// R-FEED-4). `None` when the attribute is missing.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub type_text: Option<String>,
    /// Lightning node pubkey or LN address.
    pub address: String,
    /// Custom TLV key.
    pub custom_key: Option<String>,
    /// Custom TLV value.
    pub custom_value: Option<String>,
    /// The parsed form of `split`. A missing value, or a value that is not
    /// an integer, gives 0.
    pub split: i64,
    /// The `split` attribute as the feed gives it (musicindex ADR 0086
    /// R-FEED-4). `None` when the attribute is missing.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub split_text: Option<String>,
    /// The parsed form of `fee`: `true` when the recipient is an app-fee
    /// destination. [`FeeReading`] gives the rule.
    pub fee: bool,
    /// The `fee` attribute as the feed gives it (musicindex ADR 0086
    /// R-FEED-4). `None` when the attribute is missing.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub fee_text: Option<String>,
}

impl IngestPaymentRoute {
    /// Returns `true` when `fee_text` is a value that the fee rule does not
    /// know (musicindex ADR 0086 R-FEED-4 item 5). This is the warning of
    /// that rule. The route stays in the result, and `fee` is `false`.
    #[must_use]
    pub fn fee_warning(&self) -> bool {
        FeeReading::from_text(self.fee_text.as_deref()) == FeeReading::Unknown
    }
}

/// The reading of a `fee` attribute (musicindex ADR 0086 R-FEED-4 item 5).
///
/// The rule removes the spaces at the two ends of the text and ignores the
/// letter case.
///
/// ```
/// use stophammer_parser::FeeReading;
///
/// assert_eq!(FeeReading::from_text(Some(" YES ")), FeeReading::Fee);
/// assert_eq!(FeeReading::from_text(Some("")), FeeReading::NotFee);
/// assert_eq!(FeeReading::from_text(None), FeeReading::NotFee);
/// assert_eq!(FeeReading::from_text(Some("maybe")), FeeReading::Unknown);
/// assert!(!FeeReading::Unknown.is_fee());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeReading {
    /// `true`, `1` or `yes`.
    Fee,
    /// `false`, `0`, `no`, an empty value or a missing attribute.
    NotFee,
    /// Each other value. It is not a fee, and it gives a warning.
    Unknown,
}

impl FeeReading {
    /// Reads the text of a `fee` attribute. `None` is a missing attribute.
    #[must_use]
    pub fn from_text(text: Option<&str>) -> Self {
        let Some(text) = text else {
            return Self::NotFee;
        };
        let text = text.trim();
        if ["true", "1", "yes"]
            .iter()
            .any(|known| text.eq_ignore_ascii_case(known))
        {
            Self::Fee
        } else if text.is_empty()
            || ["false", "0", "no"]
                .iter()
                .any(|known| text.eq_ignore_ascii_case(known))
        {
            Self::NotFee
        } else {
            Self::Unknown
        }
    }

    /// Returns `true` for [`FeeReading::Fee`] only.
    #[must_use]
    pub fn is_fee(self) -> bool {
        self == Self::Fee
    }
}

/// The form of a value time split (stophammer ADR 0073 §1).
///
/// A remote split names a remote item. A local split names its own
/// recipients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ValueTimeSplitForm {
    /// The split names a remote item, with an optional share and start time.
    Remote,
    /// The split names its own recipients.
    Local,
}

/// A value time split in the form the source states (stophammer ADR 0073
/// §1).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IngestValueTimeSplit {
    /// Type of the `podcast:value` block.
    pub value_type: String,
    /// Method of the `podcast:value` block.
    pub value_method: Option<String>,
    /// Position of the `podcast:value` block in its parent.
    pub block_index: u32,
    /// The form of the split: remote or local.
    pub form: ValueTimeSplitForm,
    /// Start time in seconds from the start of playback. Can be a fraction.
    pub start_time_secs: f64,
    /// Duration in seconds, or `None` when the source states none.
    pub duration_secs: Option<f64>,
    /// GUID of the remote feed to pay. `None` in the local form.
    pub remote_feed_guid: Option<String>,
    /// GUID of the remote item to pay. Can be `None` in the remote form.
    pub remote_item_guid: Option<String>,
    /// The stated `remoteStartTime`, or `None` when the source states none.
    pub remote_start_time_secs: Option<f64>,
    /// The stated `remotePercentage`, or `None` when the source states none.
    pub remote_percentage: Option<f64>,
    /// The recipients of a local split, in source order. Empty in the remote
    /// form.
    pub recipients: Vec<IngestPaymentRoute>,
}

/// Payment route type from a `podcast:valueRecipient` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteType {
    /// Lightning network node (pubkey).
    Node,
    /// Lightning address (user@domain).
    Lnaddress,
    /// Any other route type from the feed.
    Other(String),
}

#[cfg(feature = "serde")]
impl serde::Serialize for RouteType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::Node => "node",
            Self::Lnaddress => "lnaddress",
            Self::Other(value) => value,
        };
        serializer.serialize_str(value)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RouteType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "node" => Self::Node,
            "lnaddress" => Self::Lnaddress,
            _ => Self::Other(value),
        })
    }
}
