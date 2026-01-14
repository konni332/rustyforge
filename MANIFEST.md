# RustyForge.toml Overview

The `RustyForge.toml` file is the main manifest for your RustyForge project. It contains all configuration for building binaries and libraries, setting build profiles, defining features, and excluding files from the build process.  

---

## Package

The `[package]` section describes metadata about your project.

```toml
[package]
name = "example"          # required
version = "0.1.0"         # required
c-edition = "c11"         # required, e.g., c89, c99, c11, c17, c23
cpp-edition = "c++17"     # required, e.g., c++17, c++20, c++latest
authors = ["Your Name"]   # optional
license = "MIT"           # optional
description = "A short description of your project"  # optional
repository = "https://github.com/example/repo"       # optional
```

**Notes:**

- `edition` defines the C standard used when compiling.
- `authors`, `license`, `description`, and `repository` are optional but recommended for documentation and future registry support.

---

## Build

The `[build]` section specifies global build-related settings.

```toml
[build]
ignore = [
    "**/tests/**",   # ignore all files in tests
    "**/docs/**"     # ignore documentation files
]
```

**Notes:**

- `ignore` defines patterns for files or directories to **exclude from source discovery**.
- All C source files (`*.c`) outside of these patterns are automatically discovered.
- This section **does not handle compiler toolchains or flags**, those are handled in profiles or targets.

---

## Lib

The `[lib]` section configures building a library from your source code.

```toml
[lib]
name = "example"          # required
type = "static"           # optional, "static" or "shared", defaults to "static"
ignore = ["src-bin/*"]    # optional, target-specific ignore patterns
flags = ["-pthread"]      # optional, flags applied at **link time**
defines = ["LIB_EXAMPLE"] # optional, target-specific defines (-D)
```

**Notes:**

- `flags` in `[lib]` are **linker flags**, not compiler flags.
- `defines` are translated to `-D<NAME>` and applied at compile time.
- `ignore` here overrides or supplements `[build].ignore` for this target.

---

## Executable

The `[[bin]]` section configures binary targets (executables).

```toml
[[bin]]
name = "example"               # required
entry = "src/main.c"           # required, the entry point of the binary
ignore = ["src/legacy/*"]      # optional, target-specific ignore patterns
flags = ["-lpthread"]          # optional, linker flags only
defines = ["BIN_EXAMPLE"]      # optional, compile-time defines
```

**Notes:**

- Multiple binaries can be defined with multiple `[[bin]]` entries.
- Each `[[bin]]` must have an `entry` file containing `main()`.
- `flags` in a `[[bin]]` section are applied only during linking.
- `defines` are converted to `-D<NAME>` compiler flags.

---

## Profiles

Profiles define build-time configurations for different use cases.

```toml
[profile.dev]
# default []
flags = ["-Wall", "-Wextra"]   # optional, compiler flags
# default false
lto = false                    # optional, link-time optimization 
# default 0
opt_level = 0                  # optional, optimization level 0-3
# default true
debug = true                   # optional, include debug info

[profile.release]
# default []
flags = ["-Wall"]              # optional, compiler flags
# default true
lto = true                     # optional, enable link-time optimization
# default 3
opt_level = 3                  # optional, maximum optimization
# default false
debug = false                  # optional, disable debug info
```

**Notes:**

- Profile flags are applied at **compile time**.
- `lto` controls whether link-time optimizations are enabled.
- `opt_level` corresponds to typical optimization levels (0–3).
- `debug` controls debug info inclusion.

---

## Features

The `[features]` section allows you to group defines into named feature sets.

```toml
[features]
terminal-colors = ["TERM_COLORS", "RAW_MODE"]
logging = ["ENABLE_LOGGING"]
```

**Notes:**

- When a feature is activated, all defines listed are passed to the compiler (`-D<NAME>`).
- Features are **semantic groups** and do not affect linking directly.
- Useful for toggling functionality without editing multiple defines manually.
- Can be extended in the future to support subpackages and dependencies.

*For now features merely add additional defines. Making features optional and specifing features for subpackages, is planned for the dependency management system.*

---
