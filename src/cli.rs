use clap::{Args, Parser, Subcommand};

use crate::compile::types::Profile;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "rustyforge",
    about = "A simple, lightweight build tool for C",
    version = "0.4.1",
    author = "<konni332>",
    subcommand_required = true,
    arg_required_else_help = true,
    override_usage = "rustyforge <COMMAND> [OPTIONS]"
)]
pub struct ForgeArgs {
    /// show verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// show verbose output, with raw paths
    #[arg(long = "verbose-hard", global = true, conflicts_with = "verbose")]
    pub verbose_hard: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, PartialEq, Clone)]
pub enum Command {
    /// Compile and link a C project
    Build(BuildOptions),
    /// Clean the project all build artifacts, unless otherwise specified
    Clean(CleanOptions),
    /// Build and run the project
    Run(RunOptions),
    /// Rebuild the project
    Rebuild(BuildOptions),
    /// Initialize a new project in current directory
    Init,
    /// Create a new project in a new directory
    New { project_name: String },
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct RunOptions {
    /// Run the project in debug mode
    #[arg(long, conflicts_with = "release")]
    pub debug: bool,
    /// Run the project in release mode
    #[arg(long, conflicts_with = "debug")]
    pub release: bool,
    /// Clean the project before running
    #[arg(long)]
    pub clean: bool,
    /// Arguments to pass to the program
    #[arg(value_name = "ARGS", trailing_var_arg = true)]
    pub args: Vec<String>,
    /// cross compile for a different target
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct BuildOptions {
    /// specify the build profile as debug (default)
    #[arg(long, conflicts_with = "release")]
    pub debug: bool,
    /// specify the build profile as release
    #[arg(long, conflicts_with = "debug")]
    pub release: bool,
    /// cross compile for a different target
    #[arg(long)]
    pub target: Option<String>,
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct CleanOptions {
    /// clean the debug artifacts only
    #[arg(long)]
    pub debug: bool,
    /// clean the release artifacts only
    #[arg(long)]
    pub release: bool,
}

impl RunOptions {
    pub fn profile(&self) -> Option<Profile> {
        if self.release {
            Some(Profile::Release)
        } else if self.debug {
            Some(Profile::Debug)
        } else {
            None
        }
    }
    pub fn target(&self) -> Option<&String> {
        self.target.as_ref()
    }
}

impl BuildOptions {
    pub fn profile(&self) -> Option<Profile> {
        if self.release {
            Some(Profile::Release)
        } else if self.debug {
            Some(Profile::Debug)
        } else {
            None
        }
    }
    pub fn target(&self) -> Option<&String> {
        self.target.as_ref()
    }
}

impl ForgeArgs {
    pub fn profile(&self) -> Option<Profile> {
        match &self.command {
            Command::Run(opts) => opts.profile(),
            Command::Build(opts) => opts.profile(),
            Command::Rebuild(opts) => opts.profile(),
            _ => None,
        }
    }
    pub fn target(&self) -> Option<&String> {
        match &self.command {
            Command::Run(opts) => opts.target(),
            Command::Build(opts) => opts.target(),
            Command::Rebuild(opts) => opts.target(),
            _ => None,
        }
    }
}
