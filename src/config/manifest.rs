use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::CoreResult;

/// Default C language edition used when no explicit edition is specified.
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
    pub dev: Option<Profile>,
    pub release: Option<Profile>,
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

impl Profile {
    pub fn default_dev() -> Self {
        Self {
            flags: None,
            lto: Some(false),
            opt_level: Some(0),
            debug: Some(true),
        }
    }
    pub fn default_release() -> Self {
        Self {
            flags: None,
            lto: Some(true),
            opt_level: Some(3),
            debug: Some(false),
        }
    }
}

impl Manifest {
    pub fn new(name: &str) -> Self {
        Manifest {
            package: Package::new(name),
            build: Build::new(),
            lib: None,
            bin: None,
            profile: None,
            features: None,
        }
    }

    pub fn bin_template(name: &str) -> Self {
        Manifest {
            package: Package::new(name),
            build: Build::new(),
            lib: None,
            bin: Some(vec![Executable::new(name, PathBuf::from("src/main.c"))]),
            profile: None,
            features: None,
        }
    }

    pub fn lib_template(name: &str) -> Self {
        Manifest {
            package: Package::new(name),
            build: Build::new(),
            lib: Some(Lib::new(name)),
            bin: None,
            profile: None,
            features: None,
        }
    }

    /// Reads and parses manifest from `current/directory/RustyForge.toml`
    pub fn read_manifest() -> CoreResult<Self> {
        let cwd = std::env::current_dir()?;
        let manifest_path = cwd.join("RustyForge.toml");
        if !manifest_path.exists() {
            return Err(Box::new(crate::CoreError::ManifestNotFound));
        }

        let src = std::fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = match toml::from_str(&src) {
            Ok(m) => m,
            Err(e) => {
                return Err(Box::new(crate::CoreError::InvalidManifest {
                    path: manifest_path,
                    src,
                    span: e.span().unwrap_or_default(),
                    msg: e.message().to_string(),
                }));
            }
        };

        Ok(manifest)
    }
    /// Writes Self::default() to `current/directory/RustyForge.toml`
    pub fn write_default() -> CoreResult<()> {
        let manifest = Self::default();
        manifest.write_manifest()
    }

    /// Writes self to `current/directory/RustyForge.toml`
    pub fn write_manifest(&self) -> CoreResult<()> {
        let contents = toml::to_string_pretty(&self)?;
        let path = std::env::current_dir()?.join("RustyForge.toml");
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn write_new(name: &str) -> CoreResult<()> {
        let manifest = Manifest::new(name);
        manifest.write_manifest()
    }

    pub fn write_lib_template(name: &str) -> CoreResult<()> {
        let manifest = Manifest::lib_template(name);
        manifest.write_manifest()
    }

    pub fn write_bin_template(name: &str) -> CoreResult<()> {
        let manifest = Manifest::bin_template(name);
        manifest.write_manifest()
    }
}
