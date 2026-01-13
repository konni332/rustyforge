use std::sync::Arc;

use globset::GlobSet;
use rustyforge_core::{
    Cli, CoreError, CoreResult, GlobalContext, ProjectInfo, ToolConfig, internal_error,
    manifest::Manifest, shell::Verbosity, success, with_shell,
};

/// Initializes RustyForge project in the current directory
pub fn initialize_new_rustyforge() -> CoreResult<()> {
    match std::env::current_dir()?
        .file_name()
        .and_then(|s| s.to_str())
    {
        Some(name) => Manifest::write_new(name),
        None => Manifest::write_default(),
    }?;
    Ok(())
}

/// Creates an new RustyForge project
pub fn create_new_rustyforge(name: &str, bin: bool, lib: bool) -> CoreResult<()> {
    let old_cwd = std::env::current_dir()?;
    let project_dir = old_cwd.join(name);
    std::fs::create_dir_all(&project_dir)?;
    std::env::set_current_dir(project_dir)?;

    if bin {
        Manifest::write_bin_template(name)?;
    } else if lib {
        Manifest::write_lib_template(name)?;
    } else {
        Manifest::write_new(name)?;
    }

    std::env::set_current_dir(old_cwd)?;
    Ok(())
}

pub fn clean() -> CoreResult<()> {
    let path = std::env::current_dir()?.join("build");
    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }
    success!(&"Cleaned".to_string(), &"build artifacts".to_string());
    Ok(())
}

pub fn project_info(
    cli: &Cli,
    manifest: &Manifest,
    config: &ToolConfig,
    json: bool,
) -> CoreResult<()> {
    let ctx = GlobalContext::new(cli, manifest, config)?;
    let mut info = ProjectInfo {
        config: &ctx.config,
        includes: None,
        c_sources: None,
        cpp_sources: None,
    };
    let verbosity = with_shell(|sh| sh.get_verbosity());
    let ignore = Arc::new(GlobSet::empty());
    if verbosity == Verbosity::Verbose {
        info.includes = Some(ctx.discover_include_dirs(ignore.clone()));
        info.c_sources = Some(ctx.discover_c_files(ignore.clone()));
        info.cpp_sources = Some(ctx.discover_cpp_files(ignore));
    }

    if json {
        let str = match serde_json::to_string_pretty(&info) {
            Ok(s) => s,
            Err(e) => {
                // NOTE: The Runtime config layout is static and known. If it fails there is a
                // serious Bug in either our config or serde
                internal_error!("Failed to serialize runtime config: {}", e);
            }
        };

        todo!("use json string somehow: {}", str);
    } else {
        println!("{info}");
    }

    Ok(())
}

pub fn build(cli: &Cli, manifest: &Manifest, config: &ToolConfig) -> CoreResult<()> {
    let mut ctx = GlobalContext::new(cli, manifest, config)?;
    ctx.build()
}
