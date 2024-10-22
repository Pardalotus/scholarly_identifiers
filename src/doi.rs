//! DOI
//! https://doi.org
//!
//! DOIs are persistent identifiers used in scholarly metadata.
//! The DOI system is a subset of Handle. Every DOI is a Handle.
//!
//! A DOI can contain any printable Unicode character. A 'raw' DOI string starts
//! with "10.", followed by a string of numbers, then "/", then a string of any
//! printable Unicode characters. A DOI can also be encoded as URLs (which makes
//! it a resolvable identifier). When it's represented as a URL, it must be
//! carefully encoded.
//!
//! A 'raw' DOI string can be compared for equality against another 'raw' DOI string.
//!
//! DOIs are commonly turned into URLs by prepending the link resolver
//! "https://doi.org/", although other link resolvers such as
//! "https://hdl.handle.net/" or "https://dx.doi.org/" work and have been used
//! in the past.
//!
//! DOI Handbook's [Encode a DOI according per "DOI Name Encoding Rules for URL Presentation" in the DOI handbook](https://www.doi.org/doi-handbook/HTML/encoding-rules-for-urls.html)
//! sets out how DOIs should be encoded. This library follows those rules.
//!
//! However, there are many ways to encode a URL. This means that a URL representation of a DOI cannot be reliably compared for equality.
//!

use std::collections::HashSet;
use std::fmt::Write;

use crate::identifiers::{Identifier, IdentifierParseInput};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {

    /// Match for various kinds of URI scheme that may be used in a DOI URI.
    /// Because of the variety of presentations of DOIs, it's possible to find a DOI like "http://doi.org/urn:doi:10.5555/12345678"
    /// in the wild. So the URI prefixes are removed from both the start of the whole string and the start of the path.
    static ref URI_PREFIXES_SCHEME: Regex = Regex::new(r"^(https://|http://|doi:|urn:doi:|info:doi:)").unwrap();

    /// Match for hostnames of DOI resolvers.
    static ref URI_PREFIXES_HOST: Regex = Regex::new(r"^(dx.doi.org/|doi.org/)").unwrap();

    /// Match a potential DOI with an encoded slash, anchored to the start of the string.
    static ref DOI_RE : Regex = Regex::new(r"^10\.\d+(/|%2f).*").unwrap();

    /// Match a potential DOI strictly, anchored to the start of the string.
    static ref DOI_STRICT_RE : Regex = Regex::new(r"^(10\.\d+)/(.+)$").unwrap();

    /// From RFC 3986 section 2.3 Unreserved Characters
    static ref UNRESERVED_CHARACTERS : HashSet<char> = HashSet::from_iter("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~".chars());

    /// From RFC 3986 section 2.2 Reserved Characters
    static ref RESERVED_CHARACTERS : HashSet<char> = HashSet::from_iter("!$&'()*,/:;=@".chars());

    /// These characters should not be encoded in a DOI. All others must be.
    static ref DO_NOT_ENCODE : HashSet<char> = HashSet::from_iter(UNRESERVED_CHARACTERS.union(&RESERVED_CHARACTERS).copied());
}

/// Percent-encode characters according to the specific rules for DOI encoding.
fn percent_encode_for_doi(input: &str) -> String {
    let mut result_buffer = String::new();

    // For multi-byte sequences. Unicode has a maximum charcter size of 4 bytes.
    let mut char_buffer = [0; 4];

    // Map to 4-byte UTF-8 chars.
    for c in input.chars() {
        if DO_NOT_ENCODE.contains(&c) {
            result_buffer.push(c);
        } else {
            // Number of bytes used.
            let size = c.encode_utf8(&mut char_buffer).len();

            for b in char_buffer[0..size].iter() {
                // The buffer is the correct size for any Unicode character. If
                // there's a failure to write, then this should be a noisy
                // error.
                write!(&mut result_buffer, "%{:02X}", b).unwrap();
            }
        }
    }

    result_buffer
}

// Construct an Identifier containing Unicode-native string.
fn construct(decoded_raw_doi: &String) -> Option<Identifier> {
    // If the input didn't start with "10." then the input was in the wrong format.
    if let Some(matched) = DOI_STRICT_RE.captures(decoded_raw_doi) {
        let prefix = matched.get(1).unwrap().as_str();
        let suffix = matched.get(2).unwrap().as_str();

        let lowercase_suffix = suffix.to_lowercase();

        Some(Identifier::Doi {
            prefix: String::from(prefix),
            suffix: lowercase_suffix,
        })
    } else {
        None
    }
}

