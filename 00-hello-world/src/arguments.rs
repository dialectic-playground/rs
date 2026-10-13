//! Platform boundary for command-line arguments (CONV-05).
//!
//! The rest of the crate works on the UTF-8 bytes of each argument. On Unix-like systems the
//! argument bytes are taken as they are (they are interpreted as UTF-8 and echoed exactly).
//! On Windows the standard library decodes the UTF-16 command line (`GetCommandLineW`), which
//! is converted to UTF-8 here; an unpaired surrogate, which has no UTF-8 encoding, becomes
//! U+FFFD instead of causing a panic (CONV-24).

use std::ffi::OsString;

/// Converts OS arguments (without the program name) to their UTF-8 bytes.
pub fn from_os_args(args: impl IntoIterator<Item = OsString>) -> Vec<Vec<u8>> {
    args.into_iter().map(to_utf8_bytes).collect()
}

/// Returns the bytes of one argument: verbatim on Unix, UTF-8 from UTF-16 elsewhere.
#[cfg(unix)]
pub fn to_utf8_bytes(arg: OsString) -> Vec<u8> {
    use std::os::unix::ffi::OsStringExt;
    arg.into_vec()
}

/// Returns the bytes of one argument: verbatim on Unix, UTF-8 from UTF-16 elsewhere.
#[cfg(not(unix))]
pub fn to_utf8_bytes(arg: OsString) -> Vec<u8> {
    match arg.into_string() {
        Ok(text) => text.into_bytes(),
        Err(arg) => arg.to_string_lossy().into_owned().into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_keep_utf8_text() {
        let args = vec![OsString::from("-n"), OsString::from("Zoë")];
        assert_eq!(
            from_os_args(args),
            vec![b"-n".to_vec(), "Zoë".as_bytes().to_vec()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn arguments_keep_invalid_utf8_bytes_on_unix() {
        use std::os::unix::ffi::OsStringExt;
        let raw = vec![b'Z', 0xff, b'e'];
        assert_eq!(to_utf8_bytes(OsString::from_vec(raw.clone())), raw);
    }

    #[cfg(windows)]
    #[test]
    fn arguments_replace_unpaired_surrogates_on_windows() {
        use std::os::windows::ffi::OsStringExt;
        let wide = [u16::from(b'Z'), 0xD800, u16::from(b'e')];
        assert_eq!(
            to_utf8_bytes(OsString::from_wide(&wide)),
            "Z\u{FFFD}e".as_bytes()
        );
    }
}
