use std::num::NonZero;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;

use crate::{CoreError, RUSTYFORGE_VERSION, error::CoreResult};

/// RustyForge CLI
///
/// This is the main entry point for interacting with RustyForge projects.
/// Provides commands to build, run, check, clean, and inspect your project.
#[derive(Debug, Clone, Parser)]
#[command(version = RUSTYFORGE_VERSION)]
pub struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

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

        /// Name of the binary
        #[arg(long)]
        exe: Option<String>,

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

    /// Initialize a new project in a new directory
    New {
        /// Name of the new project and the new directory
        name: String,

        /// Use the binary template, conlicts with `--lib`
        #[arg(long, conflicts_with = "lib")]
        bin: bool,

        /// Use the library template (defaults static), conflicts with `--bin`
        #[arg(long)]
        lib: bool,

        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Initialize a new project in this directory
    Init {
        #[arg(long, short = 'f')]
        force: bool,
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
    pub threads: Option<NonZero<usize>>,
}

impl Cli {
    /// Parse and validate the command line arguments
    ///
    /// returns the Cli struct if the validation was successfull
    /// Sets the verbosity level
    pub fn create() -> CoreResult<Self> {
        let cli = Cli::parse();

        if cli.verbose.is_silent() && matches!(cli.command, CliCommand::Info { .. }) {
            return Err(Box::new(CoreError::CliValidation(
                "`--quiet` cannot be used together with the `info` subcommand".to_string(),
            )));
        };
        Ok(cli)
    }

    pub fn get_verbosity(&self) -> Verbosity {
        self.verbose
    }

    pub fn threads(&self) -> Option<NonZero<usize>> {
        match &self.command {
            CliCommand::Run { opts, .. } => opts.threads,
            CliCommand::Build { opts } => opts.threads,
            _ => None,
        }
    }

    pub fn profile_name(&self) -> &'static str {
        match &self.command {
            CliCommand::Run { opts, .. } => opts.profile_name(),
            CliCommand::Build { opts } => opts.profile_name(),
            _ => "dev",
        }
    }
    pub fn bin(&self) -> Option<&str> {
        match &self.command {
            CliCommand::Run { opts, .. } => opts.bin(),
            CliCommand::Build { opts } => opts.bin(),
            _ => None,
        }
    }
    pub fn lib(&self) -> bool {
        match &self.command {
            CliCommand::Run { opts, .. } => opts.lib,
            CliCommand::Build { opts } => opts.lib,
            _ => false,
        }
    }
}

impl BuildOptions {
    pub fn profile_name(&self) -> &'static str {
        if self.release { "release" } else { "dev" }
    }
    pub fn bin(&self) -> Option<&str> {
        self.bin.as_deref()
    }
}
