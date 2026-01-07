use colored::Colorize;
use std::{path::Path, process::Command, time::Duration};
use verbosio::get_verbosity;

use crate::{config::project::LinkTargetKind, utils::format_duration};

pub fn successfull_compile_msg(file_path: &Path, cmd: &Command, warnings: &[u8]) -> String {
    let warnings = String::from_utf8_lossy(warnings);
    let verbosity = verbosio::get_verbosity!();
    let path = if verbosity > 1 {
        file_path.to_string_lossy()
    } else {
        file_path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or(file_path.to_string_lossy())
    };
    #[cfg(feature = "term-colors")]
    {
        let mut msg = format!("{} [{}]\n", "Compiled".bold().green(), path);
        if verbosity > 0 {
            msg.push_str(&format!(": {}\n", display_command(cmd)));
        }
        if !warnings.is_empty() {
            msg.push_str(&warnings);
            msg.push('\n');
        }
        msg
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let mut msg = format!("{} [{}]\n", "Compiled", path);
        if verbosity > 0 {
            msg.push_str(&format!(": {}\n", display_command(cmd)));
        }
        if !warnings.is_empty() {
            msg.push_str(&warnings);
            msg.push('\n');
        }
        msg
    }
}

pub fn error_compile_msg(file_path: &Path, cmd: &Command, error: &[u8]) -> String {
    let err_msg = String::from_utf8_lossy(error);
    let verbosity = verbosio::get_verbosity!();
    let path = if verbosity > 1 {
        file_path.to_string_lossy()
    } else {
        file_path
            .file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or(file_path.to_string_lossy())
    };

    #[cfg(feature = "term-colors")]
    {
        let mut msg = format!("{} {}:\n{}", "Failed".bold().red(), path, err_msg);
        if verbosity > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        msg
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let mut msg = format!("{} {}:\n{}", "Failed", path, err_msg);
        if verbosity > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        msg
    }
}

pub fn successfull_link_msg(cmd: &Command, link_target_kind: LinkTargetKind) -> String {
    #[cfg(feature = "term-colors")]
    {
        use verbosio::get_verbosity;

        let mut msg = format!("{} {}", "Linked".bold().green(), link_target_kind);
        if get_verbosity!() > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        msg
    }
    #[cfg(not(feature = "term-colors"))]
    {
        use verbosio::get_verbosity;

        let mut msg = format!("{} [{}]", "Linked", link_target_kind);
        if get_verbosity!() > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        msg
    }
}

pub fn error_link_msg(
    cmd: Option<&Command>,
    error: &[u8],
    link_target_kind: LinkTargetKind,
) -> String {
    let err_msg = String::from_utf8_lossy(error);

    #[cfg(feature = "term-colors")]
    {
        use verbosio::get_verbosity;

        let mut msg = format!(
            "{} {}:\n{}",
            "Failed to link".bold().red(),
            link_target_kind,
            err_msg
        );
        if get_verbosity!() > 0
            && let Some(c) = cmd
        {
            msg.push_str(&format!(": {}", display_command(c)));
        }
        msg
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let mut msg = format!("{} {}:\n{}", "Failed to link", link_target_kind, err_msg);
        if get_verbosity!() > 0
            && let Some(c) = cmd
        {
            msg.push_str(&format!(": {}", display_command(c)));
        }
        msg
    }
}

pub fn no_rustyforge_initialized_msg() -> String {
    #[cfg(feature = "term-colors")]
    return format!(
        "{}: no rustyforge project found\n   {}: 'rustyforge init' to create a new project in this directory",
        "Error".bold().red(),
        "try".bold().cyan()
    );
    #[cfg(not(feature = "term-colors"))]
    return format!(
        "Error: no rustyforge project found\n   try: 'rustyforge init' to create a new project in this directory",
    );
}

pub fn rustyforge_initialized_msg() -> String {
    #[cfg(feature = "term-colors")]
    return format!("{} new rustyforge project", "Initialized".bold().green());
    #[cfg(not(feature = "term-colors"))]
    return format!("Initialized new rustyforge project");
}

