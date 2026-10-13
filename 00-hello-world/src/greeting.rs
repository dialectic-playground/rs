//! Pure construction of everything `dp-hello` prints on success (FR-01, FR-02, FR-04,
//! FR-07, FR-08, NFR-05). Every line ends in a single LF (CONV-03).

/// The help text, byte-for-byte from the user manual's "Help output" section (CONV-23).
pub const HELP_TEXT: &str = "\
Usage: dp-hello [OPTION]...
Print a greeting to standard output.

Options:
  -n, --name=NAME     greet NAME instead of World
  -c, --count=COUNT   print the greeting COUNT times (1-1000, default 1)
      --help          display this help and exit
      --version       output version information and exit
";

/// The version line (CONV-02).
pub const VERSION_LINE: &str = "dp-hello (dialectic-playground) 1.0.0\n";

/// Returns one greeting line: `Hello, ` + `name` + `!` + LF, with `name` verbatim.
pub fn greeting_line(name: &[u8]) -> Vec<u8> {
    [b"Hello, ".as_slice(), name, b"!\n"].concat()
}

/// Returns `count` greeting lines for `name`, ready to be written in one call.
pub fn greeting_text(name: &[u8], count: u16) -> Vec<u8> {
    greeting_line(name).repeat(usize::from(count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_line_default_name() {
        assert_eq!(greeting_line(b"World"), b"Hello, World!\n");
    }

    #[test]
    fn greeting_line_keeps_name_verbatim() {
        assert_eq!(greeting_line(b"Ada Lovelace"), b"Hello, Ada Lovelace!\n");
        assert_eq!(greeting_line("Zoë".as_bytes()), "Hello, Zoë!\n".as_bytes());
        assert_eq!(greeting_line(b"-x"), b"Hello, -x!\n");
        assert_eq!(greeting_line(b"a\r\nb"), b"Hello, a\r\nb!\n");
    }

    #[test]
    fn greeting_text_repeats_lines() {
        assert_eq!(greeting_text(b"Ken", 1), b"Hello, Ken!\n");
        assert_eq!(
            greeting_text(b"Ken", 3),
            b"Hello, Ken!\nHello, Ken!\nHello, Ken!\n"
        );
        assert_eq!(greeting_text(b"W", 1000).len(), 1000 * b"Hello, W!\n".len());
    }

    #[test]
    fn help_and_version_end_in_lf_only() {
        for text in [HELP_TEXT, VERSION_LINE] {
            assert!(text.ends_with('\n'));
            assert!(!text.contains('\r'));
        }
        assert_eq!(HELP_TEXT.lines().count(), 8);
    }
}
