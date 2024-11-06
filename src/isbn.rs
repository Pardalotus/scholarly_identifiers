/// ISBN
/// International Standard Book Identifier
///
/// There's a good explanation on [the Wikipedia
/// page](https://en.wikipedia.org/wiki/ISBN). A logical ISBN can be expressed
/// in either 10 or 13 digits. The parser recognises both, but represents them
/// internally in 13-digit form as it's the superset of what can be recorded.
///
/// ISBNs can optionally be formatted with hyphens. These are removed upon
/// parsing.
use crate::identifiers::{Identifier, IdentifierParseInput};

/// Weights of the numbers 0 to 9 for 10-digit validation.
const TEN_DIGIT_WEIGHTS: &[u32] = &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

/// Weights of the numbers 0 to 12 for 13-digit validation.
const THIRTEEN_DIGIT_WEIGHTS: &[u32] = &[1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1];

/// Try to parse a 10 or 13 digit ISBN. Return the digits normalized to 13
/// digits. This enables the resulting value to be compared against another
/// ISBN, whether it was expressed in 10 or 13 digit form.
pub(crate) fn try_parse(input: &IdentifierParseInput) -> Option<Identifier> {
    let upcase = &input.raw.to_uppercase();
    let less_prefix = upcase.strip_prefix("URN:ISBN:").unwrap_or(&input.raw);

    if let Some(digits) = str_to_digits(&less_prefix) {
        if validate_10_digit(&digits) {
            let as_thirteen = ten_digit_to_thirteen_digit(&digits);
            Some(Identifier::Isbn(digits_to_str(&as_thirteen)))
        } else if validate_13_digit(&digits) {
            Some(Identifier::Isbn(digits_to_str(&digits)))
        } else {
            None
        }
    } else {
        None
    }
}

/// Convert an ISBN to a URN URI.
/// Follows <https://www.iana.org/assignments/urn-formal/isbn>.
pub fn to_uri(input: &Identifier) -> Option<String> {
    match input {
        Identifier::Isbn(ref value) => Some(format!("urn:isbn:{}", value)),
        _ => None,
    }
}

/// Encode an ISBN as a stable string.
/// Will always return a String if an ISBN type is supplied.
pub(crate) fn to_stable_string(input: &Identifier) -> Option<String> {
    match input {
        Identifier::Isbn(ref value) => Some(value.clone()),
        _ => None,
    }
}

/// Return vector of integers for 10 or 13 sized ISBN.
/// If any invalid digits are found, return None.
fn str_to_digits(input: &str) -> Option<Vec<u32>> {
    let chars = Vec::from_iter(input.chars());

    let bad = chars
        .iter()
        .any(|x| !matches!(x, '0'..='9' | 'X' | 'x' | ' ' | '-'));

    if bad {
        return None;
    }

    // Flatten unboxes Options and removes nulls.
    let digits = chars.iter().filter_map(|x| match x {
        '0'..='9' => x.to_digit(10),
        'X' | 'x' => Some(10),
        _ => None,
    });

    Some(Vec::from_iter(digits))
}

fn digits_to_str(input: &[u32]) -> String {
    String::from_iter(input.iter().filter_map(|x| match x {
        0..=9 => char::from_digit(*x, 10),
        10 => Some('X'),
        _ => None,
    }))
}

/// Generate checksum for 10 digit ISBN.
fn generate_10_digit_checksum(digits: &[u32]) -> u32 {
    let mut expected_checksum = 0;
    // Exclude check digit from sum
    for i in 0..9 {
        expected_checksum += digits[i] * TEN_DIGIT_WEIGHTS[i]
    }

    11 - (expected_checksum % 11)
}

// Validate a candidate 10 digit ISBN.
// String must be composed only of digits and 'X'.
fn validate_10_digit(digits: &[u32]) -> bool {
    if digits.len() != 10 {
        return false;
    }

    generate_10_digit_checksum(digits) == digits[9]
}

/// Generate a checksum for a 13 digit ISBN from the first 12 digits.
/// Accept whole ISBN including check digit.
/// If validates, return the first digits.
fn generate_13_digit_checksum(digits: &[u32]) -> u32 {
    let mut result = 0;
    // Don't include check digit
    for i in 0..12 {
        result += digits[i] * THIRTEEN_DIGIT_WEIGHTS[i];
    }

    10 - (result % 10)
}

/// Validate and normalise 13 digit ISBN.
/// Return None if invalid, or the normalised value as a string, including the check digit.
fn validate_13_digit(digits: &[u32]) -> bool {
    if digits.len() != 13 {
        return false;
    }
    let expected_checksum = generate_13_digit_checksum(digits);
    expected_checksum == digits[12]
}

/// Convert 10 digit ISBN to 13 digit by prepending 978 and recalculating the check digit.
fn ten_digit_to_thirteen_digit(digits: &[u32]) -> Vec<u32> {
    let mut new_isbn = vec![9, 7, 8];
    // Don't take the check digit.
    new_isbn.extend(digits.iter().take(9));

    let check = generate_13_digit_checksum(&new_isbn);
    new_isbn.push(check);

    new_isbn
}

#[cfg(test)]
mod isbn_parser_tests {
    use crate::identifiers::Identifier;

    /// Correct 10 digit ISBNs are converted to 13-digit ones, with correct check digit.
    #[test]
    fn simple_10() {
        let examples = [("0306406152", "9780306406157")];

        for example in examples.iter() {
            let result = Identifier::parse(example.0);
            assert_eq!(result, Identifier::Isbn(String::from(example.1)));
        }
    }

    /// Correct 13 digit ISBNs are kept as-is.
    #[test]
    fn simple_13() {
        let examples = [
            "9780306406157",
            "9781566199094",
            "9780123456472",
            "9781413304541",
        ];

        for example in examples.iter() {
            let result = Identifier::parse(example);

            assert_eq!(result, Identifier::Isbn(String::from(*example)));
        }
    }

    #[test]
    fn hyphens_10() {
        let examples = [
            "0306406152",
            "0-306406152",
            "03-06406152",
            "030-6406152",
            "0306-406152",
            "0-3064-06152",
            "03-0640-6152",
            "030-6406-152",
            "0306-4061-52",
            "03064-0615-2",
        ];

        for example in examples.iter() {
            let result = Identifier::parse(example);
            assert_eq!(result, Identifier::Isbn(String::from("9780306406157")));
        }
    }

    #[test]
    fn hyphens_13() {
        let examples = [
            "9-780306406157",
            "97-80306406157",
            "978-0306406157",
            "9780-306406157",
            "97803-06406157",
            "97-8030-6406157",
            "978-0306-406157",
            "9780-3064-06157",
            "97803-0640-6157",
        ];

        for example in examples.iter() {
            let result = Identifier::parse(example);

            assert_eq!(result, Identifier::Isbn(String::from("9780306406157")));
        }
    }

    /// Bad checksums are not recognised as 10 digit ISBNs.
    #[test]
    fn bad_10() {
        let examples = ["0306406150"];

        for example in examples.iter() {
            let result = Identifier::parse(example);
            assert_eq!(result, Identifier::Uri(String::from(*example)));
        }
    }

    /// Bad checksums are not recognised for 13 digit ISBNs.
    #[test]
    fn bad_13() {
        let examples = [
            "9780306406157",
            "9781566199094",
            "9780123456472",
            "9781413304541",
        ];

        for example in examples.iter() {
            let result = Identifier::parse(example);

            assert_eq!(result, Identifier::Isbn(String::from(*example)));
        }
    }
}
