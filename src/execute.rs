use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{
    ForgeArgs,
    cli::{Command, ForgeOptions},
    compile::{Clang, CompilerDriver, Gcc, Msvc, types::Profile},
    config::{
        project::ProjectConfig,
        tool::{CompilerKind, ToolConfig},
    },
    fs::{
        create_profile_dir, initialize_filestructure, load_project_config, load_tool_config,
        remove_file_structure, remove_target_dir,
    },
    ui,
    utils::resolve_compiler,
};

pub fn execute(forge_args: ForgeArgs) -> Result<()> {
    if forge_args.verbose_hard {
        verbosio::set_verbosity!(2);
    } else if forge_args.verbose {
        verbosio::set_verbosity!(1);
    }
    let tool_config = load_tool_config()?;
    let project_config = load_project_config()?;
    match &forge_args.command {
        Command::Init => execute_init(),
        Command::New { project_name } => execute_new(project_name),
        Command::Build { opts } => {
            execute_build(&forge_args, opts, &project_config, &tool_config)?;
            Ok(())
        }
        Command::Rebuild { opts } => {
            execute_rebuild(&forge_args, opts, &project_config, &tool_config)
        }
        Command::Run { opts, args } => {
            execute_run(&forge_args, opts, args, &project_config, &tool_config)
        }
        Command::Clean => execute_clean(),
        Command::Remove => execute_remove(),
    }
}

fn execute_init() -> Result<()> {
    let project_name = std::env::current_dir()?
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or("new-project".to_string());

    initialize_filestructure(Some(&project_name))?;
    ui::output_rustyforge_initialized();
    Ok(())
}

fn execute_new(name: &str) -> Result<()> {
    initialize_filestructure(Some(name))?;
    ui::output_rustyforge_new(name);
    Ok(())
}

fn execute_remove() -> Result<()> {
    remove_file_structure()?;
    ui::output_rustyforge_removed();
    Ok(())
}

fn execute_build(
    args: &ForgeArgs,
    opts: &ForgeOptions,
    project_config: &ProjectConfig,
    tool_config: &ToolConfig,
) -> Result<PathBuf> {
    let project_config_compiler = match opts.profile().unwrap_or(Profile::Debug) {
        Profile::Debug => project_config
            .project
            .profile_debug
            .as_ref()
            .and_then(|p| p.compiler),
        Profile::Release => project_config
            .project
            .profile_release
            .as_ref()
            .and_then(|p| p.compiler),
    };
    create_profile_dir(args.profile().unwrap_or(Profile::Debug))?;
    let compiler_kind = resolve_compiler(tool_config.compiler, project_config_compiler)
        .context("Failed to resolve compiler kind")?;
    let root = std::env::current_dir()?;
    match compiler_kind {
        CompilerKind::Clang => {
            let mut driver = CompilerDriver::<Clang>::default_driver(root, args, project_config)?;
            driver.compile_incremental()?;
        }
        CompilerKind::Gcc => {
            let mut driver = CompilerDriver::<Gcc>::default_driver(root, args, project_config)?;
            driver.compile_incremental()?;
        }
        CompilerKind::Msvc => {
            let mut driver = CompilerDriver::<Msvc>::default_driver(root, args, project_config)?;
            driver.compile_incremental()?;
        }
    };
    Ok(PathBuf::from("place-holder"))
}

fn execute_run(
    args: &ForgeArgs,
    opts: &ForgeOptions,
    program_args: &[String],
    project_config: &ProjectConfig,
    tool_config: &ToolConfig,
) -> Result<()> {
    execute_build(args, opts, project_config, tool_config)?;
    unimplemented!()
}

fn execute_rebuild(
    args: &ForgeArgs,
    opts: &ForgeOptions,
    project_config: &ProjectConfig,
    tool_config: &ToolConfig,
) -> Result<()> {
    execute_clean()?;
    execute_build(args, opts, project_config, tool_config)?;
    Ok(())
}

fn execute_clean() -> Result<()> {
    remove_target_dir()?;
    ui::output_rustyforge_cleaned();
    Ok(())
}
