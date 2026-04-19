//! Transform operations applied to extracted XML text.

use crate::date::parse_date;
use crate::duration::parse_duration;
use crate::html::{decode_entities, strip_html};

/// A transform applied to an extracted string value before storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    /// Pass through as-is.
    None,
    /// Parse as integer (e.g. "42" -> 42).
    ParseInt,
    /// Parse RFC-2822 / ISO-8601 date to Unix seconds.
    ParseDate,
    /// Parse iTunes duration (HH:MM:SS, MM:SS, seconds) to seconds.
    ParseDuration,
    /// Strip HTML tags and decode entities.
    StripHtml,
    /// Parse explicit flag: "yes"/"true" -> true, everything else -> false.
    ExplicitBool,
    /// Decode HTML/XML entities only (no tag stripping).
    DecodeEntities,
    /// Lowercase the string.
    Lowercase,
    /// Extract the first URL from a Podcast Namespace `srcset` value.
    FirstSrcsetUrl,
}

/// Result of applying a transform: either a string or a parsed integer.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TransformResult {
    Text(String),
    Int(i64),
    Bool(bool),
}

/// Applies the given transform to a raw string value.
pub(crate) fn apply_transform(transform: Transform, value: &str) -> Option<TransformResult> {
    match transform {
        Transform::None => Some(TransformResult::Text(value.to_owned())),
        Transform::ParseInt => value.trim().parse::<i64>().ok().map(TransformResult::Int),
        Transform::ParseDate => parse_date(value).map(TransformResult::Int),
        Transform::ParseDuration => parse_duration(value).map(TransformResult::Int),
        Transform::StripHtml => Some(TransformResult::Text(strip_html(value))),
        Transform::ExplicitBool => {
            let lower = value.trim().to_lowercase();
            Some(TransformResult::Bool(lower == "yes" || lower == "true"))
        }
        Transform::DecodeEntities => Some(TransformResult::Text(decode_entities(value))),
        Transform::Lowercase => Some(TransformResult::Text(value.to_lowercase())),
        Transform::FirstSrcsetUrl => first_srcset_url(value).map(TransformResult::Text),
    }
}

fn first_srcset_url(value: &str) -> Option<String> {
    value.split(',').find_map(|candidate| {
        candidate
            .split_whitespace()
            .next()
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(str::to_owned)
    })
}
