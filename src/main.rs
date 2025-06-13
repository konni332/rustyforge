use std::io::Write;
use std::path::Path;
use crate::config::{parse_forge_file, Config};
use crate::fs_utils::{create_forge_sub_dir, ensure_necessary_files, init_forge_structure, init_hash_cache_json};
use crate::arguments::ForgeArgs;
use clap::Parser;
use crate::arguments::Command::{Build, Run, Rebuild, Clean, Init, Discover};
use crate::compile::compile;
use crate::discovery::discover;
use crate::linker::link;
use crate::ui::{print_cleaning, verbose_command, verbose_command_hard};


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
mod discovery;

fn main() {
    // parse command line arguments
    let mut args = ForgeArgs::parse();
    // set default to debug if no other option is given
    if !&args.debug && !&args.release {
        args.debug = true;
    }

    // get the current working directory
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    
    let config: Config;
    // don't check project structure if init command is used (obviously)
    if args.command != Init {
        if let Err(e) = ensure_necessary_files() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        let toml_path = Path::new("RustyForge.toml");
        let forge = parse_forge_file(toml_path.to_str().unwrap())
            .expect("Error parsing forge file. Format might be wrong.");
        config = Config {
            forge,
            args: args.clone(),
        };
        
        let targets = &config.forge.project.targets;
        if targets.iter().any(|t| t == "static" || t == "shared") {
            for dir in ["libs/out", "libs/obj"] {
                if let Err(e) = create_forge_sub_dir(dir) {
                    eprintln!("Error creating {}: {}", dir, e);
                    std::process::exit(1);
                }
            }
        }
        
        if args.debug {
            if let Err(e) = create_forge_sub_dir("debug") {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        else if args.release {
            if let Err(e) = create_forge_sub_dir("release") {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        
        match &args.command {
            Build => {
                compile(&config).expect("Error compiling.");
                link(&config).expect("Error linking.");
            }
            Rebuild => {
                clean(&config, &cwd);
                compile(&config).expect("Error compiling.");
                link(&config).expect("Error linking.");
            }
            Run(_) => {
                compile(&config).expect("Error compiling.");
                link(&config).expect("Error linking.");
                execute_target(&config, &cwd);
            }
            Clean => {
                clean(&config, &cwd);
            }
            Discover(options) => {
                discover(&options).expect("Error discovering.");
            }
            _ => {
                print!("Hi! This will never be printed.")
            } // not necessary, just for compiler error suppression
        }
    }
    // initialize forge structure if init command is used
    else {
        if let Err(e) = init_forge_structure() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_target(config: &Config, cwd: &Path) {
    let mut exe_name = config.forge.build.output.clone();

    // add .exe extension on windows
    #[cfg(target_os = "windows")]
    exe_name.push_str(".exe");

    let mut exe_path = cwd.join("forge");
    if config.args.debug {
        exe_path.push("debug");
    }
    else if config.args.release {
        exe_path.push("release");
    }
    exe_path = exe_path.join(exe_name);
    let mut cmd = std::process::Command::new(exe_path);
    
    match &config.args.command { 
        Run(run_options) => {
            cmd.args(run_options.args.clone());
        }
        _ => {}
    }
    
    if config.args.verbose {
        verbose_command(&cmd);
    }
    else if config.args.verbose_hard {
        verbose_command_hard(&cmd);
    }

    let output = cmd.output().expect("Error running executable.");

    std::io::stdout().write_all(&output.stdout).expect("Error writing stdout.");
    std::io::stderr().write_all(&output.stderr).expect("Error writing stderr.");
}

fn clean(config: &Config, cwd: &Path) {
    print_cleaning();
    if config.args.debug {
        let path = cwd.join("forge").join("debug");
        std::fs::remove_dir_all(path).expect("Error removing debug directory.");
    }
    else if config.args.release {
        let path = cwd.join("forge").join("release");
        std::fs::remove_dir_all(path).expect("Error removing release directory.");
    }
    
    let libs_path = cwd.join("forge").join("libs");
    std::fs::remove_dir_all(libs_path).expect("Error removing libs directory.");
    
    let json_path = cwd.join("forge").join(".forge").join("hash_cache.json");
    std::fs::remove_file(json_path).expect("Error removing hash cache file.");
    // reinitialize empty hash_cache.json file
    if let Err(e) = init_hash_cache_json(){
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    // reinitialize forge directory
    if config.args.debug {
        if let Err(e) = create_forge_sub_dir("debug") {
            eprintln!("Error: {}", e);
        }
    }
    else if config.args.release {
        if let Err(e) = create_forge_sub_dir("release") {
            eprintln!("Error: {}", e);
        }
    }
}