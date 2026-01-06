use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::tool::CompilerKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: Project,
    pub build: Build,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub profile_release: Option<ProfileConfig>,
    pub profile_debug: Option<ProfileConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    pub flags: Vec<String>,
    pub defines: Vec<String>,
    pub compiler: Option<CompilerKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub ignore_patterns: Vec<String>,
    pub link_targets: Vec<LinkTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTarget {
    pub name: String,
    pub user_flags: Vec<String>,
    pub kind: LinkTargetKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LinkTargetKind {
    #[serde(rename(serialize = "executable", deserialize = "executable"))]
    Executable,
    #[serde(rename(serialize = "static", deserialize = "static"))]
    StaticLibrary,
    #[serde(rename(serialize = "shared", deserialize = "shared"))]
    SharedLibrary,
}

impl Display for LinkTargetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LinkTargetKind::Executable => write!(f, "executable"),
            LinkTargetKind::StaticLibrary => write!(f, "static library"),
            LinkTargetKind::SharedLibrary => write!(f, "shared library"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub kind: LinkTargetKind,
    pub path: Option<PathBuf>,
    pub include_dirs: Vec<PathBuf>,
    pub lib_dirs: Vec<PathBuf>,
    pub libs: Vec<String>,
}

impl ProjectConfig {
    pub fn new(name: &str) -> Self {
        Self {
            project: Project::new(name),
            build: Build::new(),
            dependencies: vec![],
        }
    }
}

impl Project {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".into(),
            profile_release: None,
            profile_debug: None,
        }
    }
}

impl Build {
    pub fn new() -> Self {
        Self {
            ignore_patterns: vec![],
            link_targets: vec![],
        }
    }
}
