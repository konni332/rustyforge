use std::path::Path;
use std::process::Command;


pub fn is_valid_cflag(flag: &str) -> bool {
    let forbidden = ["-Wall", "-Wextra", "-DDEBUG", "-DNDEBUG"];

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

pub fn add_debug_cflags(cmd: &mut Command) {
    cmd.arg("-g").arg("-O0").arg("-Wall").arg("-Wextra").arg("-DDEBUG");
}

pub fn add_release_cflags(cmd: &mut Command) {
    cmd.arg("-O3").arg("-Wall").arg("-Wextra").arg("-DRELEASE").arg("-DNDEBUG");
}