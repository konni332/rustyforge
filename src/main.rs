use crate::config::{parse_forge_file, Config};
use crate::fs_utils::{create_forge_dir, create_forge_dirs, init_default_toml, init_hash_cache_json};
use crate::arguments::ForgeArgs;
use clap::Parser;
use crate::arguments::Command::{Build, Run, Rebuild, Clean, Init};
use crate::compile::compile;
use crate::linker::link;

mod config;
mod runner;
mod fs_utils;
mod compile;
mod linker;
mod utils;
mod hashes;
mod tests;
mod ui;
mod arguments;




fn main() {
    let mut args = ForgeArgs::parse();
    if !args.debug && !args.release {
        args.debug = true;
    }
    
    // parse forge config file
    let cwd = std::env::current_dir().expect("Error getting current working directory.");
    
    let toml_path = cwd.join("RustyForge.toml");
    
    // if command is init, create the forge directory and the forge file
    if args.command == Init {
        // create the forge directory if it does not exist
        create_forge_dir().expect("Error creating forge directory.");
        create_forge_dirs(".forge").expect("Error creating .forge directory.");
        init_hash_cache_json().expect("Error creating hash cache file.");
        
        // init a default RustForge.toml file
        init_default_toml().expect("Error creating default forge file.");
        
        // init include/ and src/ directories
        std::fs::create_dir_all(cwd.join("include")).expect("Error creating include directory.");
        std::fs::create_dir_all(cwd.join("src")).expect("Error creating src directory.");
        return;
    }
    
    if !toml_path.exists() {
        eprintln!("No RustyForge.toml file found. Use rustyforge init to create one.");
        std::process::exit(1);
    }
    
    
    let forge = parse_forge_file(toml_path.to_str().unwrap())
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
    
    match &config.args.command {
        Build |
        Rebuild => {
            compile(&config).expect("Error compiling.");
            link(&config);
        }
        Run(_) => {
            compile(&config).expect("Error compiling.");
            link(&config);
        }
        Clean => {
            if config.args.debug {
                let path = cwd.join("forge").join("debug");
                std::fs::remove_dir_all(path).expect("Error removing debug directory.");
            }
            else if config.args.release {
                let path = cwd.join("forge").join("release");
                std::fs::remove_dir_all(path).expect("Error removing release directory.");
            }
            let json_path = cwd.join("forge").join(".forge").join("hash_cache.json");
            std::fs::remove_file(json_path).expect("Error removing hash cache file.")
        }
        Init => {}
    }
    
    
}
