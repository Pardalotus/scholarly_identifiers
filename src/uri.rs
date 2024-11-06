//! A URI
//!
//! This is a fall-through case, as most types of identifiers are themselves URIs.
//! This parser is greedy and doesn't attempt to avoid recognising other URI types (e.g. DOI). It relies on being called after the other types.

use crate::identifiers::{Identifier, IdentifierParseInput};

pub(crate) fn try_parse(input: &IdentifierParseInput) -> Option<Identifier> {
    // Rely on the pre-compused URI.
    input
        .uri
        .as_ref()
        .map(|uri| Identifier::Uri(uri.to_string()))
}

/// Represent a URI Identifier type as a URI string.
pub(crate) fn to_uri(input: &Identifier) -> Option<String> {
    match input {
        Identifier::Uri(value) => Some(value.clone()),
        _ => None,
    }
}

/// Encode a URI as a stable string in the recommended format.
/// Will always return a String if an URI type is supplied.
pub(crate) fn to_stable_string(input: &Identifier) -> Option<String> {
    to_uri(input)
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn parse_simple() {
        assert_eq!(
            Identifier::Uri(String::from("http://example.com/"),),
            Identifier::parse("http://example.com/")
        );
    }

    /// Some strings don't look like URIs, but are.
    #[test]
    fn parse_unconventional() {
        assert_eq!(
            Identifier::Uri(String::from("an-unconventional-uri"),),
            Identifier::parse("an-unconventional-uri")
        );
    }

    /// Unencoded Unicode strings are not valid URIs, and are not parsed as such.
    #[test]
    fn parse_invalid() {
        assert_eq!(
            Identifier::String(String::from("http://example.com/®")),
            Identifier::parse("http://example.com/®")
        );
    }
}

#[cfg(test)]
mod end_to_end_tests {
    use super::*;

    #[test]
    fn parse_simple() {
        assert_eq!(
            "http://example.com/",
            Identifier::parse("http://example.com/").to_uri().unwrap()
        );
    }
}
