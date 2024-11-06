use std::str::FromStr;

use crate::{doi, isbn, orcid, ror, uri};
use http::Uri;

/// A Scholarly Identifier.
/// Each type of scholarly identifier has a different purpose, different semantics for construction, different validation and comparison.
#[derive(Debug, PartialEq)]
pub enum Identifier {
    /// DOI, Digital Object Identifier
    ///
    /// A DOI, split into prefix (e.g. "10.5555") and suffix (eg. "12345678").
    /// This representation is the native Unicode representation, i.e. is not URL-encoded.
    ///
    /// DOIs are persistent identifiers used in scholarly metadata.
    /// The DOI system is a subset of Handle. Every DOI is a Handle.
    ///
    /// A DOI can contain any printable Unicode character. A 'raw' DOI string starts
    /// with "10.", followed by a string of numbers, then "/", then a string of any
    /// printable Unicode characters. A DOI can also be encoded as URLs (which makes
    /// it a resolvable identifier). When it's represented as a URL, it must be
    /// carefully encoded.
    ///
    /// A 'raw' DOI string can be compared for equality against another 'raw' DOI string.
    ///
    /// DOIs are commonly turned into URLs by prepending the link resolver
    /// "`https://doi.org/`", although other link resolvers such as
    /// "`https://hdl.handle.net/`" or "`https://dx.doi.org/`" work and have been used
    /// in the past.
    ///
    /// DOI Handbook's [Encode a DOI according per "DOI Name Encoding Rules for URL Presentation" in the DOI handbook](https://www.doi.org/doi-handbook/HTML/encoding-rules-for-urls.html)
    /// sets out how DOIs should be encoded. This library follows those rules.
    ///
    /// However, there are many ways to encode a URL. This means that a URL representation of a DOI cannot be reliably compared for equality.
    ///
    Doi { prefix: String, suffix: String },

    /// ORCID, Open Researcher and Contributor ID
    /// An ORCID iD, expressed as a raw identifier without the link resolver.
    Orcid(String),

    /// ROR, Research Organisation Registry id.
    /// A raw identifier, without the link resolver.
    Ror(String),

    /// A valid URI, but only used when other types aren't recognised.
    Uri(String),

    /// An identifier that doesn't match any known type. The fall-through case.
    String(String),

    /// ISBN, International Standard Book Identifier
    /// Always expressed in the 13-digit form, including check-digit.
    /// Hyphens and spaces are removed.
    Isbn(String),
}

/// Signature of a function that attempts to parse to an Identifier.
type IdentifierParser = fn(input: &IdentifierParseInput) -> Option<Identifier>;

// List of parsers, in order of precedence.
const PARSERS: &[IdentifierParser] = &[
    // DOIs are a subset of Handle, so must be attempted before Handles.
    doi::try_parse,
    orcid::try_parse,
    isbn::try_parse,
    ror::try_parse,
    // URIs are greedy, so place last in the list.
    uri::try_parse,
];

impl Identifier {
    /// Parse an input string, producing an Identifier. This will always
    /// succeed, but if the type isn't recognised, an Identifier::String will be
    /// returned, which indicates that it wasn't possible to recognise it.
    pub fn parse(input: &str) -> Identifier {
        let parse_input = IdentifierParseInput::build(input);

        for parser in PARSERS.iter() {
            if let Some(result) = parser(&parse_input) {
                return result;
            }
        }

        // Fall-back case.
        Identifier::String(String::from(input))
    }

    /// Convert to a URI format, if possible.
    /// As not all identifiers have a URI representation, this might return None.
    pub fn to_uri(&self) -> Option<String> {
        // The to_uri functions take the Identifier, not the unwrapped value.
        // This allows them to guard that they are supplied the right type, which is important for a correct implementation.
        // This in turn means those functions are safe to use directly if required.
        match self {
            Identifier::Doi {
                prefix: _,
                suffix: _,
            } => doi::to_uri(self),

            Identifier::Orcid(_) => orcid::to_uri(self),

            Identifier::Uri(value) => Some(value.clone()),

            // Don't assume the String type can be converted to a URI.
            // If it had been parseable as a URI, it would have been parsed as the Identifier::Uri type.
            Identifier::String(_) => None,

            Identifier::Isbn(_) => isbn::to_uri(self),
            Identifier::Ror(_) => ror::to_uri(self),
        }
    }

    /// Represent as a simple, stable string representation.
    /// Depending on type, this is sometimes the URI representation, sometimes not.
    /// This representation is meant to be stable and consistent, so that it can be used as a key in a database.
    /// It should also be deterministically parsed back into the same value.
    pub fn to_stable_string(&self) -> String {
        // Some use of unwrap in cases where we know that the URI will always succeed when the enum variant is right.
        let maybe_string = match self {
            // Recommended format for a DOI is the URI.
            Identifier::Doi {
                prefix: _,
                suffix: _,
            } => doi::to_stable_string(self),

            Identifier::Orcid(_) => orcid::to_stable_string(self),

            Identifier::Uri(_) => uri::to_stable_string(self),

            // Don't assume the String type can be converted to a URI.
            // If it had been parseable as a URI, it would have been parsed as the Identifier::Uri type.
            Identifier::String(value) => Some(value.clone()),

            // No natural URI for ISBN.
            Identifier::Isbn(_) => isbn::to_stable_string(self),
            Identifier::Ror(_) => ror::to_stable_string(self),
        };

        // All of the above should handle representation.
        // An Option at this point is a bug. Use the fall-back debug format.
        if let Some(result) = maybe_string {
            result
        } else {
            log::error!("Failed to convert to string: {:?}", self);

            format!("{:?}", self)
        }
    }

