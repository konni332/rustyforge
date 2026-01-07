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
    Build {
        #[command(flatten)]
        opts: ForgeOptions,
    },
    /// Clean the project all build artifacts, unless otherwise specified
    Clean,
    /// Build and run the project
    Run {
        #[command(flatten)]
        opts: ForgeOptions,
        #[arg(value_name = "ARGS", trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Rebuild the project
    Rebuild {
        #[command(flatten)]
        opts: ForgeOptions,
    },
    /// Initialize a new project in current directory
    Init,
    /// Create a new project in a new directory
    New { project_name: String },
    /// Remove all files related to RustyForge in the current directory
    Remove,
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct ForgeOptions {
    /// specify the build profile as debug (default)
    #[arg(long, conflicts_with = "release")]
    pub debug: bool,
    /// specify the build profile as release
    #[arg(long, conflicts_with = "debug")]
    pub release: bool,
    /// cross compile for a different target
    #[arg(long)]
    pub target: Option<String>,

    #[arg(long)]
    pub discover_hidden: bool,
}

impl ForgeOptions {
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
    pub fn discover_hidden(&self) -> bool {
        self.discover_hidden
    }
}

impl ForgeArgs {
    pub fn profile(&self) -> Option<Profile> {
        match &self.command {
            Command::Build { opts } => opts.profile(),
            Command::Rebuild { opts } => opts.profile(),
            Command::Run { opts, .. } => opts.profile(),
            _ => None,
        }
    }
    pub fn target(&self) -> Option<&String> {
        match &self.command {
            Command::Build { opts } => opts.target.as_ref(),
            Command::Rebuild { opts } => opts.target.as_ref(),
            Command::Run { opts, .. } => opts.target.as_ref(),
            _ => None,
        }
    }
    pub fn discover_hidden(&self) -> bool {
        match &self.command {
            Command::Build { opts } => opts.discover_hidden(),
            Command::Rebuild { opts } => opts.discover_hidden(),
            Command::Run { opts, .. } => opts.discover_hidden(),
            _ => false,
        }
    }
}
