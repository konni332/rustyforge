use std::path::Path;
use std::process::Command;
use crate::config::{CompilerKind, Config};
use crossterm::style::{Stylize};
use crate::arguments;
use crate::arguments::Command::*;
use crate::arguments::{CleanOptions, BuildOptions};

pub fn check_compiler(cfg: &mut Config) {
    let comp = cfg.compiler;
    let available = match comp {
        CompilerKind::GCC => is_gcc_available(),
        CompilerKind::Clang => is_clang_available(),
        CompilerKind::MSVC => false, // TODO: Implement MSVC support
    };
    if !available {
        eprintln!("Compiler is not available: {}", comp.to_string().red());
        println!("Trying fallback to default compiler: gcc");
        if is_gcc_available() {
            cfg.compiler = CompilerKind::GCC;
        } else {
            eprintln!("[{}]", "Fallback compiler not found!\nAborting...".red());
            std::process::exit(1);
        }   
    }  
}

fn is_clang_available() -> bool {
    Command::new("clang")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}


fn is_gcc_available() -> bool {
    Command::new("gcc")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn is_valid_cflag(flag: &str) -> bool {
    let forbidden = ["-Wall", "-Wextra", "-DDEBUG", "-DNDEBUG", "-DRELEASE"];

    if forbidden.contains(&flag) {
        eprintln!("Warning: Flag '{}' is handled internally and should not be set explicitly.", flag);
        return false;
    }

    let valid_flags = [
        "-O0", "-O1", "-O2", "-O3", "-Os", "-Ofast",
        "-g", "-ggdb", "-g3",
        "-Werror", "-Wpedantic", "-Wshadow", "-Wconversion", "-Wformat", "-Wunused",
        "-Wsign-compare", "-Wfloat-equal",
        "-I", "-isystem", "-D",
        "-std=", "-fPIC", "-fvisibility=", "-march=", "-mtune=",
        "-pthread", "-pipe", "-c", "-S",
    ];

    for valid in valid_flags {
        if valid.ends_with('=') || valid == "-I" || valid == "-D" || valid == "-isystem" {
            if flag.starts_with(valid) {
                return true;
            }
        } else if flag == valid {
            return true;
        }
    }

    eprintln!("Invalid compiler flag '{}': Not recognized as valid gcc C compiler flag.", flag);
    false
}

pub fn is_valid_ldflag(flag: &str) -> bool {
    let valid_ldflags = [
        "-L",      // Library path
        "-l",      // Link library
        "-shared", // Shared library
        "-static", // Static linking
        "-pthread",
        "-Wl,",    // Pass option to linker
        "-rdynamic",
        "-pie",
        "-fPIC",
    ];

    for valid in valid_ldflags {
        if valid.ends_with(',') {
            if flag.starts_with(valid) {
                return true;
            }
        } else if flag == valid || flag.starts_with(valid) {
            return true;
        }
    }

    eprintln!("Invalid linker flag '{}': Not recognized as valid gcc linker flag.", flag);
    false
}

pub fn format_command(cmd: &Command) -> (String, Vec<String>) {
    let program = cmd.get_program().to_string_lossy().to_string();
    let args = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect();
    
    (program, args)
}

pub fn strip_cwd(arg: &str, cwd: &Path) -> String {
    let arg_path = Path::new(arg);
    if arg_path.is_absolute() && arg_path.starts_with(cwd) {
        match arg_path.strip_prefix(cwd) { 
            Ok(stripped) => stripped.to_string_lossy().to_string(),
            Err(_) => arg.to_string(),
        }
    }
    else { 
        arg.to_string()
    }
}

// TODO: Change name of subcommands
pub fn add_build_flags(command: &arguments::Command, cmd: &mut Command) {
    match command { 
        Run(opt) => {
            if opt.debug {
                add_debug_cflags(cmd);
            } 
            else {
                add_release_cflags(cmd);
            }
        }
        Build(opt) => {
            if opt.debug {
                add_debug_cflags(cmd);
            } 
            else {
                add_release_cflags(cmd);
            }
        }
        Rebuild(opt) => {
            if opt.debug {
                add_debug_cflags(cmd);
            } 
            else {
                add_release_cflags(cmd);
            }
        }
        _ => {}
    }
}

pub fn add_debug_cflags(cmd: &mut Command) {
    cmd.arg("-g").arg("-O0").arg("-Wall").arg("-Wextra").arg("-DDEBUG");
}

pub fn add_release_cflags(cmd: &mut Command) {
    cmd.arg("-O3").arg("-Wall").arg("-Wextra").arg("-DRELEASE").arg("-DNDEBUG");
}

pub fn format_lib_name(name: &mut String){
    let new_name = format!("lib{}.a", name);
    *name = new_name;
}

pub fn format_shared_lib_name(name: &mut String){
    #[cfg(target_os = "windows")]
    let new_name = format!("lib{}.dll", name);
    #[cfg(target_os = "linux")]
    let new_name = format!("lib{}.so", name);
    *name = new_name;
}

pub fn derive_clean_options(build_opt: &BuildOptions) -> CleanOptions {
    CleanOptions {
        debug: build_opt.debug,
        release: build_opt.release,
        libs: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_exact_flags() {
        let flags = [
            "-O2", "-O3", "-g", "-Werror", "-pthread", "-pipe", "-c", "-S"
        ];
        for flag in flags {
            assert!(is_valid_cflag(flag), "Expected '{}' to be valid", flag);
        }
    }

    #[test]
    fn test_valid_prefix_flags() {
        let flags = [
            "-I/usr/include",
            "-isystem/usr/local/include",
            "-DDEBUG_MODE",
            "-std=c99",
            "-fvisibility=hidden",
            "-march=native",
            "-mtune=core2"
        ];
        for flag in flags {
            assert!(is_valid_cflag(flag), "Expected '{}' to be valid", flag);
        }
    }

    #[test]
    fn test_forbidden_flags() {
        let forbidden = ["-Wall", "-Wextra", "-DDEBUG", "-DNDEBUG"];
        for flag in forbidden {
            assert!(!is_valid_cflag(flag), "Expected '{}' to be forbidden", flag);
        }
    }

    #[test]
    fn test_invalid_flags() {
        let invalid = ["-O4", "-Wbanana", "-funroll-loops", "--weirdflag"];
        for flag in invalid {
            assert!(!is_valid_cflag(flag), "Expected '{}' to be invalid", flag);
        }
    }
    #[test]
    fn test_valid_ldflags_exact() {
        let flags = [
            "-shared",
            "-static",
            "-pthread",
            "-rdynamic",
            "-pie",
            "-fPIC",
        ];
        for flag in flags {
            assert!(is_valid_ldflag(flag), "Expected '{}' to be valid", flag);
        }
    }

    #[test]
    fn test_valid_ldflags_prefix() {
        let flags = [
            "-L/usr/lib",
            "-lssl",
            "-lcrypto",
            "-Wl,--as-needed",
        ];
        for flag in flags {
            assert!(is_valid_ldflag(flag), "Expected '{}' to be valid", flag);
        }
    }

    #[test]
    fn test_invalid_ldflags() {
        let flags = [
            "-Zweird",
            "-Xlinker",
            "-unknownflag",
        ];
        for flag in flags {
            assert!(!is_valid_ldflag(flag), "Expected '{}' to be invalid", flag);
        }
    }
}
