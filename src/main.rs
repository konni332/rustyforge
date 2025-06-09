use crate::config::parse_forge_file;

mod config;
mod runner;
mod fs_utils;
mod compile;
mod link;

fn main() {
    let config = parse_forge_file("C:/Users/doepp/CLionProjects/rustyforge/RustyForge.toml")
        .expect("Error parsing forge file. Format might be wrong.");
    println!("{:?}", config);
}
