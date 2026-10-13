//! Hand-written option parser (CONV-07 … CONV-12, SRS §3.2).
//!
//! The parser only recognizes syntax: it records the last value of each option together with
//! the spelling the user wrote, collects operands, and stops at the first `--help` or
//! `--version`. Checking the values is left to [`crate::validate`].

use crate::diagnostics::UsageError;

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    /// `--help` was reached during the scan (CONV-12).
    Help,
    /// `--version` was reached during the scan (CONV-12).
    Version,
    /// Print the greeting, after validating the parsed values.
    Greet(ParsedArgs),
}

/// An option value as written, with the option spelling used for it (`-n` or `--name`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionValue {
    pub value: Vec<u8>,
    pub option: &'static str,
}

/// The syntactically valid, not yet validated, contents of a command line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedArgs {
    /// The last `-n` / `--name` value (CONV-11), if any.
    pub name: Option<OptionValue>,
    /// The last `-c` / `--count` value (CONV-11), if any.
    pub count: Option<OptionValue>,
    /// Every operand, in order (CONV-09, CONV-10).
    pub operands: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionKind {
    Name,
    Count,
    Help,
    Version,
}

/// One entry of the option table.
struct OptionSpec {
    kind: OptionKind,
    short: Option<char>,
    short_spelling: &'static str,
    long_spelling: &'static str,
}

impl OptionSpec {
    fn takes_value(&self) -> bool {
        matches!(self.kind, OptionKind::Name | OptionKind::Count)
    }
}

const OPTIONS: [OptionSpec; 4] = [
    OptionSpec {
        kind: OptionKind::Name,
        short: Some('n'),
        short_spelling: "-n",
        long_spelling: "--name",
    },
    OptionSpec {
        kind: OptionKind::Count,
        short: Some('c'),
        short_spelling: "-c",
        long_spelling: "--count",
    },
    OptionSpec {
        kind: OptionKind::Help,
        short: None,
        short_spelling: "",
        long_spelling: "--help",
    },
    OptionSpec {
        kind: OptionKind::Version,
        short: None,
        short_spelling: "",
        long_spelling: "--version",
    },
];

/// Parses the arguments (without the program name), scanning left to right.
///
/// Returns at the first `--help` / `--version` or the first syntax error, whichever comes
/// first (CONV-12). Long options must be spelled in full (CONV-08).
pub fn parse(args: &[Vec<u8>]) -> Result<Invocation, UsageError> {
    let mut parsed = ParsedArgs::default();
    let mut tokens = args.iter();
    while let Some(token) = tokens.next() {
        let token = token.as_slice();
        if token == b"--" {
            parsed.operands.extend(tokens.by_ref().cloned());
        } else if let Some(body) = token.strip_prefix(b"--") {
            let (name, inline_value) = split_long_option(body);
            let spec =
                find_long(name).ok_or_else(|| UsageError::UnrecognizedOption(token.to_vec()))?;
            if !spec.takes_value() {
                if inline_value.is_some() {
                    return Err(UsageError::ArgumentNotAllowed(spec.long_spelling));
                }
                return Ok(action(spec.kind));
            }
            let value = match inline_value {
                Some(value) => value.to_vec(),
                None => tokens
                    .next()
                    .cloned()
                    .ok_or(UsageError::LongOptionRequiresArgument(spec.long_spelling))?,
            };
            store(&mut parsed, spec, value, spec.long_spelling);
        } else if token.len() > 1 && token[0] == b'-' {
            parse_short_cluster(&token[1..], &mut tokens, &mut parsed)?;
        } else {
            parsed.operands.push(token.to_vec());
        }
    }
    Ok(Invocation::Greet(parsed))
}

