use clap::{Parser, Subcommand};

use crate::RUSTYFORGE_VERSION;

/// RustyForge CLI
///
/// This is the main entry point for interacting with RustyForge projects.
/// Provides commands to build, run, check, clean, and inspect your project.
#[derive(Debug, Clone, Parser)]
#[command(version = RUSTYFORGE_VERSION)]
pub struct Cli {
    /// Enable verbose output
    #[arg(long, short, conflicts_with = "quiet", global = true)]
    pub verbose: bool,

    /// Suppress output (quiet mode)
    #[arg(long, short, global = true, conflicts_with = "info")]
    pub quiet: bool,

    /// Activate features by name
    ///
    /// Multiple features can be specified, e.g. `--features terminal-colors,logging`
    #[arg(long, global = true)]
    pub features: Option<Vec<String>>,

    /// The subcommand to execute
    #[command(subcommand)]
    pub command: CliCommand,
}

/// RustyForge subcommands
#[derive(Debug, Clone, Subcommand)]
pub enum CliCommand {
    /// Compile and link your project
    Build {
        #[command(flatten)]
        opts: BuildOptions,
    },

    /// Compile, link, and run a binary
    ///
    /// Accepts additional arguments after `--` which are passed directly to the program.
    Run {
        #[command(flatten)]
        opts: BuildOptions,

        /// Arguments to pass to the program being executed
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Validate the manifest and discovered sources without building
    Check,

    /// Remove all build artifacts (e.g., target directory)
    Clean,

    /// Display project metadata and configuration
    Info {
        /// Output information in JSON format
        #[arg(long)]
        json: bool,
    },
}

/// Options common to Build and Run commands
#[derive(Debug, Clone, Parser)]
pub struct BuildOptions {
    /// Build using the release profile (`profile.release`)
    ///
    /// Conflicts with `--dev`.
    #[arg(long, conflicts_with = "dev")]
    pub release: bool,

    /// Build using the development profile (`profile.dev`)
    ///
    /// Conflicts with `--release`.
    #[arg(long)]
    pub dev: bool,

    /// Activate specific features for this build/run
    ///
    /// Overrides global features specified with `--features`.
    #[arg(long)]
    pub features: Option<Vec<String>>,

    /// Build only the specified binary target
    ///
    /// Conflicts with `--lib`.
    #[arg(long, conflicts_with = "lib")]
    pub bin: Option<String>,

    /// Build only the library target
    #[arg(long)]
    pub lib: bool,

    /// Number of parallel compilation threads
    #[arg(short = 'j')]
    pub threads: Option<i32>,
}

