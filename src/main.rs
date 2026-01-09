use rustyforge_core::{
    Cli, CliCommand, RunTimeConfig, ToolConfig, internal_error, manifest::Manifest,
};

use crate::{
    diagnostics::RustyForgeReport,
    execute::{create_new_rustyforge, initialize_new_rustyforge, project_info},
};

mod diagnostics;
mod execute;

fn main() -> miette::Result<()> {
    setup()?;

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

    internal_error!("unimplemented");
}

fn setup() -> miette::Result<()> {
    miette::set_panic_hook();
    miette::set_hook(Box::new(
        |_| Box::new(miette::GraphicalReportHandler::new()),
    ))?;
    Ok(())
}