// Remove the string prefixes for DOIs. Not DOI prefixes. Urgh.
fn remove_doi_prefixes(input: &String) -> String {
    // Remove leading scheme from start of string, if present.
    let no_scheme = URI_PREFIXES_SCHEME.replace(input, "").into_owned();

    // Remove leading resolver host, if present.
    let no_resolver = URI_PREFIXES_HOST.replace(&no_scheme, "").into_owned();

    // Remove leaidng scheme from path, if one was found.
    URI_PREFIXES_SCHEME.replace(&no_resolver, "").into_owned()
}

/// Parse an input string as a DOI, if recognised as a DOI.
///
/// Accepts:
///  - Plain DOI string, e.g. "10.5555/12345678", interpreted as a literal Unicode string.
///  - URL DOI, e.g. "https://doi.org/10.5555/12345678", intepreted as URL-encoded.
///
/// As a DOI can contain any printable Unicode character there's
/// no middle-ground and it's impossible to guess if a DOI is already encoded or
/// not.
///
/// If a URL DOI is incorrectly encoded, don't try to guess, just return as an
/// invalid DOI. To guess would be to break the resolvability of the identifier,
/// making it worse than useless.
pub(crate) fn try_parse(input: &IdentifierParseInput) -> Option<Identifier> {
    // DOIs are case-invariant so always lower-case them.
    let lowercase = input.raw.to_lowercase();

    // Raw DOIs can be encoded and put into a URI.
    if DOI_STRICT_RE.is_match(&lowercase) {
        construct(&lowercase)
    } else {
        // Otherwise treat this as a URI DOI, and attempt to parse.
        let less_prefixes = remove_doi_prefixes(&lowercase);

        if DOI_RE.is_match(&less_prefixes) {
            // Use [`percent_encoding::percent_decode`] rather than
            // [`percent_encoding::decode_utf8_lossy`] so this function fails when it encounters
            // invalid UTF-8 sequences.
            //
            // It's better to report invavlid DOIs than try to rescue them and end up
            // with an unintended string.
            match percent_encoding::percent_decode(less_prefixes.as_bytes()).decode_utf8() {
                Ok(decoded) => construct(&decoded.into_owned()),
                Err(err) => {
                    log::error!(
                        "Failed to decode URI component: {}, error: {}",
                        &less_prefixes,
                        err
                    );
                    None
                }
            }
        } else {
            None
        }
    }
}

/// Encode a DOI according per "DOI Name Encoding Rules for URL Presentation" in the DOI handbook.
/// https://www.doi.org/doi-handbook/HTML/encoding-rules-for-urls.html
///
/// There are options about how to encode a string into a URI. Follow a minimal stable set of rules:
///
/// 1. Don't encode RFC 3986 "unreserved characters".
/// 2. Encode all characters that are "mandatory" and "recommended" according to the DOI handbook.
/// 3. Don't encode any RFC 3986 "reserved" character that fall outside these ranges.
/// 4. Encode all other characters.
pub fn to_uri(input: &Identifier) -> Option<String> {
    match input {
        Identifier::Doi {
            ref prefix,
            ref suffix,
        } => {
            let encoded_suffix = percent_encode_for_doi(suffix);
            Some(format!("https://doi.org/{}/{}", prefix, encoded_suffix))
        }
        _ => None,
    }
}

/// Tests specifically for the parser.
#[cfg(test)]
mod doi_parser_tests {
    use super::*;

    #[test]
    fn parse_simple_raw() {
        assert_eq!(
            Identifier::Doi {
                prefix: String::from("10.5555"),
                suffix: String::from("12345678")
            },
            Identifier::parse("10.5555/12345678")
        );
    }

    #[test]
    fn parse_simple_doi_scheme() {
        assert_eq!(
            Identifier::Doi {
                prefix: String::from("10.5555"),
                suffix: String::from("12345678")
            },
            Identifier::parse("doi:10.5555/12345678"),
            "Siple DOIs should parse."
        );
    }

