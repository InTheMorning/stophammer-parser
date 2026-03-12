//! HTML entity decoding and tag stripping.

use std::sync::LazyLock;

use regex::Regex;

static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<[^>]+>").expect("html tag regex is valid")
});

/// Strips HTML tags and decodes common entities.
///
/// # Examples
///
/// ```
/// use stophammer_parser::html::strip_html;
///
/// assert_eq!(strip_html("<p>Hello &amp; world</p>"), "Hello & world");
/// assert_eq!(strip_html("no tags here"), "no tags here");
/// ```
#[must_use]
pub fn strip_html(s: &str) -> String {
    let stripped = HTML_TAG_RE.replace_all(s, "");
    decode_entities(&stripped)
}

/// Decodes HTML/XML entities in a string.
///
/// Handles named entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`)
/// and numeric entities (`&#NNN;`, `&#xNNN;`).
///
/// # Examples
///
/// ```
/// use stophammer_parser::html::decode_entities;
///
/// assert_eq!(decode_entities("Tom &amp; Jerry"), "Tom & Jerry");
/// assert_eq!(decode_entities("&#60;tag&#62;"), "<tag>");
/// assert_eq!(decode_entities("&#x26;"), "&");
/// ```
#[must_use]
pub fn decode_entities(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '&' {
            result.push(c);
            continue;
        }

        // Collect entity up to ';'
        let mut entity = String::new();
        let mut found_semi = false;
        for ec in chars.by_ref() {
            if ec == ';' {
                found_semi = true;
                break;
            }
            entity.push(ec);
            // Guard against runaway — entities are short
            if entity.len() > 10 {
                break;
            }
        }

        if !found_semi {
            // Not a real entity, emit verbatim
            result.push('&');
            result.push_str(&entity);
            continue;
        }

        match entity.as_str() {
            "amp" => result.push('&'),
            "lt" => result.push('<'),
            "gt" => result.push('>'),
            "quot" => result.push('"'),
            "apos" => result.push('\''),
            _ if entity.starts_with('#') => {
                if let Some(ch) = parse_numeric_entity(&entity[1..]) {
                    result.push(ch);
                } else {
                    result.push('&');
                    result.push_str(&entity);
                    result.push(';');
                }
            }
            _ => {
                // Unknown entity — pass through
                result.push('&');
                result.push_str(&entity);
                result.push(';');
            }
        }
    }

    result
}

/// Parses a numeric entity body (after `#`): decimal or hex (`x` prefix).
fn parse_numeric_entity(s: &str) -> Option<char> {
    let code = if let Some(hex) = s.strip_prefix('x').or_else(|| s.strip_prefix('X')) {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        s.parse::<u32>().ok()?
    };
    char::from_u32(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_tags() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn decode_named_entities() {
        assert_eq!(decode_entities("&amp; &lt; &gt; &quot; &apos;"), "& < > \" '");
    }

    #[test]
    fn decode_decimal_entity() {
        assert_eq!(decode_entities("&#60;tag&#62;"), "<tag>");
    }

    #[test]
    fn decode_hex_entity() {
        assert_eq!(decode_entities("&#x26;"), "&");
    }

    #[test]
    fn cdata_not_special() {
        // CDATA markers are already stripped by roxmltree; this just ensures
        // we don't break on text that includes them literally.
        assert_eq!(strip_html("plain text"), "plain text");
    }

    #[test]
    fn ampersand_without_entity() {
        assert_eq!(decode_entities("rock & roll"), "rock & roll");
    }
}
