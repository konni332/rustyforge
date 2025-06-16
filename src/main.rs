use std::io::Write;
use std::path::Path;
use anyhow::{Context, Result};
use crate::config::{Config};
use crate::fs_utils::{
    create_build_dir,
    create_forge_sub_dir,
    ensure_necessary_files,
    init_forge_structure,
    init_hash_cache_json,
    std_hash_cache_path,
    std_toml_path};

use crate::arguments::{set_command_defaults, CleanOptions, ForgeArgs, RunOptions};
use clap::Parser;
use crate::arguments::Command::{Build, Run, Rebuild, Clean, Init, Discover};
use crate::compile::compile;
use crate::discovery::discover;
use crate::linker::link;
use crate::ui::{print_cleaning, verbose_command, verbose_command_hard};
use crate::utils::derive_clean_options;

mod config;
mod fs_utils;
mod compile;
mod linker;
mod utils;
mod hashes;
mod tests;
mod ui;
mod arguments;
mod discovery;

fn main() -> Result<()>{
    // parse command line arguments
    let mut args = ForgeArgs::parse();
    
    // set default to debug if no other option is given
    set_command_defaults(&mut args.command);

    // get the current working directory
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    if let Init(opt) = &args.command{
        if let Err(e) = init_forge_structure(opt) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }
    if let Err(e) = ensure_necessary_files() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    
    let config = Config::read(&args);
    
    let targets = &config.forge.project.targets;
    if targets.iter().any(|t| t == "static" || t == "shared") {
        for dir in ["libs/out", "libs/obj"] {
            if let Err(e) = create_forge_sub_dir(dir) {
                eprintln!("Error creating {}: {}", dir, e);
                std::process::exit(1);
            }
        }
    }
    
    if let Err(e) = create_build_dir(&args.command) {
        eprintln!("Error: {}", e);
        std::process::exit(1);   
    }
    
    match args.command.clone() {
        Build(_) => {
            compile(&config).context("Error compiling")?;
            link(&config).expect("Error linking");
        }
        Rebuild(opt) => {
            let mut clean_opt = derive_clean_options(&opt);
            clean(&cwd, &mut clean_opt);
            compile(&config).expect("Error compiling");
            link(&config).expect("Error linking");
        }
        Run(mut opt) => {
            compile(&config).expect("Error compiling");
            link(&config).expect("Error linking");
            execute_target(&config, &cwd, &mut opt);
        }
        Clean(mut opt) => {
            clean(&cwd, &mut opt);
        }
        Discover(options) => {
            discover(&options, std_toml_path()
                .expect("Error generating standard .toml path")
            ).expect("Error discovering.");
        }
        _ => {
            print!("Hi! This will never be printed.")
        } // not necessary, just for compiler error suppression
    }
    Ok(())
}

fn execute_target(config: &Config, cwd: &Path, opt: &mut RunOptions) {
    // needs to mutable on windows!
    #[allow(unused_mut)]
    let mut exe_name = config.forge.build.output.clone();

    // add .exe extension on windows
    #[cfg(target_os = "windows")]
    exe_name.push_str(".exe");

    let mut exe_path = cwd.join("forge");
    if opt.debug {
        exe_path.push("debug");
    }
    else if opt.release {
        exe_path.push("release");
    }
    exe_path = exe_path.join(exe_name);
    let mut cmd = std::process::Command::new(exe_path);
    
    cmd.args(opt.args.clone());
    
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

fn clean(cwd: &Path, opt: &mut CleanOptions) {
    print_cleaning();
    // if none are specified, clean everything
    if !opt.debug && !opt.release && !opt.libs {
        opt.debug = true;
        opt.release = true;
        opt.libs = true;   
    }
    if opt.debug {
        let path = cwd.join("forge").join("debug");
        if path.exists() {
            std::fs::remove_dir_all(path).expect("Error removing debug directory.");
        }
    }
    if opt.release {
        let path = cwd.join("forge").join("release");
        if path.exists() {
            std::fs::remove_dir_all(path).expect("Error removing release directory.");
        }
    }
    if opt.libs {
        let libs_path = cwd.join("forge").join("libs");
        if libs_path.exists() {
            std::fs::remove_dir_all(libs_path).expect("Error removing libs directory.");
        }
    }
    let json_path = cwd.join("forge").join(".forge").join("hash_cache.json");
    if json_path.exists()  {
        std::fs::remove_file(json_path).expect("Error removing hash cache file.");
    }
    // reinitialize empty hash_cache.json file
    if let Err(e) = init_hash_cache_json(std_hash_cache_path().expect("Error getting hash cache path.")){
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    // reinitialize forge directory
    if opt.debug {
        if let Err(e) = create_forge_sub_dir("debug") {
            eprintln!("Error: {}", e);
        }
    }
    else if opt.release {
        if let Err(e) = create_forge_sub_dir("release") {
            eprintln!("Error: {}", e);
        }
    }
    else if opt.libs {
        if let Err(e) = create_forge_sub_dir("libs") {
            eprintln!("Error: {}", e);
        }
    }
}