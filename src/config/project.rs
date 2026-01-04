use serde::{Deserialize, Serialize};

use crate::config::tool::CompilerKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: Project,
    pub compilation: Compilation,
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
pub struct Compilation {
    pub ignore_patterns: Vec<String>,
}

impl ProjectConfig {
    pub fn new(name: &str) -> Self {
        Self {
            project: Project::new(name),
            compilation: Compilation::new(),
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

impl Compilation {
    pub fn new() -> Self {
        Self {
            ignore_patterns: vec![],
        }
    }
}
