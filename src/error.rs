//! Canonical error type for feed parsing failures.

use std::fmt;

/// Error returned when feed parsing fails.
///
/// Provides structured access to the failure reason without
/// exposing internal parser types.
///
/// # Examples
///
/// ```
/// use stophammer_parser::{profile, ParseError};
///
/// let result = profile::stophammer().parse("<not xml");
/// assert!(result.is_err());
/// assert!(result.unwrap_err().is_xml());
/// ```
#[derive(Debug)]
pub struct ParseError {
    pub(crate) kind: ErrorKind,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// XML is not well-formed.
    Xml(roxmltree::Error),
    /// Document has no `<channel>` element.
    NoChannel,
    /// Feed has no `<title>`.
    NoTitle {
        /// The channel's `podcast:medium`, if the channel declares one.
        raw_medium: Option<String>,
    },
    /// Feed has no `podcast:guid`.
    NoGuid {
        /// The channel's `podcast:medium`, if the channel declares one.
        raw_medium: Option<String>,
    },
}

impl ParseError {
    /// Returns `true` if the error is due to malformed XML.
    #[must_use]
    pub fn is_xml(&self) -> bool {
        matches!(self.kind, ErrorKind::Xml(_))
    }

    /// Returns `true` if the error is due to a missing required field.
    #[must_use]
    pub fn is_missing_field(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::NoChannel | ErrorKind::NoTitle { .. } | ErrorKind::NoGuid { .. }
        )
    }

    /// Returns the channel's raw `podcast:medium`, when the parser found a
    /// `<channel>` before it stopped at a missing `<title>` or a missing
    /// `podcast:guid`.
    ///
    /// Returns `None` for every other error kind, and `None` when the
    /// channel has no `podcast:medium`. See stophammer ADR 0074 §3a.
    #[must_use]
    pub fn raw_medium(&self) -> Option<&str> {
        match &self.kind {
            ErrorKind::NoTitle { raw_medium } | ErrorKind::NoGuid { raw_medium } => {
                raw_medium.as_deref()
            }
            ErrorKind::Xml(_) | ErrorKind::NoChannel => None,
        }
    }

    /// Returns `true` if the parser found a `<channel>` element before the
    /// error.
    ///
    /// This holds for a missing `<title>` or a missing `podcast:guid`. See
    /// stophammer ADR 0074 §3a.
    #[must_use]
    pub fn has_channel(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::NoTitle { .. } | ErrorKind::NoGuid { .. }
        )
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Xml(e) => write!(f, "malformed XML: {e}"),
            ErrorKind::NoChannel => write!(f, "missing <channel> element"),
            ErrorKind::NoTitle { .. } => write!(f, "missing <title> element"),
            ErrorKind::NoGuid { .. } => write!(f, "missing podcast:guid"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Xml(e) => Some(e),
            _ => None,
        }
    }
}

impl From<roxmltree::Error> for ParseError {
    fn from(e: roxmltree::Error) -> Self {
        Self {
            kind: ErrorKind::Xml(e),
        }
    }
}
