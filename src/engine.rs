//! Feed parsing engine.
//!
//! The engine applies declarative rules to an XML DOM, extracting
//! structured feed and track data.

use std::collections::HashSet;

use crate::error::{ErrorKind, ParseError};
use crate::phase::Phase;
use crate::rule::{FeedField, Rule, Source, Target, TrackField};
use crate::transform::{apply_transform, TransformResult};
use crate::types::{
    IngestFeedData, IngestPaymentRoute, IngestTrackData, IngestValueTimeSplit, RouteType,
};

/// Podcast namespace URI used in namespace-aware feeds.
const PODCAST_NS: &str = "https://podcastindex.org/namespace/1.0";

/// The feed parsing engine.
///
/// Applies configured rules to extract structured data from RSS/Podcast XML.
///
/// # Examples
///
/// ```
/// use stophammer_parser::profile;
///
/// let xml = r#"<?xml version="1.0"?>
/// <rss xmlns:podcast="https://podcastindex.org/namespace/1.0"
///      xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
///   <channel>
///     <title>My Podcast</title>
///     <podcast:guid>abc-123</podcast:guid>
///   </channel>
/// </rss>"#;
///
/// let parser = profile::stophammer();
/// let feed = parser.parse(xml).unwrap();
/// assert_eq!(feed.title, "My Podcast");
/// assert_eq!(feed.feed_guid, "abc-123");
/// ```
pub struct FeedParser {
    pub(crate) feed_rules: Vec<Rule>,
    pub(crate) track_rules: Vec<Rule>,
    pub(crate) fallback_guid: Option<String>,
    pub(crate) phases: HashSet<Phase>,
}

impl FeedParser {
    /// Creates a new builder for configuring a `FeedParser`.
    #[must_use]
    pub fn builder() -> ParserBuilder {
        ParserBuilder {
            phases: HashSet::new(),
            fallback_guid: None,
            feed_rules: Vec::new(),
            track_rules: Vec::new(),
        }
    }

    /// Parses an RSS/Podcast XML document into structured feed data.
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if:
    /// - The XML is malformed
    /// - No `<channel>` element is found
    /// - The feed has no `<title>`
    /// - The feed has no `podcast:guid` and no fallback was configured
    pub fn parse(&self, xml: &str) -> Result<IngestFeedData, ParseError> {
        let doc = roxmltree::Document::parse(xml)?;
        let root = doc.root_element();

        // Find <channel> — it may be direct child of <rss> or the root itself
        let channel = find_channel(&root)
            .ok_or(ParseError { kind: ErrorKind::NoChannel })?;

        // Apply feed-level rules
        let mut feed = FeedDataBuilder::default();
        for rule in &self.feed_rules {
            if !self.phases.contains(&rule.phase) {
                continue;
            }
            if let Target::Feed(field) = rule.target {
                if feed.is_set(field) {
                    continue;
                }
                if let Some(value) = extract_source(&channel, &rule.source)
                    && let Some(result) = apply_transform(rule.transform, &value)
                {
                    feed.set(field, result);
                }
            }
        }

        // Apply fallback guid
        if feed.feed_guid.is_none()
            && let Some(ref fallback) = self.fallback_guid
        {
            feed.feed_guid = Some(fallback.clone());
        }

        // Extract feed-level payment routes (Phase2)
        let feed_payment_routes = if self.phases.contains(&Phase::Phase2) {
            extract_payment_routes(&channel)
        } else {
            Vec::new()
        };

        // Validate required fields
        let title = feed.title.ok_or(ParseError { kind: ErrorKind::NoTitle })?;
        let feed_guid = feed.feed_guid.ok_or(ParseError { kind: ErrorKind::NoGuid })?;

        // Parse items
        let tracks = self.parse_items(&channel);

        Ok(IngestFeedData {
            feed_guid,
            title,
            description: feed.description,
            image_url: feed.image_url,
            language: feed.language,
            explicit: feed.explicit,
            itunes_type: feed.itunes_type,
            raw_medium: feed.raw_medium,
            author_name: feed.author_name,
            owner_name: feed.owner_name,
            pub_date: feed.pub_date,
            feed_payment_routes,
            tracks,
        })
    }

    /// Parses all `<item>` elements within the channel.
    fn parse_items(&self, channel: &roxmltree::Node) -> Vec<IngestTrackData> {
        let mut tracks = Vec::new();

        for item in channel.children().filter(|n| n.has_tag_name("item")) {
            if let Some(track) = self.parse_item(&item) {
                tracks.push(track);
            }
        }

        tracks
    }

