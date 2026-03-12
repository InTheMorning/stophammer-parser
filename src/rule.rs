//! Declarative extraction rules.
//!
//! A [`Rule`] declares: extract from [`Source`] -> apply [`Transform`] -> store in [`Target`].

use crate::phase::Phase;
use crate::transform::Transform;

/// A declarative extraction rule.
///
/// Rules are evaluated in order. The first rule that produces a value
/// for a given target wins — later rules for the same target are skipped.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Which phase this rule belongs to.
    pub phase: Phase,
    /// Where to find the value in the XML DOM.
    pub source: Source,
    /// How to transform the extracted text.
    pub transform: Transform,
    /// Which output field to populate.
    pub target: Target,
}

/// Where to find a value in the XML DOM.
#[derive(Debug, Clone)]
pub enum Source {
    /// Text content of first matching child element by local name.
    ChildText {
        /// Local element name (e.g. "title", "guid").
        tag: &'static str,
        /// Optional namespace URI filter.
        ns: Option<&'static str>,
    },
    /// Text content, trying multiple tags in order (first match wins).
    ChildTextFallback {
        /// List of (`local_name`, namespace) pairs to try.
        tags: &'static [(&'static str, Option<&'static str>)],
    },
    /// Attribute on first matching child element.
    ChildAttr {
        /// Local element name.
        tag: &'static str,
        /// Optional namespace URI filter.
        ns: Option<&'static str>,
        /// Attribute name.
        attr: &'static str,
    },
    /// Text of a nested element: parent > child.
    NestedText {
        /// Parent element local name.
        parent: &'static str,
        /// Parent namespace URI.
        parent_ns: Option<&'static str>,
        /// Child element local name.
        child: &'static str,
        /// Child namespace URI.
        child_ns: Option<&'static str>,
    },
    /// Attribute on a matching element (e.g. href on itunes:image).
    Attr {
        /// Element local name.
        tag: &'static str,
        /// Optional namespace URI filter.
        ns: Option<&'static str>,
        /// Attribute name.
        attr: &'static str,
    },
}

/// Which output struct field to populate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// A field on `IngestFeedData`.
    Feed(FeedField),
    /// A field on `IngestTrackData`.
    Track(TrackField),
}

/// Feed-level output fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedField {
    /// `feed_guid`
    FeedGuid,
    /// `title`
    Title,
    /// `description`
    Description,
    /// `image_url`
    ImageUrl,
    /// `language`
    Language,
    /// `explicit`
    Explicit,
    /// `itunes_type`
    ItunesType,
    /// `raw_medium`
    RawMedium,
    /// `author_name`
    AuthorName,
    /// `owner_name`
    OwnerName,
    /// `pub_date`
    PubDate,
}

/// Track-level output fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackField {
    /// `track_guid`
    TrackGuid,
    /// `title`
    Title,
    /// `pub_date`
    PubDate,
    /// `duration_secs`
    DurationSecs,
    /// `enclosure_url`
    EnclosureUrl,
    /// `enclosure_type`
    EnclosureType,
    /// `enclosure_bytes`
    EnclosureBytes,
    /// `track_number`
    TrackNumber,
    /// `season`
    Season,
    /// `explicit`
    Explicit,
    /// `description`
    Description,
    /// `author_name`
    AuthorName,
}
