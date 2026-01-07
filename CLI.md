# RustyForge CLI Reference

This document describes all available commands, options, and arguments for the RustyForge CLI.

---

## Usage

```
rustyforge [OPTIONS] <COMMAND> [ARGS]
```

**Global Options:**

| Option               | Description                                                                                              |
|----------------------|----------------------------------------------------------------------------------------------------------|
| `-v`, `--verbose`    | Enable verbose output. Conflicts with `--quiet`.                                                         |
| `-q`, `--quiet`      | Suppress output. Conflicts with `--verbose`.                                                             |
| `--features <list>`  | Activate one or more features globally. Comma-separated list, e.g. `--features terminal-colors,logging`. |

---

## Commands

### build

Compile and link your project.

```
rustyforge build [OPTIONS]
```

**Options (BuildOptions):**

| Option                    | Description                                                                      |
|---------------------------|----------------------------------------------------------------------------------|
| `--release`               | Build using the release profile (`profile.release`). Conflicts with `--dev`.     |
| `--dev`                   | Build using the development profile (`profile.dev`). Conflicts with `--release`. |
| `--features <list>`       | Activate features for this build. Overrides global `--features`.                 |
| `--bin <name>`            | Build only the specified binary. Conflicts with `--lib`.                         |
| `--lib`                   | Build only the library target.                                                   |
| `-j <num>`                | Number of parallel compilation threads. Defaults to the number of CPU cores.     |

---

### run

Compile, link, and run a binary. Additional program arguments can be provided after `--`.

```
rustyforge run [OPTIONS] -- [ARGS]
```

**Options:**

- Same as `build`.
- `--bin <name>` is required if multiple binaries exist.
- All arguments after `--` are passed directly to the program.

**Example:**

```
rustyforge run --release --bin nettool -- -p 8080
```

---

### check

Validate manifest and discovered sources without building.

```
rustyforge check
```

- Ensures all required fields in `[package]` exist.
- Verifies that `[[bin]]` entries and library sources exist.
- Checks ignore patterns and feature/define consistency.

---

### clean

Remove all build artifacts.

```
rustyforge clean
```

- Removes the `target/` directory.

---

### info

Display project metadata and configuration.

```
rustyforge info [OPTIONS]
```

**Options:**

| Option   | Description                        |
|----------|------------------------------------|
| `--json` | Output information in JSON format. |

- Shows package name, version, edition, authors, license, repository.
- Lists binaries and library targets.
- Shows profiles with compiler flags, defines, LTO, optimization level, and debug info.

---

### list-bins

List all binaries in the project.

```
rustyforge list-bins
```

- Outputs binary names and entry paths.

---

### list-features

List all features defined in `[features]` and their associated defines.

```
rustyforge list-features
```

- Shows feature names and the `-D` defines they enable.
- Useful to see available features before activating them in build/run.

---

## Notes

- By default, invoking `rustyforge` with no subcommand is equivalent to `rustyforge build`.
- Feature activation order is deterministic:
  1. Profile defines
  2. Target defines (lib or bin)
  3. Activated features
- Compiler flags (`flags`) from the profile are used in the **compile phase**.
- Target flags (`flags`) from `[[bin]]` or `[lib]` are used in the **link phase**.
- LTO, optimization level, and debug settings come from the selected profile.
