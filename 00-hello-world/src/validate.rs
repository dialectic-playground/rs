//! Validation of parsed values (FR-03, FR-05, FR-06, FR-09).
//!
//! Only the winning (last) value of each option is checked (FR-06).

use crate::cli::{OptionValue, ParsedArgs};
use crate::diagnostics::UsageError;

/// Name greeted when `--name` is absent (SRS §1.3).
pub const DEFAULT_NAME: &str = "World";
/// Greeting lines printed when `--count` is absent (SRS §1.3).
pub const DEFAULT_COUNT: u16 = 1;
/// Smallest accepted `--count` (FR-04).
pub const MIN_COUNT: u16 = 1;
/// Largest accepted `--count` (FR-04).
pub const MAX_COUNT: u16 = 1000;

/// A validated request: a non-empty name and a count in `MIN_COUNT..=MAX_COUNT`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub name: Vec<u8>,
    pub count: u16,
}

/// Turns parsed arguments into a [`Config`], checking operands, then the name, then the
/// count.
pub fn validate(parsed: ParsedArgs) -> Result<Config, UsageError> {
    if let Some(operand) = parsed.operands.into_iter().next() {
        return Err(UsageError::ExtraOperand(operand));
    }
    let name = match parsed.name {
        Some(name) => validate_name(name)?,
        None => DEFAULT_NAME.as_bytes().to_vec(),
    };
    let count = match parsed.count {
        Some(count) => validate_count(count)?,
        None => DEFAULT_COUNT,
    };
    Ok(Config { name, count })
}

/// Accepts any non-empty name, verbatim (FR-02, FR-03).
pub fn validate_name(name: OptionValue) -> Result<Vec<u8>, UsageError> {
    if name.value.is_empty() {
        return Err(invalid(name));
    }
    Ok(name.value)
}

/// Accepts ASCII decimal digits only (leading zeros allowed) with a value in
/// `MIN_COUNT..=MAX_COUNT` (FR-04, FR-05). Arbitrarily long inputs cannot overflow.
pub fn validate_count(count: OptionValue) -> Result<u16, UsageError> {
    match parse_count(&count.value) {
        Some(value) => Ok(value),
        None => Err(invalid(count)),
    }
}

fn parse_count(digits: &[u8]) -> Option<u16> {
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let mut value: u16 = 0;
    for digit in digits {
        value = value * 10 + u16::from(digit - b'0');
        if value > MAX_COUNT {
            return None;
        }
    }
    (value >= MIN_COUNT).then_some(value)
}

fn invalid(option_value: OptionValue) -> UsageError {
    UsageError::InvalidArgument {
        value: option_value.value,
        option: option_value.option,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn option(value: &str, option: &'static str) -> OptionValue {
        OptionValue {
            value: value.as_bytes().to_vec(),
            option,
        }
    }

    fn count(value: &str) -> Result<u16, UsageError> {
        validate_count(option(value, "--count"))
    }

    fn invalid_count(value: &str) -> Result<u16, UsageError> {
        Err(UsageError::InvalidArgument {
            value: value.as_bytes().to_vec(),
            option: "--count",
        })
    }

    #[test]
    fn validate_defaults() {
        let config = validate(ParsedArgs::default());
        assert_eq!(
            config,
            Ok(Config {
                name: b"World".to_vec(),
                count: 1
            })
        );
    }

    #[test]
    fn validate_count_boundaries() {
        assert_eq!(count("0"), invalid_count("0"));
        assert_eq!(count("1"), Ok(1));
        assert_eq!(count("1000"), Ok(1000));
        assert_eq!(count("1001"), invalid_count("1001"));
        assert_eq!(
            count("99999999999999999999999"),
            invalid_count("99999999999999999999999")
        );
    }

    #[test]
    fn validate_count_leading_zeros() {
        assert_eq!(count("007"), Ok(7));
        assert_eq!(count("0001000"), Ok(1000));
        assert_eq!(count("000"), invalid_count("000"));
    }

    #[test]
    fn validate_count_non_digits() {
        for text in ["", "abc", "-1", "+1", " 1", "1 ", "1.0", "1e3", "٣", "３"] {
            assert_eq!(count(text), invalid_count(text), "{text:?}");
        }
    }

    #[test]
    fn validate_count_reports_spelling() {
        assert_eq!(
            validate_count(option("x", "-c")),
            Err(UsageError::InvalidArgument {
                value: b"x".to_vec(),
                option: "-c"
            })
        );
    }

    #[test]
    fn validate_empty_name() {
        for spelling in ["-n", "--name"] {
            assert_eq!(
                validate_name(option("", spelling)),
                Err(UsageError::InvalidArgument {
                    value: Vec::new(),
                    option: spelling
                })
            );
        }
        assert_eq!(validate_name(option(" ", "-n")), Ok(b" ".to_vec()));
    }

    #[test]
    fn validate_only_winning_values() {
        let parsed = crate::cli::parse(&[
            b"-c".to_vec(),
            b"0".to_vec(),
            b"-c".to_vec(),
            b"2".to_vec(),
            b"--name=".to_vec(),
            b"-nB".to_vec(),
        ]);
        let Ok(crate::cli::Invocation::Greet(parsed)) = parsed else {
            panic!("expected a greeting");
        };
        assert_eq!(
            validate(parsed),
            Ok(Config {
                name: b"B".to_vec(),
                count: 2
            })
        );
    }

    #[test]
    fn validate_operands_rejected_first() {
        let parsed = ParsedArgs {
            name: Some(option("", "-n")),
            count: None,
            operands: vec![b"first".to_vec(), b"second".to_vec()],
        };
        assert_eq!(
            validate(parsed),
            Err(UsageError::ExtraOperand(b"first".to_vec()))
        );
    }

    #[test]
    fn validate_name_before_count() {
        let parsed = ParsedArgs {
            name: Some(option("", "--name")),
            count: Some(option("0", "-c")),
            operands: Vec::new(),
        };
        assert_eq!(
            validate(parsed),
            Err(UsageError::InvalidArgument {
                value: Vec::new(),
                option: "--name"
            })
        );
    }
}
