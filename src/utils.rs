use std::ffi::OsStr;
use std::num::NonZero;
use std::process::Command;

/// Returns a shell-like string representation of a [`std::process::Command`].
///
/// The command is rendered as it would typically appear in a POSIX shell:
/// - The program and all arguments are joined with spaces
/// - Arguments containing special characters are safely single-quoted
/// - Existing single quotes are escaped in a shell-compatible way
///
/// This function is intended for diagnostics, logging, and error reporting.
/// The returned string is **not** guaranteed to be a lossless or executable
/// reconstruction of the original command, but it is suitable for display.
///
/// # Parameters
///
/// - `cmd`: The command to render.
///
/// # Returns
///
/// A human-readable string representation of the command.
pub fn display_command(cmd: &Command) -> String {
    let mut parts = Vec::new();

    parts.push(quote(cmd.get_program()));

    for arg in cmd.get_args() {
        parts.push(quote(arg));
    }

    parts.join(" ")
}

fn quote(s: &OsStr) -> String {
    let s = s.to_string_lossy();

    if s.is_empty() {
        return "''".to_string();
    }

    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '='))
    {
        return s.to_string();
    }

    let escaped = s.replace('\'', r"'\''");

    format!("'{}'", escaped)
}

/// Returns the maximum number of threads that should be used for parallel work.
///
/// This function queries the system for the available level of parallelism
/// and guarantees a non-zero result. If the system query fails, it falls back
/// to a single-threaded configuration.
///
/// This is typically used to size thread pools or limit parallel compilation.
///
/// # Returns
///
/// A non-zero number representing the maximum recommended parallel threads.
pub fn get_number_of_max_parallel_threads() -> NonZero<usize> {
    match std::thread::available_parallelism() {
        Ok(threads) => threads,
        Err(_) => NonZero::new(1usize).unwrap(),
    }
}
