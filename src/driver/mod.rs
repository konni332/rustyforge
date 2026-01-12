mod cache;
mod cannonical_command;
mod compiler;
mod discovery;
mod runtime;
mod runtime_info;
mod toolchain;
mod toolchain_resolve;
mod utils;

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use crate::Cli;
use crate::CoreError;
use crate::CoreResult;
use crate::ProgressBar;
use crate::ToolConfig;
use crate::config::Manifest;
use crate::config::tool_config::CCompilerKind;
use crate::config::tool_config::CppCompilerKind;
use crate::diagnostics::RustyForgeDiagnostic;
use crate::driver::cache::BuildCache;
use crate::driver::cache::CacheFile;
use crate::driver::cannonical_command::CannonicalCommand;
use crate::driver::compiler::CompileContext;
use crate::driver::compiler::CompileResult;
use crate::driver::runtime::Profile;
use crate::driver::runtime::RuntimeToolchain;
use crate::driver::runtime::Target;
use crate::driver::toolchain::CCompiler;
use crate::driver::toolchain::Clang;
use crate::driver::toolchain::CppCompiler;
use crate::driver::toolchain::Gcc;
use crate::driver::toolchain::Intel;
use crate::driver::toolchain::Msvc;
use crate::internal_error;
use crate::status;
use crate::success;
use crate::utils::display_command;
use crate::with_shell;
use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
pub use runtime::RunTimeConfig;
pub use runtime::TargetKind;
pub use runtime_info::get_project_info_string;
use std::hash::{Hash, Hasher};

pub struct GlobalContext<'ctx> {
    pub cwd: PathBuf,
    pub config: RunTimeConfig<'ctx>,
    pool: Arc<ThreadPool>,
    cache: CacheFile<BuildCache>,
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

        let cache_path = cwd.join("build").join("build.cache");
        let cache = CacheFile::new(cache_path)?;

        Ok(Self {
            cwd,
            config,
            pool,
            cache,
        })
    }

    pub fn build(&mut self) -> CoreResult<()> {
        for target in self.config.targets.clone() {
            let CompileResult { cmds, has_cpp } = self.compile(&target)?;
            let failed = self.execute_build_commands(&cmds)?;
            if failed {
                return Err(Box::new(CoreError::BuildFailed));
            }
        }
        Ok(())
    }

    fn execute_build_commands(&self, ccmds: &[(CannonicalCommand, u64)]) -> CoreResult<bool> {
        let mut failed = false;
        let mut pb = ProgressBar::new(ccmds.len());
        for (ccmd, hash) in ccmds {
            status!(&"Compiling".to_string(), &pb.render());
            pb.next();

            if self.cache.contains(hash) {
                continue;
            } else {
                self.cache.insert(hash, PathBuf::new());
            }
            let mut cmd = std::process::Command::from(ccmd);
            let output = cmd.output()?;
            if !output.status.success() {
                let diagnostic = RustyForgeDiagnostic::BuildCommandFail {
                    cmd: display_command(&cmd),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                };
                with_shell(|sh| {
                    sh.print_miette(&diagnostic);
                });
                failed = true;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        success!(&"Compiled".to_string());
        Ok(failed)
    }

    fn compile(&mut self, target: &Target) -> CoreResult<CompileResult> {
        let ignore = if let Some(patterns) = target.ignore.as_ref() {
            let globs: Vec<Glob> = patterns
                .iter()
                .filter_map(|pat| Glob::new(pat).ok())
                .collect();
            Arc::new(GlobSet::new(globs)?)
        } else {
            let globs: Vec<Glob> = Vec::new();
            Arc::new(GlobSet::new(globs)?)
        };
        self.create_file_structure(target)?;
        let c_files = self.discover_c_files(ignore.clone());
        let cpp_files = self.discover_cpp_files(ignore.clone());
        let includes = self.discover_include_dirs(ignore.clone());
        let obj_dir = self.get_object_dir(target);
        let compile_ctx = CompileContext::new(
            &includes,
            &c_files,
            &cpp_files,
            target,
            &self.config.profile,
            &self.config.toolchain,
            &mut self.cache,
        );
        compile_ctx.build(&obj_dir)
    }
}