    #[test]
    fn parse_doi_resolver() {
        let expected = Identifier::Doi {
            prefix: String::from("10.5555"),
            suffix: String::from("12345678"),
        };

        // Standard resolver.
        assert_eq!(
            expected,
            Identifier::parse("https://doi.org/10.5555/12345678"),
            "URLs on the DOI resolver using HTTPS should parse."
        );

        // Standard resolver.
        assert_eq!(
            expected,
            Identifier::parse("http://doi.org/10.5555/12345678"),
            "URLs on the DOI resolver using HTTP should parse."
        );

        // Old resolver http
        assert_eq!(
            expected,
            Identifier::parse("https://dx.doi.org/10.5555/12345678"),
            "Old resolver using HTTP should be recognised."
        );

        // Old resolver https
        assert_eq!(
            expected,
            Identifier::parse("http://dx.doi.org/10.5555/12345678"),
            "Old resolver using HTTPS should be recognised."
        );
    }

    #[test]
    fn parse_doi_schemes() {
        let expected = Identifier::Doi {
            prefix: String::from("10.5555"),
            suffix: String::from("12345678"),
        };

        assert_eq!(expected, Identifier::parse("doi:10.5555/12345678"));
        assert_eq!(expected, Identifier::parse("info:doi:10.5555/12345678"));
        assert_eq!(expected, Identifier::parse("urn:doi:10.5555/12345678"));
    }

    #[test]
    fn lower_case() {
        let expected = Identifier::Doi {
            prefix: String::from("10.5555"),
            suffix: String::from("abcdefg"),
        };

        assert_eq!(expected, Identifier::parse("10.5555/abcdefg"));
        assert_eq!(
            expected,
            Identifier::parse("https://doi.org/10.5555/abcdefg")
        );
        assert_eq!(expected, Identifier::parse("10.5555/ABCDEFG"));
        assert_eq!(
            expected,
            Identifier::parse("https://doi.org/10.5555/ABCDEFG")
        );
    }

    /// See https://en.wikipedia.org/wiki/Serial_Item_and_Contribution_Identifier
    /// SICIs can contain all manner of interesting characters, including a terminal '#'.
    #[test]
    fn sici() {
        let expected = Identifier::Doi {
            prefix: String::from("10.1002"),
            suffix: String::from("(sici)1099-050x(199823/24)37:3/4<197::aid-hrm2>3.0.co;2-#"),
        };

        assert_eq!(
            expected,
            Identifier::parse("10.1002/(SICI)1099-050X(199823/24)37:3/4<197::AID-HRM2>3.0.CO;2-#",),
            "SICI as a plain DOI should parse. Contains a terminal '#' character."
        );

        assert_eq!(
            expected,
            Identifier::parse("https://doi.org/10.1002%2F%28sici%291099-050x%28199823%2F24%2937%3A3%2F4%3C197%3A%3Aaid-hrm2%3E3.0.co%3B2-%23"),
            "SICI as URL, with every possible character encoded, should parse."
        );

        assert_eq!(
            expected,
            Identifier::parse("https://doi.org/10.1002/(sici)1099-050x(199823/24)37:3/4%3C197::aid-hrm2%3E3.0.co;2-%23"),
            "SICI as URL, with only characters recommended by DOI handbook, should parse."
        )
    }