    /// Parses a single `<item>` element into track data.
    fn parse_item(&self, item: &roxmltree::Node) -> Option<IngestTrackData> {
        let mut track = TrackDataBuilder::default();

        for rule in &self.track_rules {
            if !self.phases.contains(&rule.phase) {
                continue;
            }
            if let Target::Track(field) = rule.target {
                if track.is_set(field) {
                    continue;
                }
                if let Some(value) = extract_source(item, &rule.source)
                    && let Some(result) = apply_transform(rule.transform, &value)
                {
                    track.set(field, result);
                }
            }
        }

        // Track guid and title are required
        let track_guid = track.track_guid?;
        let title = track.title?;

        // Extract payment routes (Phase2)
        let payment_routes = if self.phases.contains(&Phase::Phase2) {
            extract_payment_routes(item)
        } else {
            Vec::new()
        };

        // Extract value time splits (Phase3)
        let value_time_splits = if self.phases.contains(&Phase::Phase3) {
            extract_value_time_splits(item)
        } else {
            Vec::new()
        };

        Some(IngestTrackData {
            track_guid,
            title,
            pub_date: track.pub_date,
            duration_secs: track.duration_secs,
            enclosure_url: track.enclosure_url,
            enclosure_type: track.enclosure_type,
            enclosure_bytes: track.enclosure_bytes,
            track_number: track.track_number,
            season: track.season,
            explicit: track.explicit,
            description: track.description,
            author_name: track.author_name,
            payment_routes,
            value_time_splits,
        })
    }
}

/// Builder for configuring a [`FeedParser`].
pub struct ParserBuilder {
    pub(crate) phases: HashSet<Phase>,
    pub(crate) fallback_guid: Option<String>,
    pub(crate) feed_rules: Vec<Rule>,
    pub(crate) track_rules: Vec<Rule>,
}

impl ParserBuilder {
    /// Enables all defined phases.
    #[must_use]
    pub fn all_phases(mut self) -> Self {
        for &p in Phase::all() {
            self.phases.insert(p);
        }
        self
    }

    /// Enables a single phase.
    #[must_use]
    pub fn phase(mut self, p: Phase) -> Self {
        self.phases.insert(p);
        self
    }

    /// Enables multiple phases.
    #[must_use]
    pub fn phases(mut self, ps: &[Phase]) -> Self {
        for &p in ps {
            self.phases.insert(p);
        }
        self
    }

    /// Sets a fallback GUID for feeds that lack `podcast:guid`.
    #[must_use]
    pub fn fallback_guid(mut self, guid: impl Into<String>) -> Self {
        self.fallback_guid = Some(guid.into());
        self
    }

    /// Adds a single extraction rule, routed to feed or track rules
    /// based on its target.
    #[must_use]
    pub fn rule(mut self, rule: Rule) -> Self {
        match rule.target {
            Target::Feed(_) => self.feed_rules.push(rule),
            Target::Track(_) => self.track_rules.push(rule),
        }
        self
    }

    /// Builds the configured `FeedParser`.
    #[must_use]
    pub fn build(self) -> FeedParser {
        FeedParser {
            feed_rules: self.feed_rules,
            track_rules: self.track_rules,
            fallback_guid: self.fallback_guid,
            phases: self.phases,
        }
    }
}

// --- Extraction helpers ---

/// Finds the `<channel>` element in the document.
fn find_channel<'a>(root: &'a roxmltree::Node<'a, 'a>) -> Option<roxmltree::Node<'a, 'a>> {
    // Could be <rss><channel> or just <channel> at root
    if root.has_tag_name("channel") {
        return Some(*root);
    }
    root.children().find(|n| n.has_tag_name("channel"))
}

/// Extracts a text value from a node according to a [`Source`] specification.
fn extract_source(node: &roxmltree::Node, source: &Source) -> Option<String> {
    match source {
        Source::ChildText { tag, ns } => {
            find_child(node, tag, *ns)
                .and_then(|n| child_text(&n))
        }
        Source::ChildTextFallback { tags } => {
            for &(tag, ref ns) in *tags {
                if let Some(text) = find_child(node, tag, *ns)
                    .and_then(|n| child_text(&n))
                {
                    return Some(text);
                }
            }
            None
        }
        Source::ChildAttr { tag, ns, attr } => {
            find_child(node, tag, *ns)
                .and_then(|n| n.attribute(*attr).map(String::from))
        }
        Source::NestedText { parent, parent_ns, child, child_ns } => {
            let parent_node = find_child(node, parent, *parent_ns)?;
            let child_node = find_child(&parent_node, child, *child_ns)?;
            child_text(&child_node)
        }
        Source::Attr { tag, ns, attr } => {
            find_child(node, tag, *ns)
                .and_then(|n| n.attribute(*attr).map(String::from))
        }
    }
}

