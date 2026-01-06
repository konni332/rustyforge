use colored::Colorize;
use std::{path::Path, process::Command};

use crate::config::project::LinkTargetKind;

pub fn output_successfull_compile(file_path: &Path, cmd: &Command) {
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
        let mut msg = format!("{} {}", "Compiled".bold().green(), path);
        if verbosity > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        println!("{}", msg);
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let mut msg = format!("{} [{}]", "Compiled", path);
        if verbosity > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        println!("{}", msg);
    }
}

pub fn output_error_compile(file_path: &Path, cmd: &Command, error: &[u8]) {
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
        println!("{}", msg);
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let mut msg = format!("{} {}:\n{}", "Failed", path, err_msg);
        if verbosity > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        println!("{}", msg);
    }
}

pub fn output_successfull_link(cmd: &Command, link_target_kind: LinkTargetKind) {
    #[cfg(feature = "term-colors")]
    {
        use verbosio::get_verbosity;

        let mut msg = format!("{} {}", "Linked".bold().green(), link_target_kind);
        if get_verbosity!() > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        println!("{}", msg);
    }
    #[cfg(not(feature = "term-colors"))]
    {
        use verbosio::get_verbosity;

        let mut msg = format!("{} [{}]", "Linked", link_target_kind);
        if get_verbosity!() > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        println!("{}", msg);
    }
}

pub fn output_error_link(cmd: &Command, error: &[u8], link_target_kind: LinkTargetKind) {
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
        if get_verbosity!() > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        println!("{}", msg);
    }
    #[cfg(not(feature = "term-colors"))]
    {
        let mut msg = format!("{} {}:\n{}", "Failed to link", link_target_kind, err_msg);
        if get_verbosity!() > 0 {
            msg.push_str(&format!(": {}", display_command(cmd)));
        }
        println!("{}", msg);
    }
}

pub fn output_no_rustyforge_initialized() {
    #[cfg(feature = "term-colors")]
    println!(
        "{}: no rustyforge project found\n   {}: 'rustyforge init' to create a new project in this directory",
        "Error".bold().red(),
        "try".bold().cyan()
    );
    #[cfg(not(feature = "term-colors"))]
    println!(
        "Error: no rustyforge project found\n   try: 'rustyforge init' to create a new project in this directory",
    );
}

pub fn output_rustyforge_initialized() {
    #[cfg(feature = "term-colors")]
    println!("{} new rustyforge project", "Initialized".bold().green());
    #[cfg(not(feature = "term-colors"))]
    println!("Initialized new rustyforge project");
}

pub fn output_rustyforge_new(name: &str) {
    #[cfg(feature = "term-colors")]
    println!(
        "{} new rustyforge project in: {}",
        "Initialized".bold().green(),
        name
    );
    #[cfg(not(feature = "term-colors"))]
    println!("Initialized new rustyforge project in: {}", name);
}

pub fn output_rustyforge_removed() {
    #[cfg(feature = "term-colors")]
    println!("{} rustyforge project", "Removed".bold().red());
    #[cfg(not(feature = "term-colors"))]
    println!("Removed rustyforge project");
}

pub fn output_rustyforge_cleaned() {
    #[cfg(feature = "term-colors")]
    println!("{} rustyforge project", "Cleaned".bold().green());
    #[cfg(not(feature = "term-colors"))]
    println!("Cleaned rustyforge project");
}

pub fn output_discovered_file<P: AsRef<Path>>(path: P) {
    #[cfg(feature = "term-colors")]
    println!(
        "{}: {}",
        "Discovered file".bold().green(),
        path.as_ref().display()
    );
    #[cfg(not(feature = "term-colors"))]
    println!("Discovered file: {}", path.as_ref().display());
}

pub fn output_discovered_dir<P: AsRef<Path>>(path: P) {
    #[cfg(feature = "term-colors")]
    println!(
        "{}: {}",
        "Discovered dir".bold().green(),
        path.as_ref().display()
    );
    #[cfg(not(feature = "term-colors"))]
    println!("Discovered dir: {}", path.as_ref().display());
}

pub fn output_run_exit_code(exit_code: i32) {
    let success = exit_code == 0;
    #[cfg(feature = "term-colors")]
    {
        let exited = if success {
            "Exited".bold().green()
        } else {
            "Exited".bold().red()
        };
        println!("{} with code: {}", exited, exit_code);
    }
    #[cfg(not(feature = "term-colors"))]
    println!("Exited with code: {}", exit_code);
}

pub fn output_run_exit_signal() {
    #[cfg(feature = "term-colors")]
    println!("{} with signal", "Exited".bold().red());
    #[cfg(not(feature = "term-colors"))]
    println!("Exited with signale");
}

fn shell_escape(s: &std::ffi::OsStr) -> String {
    let s = s.to_string_lossy();
    if s.contains([' ', '"', '\'']) {
        format!("'{}'", s.replace('\'', r"'\''"))
    } else {
        s.into_owned()
    }
}

fn display_command(cmd: &Command) -> String {
    let program = shell_escape(cmd.get_program());

    let args = cmd
        .get_args()
        .map(shell_escape)
        .collect::<Vec<_>>()
        .join(" ");

    format!("{program} {args}")
}
