use crate::config::parse_forge_file;
use crate::fs_utils::create_forge_dir;

mod config;
mod runner;
mod fs_utils;
mod compile;
mod linker;

fn main() {
    // parse forge config file
    let mut cwd = std::env::current_dir().expect("Error getting current working directory.");
    cwd.push("RustyForge.toml");
    let config = parse_forge_file(cwd.to_str().unwrap())
        .expect("Error parsing forge file. Format might be wrong.");
    
    // create the forge directory if it does not exist
    create_forge_dir().expect("Error creating forge directory.");
    
    // compile necessary files
    compile::compile(&config).expect("Error compiling project.");
    
    // link files
    linker::link(&config);
}
