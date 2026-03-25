//! Preconfigured parser profiles.
//!
//! These provide ready-to-use [`FeedParser`](crate::engine::FeedParser) instances
//! with standard rule sets for common use cases.

use crate::engine::FeedParser;
use crate::phase::Phase;
use crate::rule::{FeedField, Rule, Source, Target, TrackField};
use crate::transform::Transform;

/// Podcast namespace URI.
const PODCAST_NS: &str = "https://podcastindex.org/namespace/1.0";

/// iTunes namespace URI.
const ITUNES_NS: &str = "http://www.itunes.com/dtds/podcast-1.0.dtd";

/// Full stophammer parser with all phases enabled.
///
/// Extracts all supported fields from RSS 2.0, iTunes, and podcast
/// namespace elements.
///
/// # Examples
///
/// ```
/// let parser = stophammer_parser::profile::stophammer();
/// // parser is ready to parse any podcast RSS feed
/// ```
#[must_use]
pub fn stophammer() -> FeedParser {
    FeedParser::builder()
        .all_phases()
        .feed_rule(feed_rules())
        .track_rule(track_rules())
        .build()
}

/// Full stophammer parser with a fallback GUID.
///
/// Used by the importer when the `PodcastIndex` database provides a GUID
/// for feeds that lack `podcast:guid`.
#[must_use]
pub fn stophammer_with_fallback(guid: String) -> FeedParser {
    FeedParser::builder()
        .all_phases()
        .feed_rule(feed_rules())
        .track_rule(track_rules())
        .fallback_guid(guid)
        .build()
}

/// Minimal parser — RSS 2.0 core and Phase 1 only.
///
/// Useful for testing or lightweight extraction where iTunes and payment
/// data are not needed.
#[must_use]
pub fn minimal() -> FeedParser {
    FeedParser::builder()
        .phase(Phase::Rss2Core)
        .phase(Phase::Phase1)
        .feed_rule(feed_rules())
        .track_rule(track_rules())
        .build()
}

/// Full stophammer parser with a fallback GUID and custom phase set.
#[must_use]
pub fn stophammer_with_phases(guid: String, phases: &[Phase]) -> FeedParser {
    FeedParser::builder()
        .phases(phases)
        .fallback_guid(guid)
        .feed_rule(feed_rules())
        .track_rule(track_rules())
        .build()
}

/// Full stophammer parser with custom phase set (no fallback GUID).
#[must_use]
pub fn stophammer_phases_only(phases: &[Phase]) -> FeedParser {
    FeedParser::builder()
        .phases(phases)
        .feed_rule(feed_rules())
        .track_rule(track_rules())
        .build()
}

// Extend ParserBuilder with batch rule insertion
impl crate::engine::ParserBuilder {
    /// Adds all rules from a batch function returning feed rules.
    #[must_use]
    pub fn feed_rule(mut self, rules: Vec<Rule>) -> Self {
        self.feed_rules.extend(rules);
        self
    }

    /// Adds all rules from a batch function returning track rules.
    #[must_use]
    pub fn track_rule(mut self, rules: Vec<Rule>) -> Self {
        self.track_rules.extend(rules);
        self
    }
}

/// Returns all feed-level extraction rules.
#[expect(
    clippy::too_many_lines,
    reason = "the feed rule table is intentionally declared inline for auditability"
)]
fn feed_rules() -> Vec<Rule> {
    vec![
        // Phase1: podcast:guid
        Rule {
            phase: Phase::Phase1,
            source: Source::ChildText {
                tag: "guid",
                ns: Some(PODCAST_NS),
            },
            transform: Transform::None,
            target: Target::Feed(FeedField::FeedGuid),
        },
        // Phase1: podcast:medium
        Rule {
            phase: Phase::Phase1,
            source: Source::ChildText {
                tag: "medium",
                ns: Some(PODCAST_NS),
            },
            transform: Transform::None,
            target: Target::Feed(FeedField::RawMedium),
        },
        // RSS2: title
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "title",
                ns: None,
            },
            transform: Transform::DecodeEntities,
            target: Target::Feed(FeedField::Title),
        },
        // RSS2: description (with HTML stripping)
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "description",
                ns: None,
            },
            transform: Transform::StripHtml,
            target: Target::Feed(FeedField::Description),
        },
        // iTunes: summary as description fallback
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "summary",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::StripHtml,
            target: Target::Feed(FeedField::Description),
        },
        // RSS2: language
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "language",
                ns: None,
            },
            transform: Transform::None,
            target: Target::Feed(FeedField::Language),
        },
        // RSS2: pubDate
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "pubDate",
                ns: None,
            },
            transform: Transform::ParseDate,
            target: Target::Feed(FeedField::PubDate),
        },
        // RSS2: lastBuildDate as pubDate fallback
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "lastBuildDate",
                ns: None,
            },
            transform: Transform::ParseDate,
            target: Target::Feed(FeedField::PubDate),
        },
        // iTunes: image (href attribute)
        Rule {
            phase: Phase::Itunes,
            source: Source::Attr {
                tag: "image",
                ns: Some(ITUNES_NS),
                attr: "href",
            },
            transform: Transform::None,
            target: Target::Feed(FeedField::ImageUrl),
        },
        // RSS2: image > url fallback
        Rule {
            phase: Phase::Rss2Core,
            source: Source::NestedText {
                parent: "image",
                parent_ns: None,
                child: "url",
                child_ns: None,
            },
            transform: Transform::None,
            target: Target::Feed(FeedField::ImageUrl),
        },
        // iTunes: explicit
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "explicit",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::ExplicitBool,
            target: Target::Feed(FeedField::Explicit),
        },
        // iTunes: type
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "type",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::None,
            target: Target::Feed(FeedField::ItunesType),
        },
        // iTunes: author
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "author",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::DecodeEntities,
            target: Target::Feed(FeedField::AuthorName),
        },
        // iTunes: owner > name (Sprint 5 fix: nested extraction)
        Rule {
            phase: Phase::Itunes,
            source: Source::NestedText {
                parent: "owner",
                parent_ns: Some(ITUNES_NS),
                child: "name",
                child_ns: Some(ITUNES_NS),
            },
            transform: Transform::DecodeEntities,
            target: Target::Feed(FeedField::OwnerName),
        },
    ]
}

