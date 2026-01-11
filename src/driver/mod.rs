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
use std::sync::Arc;

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
pub use runtime::RunTimeConfig;
pub use runtime::TargetKind;
pub use runtime_info::get_project_info_string;

use crate::Cli;
use crate::CoreResult;
use crate::ToolConfig;
use crate::config::Manifest;
use crate::config::tool_config::CCompilerKind;
use crate::config::tool_config::CppCompilerKind;
use crate::driver::cannonical_command::CannonicalCommand;
use crate::driver::runtime::Target;
use crate::driver::toolchain::CCompiler;
use crate::driver::toolchain::Clang;
use crate::driver::toolchain::CppCompiler;
use crate::driver::toolchain::Gcc;
use crate::driver::toolchain::Intel;
use crate::driver::toolchain::Msvc;
use crate::internal_error;

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

    fn compile_target_cmds(&self, target: &Target) -> CoreResult<CompilationResult> {
        let output_dir = self.get_object_dir(target);
        let ignore_set = build_ignore_set(target)?;
        let c_files = self.discover_c_files(ignore_set.clone());
        let cpp_files = self.discover_cpp_files(ignore_set.clone());

        let ccomp = make_c_compiler(self.config.toolchain.c_compiler);
        let mut c_commands =
            self.make_commands_for_c_files(&*ccomp, &c_files, target, &output_dir)?;

        let cppcomp = make_cpp_compiler(self.config.toolchain.cpp_compiler);
        let cpp_commands =
            self.make_commands_for_cpp_files(&*cppcomp, &cpp_files, target, &output_dir)?;

        let has_cpp = !cpp_commands.is_empty();
        c_commands.extend(cpp_commands);
        Ok(CompilationResult {
            has_cpp,
            cmds: c_commands,
        })
    }

    fn make_commands_for_c_files(
        &self,
        compiler: &dyn CCompiler,
        files: &[PathBuf],
        target: &Target,
        object_dir: &Path,
    ) -> CoreResult<Vec<CannonicalCommand>> {
        let profile = &self.config.profile;
        let pool = self.pool.clone();

        pool.install(|| {
            files
                .par_iter()
                .map(|path| {
                    let output = object_path(object_dir, path);
                    compiler.compile_unit_cmd(path, &output, profile, target)
                })
                .collect()
        })
    }

    fn make_commands_for_cpp_files(
        &self,
        compiler: &dyn CppCompiler,
        files: &[PathBuf],
        target: &Target,
        object_dir: &Path,
    ) -> CoreResult<Vec<CannonicalCommand>> {
        let profile = &self.config.profile;
        let pool = self.pool.clone();

        pool.install(|| {
            files
                .par_iter()
                .map(|path| {
                    let output = object_path(object_dir, path);
                    compiler.compile_unit_cmd(path, &output, profile, target)
                })
                .collect()
        })
    }
}

pub struct CompilationResult {
    pub has_cpp: bool,
    pub cmds: Vec<CannonicalCommand>,
}

fn build_ignore_set(target: &Target) -> CoreResult<Arc<GlobSet>> {
    let mut builder = GlobSetBuilder::new();
    if let Some(ignore_patterns) = &target.ignore {
        for pat in ignore_patterns {
            let glob = Glob::new(pat)?;
            builder.add(glob);
        }
    }
    Ok(Arc::new(builder.build()?))
}

fn make_c_compiler(kind: CCompilerKind) -> Box<dyn CCompiler> {
    match kind {
        CCompilerKind::Gcc => Box::new(Gcc),
        CCompilerKind::Clang => Box::new(Clang),
        CCompilerKind::Msvc => Box::new(Msvc),
        CCompilerKind::Icc => Box::new(Intel),
    }
}

fn make_cpp_compiler(kind: CppCompilerKind) -> Box<dyn CppCompiler> {
    match kind {
        CppCompilerKind::Gpp => Box::new(Gcc),
        CppCompilerKind::Clangpp => Box::new(Clang),
        CppCompilerKind::Msvc => Box::new(Msvc),
        CppCompilerKind::Icc => Box::new(Intel),
    }
}

fn object_path(dir: &Path, src_file: &Path) -> PathBuf {
    let file_name = match src_file.file_name() {
        Some(n) => n,
        None => {
            internal_error!("Failed to determine source file, filename");
        }
    };

    dir.join(file_name)
}
