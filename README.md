# rustyforge

RustyForge is a minimal, fast, and cross-platform build system for C.
It’s designed for simplicity and ease of use, with modern features like toml-based configuration, parallel builds,
and hash-based caching.
Ideal for both beginners and experienced developers looking for a streamlined alternative to Make.


---

## Current Features

- Parsing the `RustyForge.toml` configuration file  
- Hash-based caching, to avoid unnecessary compilation (including `.h` files)
- Compiling individual `.c` files to `.o` files with correct include paths  
- Parallel compilation, for faster builds
- Linking `.o` files into the final executable  
- Support for linking and building static (`.a`) and dynamic (`.so`/`.dll`) libraries
- Cross-platform support (Windows/Linux) with proper handling of paths and executable extensions
- Uses `gcc` as the compiler
---

## Planned Features

- Test targets and automated test execution 
- Cross compilation
- Support for more compilers (`clang` `tcc`)
- Support for non gcc-based toolchains

---

## CLion Plugin (Planned)

- Automatically detect and add new source files to the build configuration  
- Show build errors directly in the IDE  
- Facilitate easy maintenance of `RustyForge.toml`  
- Syntax highlighting and autocomplete for `RustyForge.toml`

---

## Installation

1. Clone rustyforge repository  
2. Install Rust toolchain (Rust 1.70 or newer recommended)  
3. Run `cargo build --release`  
4. Place the binary in your system path or run directly

---

## Usage

- Create a project with a `RustyForge.toml` file  
- Run `rustyforge --help` to see usage

### Examples `Commands`
````shell
rustyforge init
rustyforge discover
rustyforge build --verbose
rustyforge clean
````

---

## Example `RustyForge.toml`

```toml
[project]
name = "project-name"
targets = ["bin", "shared", "static"]

[build]
src = ["src/main.c", "src/foo.c"]
include_dirs = ["include"]
output = "lib_or_executable_name"

[dependencies]
libraries = ["bar"]
library_paths = ["libs"]
include_dirs = ["libs/include"]
posix_libraries = ["m", "pthreads"]
```

---

## Contact / Contributing

Open issues or pull requests for questions, ideas, or contributions

---

## License

MIT License
