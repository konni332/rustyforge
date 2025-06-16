use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "rustyforge",
    about = "A simple, lightweight build tool for C",
    version = "0.4.0",
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
    /// Initialize a new, very rusty, forge
    Init(InitOptions),
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
    /// Run the project in debug mode
    #[arg(long, conflicts_with = "release")]   
    pub debug: bool,
    /// Run the project in release mode
    #[arg(long, conflicts_with = "debug")]  
    pub release: bool,
    /// Clean the project before running
    #[arg(long)]
    pub clean: bool,
    /// Specify the compiler to use
    #[arg(long)]
    pub compiler: Option<String>,
    /// Arguments to pass to the program
    #[arg(value_name = "ARGS", trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct BuildOptions {
    /// specify the build profile as debug (default)
    #[arg(long, conflicts_with = "release")]
    pub debug: bool,
    /// specify the build profile as release
    #[arg(long, conflicts_with = "debug")]   
    pub release: bool,
    /// specify the compiler to use
    #[arg(long)]
    pub compiler: Option<String>,
}


#[derive(Args, Debug, PartialEq, Clone)]
pub struct InitOptions {
    /// initialize with a specific compiler
    #[arg(long)]
    pub compiler: Option<String>,
}

#[derive(Args, Debug, PartialEq, Clone)]
pub struct CleanOptions {
    /// clean the debug artifacts only
    #[arg(long)]
    pub debug: bool,
    /// clean the release artifacts only
    #[arg(long)]
    pub release: bool,
    /// clean the libs artifacts only
    #[arg(long)]
    pub libs: bool,
}

pub fn set_command_defaults(cmd: &mut Command) {
    match cmd {
        Command::Build(opts) => {
            if !opts.debug && !opts.release {
                opts.debug = true;
                return;
            }
        },
        Command::Clean(opts) => {
            if !opts.debug && !opts.release && !opts.libs {
                opts.debug = true;
                opts.release = true;
                opts.libs = true;
            }
        },
        Command::Run(opts) => {
            if !opts.debug && !opts.release {
                opts.debug = true;
                return;
            }
        }
        Command::Rebuild(opts) => {
            if !opts.debug && !opts.release {
                opts.debug = true;
                return;
            }
        }
        Command::Init(opts) => {
            if opts.compiler.is_none() {
                opts.compiler = Some("gcc".to_string());
            }
        }
        Command::Discover(_) => {}
    }
}