/// Parses the letters after the `-` of one token (CONV-07). Every dp-hello short option
/// takes a value, so the first known letter consumes the rest of the token or the next one.
fn parse_short_cluster<'a>(
    mut letters: &[u8],
    tokens: &mut impl Iterator<Item = &'a Vec<u8>>,
    parsed: &mut ParsedArgs,
) -> Result<(), UsageError> {
    while !letters.is_empty() {
        let (letter, rest) = split_first_char(letters);
        let spec = find_short(letter).ok_or_else(|| UsageError::InvalidOption(letter.to_vec()))?;
        letters = rest;
        if spec.takes_value() {
            let value = if rest.is_empty() {
                let short = spec.short.unwrap_or_default();
                tokens
                    .next()
                    .cloned()
                    .ok_or(UsageError::ShortOptionRequiresArgument(short))?
            } else {
                rest.to_vec()
            };
            store(parsed, spec, value, spec.short_spelling);
            return Ok(());
        }
    }
    Ok(())
}

/// Records a value; a later occurrence replaces an earlier one (CONV-11).
fn store(parsed: &mut ParsedArgs, spec: &OptionSpec, value: Vec<u8>, option: &'static str) {
    let slot = match spec.kind {
        OptionKind::Name => &mut parsed.name,
        OptionKind::Count => &mut parsed.count,
        OptionKind::Help | OptionKind::Version => return,
    };
    *slot = Some(OptionValue { value, option });
}

fn action(kind: OptionKind) -> Invocation {
    match kind {
        OptionKind::Help => Invocation::Help,
        _ => Invocation::Version,
    }
}

/// Splits `name=value` at the first `=`; the value is `None` when there is no `=`.
fn split_long_option(body: &[u8]) -> (&[u8], Option<&[u8]>) {
    match body.iter().position(|&byte| byte == b'=') {
        Some(index) => (&body[..index], Some(&body[index + 1..])),
        None => (body, None),
    }
}

/// Splits off the first character: one UTF-8 scalar value, or one byte of an invalid
/// sequence, so that `-é` reports `'é'` and invalid input never panics (CONV-24).
fn split_first_char(bytes: &[u8]) -> (&[u8], &[u8]) {
    let length = bytes
        .utf8_chunks()
        .next()
        .map(|chunk| match chunk.valid().chars().next() {
            Some(character) => character.len_utf8(),
            None => 1,
        })
        .unwrap_or(1)
        .min(bytes.len());
    bytes.split_at(length)
}

fn find_long(name: &[u8]) -> Option<&'static OptionSpec> {
    OPTIONS
        .iter()
        .find(|spec| spec.long_spelling.as_bytes()[2..] == *name)
}

