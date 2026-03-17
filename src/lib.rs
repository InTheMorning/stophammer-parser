//! Declarative RSS/Podcast XML extraction engine.
//!
//! `stophammer-parser` provides a rule-based parser that extracts structured
//! feed and track data from podcast RSS XML. Rules declare *where* to find
//! a value in the DOM, *how* to transform it, and *which* output field to
//! populate.
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

pub use engine::FeedParser;
pub use error::ParseError;
pub use types::{
    IngestFeedData, IngestLiveItemData, IngestPaymentRoute, IngestRemoteFeedRef, IngestTrackData,
    IngestValueTimeSplit, RouteType,
};
