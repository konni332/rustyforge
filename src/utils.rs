


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

