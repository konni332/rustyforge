use std::{path::PathBuf, sync::Arc};

use globset::GlobSet;
use rustyforge_core::{
    BuildResult, Cli, CliCommand, CoreError, CoreResult, GlobalContext, ProjectInfo, ToolConfig,
    error, internal_error, manifest::Manifest, shell::Verbosity, success, with_shell,
};

/// Initializes RustyForge project in the current directory
pub fn initialize_new_rustyforge(force: bool) -> CoreResult<()> {
    let cwd = std::env::current_dir()?;
    if cwd.join("RustyForge.toml").exists() && !force {
        return Err(Box::new(CoreError::AlreadyInitialized));
    }
    match cwd.file_name().and_then(|s| s.to_str()) {
        Some(name) => Manifest::write_new(name),
        None => Manifest::write_default(),
    }?;
    success!(&"Initialized", &"new project");
    Ok(())
}

/// Creates an new RustyForge project
pub fn create_new_rustyforge(name: &str, bin: bool, lib: bool, force: bool) -> CoreResult<()> {
    let old_cwd = std::env::current_dir()?;
    let project_dir = old_cwd.join(name);
    std::fs::create_dir_all(&project_dir)?;
    std::env::set_current_dir(&project_dir)?;
    if project_dir.join("RustyForge.toml").exists() && !force {
        return Err(Box::new(CoreError::AlreadyInitialized));
    }
    if bin {
        Manifest::write_bin_template(name)?;
    } else if lib {
        Manifest::write_lib_template(name)?;
    } else {
        Manifest::write_new(name)?;
    }

    std::env::set_current_dir(old_cwd)?;

    success!(&"Created", &format!("new project: {name}"));
    Ok(())
}

pub fn clean() -> CoreResult<()> {
    let path = std::env::current_dir()?.join("build");
    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }
    success!(&"Cleaned", &"build artifacts");
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
        let build_dir = ctx.get_build_dir();
        std::fs::create_dir_all(&build_dir)?;
        let path = build_dir.join("info.json");
        std::fs::write(path, str)?;
    } else {
        println!("{info}");
    }

    Ok(())
}

pub fn build(cli: &Cli, manifest: &Manifest, config: &ToolConfig) -> CoreResult<BuildResult> {
    let mut ctx = GlobalContext::new(cli, manifest, config)?;
    ctx.build()
}

pub fn run(cli: &Cli, build_res: &BuildResult, manifest: &Manifest) -> CoreResult<()> {
    let (mut cmd, executable) = match &cli.command {
        CliCommand::Run { args, exe, .. } => {
            let executable = determine_executable(exe, build_res, manifest)?;
            let mut cmd = std::process::Command::new(&executable);
            cmd.args(args);
            (cmd, executable)
        }
        _ => {
            internal_error!("run was executed but cli did not contain run command");
        }
    };

    success!(&"Running", &executable.display());
    let status = cmd.status()?;
    if status.success() {
        success!(
            &"Exited",
            &format!("with exit code '{}'", status.code().unwrap_or_default())
        );
    } else {
        error!(
            &"Exited",
            &format!("with exit code '{}'", status.code().unwrap_or_default())
        );
    }
    Ok(())
}

fn determine_executable(
    bin: &Option<String>,
    build_res: &BuildResult,
    manifest: &Manifest,
) -> CoreResult<PathBuf> {
    let path = if let Some(bin_name) = bin {
        build_res.exe_paths.get(bin_name)
    } else {
        let first_in_manifest = manifest.bin.as_ref().and_then(|bins| bins.first());
        if let Some(first) = first_in_manifest {
            build_res.exe_paths.get(&first.name)
        } else {
            return Err(Box::new(CoreError::NoExeFound));
        }
    };
    if let Some(path) = path {
        Ok(path.to_path_buf())
    } else {
        Err(Box::new(CoreError::ExeNotFound {
            name: bin.as_ref().map(|s| s.as_str()).unwrap_or("any").into(),
        }))
    }
}