    /// Convert to a pair of simple stable string representation and a numeric type id.
    /// The simple string is usually not the URI format.
    /// These type IDs are defined to be stable, and should not be altered.
    pub fn to_id_string_pair(&self) -> (String, u32) {
        let maybe_result = match self {
            Identifier::Doi {
                prefix: _,
                suffix: _,
            } => (doi::to_stable_string(self), 1),
            Identifier::Orcid(_) => (orcid::to_stable_string(self), 2),
            Identifier::Ror(_) => (ror::to_stable_string(self), 3),
            Identifier::Uri(_) => (uri::to_stable_string(self), 4),
            Identifier::String(value) => (Some(value.clone()), 5),
            Identifier::Isbn(_) => (isbn::to_stable_string(self), 6),
        };

        // All of the above should handle representations.
        // A None at this point is a bug. Use the fall-back debug format.
        match maybe_result {
            (Some(value), type_id) => (value, type_id),
            _ => {
                log::error!("Failed to convert to string: {:?}", self);
                (format!("{:?}", self), 5)
            }
        }
    }

    /// Construct from a (type id, string) pair.
    pub fn from_id_string_pair(input_str: &str, type_id: u32) -> Option<Identifier> {
        let parse_input = IdentifierParseInput::build(input_str);

        match type_id {
            1 => doi::try_parse(&parse_input),
            2 => orcid::try_parse(&parse_input),
            3 => ror::try_parse(&parse_input),
            4 => uri::try_parse(&parse_input),
            5 => Some(Identifier::String(String::from(input_str))),
            6 => isbn::try_parse(&parse_input),
            _ => {
                log::error!("Unrecognised type id {}", type_id);
                None
            }
        }
    }
}

/// Intermediary representation of an input with pre-computed values needed by various parsers.
#[derive(Debug)]
pub(crate) struct IdentifierParseInput {
    pub raw: String,

    pub uri: Option<Uri>,
}

impl IdentifierParseInput {
    fn build(input: &str) -> IdentifierParseInput {
        // Nearly all identifier types want the input parsed to a URI.
        let valid_uri = match Uri::from_str(input) {
            Ok(result) => Some(result),
            Err(_) => None,
        };

        IdentifierParseInput {
            raw: String::from(input),
            uri: valid_uri,
        }
    }

    /// Return the path with the leading slash removed.
    /// There may not be a leading slash.
    pub(crate) fn path_no_slash(&self) -> Option<String> {
        match self.uri {
            Some(ref uri) => {
                let path = uri.path();

                Some(String::from(if let Some(rest) = path.strip_prefix("/") {
                    rest
                } else {
                    path
                }))
            }
            _ => None,
        }
    }

    pub(crate) fn path_no_slash_uppercase(&self) -> Option<String> {
        self.path_no_slash().map(|path| path.to_uppercase())
    }

    pub(crate) fn host(&self) -> Option<&str> {
        match &self.uri {
            Some(uri) => uri.host(),
            _ => None,
        }
    }

    pub(crate) fn host_lowercase(&self) -> Option<String> {
        self.host().map(|x| x.to_lowercase())
    }
}

// End-to-end tests for each type.
#[cfg(test)]
mod doi_end_to_end_tests {
    use super::*;

    #[test]
    fn stable() {
        let inputs = vec![
            // DOI
            "doi.org/10.5555/12345678",
            "https://doi.org/10.5555/12345678",
            "10.5555/12345678",
            "10.5555/1234e®🄮™5678",
            // ISBN
            "0306406152",
            // ORCID
            "https://orcid.org/0000-0002-1694-233X",
            // ROR
            "https://ror.org/02twcfp32",
            // URI
            "https://example.com",
            // String
            "hello",
        ];

        for input in inputs.iter() {
            let parsed = Identifier::parse(input);

            let as_string = parsed.to_stable_string();

            // Not all types produce a URI.
            let as_uri = parsed.to_uri();

            let (id_value, id_type) = parsed.to_id_string_pair();

            let as_string_parsed = Identifier::parse(&as_string);
            assert_eq!(
                as_string_parsed, parsed,
                "Expected string type to round-trip identical"
            );

            // Those types that do parse to URIs should round-trip.
            if let Some(uri) = as_uri {
                let as_uri_parsed = Identifier::parse(&uri);

                assert_eq!(
                    as_uri_parsed, parsed,
                    "Expected URI type to round-trip identical"
                );
            }

            let as_pair_parsed = Identifier::from_id_string_pair(&id_value, id_type)
                .expect("Expected as_pair to return a result");
            assert_eq!(
                as_pair_parsed, parsed,
                "Expected pair to round-trip identical"
            );
        }
    }
}
