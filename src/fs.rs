use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::project::ProjectConfig;

pub fn initialize_filestructure() -> Result<()> {
    let global_config_dir = global_config_dir()?;
    std::fs::create_dir_all(&global_config_dir).context(format!(
        "Failed to create global config dir at: {}",
        global_config_dir.display(),
    ))?;

    let cwd = std::env::current_dir().context("Failed to determine current working directory")?;
    let toml_path = toml_path(&cwd);
    if !toml_path.exists() {
        let project_name = cwd
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("project-name");
        let project_config = ProjectConfig::new(project_name);
        let project_config_str = toml::to_string_pretty(&project_config)?;
        std::fs::write(toml_path, project_config_str)?;
    }
    Ok(())
}

pub fn toml_path<P: AsRef<Path>>(root: P) -> PathBuf {
    root.as_ref().join("RustyForge.toml")
}

pub fn config_dir<P: AsRef<Path>>(root: P) -> PathBuf {
    root.as_ref().join(".rustyforge/")
}

pub fn global_config_dir() -> Result<PathBuf> {
    Ok(dirs_next::config_dir()
        .context("Failed to fetch OS config directory")?
        .join("rustyforge"))
}

pub fn target_dir<P: AsRef<Path>>(root: P) -> PathBuf {
    root.as_ref().join("target/")
}

pub fn debug_dir<P: AsRef<Path>>(root: P) -> PathBuf {
    target_dir(root).join("debug/")
}

pub fn release_dir<P: AsRef<Path>>(root: P) -> PathBuf {
    target_dir(root).join("release/")
}

pub fn object_dir<P: AsRef<Path>>(profile_dir: P) -> PathBuf {
    profile_dir.as_ref().join("object/")
}

pub fn build_cache_path<P: AsRef<Path>>(root: P) -> PathBuf {
    root.as_ref().join("build.cache")
}
