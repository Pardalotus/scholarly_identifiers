use std::str::FromStr;

use crate::{doi, isbn, orcid, uri};
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
        Identifier::String(String::from(input))
    }

    /// Convert to a URI format, if possible.
    pub fn to_uri(&self) -> Option<String> {
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
