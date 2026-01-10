mod cache;
mod runtime;
mod runtime_info;
mod toolchain_resolve;
use std::path::PathBuf;

pub use runtime::RunTimeConfig;
pub use runtime::TargetKind;
pub use runtime_info::get_project_info_string;

use crate::Cli;
use crate::CoreResult;
use crate::ToolConfig;
use crate::config::Manifest;

pub struct GlobalContext<'a> {
    pub cwd: PathBuf,

    pub config: RunTimeConfig<'a>,
}

impl<'a> GlobalContext<'a> {
    pub fn new(cli: &'a Cli, manifest: &'a Manifest, config: &'a ToolConfig) -> CoreResult<Self> {
        let cwd = std::env::current_dir()?;
        let config = RunTimeConfig::new(cli, manifest, config)?;

        Ok(Self { cwd, config })
    }
}
