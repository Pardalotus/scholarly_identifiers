//! ORCID
//! See <https://orcid.org>
//!
//! Contributor ID, used to identifier authors.

use crate::identifiers::{Identifier, IdentifierParseInput};
use lazy_static::lazy_static;
use regex::Regex;

/// Host expressed upper case to avoid multiple case conversions.
const HOST: &str = "orcid.org";

lazy_static! {

    // Match an ORCID id.
    static ref ORCID_RE: Regex = Regex::new(r"^(\d{4})-(\d{4})-(\d{4})-(\d{3})([\dX])$").unwrap();
}

/// Parse an input string as an ORCID id.
pub(crate) fn try_parse(input: &IdentifierParseInput) -> Option<Identifier> {
    if let Some(path) = input.path_no_slash_uppercase() {
        match input.host_lowercase() {
            Some(x) if x.eq(HOST) => {
                if validate_check_digit(&path) {
                    Some(Identifier::Orcid(path))
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Generate check digit for ORCID ID.
//  See <https://support.orcid.org/hc/en-us/articles/360006897674-Structure-of-the-ORCID-Identifier>
fn generate_check_digit(base_digits: &str) -> Option<String> {
    let mut total = 0;
    for digit in base_digits.chars() {
        // Calling function should guard against sending non-digits.#
        // But if it's not possible to parse, return None.
        if let Some(value) = digit.to_digit(10) {
            total = (total + value) * 2
        } else {
            return None;
        }
    }

    let remainder = total % 11;
    let result = (12 - remainder) % 11;

    Some(if result == 10 {
        String::from("X")
    } else {
        result.to_string()
    })
}

fn validate_check_digit(orcid_id: &str) -> bool {
    // Check the right length and syntax, also extract numbers and check digit.
    match ORCID_RE.captures(orcid_id) {
        Some(groups) => {
            // Ensure that all 4 groups, plus implicit group 0, were found.
            // Guards the unwraps.
            if groups.len() < 5 {
                false
            } else {
                // Groups
                let group_1 = groups.get(1).unwrap().as_str();
                let group_2 = groups.get(2).unwrap().as_str();
                let group_3 = groups.get(3).unwrap().as_str();

                // Group 4 excludes the check digit.
                let group_4 = groups.get(4).unwrap().as_str();

                let digits = format!("{}{}{}{}", group_1, group_2, group_3, group_4);

                let expected_check = generate_check_digit(&digits);

                let check = groups.get(5).unwrap().as_str();

                matches!(expected_check, Some(ref value) if value == check)
            }
        }
        _ => false,
    }
}

/// Encode an ORCID ID as a URI
/// Will always return a result if an ORCID type is supplied.
pub fn to_uri(input: &Identifier) -> Option<String> {
    match input {
        Identifier::Orcid(value) => Some(format!("https://orcid.org/{}", value)),
        _ => None,
    }
}

/// Encode an ORCID ID as a stable string in the recommended format.
/// https://support.orcid.org/hc/en-us/articles/360006897674-Structure-of-the-ORCID-Identifier
/// Will always return a String if an ORCID type is supplied.
pub(crate) fn to_stable_string(input: &Identifier) -> Option<String> {
    to_uri(input)
}

#[cfg(test)]
mod orcid_parser_tests {
    use super::*;

    #[test]
    fn simple_orcid() {
        assert_eq!(
            Identifier::Orcid(String::from("0000-0002-1028-6941")),
            Identifier::parse("https://orcid.org/0000-0002-1028-6941")
        );
    }

    #[test]
    fn good_checksums() {
        assert_eq!(
            Identifier::Orcid(String::from("0000-0002-1694-233X")),
            Identifier::parse("https://orcid.org/0000-0002-1694-233X")
        );

        assert_eq!(
            Identifier::Orcid(String::from("0000-0001-5109-3700")),
            Identifier::parse("https://orcid.org/0000-0001-5109-3700")
        );
    }

    /// Swapped checksums from the two examples in [`good_checksum`] to make two invalid IDs.
    #[test]
    fn bad_checksums() {
        assert_eq!(
            Identifier::Uri(String::from("https://orcid.org/0000-0002-1694-2330")),
            Identifier::parse("https://orcid.org/0000-0002-1694-2330"),
            "Bad checksum should parse as a URI not an ORCID ID."
        );

        assert_eq!(
            Identifier::Uri(String::from("https://orcid.org/0000-0001-5109-370X")),
            Identifier::parse("https://orcid.org/0000-0001-5109-370X"),
            "Bad checksum should parse as a URI not an ORCID ID."
        );
    }

    #[test]
    fn case() {
        let expected = Identifier::Orcid(String::from("0000-0002-1694-233X"));

        assert_eq!(
            expected,
            Identifier::parse("https://orcid.org/0000-0002-1694-233x"),
            "Lower case ORCID URI should parse."
        );

        assert_eq!(
            expected,
            Identifier::parse("HTTPS://ORCID.ORG/0000-0002-1694-233X"),
            "Upper case ORCID URI should parse."
        );
    }

    #[test]
    fn scheme() {
        let expected = Identifier::Orcid(String::from("0000-0002-1694-233X"));

        assert_eq!(
            expected,
            Identifier::parse("https://orcid.org/0000-0002-1694-233x"),
            "HTTPS ORCID URI should parse."
        );

        assert_eq!(
            expected,
            Identifier::parse("http://orcid.org/0000-0002-1694-233x"),
            "HTTP ORCID URI should parse."
        );
    }
}

/// Tests for the end-to-end behaviour of the parser and then conversion back to URI.
#[cfg(test)]
mod orcid_end_to_end_tests {
    use super::*;

    #[test]
    fn parse_simple() {
        assert_eq!(
            "https://orcid.org/0000-0002-1694-233X",
            Identifier::parse("https://orcid.org/0000-0002-1694-233X")
                .to_uri()
                .unwrap()
        );
    }
}
