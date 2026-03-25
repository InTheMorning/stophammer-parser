//! Podcast namespace phase definitions.
//!
//! Phases map to the podcast namespace specification phases.
//! The parser builder selects which normalized extractors to enable.
//! Generic `podcast_namespace` preservation is always on so callers retain the
//! complete Podcast Namespace 1.0 XML surface regardless of phase filtering.

/// A podcast namespace specification phase.
///
/// Only rules whose phase is enabled will execute during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// RSS 2.0 base spec: title, guid, enclosure, pubDate.
    Rss2Core,
    /// iTunes namespace: duration, explicit, author, image, type, season, episode.
    Itunes,
    /// Phase 1: `podcast:guid`, `podcast:medium`.
    Phase1,
    /// Phase 2: `podcast:value` (payment routes).
    Phase2,
    /// Phase 3: `podcast:valueTimeSplit`, `podcast:remoteItem`.
    Phase3,
    /// Phase 4: `podcast:person`.
    Phase4,
    /// Phase 5: `podcast:images` (future).
    Phase5,
    /// Phase 6: typed `podcast:txt` claims such as `purpose="npub"`.
    Phase6,
    /// Not yet assigned to a phase.
    Pending,
}

impl Phase {
    /// Returns all defined phases.
    #[must_use]
    pub fn all() -> &'static [Phase] {
        &[
            Self::Rss2Core,
            Self::Itunes,
            Self::Phase1,
            Self::Phase2,
            Self::Phase3,
            Self::Phase4,
            Self::Phase5,
            Self::Phase6,
            Self::Pending,
        ]
    }
}
