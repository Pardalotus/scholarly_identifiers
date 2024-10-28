use std::str::FromStr;

use crate::{doi, isbn, orcid, uri};
use http::Uri;

/// A Scholarly Identifier.
/// Each type of scholarly identifier has a different purpose, different semantics for construction, different validation and comparison.
#[derive(Debug, PartialEq)]
pub enum Identifier {
    // A DOI, split into prefix (e.g. "10.5555") and suffix (eg. "12345678").
    // This representation is the native Unicode representation, i.e. is not URL-encoded.
    Doi { prefix: String, suffix: String },

    Orcid { value: String },

    // A valid URI, but only used when other types aren't recognised.
    Uri(String),

    // An identifier that doesn't match any known type. The fall-through case.
    String { value: String },

    // A 13-digit ISBN, without hyphens or spaces.
    Isbn(String),
}

/// Signature of a function that attempts to parse to an Identifier.
type IdentifierParser = fn(input: &IdentifierParseInput) -> Option<Identifier>;

// List of parsers, in order of prededence.
const PARSERS: &[IdentifierParser] = &[
    // DOIs are a subset of Handle, so must be attempted before Handles.
    doi::try_parse,
    orcid::try_parse,
    isbn::try_parse,
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
        Identifier::String {
            value: String::from(input),
        }
    }

    /// Convert to a URI format, if possible.
    pub fn to_uri(&self) -> Option<String> {
        match self {
            Identifier::Doi {
                prefix: _,
                suffix: _,
            } => doi::to_uri(self),

            Identifier::Orcid { value: _ } => orcid::to_uri(self),

            Identifier::Uri(value) => Some(value.clone()),

            // Don't assume the String type can be converted to a URI.
            // If it had been parseable as a URI, it would have been parsed as the Identifier::Uri type.
            Identifier::String { value: _ } => None,

            // No natural URI for ISBN.
            Identifier::Isbn(_) => None,
        }
    }
}

/// Intermediary representation of an input with pre-computed values needed by various parsers.
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
                Some(String::from(if path.starts_with("/") {
                    &path[1..]
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
