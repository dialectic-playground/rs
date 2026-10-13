//! `dp-hello`: print a greeting to standard output (curriculum project 00).
//!
//! The crate is split into one module per responsibility (SRS NFR-04):
//!
//! | Module | Responsibility |
//! | :-- | :-- |
//! | [`arguments`] | Platform boundary: OS command-line arguments to UTF-8 bytes (CONV-05) |
//! | [`cli`] | Option parsing: tokens to a [`cli::Invocation`] (CONV-07 … CONV-12) |
//! | [`validate`] | Validation: parsed values to a [`validate::Config`] (FR-03 … FR-06, FR-09) |
//! | [`greeting`] | Greeting construction: pure functions (FR-01, FR-02, FR-04, NFR-05) |
//! | [`diagnostics`] | Usage errors and their exact messages (CONV-13 … CONV-15) |
//! | [`output`] | Writing bytes to stdout/stderr, quietly stopping on a broken pipe (CONV-25) |
//!
//! [`run`] wires them together and is the only place that maps outcomes to exit statuses.

pub mod arguments;
pub mod cli;
pub mod diagnostics;
pub mod greeting;
pub mod output;
pub mod validate;

use std::io::Write;

/// Canonical tool name used in every message (CONV-01).
pub const TOOL: &str = "dp-hello";

/// Exit status for success, help, and version (SRS §3.7).
pub const EXIT_SUCCESS: u8 = 0;
/// Exit status when standard output cannot be written for a reason other than EPIPE.
pub const EXIT_OUTPUT_FAILURE: u8 = 1;
/// Exit status for usage errors (CONV-14).
pub const EXIT_USAGE: u8 = 2;

/// Runs `dp-hello` on `arguments` (the command line without the program name, as UTF-8
/// bytes) and returns the exit status.
///
/// Nothing is written to `stdout` unless the arguments are valid (FR-11): parsing and
/// validation complete before any output is produced.
pub fn run(arguments: &[Vec<u8>], stdout: &mut impl Write, stderr: &mut impl Write) -> u8 {
    let outcome = cli::parse(arguments).and_then(|invocation| match invocation {
        cli::Invocation::Help => Ok(greeting::HELP_TEXT.as_bytes().to_vec()),
        cli::Invocation::Version => Ok(greeting::VERSION_LINE.as_bytes().to_vec()),
        cli::Invocation::Greet(parsed) => validate::validate(parsed)
            .map(|config| greeting::greeting_text(&config.name, config.count)),
    });
    match outcome {
        Ok(text) => match output::write_output(stdout, &text) {
            output::WriteOutcome::Written | output::WriteOutcome::ReaderGone => EXIT_SUCCESS,
            output::WriteOutcome::Failed => EXIT_OUTPUT_FAILURE,
        },
        Err(error) => {
            // A failure to write the diagnostic itself cannot be reported anywhere.
            let _ = output::write_output(stderr, &error.report());
            EXIT_USAGE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invoke(args: &[&str]) -> (Vec<u8>, Vec<u8>, u8) {
        let arguments: Vec<Vec<u8>> = args.iter().map(|arg| arg.as_bytes().to_vec()).collect();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let status = run(&arguments, &mut stdout, &mut stderr);
        (stdout, stderr, status)
    }

    #[test]
    fn run_default_greeting() {
        assert_eq!(invoke(&[]), (b"Hello, World!\n".to_vec(), Vec::new(), 0));
    }

    #[test]
    fn run_repeated_named_greeting() {
        let (stdout, stderr, status) = invoke(&["-c2", "-nZoë"]);
        assert_eq!(stdout, "Hello, Zoë!\nHello, Zoë!\n".as_bytes());
        assert!(stderr.is_empty());
        assert_eq!(status, 0);
    }

    #[test]
    fn run_help_and_version_exit_zero() {
        assert_eq!(
            invoke(&["--help", "extra", "-x"]).0,
            greeting::HELP_TEXT.as_bytes()
        );
        assert_eq!(invoke(&["--version"]).0, greeting::VERSION_LINE.as_bytes());
    }

    #[test]
    fn run_usage_error_writes_only_stderr() {
        let (stdout, stderr, status) = invoke(&["--count", "0"]);
        assert!(stdout.is_empty());
        assert_eq!(
            stderr,
            b"dp-hello: invalid argument '0' for '--count'\n\
              Try 'dp-hello --help' for more information.\n"
        );
        assert_eq!(status, 2);
    }

    #[test]
    fn run_output_failure_is_silent() {
        struct Closed;
        impl Write for Closed {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut stderr = Vec::new();
        assert_eq!(run(&[], &mut Closed, &mut stderr), 0);
        assert!(stderr.is_empty());
    }
}
