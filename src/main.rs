use std::process::ExitCode;

use rustyforge_core::{
    Cli, CliCommand, ProgressBar, RunTimeConfig, ToolConfig, drop_eprint, drop_eprintln,
    drop_print, drop_println, error, info, internal_error, manifest::Manifest, status, success,
    warn, with_shell,
};

use crate::{
    diagnostics::RustyForgeReport,
    execute::{create_new_rustyforge, initialize_new_rustyforge, project_info},
};

mod diagnostics;
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

    info!(&"hello".to_string(), &"info".to_string());
    warn!(&"hello".to_string(), &"warn".to_string());
    error!(&"hello".to_string(), &"error".to_string());
    success!(&"hello".to_string(), &"success".to_string());

    drop_print!("hello drop print");
    drop_println!("hello drop println");
    drop_eprint!("hello drop eprint");
    drop_eprintln!("hello drop eprintln");

    let mut pb = ProgressBar::new(10);

    for _ in 0..10 {
        success!(&"Compiled".to_string(), &"/home/some/file");
        status!(&"Compiling".to_string(), &pb.render());
        pb.next();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    success!(&"Finished".to_string());

    internal_error!("unimplemented");
}

fn setup() {
    miette::set_panic_hook();
}
