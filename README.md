# rustyforge

A simple, fast build manager for C projects with minimal configuration.

---

## Current Features

- Parsing `rustyforge.toml` configuration file  
- Automatically determining `.c` files to compile based on timestamps and header dependencies  
- Compiling individual `.c` files to `.o` files with correct include paths  
- Linking `.o` files into the final executable  
- Cross-platform support (Windows/Linux) with proper handling of paths and executable extensions  
- Uses `gcc` as the compiler

---

## Planned Features

- Support for building and linking static and dynamic libraries  
- Hash-based caching to avoid unnecessary recompilation  
- Parallel compilation for faster builds  
- Test targets and automated test execution  
- Advanced configuration options (build types, cross-compilation, custom flags)  
- Logging and debugging options for better traceability  
- CLion plugin for automatic management of `rustyforge.toml` and seamless IDE workflow  
- Syntax highlighting and autocomplete for `rustyforge.toml` in IDEs

---

## CLion Plugin (Planned)

- Goal: Automatically detect and add new source files to build configuration  
- Show build errors directly in the IDE  
- Facilitate easy maintenance of `rustyforge.toml`  
- Implementation language: Kotlin (recommended for JetBrains plugins)  
- Effort: Medium – basic version realistic within a few weeks

---

## Installation

1. Clone rustyforge repository  
2. Install Rust toolchain (Rust 1.70 or newer recommended)  
3. Run `cargo build --release`  
4. Place the binary in your system path or run directly

---

## Usage

- Create a project with a `rustyforge.toml` file  
- Run `rustyforge build` to compile and link the project  
- Additional commands planned: `rustyforge clean`, `rustyforge test`

---

## Example `rustyforge.toml`

```toml
[build]
src = ["src/main.c", "src/utils.c"]
include_dirs = ["include/"]
target_exe = "my_project"
link = ["src/runner.c"]

[libs]
static = ["libfoo.a"]
dynamic = ["libbar.so"]
```

---

## Contact / Contributing

Feel free to open issues or pull requests for questions, ideas, or contributions.

---

## License

MIT License