/// Returns all track-level extraction rules.
#[expect(
    clippy::too_many_lines,
    reason = "the track rule table is intentionally declared inline for auditability"
)]
fn track_rules() -> Vec<Rule> {
    vec![
        // RSS2: guid
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "guid",
                ns: None,
            },
            transform: Transform::None,
            target: Target::Track(TrackField::TrackGuid),
        },
        // RSS2: title
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "title",
                ns: None,
            },
            transform: Transform::DecodeEntities,
            target: Target::Track(TrackField::Title),
        },
        // RSS2: pubDate
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "pubDate",
                ns: None,
            },
            transform: Transform::ParseDate,
            target: Target::Track(TrackField::PubDate),
        },
        // RSS2: enclosure attributes
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildAttr {
                tag: "enclosure",
                ns: None,
                attr: "url",
            },
            transform: Transform::None,
            target: Target::Track(TrackField::EnclosureUrl),
        },
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildAttr {
                tag: "enclosure",
                ns: None,
                attr: "type",
            },
            transform: Transform::None,
            target: Target::Track(TrackField::EnclosureType),
        },
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildAttr {
                tag: "enclosure",
                ns: None,
                attr: "length",
            },
            transform: Transform::ParseInt,
            target: Target::Track(TrackField::EnclosureBytes),
        },
        // iTunes: duration
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "duration",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::ParseDuration,
            target: Target::Track(TrackField::DurationSecs),
        },
        // iTunes: episode number
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "episode",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::ParseInt,
            target: Target::Track(TrackField::TrackNumber),
        },
        // Podcast: episode number (fallback)
        Rule {
            phase: Phase::Phase1,
            source: Source::ChildText {
                tag: "episode",
                ns: Some(PODCAST_NS),
            },
            transform: Transform::ParseInt,
            target: Target::Track(TrackField::TrackNumber),
        },
        // iTunes: season
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "season",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::ParseInt,
            target: Target::Track(TrackField::Season),
        },
        // Podcast: season (fallback)
        Rule {
            phase: Phase::Phase1,
            source: Source::ChildText {
                tag: "season",
                ns: Some(PODCAST_NS),
            },
            transform: Transform::ParseInt,
            target: Target::Track(TrackField::Season),
        },
        // iTunes: explicit (Sprint 5 fix: accepts both "yes" and "true")
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "explicit",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::ExplicitBool,
            target: Target::Track(TrackField::Explicit),
        },
        // RSS2: description (with HTML stripping)
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "description",
                ns: None,
            },
            transform: Transform::StripHtml,
            target: Target::Track(TrackField::Description),
        },
        // iTunes: summary as description fallback
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "summary",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::StripHtml,
            target: Target::Track(TrackField::Description),
        },
        // iTunes: author
        Rule {
            phase: Phase::Itunes,
            source: Source::ChildText {
                tag: "author",
                ns: Some(ITUNES_NS),
            },
            transform: Transform::DecodeEntities,
            target: Target::Track(TrackField::AuthorName),
        },
        // RSS2: author fallback
        Rule {
            phase: Phase::Rss2Core,
            source: Source::ChildText {
                tag: "author",
                ns: None,
            },
            transform: Transform::DecodeEntities,
            target: Target::Track(TrackField::AuthorName),
        },
    ]
}
