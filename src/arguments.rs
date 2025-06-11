use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "rustyforge",
    about = "A simple, lightweight build tool for C",
    version = "0.1.0",
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
    
    /// specify the build profile as debug (default)
    #[arg(long, global = true, conflicts_with = "release")]
    pub debug: bool,
    
    /// specify the build profile as release
    #[arg(long, global = true, conflicts_with = "debug")]
    pub release: bool,
    
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, PartialEq, Clone)]
pub enum Command {
    Build,
    Clean,
    Run(RunOptions),
    Rebuild,
    Init,
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct RunOptions {
    #[arg(long)]
    pub clean: bool,
}


