use std::process::ExitCode;

use rustyforge_core::{
    Cli, CliCommand, RustyForgeReport, ToolConfig, manifest::Manifest, with_shell,
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
    with_shell(|sh| sh.set_verbosity(&cli.get_verbosity()));

    match &cli.command {
        CliCommand::New {
            name,
            bin,
            lib,
            force,
        } => {
            create_new_rustyforge(name, *bin, *lib, *force).map_err(RustyForgeReport::from)?;
            return Ok(());
        }
        CliCommand::Init { force } => {
            initialize_new_rustyforge(*force).map_err(RustyForgeReport::from)?;
            return Ok(());
        }
        _ => {}
    }

    let manifest = Manifest::read_manifest().map_err(RustyForgeReport::from)?;

    let config = ToolConfig::resolve_config().map_err(RustyForgeReport::from)?;

    if let CliCommand::Info { json } = &cli.command {
        project_info(&cli, &manifest, &config, *json).map_err(RustyForgeReport::from)?;
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
