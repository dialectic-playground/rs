//! Entry point of `dp-hello`: hands the raw command line and the standard streams to
//! [`dp_hello::run`] and turns its result into the process exit status.

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments = dp_hello::arguments::from_os_args(std::env::args_os().skip(1));
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let status = dp_hello::run(&arguments, &mut stdout, &mut stderr);
    // Errors were already handled by `run`; flushing here only completes any buffered bytes.
    let _ = stdout.flush();
    ExitCode::from(status)
}