pub fn rustyforge_new_msg(name: &str) -> String {
    #[cfg(feature = "term-colors")]
    return format!(
        "{} new rustyforge project in: {}",
        "Initialized".bold().green(),
        name
    );
    #[cfg(not(feature = "term-colors"))]
    return format!("Initialized new rustyforge project in: {}", name);
}

pub fn rustyforge_removed_msg() -> String {
    #[cfg(feature = "term-colors")]
    return format!("{} rustyforge project", "Removed".bold().red());
    #[cfg(not(feature = "term-colors"))]
    return format!("Removed rustyforge project");
}

pub fn rustyforge_cleaned_msg() -> String {
    #[cfg(feature = "term-colors")]
    return format!("{} rustyforge project", "Cleaned".bold().green());
    #[cfg(not(feature = "term-colors"))]
    return format!("Cleaned rustyforge project");
}

pub fn discovered_file_msg<P: AsRef<Path>>(path: P) -> String {
    #[cfg(feature = "term-colors")]
    return format!(
        "{}: {}",
        "Discovered file".bold().green(),
        path.as_ref().display()
    );
    #[cfg(not(feature = "term-colors"))]
    return format!("Discovered file: {}", path.as_ref().display());
}

pub fn discovered_dir_msg<P: AsRef<Path>>(path: P) -> String {
    #[cfg(feature = "term-colors")]
    return format!(
        "{}: {}",
        "Discovered dir".bold().green(),
        path.as_ref().display()
    );
    #[cfg(not(feature = "term-colors"))]
    return format!("Discovered dir: {}", path.as_ref().display());
}

pub fn run_exit_code_msg(exit_code: i32) -> String {
    let success = exit_code == 0;
    #[cfg(feature = "term-colors")]
    {
        let exited = if success {
            "Exited".bold().green()
        } else {
            "Exited".bold().red()
        };
        format!("{} with code: {}", exited, exit_code)
    }
    #[cfg(not(feature = "term-colors"))]
    return format!("Exited with code: {}", exit_code);
}

pub fn run_exit_signal_msg() -> String {
    #[cfg(feature = "term-colors")]
    return format!("{} with signal", "Exited".bold().red());
    #[cfg(not(feature = "term-colors"))]
    return format!("Exited with signal");
}

fn shell_escape(s: &std::ffi::OsStr) -> String {
    let s = s.to_string_lossy();
    if s.contains([' ', '"', '\'']) {
        format!("'{}'", s.replace('\'', r"'\''"))
    } else {
        s.into_owned()
    }
}

pub fn display_command(cmd: &Command) -> String {
    let program = shell_escape(cmd.get_program());
    if get_verbosity!() > 1 {
        let args = cmd
            .get_args()
            .map(shell_escape)
            .collect::<Vec<_>>()
            .join(" ");

        format!("{program} {args}")
    } else {
        program
    }
}

pub fn running_command_msg(cmd: &Command) -> String {
    #[cfg(feature = "term-colors")]
    let mut msg = format!("{} ", "Running".bold().green());
    #[cfg(not(feature = "term-colors"))]
    let mut msg = format!("Running ");

    msg.push_str(&format!("  {}", display_command(cmd)));
    msg
}

pub fn finished_linking_msg(d: Duration) -> String {
    #[cfg(feature = "term-colors")]
    return format!(
        "{} [{}]",
        "Linking finished".bold().green(),
        format_duration(d)
    );
    #[cfg(not(feature = "term-colors"))]
    return format!("Linking finished [{}]", format_duration(d));
}

pub fn finished_compilation_msg(d: Duration) -> String {
    #[cfg(feature = "term-colors")]
    return format!(
        "{} [{}]",
        "Compilation finished".bold().green(),
        format_duration(d)
    );
    #[cfg(not(feature = "term-colors"))]
    return format!("Compilation finished [{}]", format_duration(d));
}
