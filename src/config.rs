use std::fmt::Display;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::ForgeArgs;
use crate::fs_utils::std_toml_path;
use crate::utils::check_compiler;
use crate::arguments::Command::*;
use crate::config::CompilerKind::GCC;

pub struct Config {
    pub forge: Forge,
    pub args: ForgeArgs,
    pub compiler: CompilerKind,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CompilerKind {
    GCC,
    Clang,
    MSVC,
}

impl Display for CompilerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerKind::GCC => write!(f, "gcc"),
            CompilerKind::Clang => write!(f, "clang"),
            CompilerKind::MSVC => write!(f, "msvc"),
        }
    }   
}

fn determine_compiler_kind(forge: &Forge, args: &ForgeArgs) -> CompilerKind {
    let arg_compiler = match &args.command { 
        Run(opt) => {
            opt.compiler.clone()
        }
        Build(opt) => {
            opt.compiler.clone()
        }
        Rebuild(opt) => {
            opt.compiler.clone()
        }
        _ => None,
    };
    let conf_compiler = &forge.build.compiler;
    let kind = match arg_compiler { 
        // prioritize: args > forge
        Some(comp) => {
            match_compiler_kind(&comp)
        }
        None => {
            match conf_compiler {
                Some(comp) => {
                    match_compiler_kind(comp)
                }
                None => CompilerKind::GCC, // default compiler
            }
        }
    };
    kind
}

fn match_compiler_kind(str: &String) -> CompilerKind {
    match str.as_str() {
        "gcc" => CompilerKind::GCC,
        "clang" => CompilerKind::Clang,
        "mscv" => CompilerKind::MSVC,
        _ => {
            eprintln!("Error: Invalid compiler specified: {}", str);
            println!("Fallback to default compiler: gcc");
            CompilerKind::GCC
        }
    }
}

impl Config {
    pub fn read(args: &ForgeArgs) -> Self {
        let toml_path = std_toml_path().expect("Could not generate standard TOML path");
        let forge = parse_forge_file(toml_path.to_str().unwrap())
            .expect("Could not parse TOML file");
        
        let compiler = determine_compiler_kind(&forge, args);
        let mut cfg = Config { forge, args: args.clone(), compiler };
        check_compiler(&mut cfg);
        cfg
    }
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
    pub compiler: Option<String>,
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
