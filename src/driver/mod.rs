mod cache;
mod discovery;
mod runtime;
mod runtime_info;
mod toolchain_resolve;
mod utils;

use std::path::PathBuf;
use std::sync::Arc;

use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
pub use runtime::RunTimeConfig;
pub use runtime::TargetKind;
pub use runtime_info::get_project_info_string;

use crate::Cli;
use crate::CoreResult;
use crate::ToolConfig;
use crate::config::Manifest;

pub struct GlobalContext<'ctx> {
    pub cwd: PathBuf,

    pub config: RunTimeConfig<'ctx>,

    pool: Arc<ThreadPool>,
}

impl<'ctx> GlobalContext<'ctx> {
    pub fn new(
        cli: &'ctx Cli,
        manifest: &'ctx Manifest,
        config: &'ctx ToolConfig,
    ) -> CoreResult<Self> {
        let cwd = std::env::current_dir()?;
        let config = RunTimeConfig::new(cli, manifest, config)?;

        let pool = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(config.meta.threads)
                .build()?,
        );

        Ok(Self { cwd, config, pool })
    }
}