/// Finds the first child element matching a local name and optional namespace.
fn find_child<'a>(
    node: &'a roxmltree::Node<'a, '_>,
    tag: &str,
    ns: Option<&str>,
) -> Option<roxmltree::Node<'a, 'a>> {
    node.children().find(|n| {
        n.is_element()
            && n.tag_name().name() == tag
            && match ns {
                Some(uri) => n.tag_name().namespace() == Some(uri),
                None => true,
            }
    })
}

/// Extracts text content from an element, handling both direct text and nested text nodes.
fn child_text(node: &roxmltree::Node) -> Option<String> {
    // Collect all text children (handles mixed content, CDATA, etc.)
    let text: String = node.children()
        .filter(roxmltree::Node::is_text)
        .filter_map(|n| n.text())
        .collect();

    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// Extracts `podcast:value > podcast:valueRecipient` payment routes from a node.
fn extract_payment_routes(node: &roxmltree::Node) -> Vec<IngestPaymentRoute> {
    let mut routes = Vec::new();

    for value_node in node.children().filter(|n| {
        n.is_element() && n.tag_name().name() == "value"
            && n.tag_name().namespace() == Some(PODCAST_NS)
    }) {
        for recipient in value_node.children().filter(|n| {
            n.is_element() && n.tag_name().name() == "valueRecipient"
                && n.tag_name().namespace() == Some(PODCAST_NS)
        }) {
            let address = match recipient.attribute("address") {
                Some(a) if !a.trim().is_empty() => a.trim().to_owned(),
                _ => continue, // Skip recipients without an address
            };

            let route_type_str = recipient.attribute("type").unwrap_or("node");
            let route_type = if route_type_str.eq_ignore_ascii_case("lnaddress") {
                RouteType::Lnaddress
            } else {
                // Default to Node (Sprint 5 fix: was incorrectly "lightning")
                RouteType::Node
            };

            let split = recipient
                .attribute("split")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);

            let fee = recipient
                .attribute("fee")
                .is_some_and(|f| f.eq_ignore_ascii_case("true") || f == "1");

            routes.push(IngestPaymentRoute {
                recipient_name: recipient.attribute("name").map(String::from),
                route_type,
                address,
                custom_key: recipient.attribute("customKey").map(String::from),
                custom_value: recipient.attribute("customValue").map(String::from),
                split,
                fee,
            });
        }
    }

    routes
}

/// Extracts `podcast:valueTimeSplit` entries with `podcast:remoteItem` children.
fn extract_value_time_splits(node: &roxmltree::Node) -> Vec<IngestValueTimeSplit> {
    let mut splits = Vec::new();

    for value_node in node.children().filter(|n| {
        n.is_element() && n.tag_name().name() == "value"
            && n.tag_name().namespace() == Some(PODCAST_NS)
    }) {
        for vts in value_node.children().filter(|n| {
            n.is_element() && n.tag_name().name() == "valueTimeSplit"
                && n.tag_name().namespace() == Some(PODCAST_NS)
        }) {
            // Skip if remotePercentage is present
            if vts.attribute("remotePercentage").is_some() {
                continue;
            }

            let Some(start_time_secs) = vts
                .attribute("startTime")
                .and_then(|s| s.parse::<i64>().ok())
            else {
                continue;
            };

            let duration_secs = vts
                .attribute("duration")
                .and_then(|s| s.parse::<i64>().ok());

            let split = vts
                .attribute("split")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);

            // Sprint 5 fix: read GUIDs from podcast:remoteItem *child element*
            let remote_item = vts.children().find(|n| {
                n.is_element() && n.tag_name().name() == "remoteItem"
                    && n.tag_name().namespace() == Some(PODCAST_NS)
            });

            let Some(remote) = remote_item else {
                continue;
            };

            let remote_feed_guid = match remote.attribute("feedGuid") {
                Some(g) if !g.trim().is_empty() => g.trim().to_owned(),
                _ => continue,
            };

            let remote_item_guid = match remote.attribute("itemGuid") {
                Some(g) if !g.trim().is_empty() => g.trim().to_owned(),
                _ => continue,
            };

            splits.push(IngestValueTimeSplit {
                start_time_secs,
                duration_secs,
                remote_feed_guid,
                remote_item_guid,
                split,
            });
        }
    }

    splits
}

// --- Feed/Track data builders ---

#[derive(Default)]
struct FeedDataBuilder {
    feed_guid: Option<String>,
    title: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
    language: Option<String>,
    explicit: bool,
    explicit_set: bool,
    itunes_type: Option<String>,
    raw_medium: Option<String>,
    author_name: Option<String>,
    owner_name: Option<String>,
    pub_date: Option<i64>,
}