fn find_short(letter: &[u8]) -> Option<&'static OptionSpec> {
    OPTIONS.iter().find(|spec| {
        !spec.short_spelling.is_empty() && spec.short_spelling.as_bytes()[1..] == *letter
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(args: &[&str]) -> Result<Invocation, UsageError> {
        let args: Vec<Vec<u8>> = args.iter().map(|arg| arg.as_bytes().to_vec()).collect();
        parse(&args)
    }

    fn greet(args: &[&str]) -> ParsedArgs {
        match parse_strs(args) {
            Ok(Invocation::Greet(parsed)) => parsed,
            other => panic!("expected a greeting for {args:?}, got {other:?}"),
        }
    }

    fn value(value: &str, option: &'static str) -> Option<OptionValue> {
        Some(OptionValue {
            value: value.as_bytes().to_vec(),
            option,
        })
    }

    #[test]
    fn parse_no_arguments() {
        assert_eq!(greet(&[]), ParsedArgs::default());
    }

    #[test]
    fn parse_name_spellings() {
        assert_eq!(greet(&["-n", "Ada"]).name, value("Ada", "-n"));
        assert_eq!(greet(&["-nAda"]).name, value("Ada", "-n"));
        assert_eq!(greet(&["--name", "Ada"]).name, value("Ada", "--name"));
        assert_eq!(greet(&["--name=Ada"]).name, value("Ada", "--name"));
        assert_eq!(greet(&["--name=a=b"]).name, value("a=b", "--name"));
    }

    #[test]
    fn parse_name_may_start_with_dash() {
        assert_eq!(greet(&["--name=-x"]).name, value("-x", "--name"));
        assert_eq!(greet(&["-n-x"]).name, value("-x", "-n"));
        assert_eq!(greet(&["--name", "-x"]).name, value("-x", "--name"));
        assert_eq!(greet(&["-n", "--"]).name, value("--", "-n"));
    }

    #[test]
    fn parse_empty_attached_values() {
        assert_eq!(greet(&["--name="]).name, value("", "--name"));
        assert_eq!(greet(&["-n", ""]).name, value("", "-n"));
    }

    #[test]
    fn parse_bundled_short_options() {
        let parsed = greet(&["-c2", "-nKen"]);
        assert_eq!(parsed.count, value("2", "-c"));
        assert_eq!(parsed.name, value("Ken", "-n"));
        assert_eq!(greet(&["-cn"]).count, value("n", "-c"));
    }

    #[test]
    fn parse_last_occurrence_wins() {
        let parsed = greet(&["-n", "A", "--name=B", "-c", "x", "--count", "3"]);
        assert_eq!(parsed.name, value("B", "--name"));
        assert_eq!(parsed.count, value("3", "--count"));
    }

    #[test]
    fn parse_collects_operands() {
        assert_eq!(
            greet(&["extra", "-n", "A"]).operands,
            vec![b"extra".to_vec()]
        );
        assert_eq!(greet(&["-"]).operands, vec![b"-".to_vec()]);
        assert_eq!(
            greet(&["--", "-n", "x"]).operands,
            vec![b"-n".to_vec(), b"x".to_vec()]
        );
        assert_eq!(greet(&["--", "--help"]).operands, vec![b"--help".to_vec()]);
    }

    #[test]
    fn parse_help_and_version_stop_the_scan() {
        assert_eq!(parse_strs(&["--help", "extra", "-x"]), Ok(Invocation::Help));
        assert_eq!(
            parse_strs(&["--version", "--help"]),
            Ok(Invocation::Version)
        );
        assert_eq!(parse_strs(&["-c", "0", "--help"]), Ok(Invocation::Help));
        assert_eq!(parse_strs(&["extra", "--version"]), Ok(Invocation::Version));
    }

    #[test]
    fn parse_error_before_help_wins() {
        assert_eq!(
            parse_strs(&["-x", "--help"]),
            Err(UsageError::InvalidOption(b"x".to_vec()))
        );
    }

    #[test]
    fn parse_unknown_short_option() {
        assert_eq!(
            parse_strs(&["-x"]),
            Err(UsageError::InvalidOption(b"x".to_vec()))
        );
        assert_eq!(
            parse_strs(&["-é"]),
            Err(UsageError::InvalidOption("é".as_bytes().to_vec()))
        );
        assert_eq!(
            parse_strs(&["-xn"]),
            Err(UsageError::InvalidOption(b"x".to_vec()))
        );
    }

    #[test]
    fn parse_unknown_short_option_with_invalid_utf8() {
        let args = vec![vec![b'-', 0xff, b'n']];
        assert_eq!(parse(&args), Err(UsageError::InvalidOption(vec![0xff])));
    }

    #[test]
    fn parse_unrecognized_long_options() {
        for token in [
            "--nam",
            "--nme",
            "--names",
            "--nam=Ada",
            "--NAME",
            "--=x",
            "--h",
        ] {
            assert_eq!(
                parse_strs(&[token, "Ada"]),
                Err(UsageError::UnrecognizedOption(token.as_bytes().to_vec())),
                "{token}"
            );
        }
    }

    #[test]
    fn parse_missing_arguments() {
        assert_eq!(
            parse_strs(&["-n"]),
            Err(UsageError::ShortOptionRequiresArgument('n'))
        );
        assert_eq!(
            parse_strs(&["-c"]),
            Err(UsageError::ShortOptionRequiresArgument('c'))
        );
        assert_eq!(
            parse_strs(&["--name"]),
            Err(UsageError::LongOptionRequiresArgument("--name"))
        );
        assert_eq!(
            parse_strs(&["--count"]),
            Err(UsageError::LongOptionRequiresArgument("--count"))
        );
    }

    #[test]
    fn parse_help_and_version_take_no_argument() {
        assert_eq!(
            parse_strs(&["--help="]),
            Err(UsageError::ArgumentNotAllowed("--help"))
        );
        assert_eq!(
            parse_strs(&["--version=1"]),
            Err(UsageError::ArgumentNotAllowed("--version"))
        );
    }
}
