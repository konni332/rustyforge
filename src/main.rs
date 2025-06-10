use crate::config::{parse_forge_file, Config};
use crate::fs_utils::{create_forge_dir, create_forge_dirs, init_hash_cache_json};

mod config;
mod runner;
mod fs_utils;
mod compile;
mod linker;
mod utils;
mod hashes;
mod tests;
mod ui;

use clap::{Parser};
use serde::Deserialize;

#[derive(Parser, Debug, Deserialize)]
#[command(
    name = "rustyforge",
    version,
    about = "Forging builds for C projects – simple, fast, and portable."
)]
pub struct ForgeArgs {
    #[arg(long, conflicts_with = "debug")]
    pub release: bool,

    #[arg(long, conflicts_with = "release")]
    pub debug: bool,

    #[arg(long = "rebuild-all")]
    pub rebuild_all: bool,

    /// Maximum number of threads (-1 = auto)
    #[arg(long, default_value_t = -1)]
    pub threads: isize,

    #[arg(long = "no-link")]
    pub no_link: bool,

    #[arg(long)]
    pub verbose: bool,

    #[arg(long = "dry-run")]
    pub dry_run: bool,

    #[arg(long)]
    pub config: Option<String>,
    
    #[arg(long = "verbose-hard", conflicts_with = "verbose")]
    pub verbose_hard: bool,
}


fn main() {
    let mut args = ForgeArgs::parse();
    if !args.debug && !args.release {
        args.debug = true;
    }
    
    // parse forge config file
    let mut cwd = std::env::current_dir().expect("Error getting current working directory.");
    
    match &args.config {
        Some(str) => cwd.push(str.clone()),
        None => cwd.push("RustyForge.toml"),
    }
    
    
    let forge = parse_forge_file(cwd.to_str().unwrap())
        .expect("Error parsing forge file. Format might be wrong.");
    
    let config = Config {
        forge,
        args,
    };
    
    // create the forge directory if it does not exist
    create_forge_dir().expect("Error creating forge directory.");
    // create the .forge directory if it does not exist
    create_forge_dirs(".forge").expect("Error creating .forge directory.");
    // create the .forge/hash_cache.json file if it does not exist
    init_hash_cache_json().expect("Error creating hash cache file.");
    // create the necessary subdirectories
    if config.args.debug {
        create_forge_dirs("debug").expect("Error creating debug directories.");
    }
    else if config.args.release {
        create_forge_dirs("release").expect("Error creating release directories.");
    }
    
    // compile necessary files
    compile::compile(&config).expect("Error compiling project.");
    
    // link files
    if !config.args.no_link {
        linker::link(&config);
    }
}
