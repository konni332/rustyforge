use std::io::Write;
use std::path::Path;
use crate::config::{parse_forge_file, Config};
use crate::fs_utils::{ensure_necessary_files, init_forge_structure};
use crate::arguments::ForgeArgs;
use clap::Parser;
use crate::arguments::Command::{Build, Run, Rebuild, Clean, Init};
use crate::compile::compile;
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




fn main() -> std::io::Result<()>{
    // parse command line arguments
    let mut args = ForgeArgs::parse();
    if !&args.debug && !&args.release {
        args.debug = true;
    }
    
    // get the current working directory
    let cwd = std::env::current_dir()?;
    
    let config: Config;
    // don't check project structure if init command is used (obviously)
    if args.command != Init {
        ensure_necessary_files()?;
        let toml_path = Path::new("RustyForge.toml");
        let forge = parse_forge_file(toml_path.to_str().unwrap())
            .expect("Error parsing forge file. Format might be wrong.");
        config = Config {
            forge,
            args: args.clone(),
        };

        match &args.command {
            Build |
            Rebuild => {

                compile(&config).expect("Error compiling.");
                link(&config);
            }
            Run(_) => {
                compile(&config).expect("Error compiling.");
                link(&config);
                execute_target(&config, &cwd);
            }
            Clean => {
                print_cleaning();
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
            _ => {} // not necessary, just for compiler error suppression
        }
    }
    else {
        init_forge_structure()?;
    }
    Ok(())
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
