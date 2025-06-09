use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct Forge {
    pub project: Project,
    pub build: Build,
}

#[derive(Deserialize, Debug)]
pub struct Project {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
}

#[derive(Deserialize, Debug)]
pub struct Build {
    pub src: Vec<String>,
    pub include_dirs: Vec<String>,
    pub output: String,
    pub cflags: Option<Vec<String>>,
    pub ldflags: Option<Vec<String>>,
}

pub fn parse_forge_file(path: &str) -> Result<Forge, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let forge: Forge = toml::from_str(&contents)?;
    Ok(forge)
}
