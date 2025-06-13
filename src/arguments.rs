use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "rustyforge",
    about = "A simple, lightweight build tool for C",
    version = "0.3.0",
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
    /// Compile and link a C project
    Build,
    /// Clean the project (executable, library, and object files)
    Clean,
    /// Build and run the project
    Run(RunOptions),
    /// Rebuild the project
    Rebuild,
    /// Initialize a new, very rusty, forge
    Init,
    /// Discover '.c' files and include directories
    Discover(DiscoverOptions),
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct DiscoverOptions {
    /// Automatically add all files to the project
    #[arg(long)]
    pub auto: bool,
    /// Ignore files that match the given pattern, file names, or paths
    #[arg(long, value_name = "PATTERN", num_args = 1)]
    pub ignore: Vec<String>,
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct RunOptions {
    /// Clean the project before running
    #[arg(long)]
    pub clean: bool,
    /// Arguments to pass to the program
    #[arg(value_name = "ARGS", trailing_var_arg = true)]
    pub args: Vec<String>,
}


