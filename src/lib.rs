//! Declarative RSS/Podcast XML extraction engine.
//!
//! `stophammer-parser` provides a rule-based parser that extracts structured
//! feed and track data from podcast RSS XML. Rules declare *where* to find
//! a value in the DOM, *how* to transform it, and *which* output field to
//! populate. In addition to those normalized fields, the parser preserves the
//! full Podcast Namespace 1.0 tag surface in `IngestFeedData::podcast_namespace`
//! so callers can retain spec-defined elements that do not yet have dedicated
//! typed fields.
//!
//! # Quick start
//!
//! ```
//! use stophammer_parser::profile;
//!
//! let xml = r#"<?xml version="1.0"?>
//! <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
//!      xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
//!   <channel>
//!     <title>My Podcast</title>
//!     <podcast:guid>abc-123</podcast:guid>
//!     <item>
//!       <guid>ep-1</guid>
//!       <title>Episode 1</title>
//!     </item>
//!   </channel>
//! </rss>"#;
//!
//! let parser = profile::stophammer();
//! let feed = parser.parse(xml).unwrap();
//! assert_eq!(feed.title, "My Podcast");
//! assert_eq!(feed.tracks.len(), 1);
//! ```

pub mod date;
pub mod duration;
pub mod engine;
pub mod error;
pub mod html;
pub mod phase;
pub mod profile;
pub mod rule;
pub mod transform;
pub mod types;

pub use engine::{FeedParser, extract_podcast_namespace};
pub use error::ParseError;
pub use types::{
    IngestEntityId, IngestFeedData, IngestLink, IngestLiveItemData, IngestPaymentRoute,
    IngestPerson, IngestPodcastNamespaceSnapshot, IngestPodcastNamespaceTag, IngestRemoteFeedRef,
    IngestTrackData, IngestValueTimeSplit, RouteType,
};
