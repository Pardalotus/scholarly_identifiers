//! ROR, Research Organisation Registry identifier

use std::collections::HashMap;

use crate::identifiers::{Identifier, IdentifierParseInput};
use lazy_static::lazy_static;
use regex::Regex;

const HOST: &str = "ror.org";

lazy_static! {
    /// Group 1 is the identifier, group 2 is the checksum digit.
    static ref PATH_RE: Regex = Regex::new(r"^(0[a-hj-km-np-tv-z|0-9]{6})([0-9]{2})$").unwrap();

    /// With reference to Douglas Crockford's Base32 implementation.
    /// See <https://www.crockford.com/base32.html>.
    static ref BASE_32_DECODE: HashMap<char, u64> = "0123456789abcdefghjkmnpqrstvwxyz"
        .chars()
        .enumerate()
        .map(|(i, c)| (c, i as u64 ))
        .collect();

}

/// Parse an input string as a ROR id.
pub(crate) fn try_parse(input: &IdentifierParseInput) -> Option<Identifier> {
    if let Some(host) = input.host_lowercase() {
        if host.eq(HOST) {
            if let Some(path) = input.path_no_slash() {
                if validate_check_digit(&path) {
                    Some(Identifier::Ror(path))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn validate_check_digit(path: &str) -> bool {
    match PATH_RE.captures(path) {
        // Only accept the two groups (plus implicit group).
        Some(groups) if groups.len() != 3 => false,
        Some(groups) => {
            // Groups
            let identifier = groups.get(1).unwrap().as_str();
            let check_digit = groups.get(2).unwrap().as_str();

            if let Ok(check_digit_value) = check_digit.parse::<u64>() {
                expected_check_digit(identifier) == check_digit_value
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Generate the check digit.
/// If unexpected characters are found then they are ignored.
fn expected_check_digit(identifier: &str) -> u64 {
    let parsed = identifier.chars().fold(0_u64, |acc, c| {
        // We guarded against unknown digits so it's safe to ignore them.
        (acc * 32) + BASE_32_DECODE.get(&c).unwrap_or(&0)
    });

    98 - ((parsed * 100) % 97)
}

pub(crate) fn to_uri(input: &Identifier) -> Option<String> {
    match input {
        Identifier::Ror(value) => Some(format!("https://ror.org/{}", value)),
        _ => None,
    }
}

#[cfg(test)]
mod ror_parser_tests {
    use crate::identifiers::Identifier;

    #[test]
    fn simple() {
        assert_eq!(
            Identifier::Ror(String::from("02mhbdp94")),
            Identifier::parse("https://ror.org/02mhbdp94"),
            "Simple ROR URI accepted"
        );

        assert_eq!(
            Identifier::Ror(String::from("02twcfp32")),
            Identifier::parse("https://ror.org/02twcfp32"),
            "Simple ROR URI accepted"
        );
    }

    #[test]
    fn uri() {
        assert_eq!(
            Some(String::from("https://ror.org/02mhbdp94")),
            Identifier::parse("https://ror.org/02mhbdp94").to_uri(),
            "ROR URI parse and converts end-to-end"
        );
    }

    #[test]
    fn case_sensitive() {
        // 1-character upcase from previous examples.
        assert_eq!(
            Identifier::Uri(String::from("https://ror.org/02Mhbdp94")),
            Identifier::parse("https://ror.org/02Mhbdp94"),
            "ROR not recognised with upper-case."
        );

        assert_eq!(
            Identifier::Uri(String::from("https://ror.org/02twcfP32")),
            Identifier::parse("https://ror.org/02twcfP32"),
            "Simple ROR URI accepted"
        );
    }

    #[test]
    fn checksum() {
        // Good example 1.
        assert_eq!(
            Identifier::Ror(String::from("02mhbdp94")),
            Identifier::parse("https://ror.org/02mhbdp94"),
        );

        // Digit changed in the identifier.
        assert_eq!(
            Identifier::Uri(String::from("https://ror.org/03mhbdp94")),
            Identifier::parse("https://ror.org/03mhbdp94"),
        );

        // Checksum changed.
        assert_eq!(
            Identifier::Uri(String::from("https://ror.org/02mhbdp99")),
            Identifier::parse("https://ror.org/02mhbdp99"),
        );

        // Good example 2.
        assert_eq!(
            Identifier::Ror(String::from("02twcfp32")),
            Identifier::parse("https://ror.org/02twcfp32"),
            "Simple ROR URI accepted"
        );

        // Digit changed in identifier.
        assert_eq!(
            Identifier::Uri(String::from("https://ror.org/02tw3fp32")),
            Identifier::parse("https://ror.org/02tw3fp32"),
            "Simple ROR URI accepted"
        );

        // Checksum changed.
        assert_eq!(
            Identifier::Uri(String::from("https://ror.org/02twcfp39")),
            Identifier::parse("https://ror.org/02twcfp39"),
            "Simple ROR URI accepted"
        );
    }
}