impl FeedDataBuilder {
    fn is_set(&self, field: FeedField) -> bool {
        match field {
            FeedField::FeedGuid => self.feed_guid.is_some(),
            FeedField::Title => self.title.is_some(),
            FeedField::Description => self.description.is_some(),
            FeedField::ImageUrl => self.image_url.is_some(),
            FeedField::Language => self.language.is_some(),
            FeedField::Explicit => self.explicit_set,
            FeedField::ItunesType => self.itunes_type.is_some(),
            FeedField::RawMedium => self.raw_medium.is_some(),
            FeedField::AuthorName => self.author_name.is_some(),
            FeedField::OwnerName => self.owner_name.is_some(),
            FeedField::PubDate => self.pub_date.is_some(),
        }
    }

    fn set(&mut self, field: FeedField, value: TransformResult) {
        match (field, value) {
            (FeedField::FeedGuid, TransformResult::Text(v)) => self.feed_guid = Some(v),
            (FeedField::Title, TransformResult::Text(v)) => self.title = Some(v),
            (FeedField::Description, TransformResult::Text(v)) => self.description = Some(v),
            (FeedField::ImageUrl, TransformResult::Text(v)) => self.image_url = Some(v),
            (FeedField::Language, TransformResult::Text(v)) => self.language = Some(v),
            (FeedField::Explicit, TransformResult::Bool(v)) => {
                self.explicit = v;
                self.explicit_set = true;
            }
            (FeedField::ItunesType, TransformResult::Text(v)) => self.itunes_type = Some(v),
            (FeedField::RawMedium, TransformResult::Text(v)) => self.raw_medium = Some(v),
            (FeedField::AuthorName, TransformResult::Text(v)) => self.author_name = Some(v),
            (FeedField::OwnerName, TransformResult::Text(v)) => self.owner_name = Some(v),
            (FeedField::PubDate, TransformResult::Int(v)) => self.pub_date = Some(v),
            _ => {} // Type mismatch — silently skip
        }
    }
}

#[derive(Default)]
struct TrackDataBuilder {
    track_guid: Option<String>,
    title: Option<String>,
    pub_date: Option<i64>,
    duration_secs: Option<i64>,
    enclosure_url: Option<String>,
    enclosure_type: Option<String>,
    enclosure_bytes: Option<i64>,
    track_number: Option<i64>,
    season: Option<i64>,
    explicit: bool,
    explicit_set: bool,
    description: Option<String>,
    author_name: Option<String>,
}

impl TrackDataBuilder {
    fn is_set(&self, field: TrackField) -> bool {
        match field {
            TrackField::TrackGuid => self.track_guid.is_some(),
            TrackField::Title => self.title.is_some(),
            TrackField::PubDate => self.pub_date.is_some(),
            TrackField::DurationSecs => self.duration_secs.is_some(),
            TrackField::EnclosureUrl => self.enclosure_url.is_some(),
            TrackField::EnclosureType => self.enclosure_type.is_some(),
            TrackField::EnclosureBytes => self.enclosure_bytes.is_some(),
            TrackField::TrackNumber => self.track_number.is_some(),
            TrackField::Season => self.season.is_some(),
            TrackField::Explicit => self.explicit_set,
            TrackField::Description => self.description.is_some(),
            TrackField::AuthorName => self.author_name.is_some(),
        }
    }

    fn set(&mut self, field: TrackField, value: TransformResult) {
        match (field, value) {
            (TrackField::TrackGuid, TransformResult::Text(v)) => self.track_guid = Some(v),
            (TrackField::Title, TransformResult::Text(v)) => self.title = Some(v),
            (TrackField::PubDate, TransformResult::Int(v)) => self.pub_date = Some(v),
            (TrackField::DurationSecs, TransformResult::Int(v)) => self.duration_secs = Some(v),
            (TrackField::EnclosureUrl, TransformResult::Text(v)) => self.enclosure_url = Some(v),
            (TrackField::EnclosureType, TransformResult::Text(v)) => self.enclosure_type = Some(v),
            (TrackField::EnclosureBytes, TransformResult::Int(v)) => self.enclosure_bytes = Some(v),
            (TrackField::TrackNumber, TransformResult::Int(v)) => self.track_number = Some(v),
            (TrackField::Season, TransformResult::Int(v)) => self.season = Some(v),
            (TrackField::Explicit, TransformResult::Bool(v)) => {
                self.explicit = v;
                self.explicit_set = true;
            }
            (TrackField::Description, TransformResult::Text(v)) => self.description = Some(v),
            (TrackField::AuthorName, TransformResult::Text(v)) => self.author_name = Some(v),
            _ => {} // Type mismatch — silently skip
        }
    }
}
