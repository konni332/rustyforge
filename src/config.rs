use serde::{Deserialize, Serialize};
use std::fs;
use crate::ForgeArgs;

pub struct Config {
    pub forge: Forge,
    pub args: ForgeArgs,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Forge {
    pub project: Project,
    pub build: Build,
    pub dependencies: Option<Dependencies>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Project {
    pub name: String,
    pub targets: Vec<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Build {
    pub src: Vec<String>,
    pub include_dirs: Vec<String>,
    pub output: String,
    pub cflags: Option<Vec<String>>,
    pub ldflags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Dependencies {
    pub libraries: Vec<String>,
    pub library_paths: Vec<String>,
    pub include_dirs: Vec<String>,
    pub posix_libraries: Vec<String>,
}

pub fn parse_forge_file(path: &str) -> Result<Forge, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let forge: Forge = toml::from_str(&contents)?;
    Ok(forge)
}
