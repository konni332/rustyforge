use std::process::ExitCode;

use rustyforge_core::{
    Cli, CliCommand, RunTimeConfig, RustyForgeReport, ToolConfig, manifest::Manifest, with_shell,
};

use crate::execute::{
    build, clean, create_new_rustyforge, initialize_new_rustyforge, project_info,
};

mod execute;

fn main() -> ExitCode {
    setup();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(report) => {
            with_shell(|sh| sh.print_miette(&report));
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), RustyForgeReport> {
    let cli = Cli::create().map_err(RustyForgeReport::from)?;

    match &cli.command {
        CliCommand::New { name, bin, lib } => {
            create_new_rustyforge(name, *bin, *lib).map_err(RustyForgeReport::from)?;
            return Ok(());
        }
        CliCommand::Init => {
            initialize_new_rustyforge().map_err(RustyForgeReport::from)?;
            return Ok(());
        }
        _ => {}
    }

    let manifest = Manifest::read_manifest().map_err(RustyForgeReport::from)?;

    let config = ToolConfig::resolve_config().map_err(RustyForgeReport::from)?;

    let runtime_config =
        RunTimeConfig::new(&cli, &manifest, &config).map_err(RustyForgeReport::from)?;

    if let CliCommand::Info { json } = &cli.command {
        project_info(&runtime_config, *json).map_err(RustyForgeReport::from)?;
        return Ok(());
    }

    if let CliCommand::Clean = &cli.command {
        clean()?;
        return Ok(());
    }

    build(&cli, &manifest, &config)?;

    Ok(())
}

fn setup() {
    miette::set_panic_hook();
}