    /// Deal with URL encodings.
    #[test]
    fn mandatory_encodings() {
        // DOI handbook https://www.doi.org/doi-handbook/HTML/encoding-rules-for-urls.html
        // Specifies mandatory encoding of:
        //
        // % %25
        // " %22
        // # %23
        // SPACE %20
        // ? %3F
        //
        assert_eq!(
            Identifier::Doi {
                prefix: String::from("10.5555"),
                // String contains a quote mark mid way through.
                // It's a little clearer to use a raw string than escape the quote.
                suffix: String::from(r##"%"# ?"##),
            },
            Identifier::parse("https://doi.org/10.5555/%25%22%23%20%3F"),
            "Handle mandatory DOI URL encodings."
        );
    }

    ///  DOI handbook specify recommended encoding of
    /// < %3C, > %3E, { %7B, } %7D, ^ %5E, [ %5B, ] %5D
    /// ` %60, | %7C, \ %5C, + %2B
    #[test]
    fn recommended_encodings() {
        let expected = Identifier::Doi {
            prefix: String::from("10.5555"),
            suffix: String::from(r##"<>{}^[]`|\+"##),
        };

        assert_eq!(
            expected,
            Identifier::parse("https://doi.org/10.5555/%3C%3E%7B%7D%5E%5B%5D%60%7C%5C%2B"),
            "Should parse fully encoded URL."
        );

        assert_eq!(
            expected,
            Identifier::parse(r##"10.5555/<>{}^[]`|\+"##),
            "Should parse unencoded plain DOI containing recommended encoded characters."
        );

        // The "+" is encoded.
        assert_eq!(
            expected,
            Identifier::parse(r##"https://doi.org/10.5555/<>{}^[]`|\%2B"##),
            "Should parse encoded URL with some encoded, some not."
        );

        // Alternating encoded, unencoded.
        assert_eq!(
            expected,
            Identifier::parse(r##"https://doi.org/10.5555/<%3E{%7D^[%5D`%7C\%2B"##,),
            "Should parse encoded URL with some encoded, some not"
        );
    }

    /// Test the boundaries of the regexes.
    #[test]
    fn regexes_valid() {
        let examples = vec![
            "10.12345/12345678",
            "10.12345//12345678",
            "10.12345%2f12345678",
            "10.12345%2F12345678",
            "10.1103/physrevlett.103.157203",
            "10.1111/1467%20106478.00146",
            "https://doi.org/10.1002/(sici)1099-050x(199823/24)37:3/4%3C197::aid-hrm2%3E3.0.co;2-%23{",
            "https://doi.org/10.1002/(sici)1099-050x(199823/24)37:3/4%3C197::aid-hrm2%3E3.0.co;2-%23%7C",
            "https://doi.org/10.1675/1524-4695(2003)026[0119:iosoga]2.0.co;2"
        ];

        for example in examples {
            let parsed = Identifier::parse(example);
            match parsed {
                Identifier::Doi {
                    prefix: _,
                    suffix: _,
                } => {}
                _ => assert!(false, "Should parse as a DOI"),
            }
        }
    }
}

#[cfg(test)]
mod doi_parser_negative_tests {
    use super::*;

    /// Some URLs on the doi.org domain aren't DOIs.
    #[test]
    fn non_dois() {
        let url1 = "https://www.doi.org/the-identifier/what-is-a-doi/";
        assert_eq!(
            Identifier::Uri {
                value: String::from(url1)
            },
            Identifier::parse(url1),
            "URL on doi.org should not be parsed as DOI if it doesn't have the syntax."
        );

        let url2 = "https://www.doi.org/the-identifier/what-is-a-doi/";
        assert_eq!(
            Identifier::Uri {
                value: String::from(url2)
            },
            Identifier::parse(url2),
            "https://www.doi.org/images/logos/header_logo_cropped.svg"
        )
    }

    /// Some landing pages contain DOI strings, but should not be considered to be DOIs.
    #[test]
    fn landing_page() {
        let plos = "https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0190046";
        assert_eq!(
            Identifier::Uri {
                value: String::from(plos)
            },
            Identifier::parse(plos),
            "Landing page is not a DOI."
        );

        let wiley = "https://onlinelibrary.wiley.com/doi/10.1111/j.1751-0813.2010.00564.x";
        assert_eq!(
            Identifier::Uri {
                value: String::from(wiley)
            },
            Identifier::parse(wiley),
            "Landing page is not a DOI."
        );
    }

    /// Test the boundaries of the regexes for negative cases.
    #[test]
    fn regexes_invalid() {
        let examples = vec![
            "https://doi.org/1012345/12345678",
            "1012345/12345678",
            "10.12345%212345678",
            "10 12345/12345678",
            "10/12345/12345678",
            "101067",
            "10-092322",
            " 10.12345/12345678",
            "/10.12345/12345678",
            "-10.12345/12345678",
            "110.12345/12345678",
            "a10.12345/12345678",
        ];

        for example in examples {
            let parsed = Identifier::parse(example);
            if let Identifier::Doi {
                prefix: _,
                suffix: _,
            } = parsed
            {
                assert!(false, "Should not parse {} as a DOI", &example)
            }
        }
    }
}

/// Tests for the end-to-end behaviour of the parser and then conversion back to URI.
#[cfg(test)]
mod doi_end_to_end_tests {
    use super::*;

    #[test]
    fn parse_simple() {
        assert_eq!(
            "https://doi.org/10.5555/12345678",
            Identifier::parse("10.5555/12345678").to_uri().unwrap()
        );
    }

    /// The full test for parsing encoded URLs in prior test cases.
    /// Test that a URL in the recommended format is end-tripped to be identical.
    #[test]
    fn normalise_url() {
        // Correctly encoded URL.
        let correct = "https://doi.org/10.5555/%3C%3E%7B%7D%5E%5B%5D%60%7C%5C%2B";

        assert_eq!(correct, Identifier::parse(correct).to_uri().unwrap());
    }
}
