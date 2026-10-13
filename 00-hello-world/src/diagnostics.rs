//! Usage errors and their exact diagnostics (CONV-13, CONV-14, CONV-15).
//!
//! This is the only place that knows the wording of a usage error.

use crate::TOOL;

/// A usage error. Every variant carries the bytes it must echo, exactly as the user wrote
/// them (CONV-15); `option` fields hold the option as written, e.g. `-n` or `--name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageError {
    /// An unknown short option letter (the UTF-8 bytes of one character).
    InvalidOption(Vec<u8>),
    /// An unknown long option: the full token, including any `=value`.
    UnrecognizedOption(Vec<u8>),
    /// A short option (its letter) given without its required argument.
    ShortOptionRequiresArgument(char),
    /// A long option (e.g. `--name`) given without its required argument.
    LongOptionRequiresArgument(&'static str),
    /// A long option that takes no argument (e.g. `--help`) given `=value`.
    ArgumentNotAllowed(&'static str),
    /// An option value that fails validation.
    InvalidArgument {
        value: Vec<u8>,
        option: &'static str,
    },
    /// The first operand (dp-hello takes none).
    ExtraOperand(Vec<u8>),
}

impl UsageError {
    /// The diagnostic message, without the `dp-hello: ` prefix and the trailing LF.
    pub fn message(&self) -> Vec<u8> {
        match self {
            Self::InvalidOption(letter) => quoted(b"invalid option -- '", letter, b"'"),
            Self::UnrecognizedOption(token) => quoted(b"unrecognized option '", token, b"'"),
            Self::ShortOptionRequiresArgument(letter) => {
                format!("option requires an argument -- '{letter}'").into_bytes()
            }
            Self::LongOptionRequiresArgument(option) => {
                format!("option '{option}' requires an argument").into_bytes()
            }
            Self::ArgumentNotAllowed(option) => {
                format!("option '{option}' doesn't allow an argument").into_bytes()
            }
            Self::InvalidArgument { value, option } => {
                let suffix = format!("' for '{option}'");
                quoted(b"invalid argument '", value, suffix.as_bytes())
            }
            Self::ExtraOperand(operand) => quoted(b"extra operand '", operand, b"'"),
        }
    }

    /// The complete stderr report: the diagnostic line and the CONV-14 hint, LF-terminated.
    pub fn report(&self) -> Vec<u8> {
        let mut report = format!("{TOOL}: ").into_bytes();
        report.extend_from_slice(&self.message());
        report
            .extend_from_slice(format!("\nTry '{TOOL} --help' for more information.\n").as_bytes());
        report
    }
}

/// Concatenates `prefix`, the user-supplied `value`, and `suffix` without escaping.
fn quoted(prefix: &[u8], value: &[u8], suffix: &[u8]) -> Vec<u8> {
    [prefix, value, suffix].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(error: UsageError) -> String {
        String::from_utf8(error.message()).expect("test messages are UTF-8")
    }

    #[test]
    fn diagnostics_match_conv13() {
        assert_eq!(
            message(UsageError::InvalidOption(b"x".to_vec())),
            "invalid option -- 'x'"
        );
        assert_eq!(
            message(UsageError::UnrecognizedOption(b"--nam=Ada".to_vec())),
            "unrecognized option '--nam=Ada'"
        );
        assert_eq!(
            message(UsageError::ShortOptionRequiresArgument('n')),
            "option requires an argument -- 'n'"
        );
        assert_eq!(
            message(UsageError::LongOptionRequiresArgument("--count")),
            "option '--count' requires an argument"
        );
        assert_eq!(
            message(UsageError::ArgumentNotAllowed("--version")),
            "option '--version' doesn't allow an argument"
        );
        assert_eq!(
            message(UsageError::InvalidArgument {
                value: Vec::new(),
                option: "-n"
            }),
            "invalid argument '' for '-n'"
        );
        assert_eq!(
            message(UsageError::ExtraOperand(b"extra".to_vec())),
            "extra operand 'extra'"
        );
    }

    #[test]
    fn diagnostics_report_adds_prefix_and_hint() {
        assert_eq!(
            UsageError::InvalidOption(b"x".to_vec()).report(),
            b"dp-hello: invalid option -- 'x'\nTry 'dp-hello --help' for more information.\n"
        );
    }

    #[test]
    fn diagnostics_echo_values_without_escaping() {
        let value = "Zo\u{eb} 'q'".as_bytes().to_vec();
        let error = UsageError::InvalidArgument {
            value,
            option: "--count",
        };
        assert_eq!(
            message(error),
            "invalid argument 'Zo\u{eb} 'q'' for '--count'"
        );
    }
}
