use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const DEFAULT_C_EDITION: &str = "c11";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    pub package: Package,
    pub build: Build,
    pub lib: Option<Lib>,
    pub bin: Option<Vec<Executable>>,
    pub profile: Option<Profiles>,
    pub features: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Build {
    ignore: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lib {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: LibType,
    pub ignore: Option<Vec<String>>,
    pub flags: Option<Vec<String>>,
    pub defines: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Executable {
    pub name: String,
    pub entry: PathBuf,
    pub ignore: Option<Vec<String>>,
    pub flags: Option<Vec<String>>,
    pub defines: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy)]
pub enum LibType {
    #[default]
    #[serde(rename = "static")]
    Static,
    #[serde(rename = "shared")]
    Shared,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profiles {
    dev: Option<Profile>,
    release: Option<Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub flags: Option<Vec<String>>,
    pub lto: Option<bool>,
    pub opt_level: Option<i32>,
    pub debug: Option<bool>,
}

impl Lib {
    pub fn new(name: &str) -> Self {
        Lib {
            name: name.into(),
            ty: LibType::Static,
            ..Default::default()
        }
    }
}

impl Executable {
    pub fn new<P: AsRef<Path>>(name: &str, entry: P) -> Self {
        Self {
            name: name.into(),
            entry: entry.as_ref().to_path_buf(),
            ..Default::default()
        }
    }
}

impl Build {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Package {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

impl Default for Package {
    fn default() -> Self {
        Self {
            name: "<default>".into(),
            version: "0.1.0".into(),
            edition: DEFAULT_C_EDITION.into(),
            authors: None,
            license: None,
            description: None,
            repository: None,
        }
    }
}
