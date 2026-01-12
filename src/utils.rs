use std::ffi::OsStr;
use std::num::NonZero;
use std::process::Command;

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

pub fn get_number_of_max_parallel_threads() -> NonZero<usize> {
    match std::thread::available_parallelism() {
        Ok(threads) => threads,
        Err(_) => NonZero::new(1usize).unwrap(),
    }
}
