//! Output: writes prepared bytes to a stream (FR-01, CONV-03, CONV-25).
//!
//! Rust's standard streams never translate line endings, so the LF-only bytes reach
//! stdout/stderr unchanged on every platform (Windows included: redirected output is written
//! as raw bytes, and console output is valid UTF-8). The standard library ignores SIGPIPE, so a
//! vanished reader shows up as an `ErrorKind::BrokenPipe` error, which ends the tool quietly.

use std::io::{ErrorKind, Write};

/// The result of writing one block of output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOutcome {
    /// Every byte was written and flushed.
    Written,
    /// The reader went away (EPIPE); stop without a diagnostic (CONV-25).
    ReaderGone,
    /// Any other write error.
    Failed,
}

/// Writes all of `bytes` to `stream` and flushes it.
pub fn write_output(stream: &mut impl Write, bytes: &[u8]) -> WriteOutcome {
    match stream.write_all(bytes).and_then(|()| stream.flush()) {
        Ok(()) => WriteOutcome::Written,
        Err(error) if error.kind() == ErrorKind::BrokenPipe => WriteOutcome::ReaderGone,
        Err(_) => WriteOutcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingWriter(ErrorKind);

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(self.0.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn output_writes_bytes_unchanged() {
        let mut buffer = Vec::new();
        assert_eq!(write_output(&mut buffer, b"a\nb\n"), WriteOutcome::Written);
        assert_eq!(buffer, b"a\nb\n");
    }

    #[test]
    fn output_broken_pipe_is_quiet() {
        let mut writer = FailingWriter(ErrorKind::BrokenPipe);
        assert_eq!(write_output(&mut writer, b"x"), WriteOutcome::ReaderGone);
    }

    #[test]
    fn output_other_errors_fail() {
        let mut writer = FailingWriter(ErrorKind::Other);
        assert_eq!(write_output(&mut writer, b"x"), WriteOutcome::Failed);
    }
}
