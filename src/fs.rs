use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{
    compile::types::Profile,
    config::{project::ProjectConfig, tool::ToolConfig},
    ui::{self, no_rustyforge_initialized_msg},
};

pub fn initialize_filestructure(project_name: Option<&str>) -> Result<()> {
    let global_config_dir = global_config_dir()?;
    std::fs::create_dir_all(&global_config_dir).context(format!(
        "Failed to create global config dir at: {}",
        global_config_dir.display(),
    ))?;

    let cwd = std::env::current_dir().context("Failed to determine current working directory")?;
    let toml_path = toml_path(&cwd);
    if !toml_path.exists() {
        let project_name = match project_name {
            Some(n) => n,
            None => {
                println!("{}", ui::no_rustyforge_initialized_msg());
                return Ok(());
            }
        };
        let project_config = ProjectConfig::new(project_name);
        let project_config_str = toml::to_string_pretty(&project_config)?;
        std::fs::write(toml_path, project_config_str)?;
    }
    let config_dir = config_dir(&cwd);
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_path(&cwd);
    if !config_path.exists() {
        let config = ToolConfig::default();
        let config_str = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, config_str)?;
    }
    Ok(())
}

pub fn create_profile_dir(profile: Profile) -> Result<()> {
    let cwd = std::env::current_dir()?;
    match profile {
        Profile::Debug => {
            let debug_dir = debug_dir(cwd);
            std::fs::create_dir_all(debug_dir)?;
        }
        Profile::Release => {
            let release_dir = release_dir(cwd);
            std::fs::create_dir_all(release_dir)?;
        }
    }
    Ok(())
}

pub fn profile_dir(profile: Profile) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    match profile {
        Profile::Debug => {
            let debug_dir = debug_dir(cwd);
            Ok(debug_dir)
        }
        Profile::Release => {
            let release_dir = release_dir(cwd);
            Ok(release_dir)
        }
    }
}

pub fn remove_file_structure() -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to determine current working directory")?;
    let toml_path = toml_path(&cwd);
    if toml_path.exists() {
        std::fs::remove_file(toml_path)?;
    }
    let config_dir = config_dir(&cwd);
    if config_dir.exists() {
        std::fs::remove_dir_all(config_dir)?;
    }
    remove_target_dir()?;
    Ok(())
}

pub fn remove_target_dir() -> Result<()> {
    let cwd = std::env::current_dir().context("Failed to determine current working directory")?;
    let target_dir = target_dir(cwd);
    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir)?;
    }
    Ok(())
}

pub fn toml_path<P: AsRef<Path>>(root: P) -> PathBuf {
    root.as_ref().join("RustyForge.toml")
}

pub fn config_dir<P: AsRef<Path>>(root: P) -> PathBuf {
    root.as_ref().join(".rustyforge/")
}

pub fn config_path<P: AsRef<Path>>(root: P) -> PathBuf {
    config_dir(root).join("config.toml")
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
    target_dir(root).join("build.cache")
}

pub fn load_tool_config() -> Result<ToolConfig> {
    let cwd = std::env::current_dir()?;
    let config_path = config_path(&cwd);

    let config_str = std::fs::read_to_string(config_path).context("No config.toml found")?;
    let config = toml::from_str(&config_str)?;
    Ok(config)
}

pub fn load_project_config() -> Result<Option<ProjectConfig>> {
    let cwd = std::env::current_dir()?;
    let config_path = toml_path(&cwd);
    if !config_path.exists() {
        let msg = no_rustyforge_initialized_msg();
        println!("{}", msg);
    }
    let config_str = std::fs::read_to_string(config_path).context("No RustyForge.toml found")?;
    let config = toml::from_str(&config_str)?;
    Ok(Some(config))
}
