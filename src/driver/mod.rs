mod archiver;
mod cache;
mod cannonical_command;
mod compiler;
mod discovery;
mod linker;
mod runtime;
mod runtime_info;
mod toolchain;
mod toolchain_resolve;
mod utils;

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use crate::Cli;
use crate::CoreError;
use crate::CoreResult;
use crate::ProgressBar;
use crate::ToolConfig;
use crate::config::Manifest;
use crate::config::tool_config::CCompilerKind;
use crate::config::tool_config::CppCompilerKind;
use crate::diagnostics::RustyForgeDiagnostic;
use crate::driver::archiver::ArchiverContext;
use crate::driver::cache::BuildCache;
use crate::driver::cache::CacheFile;
use crate::driver::cannonical_command::CannonicalCommand;
use crate::driver::compiler::CompileContext;
use crate::driver::compiler::CompileResult;
use crate::driver::linker::LinkContext;
use crate::driver::runtime::Profile;
use crate::driver::runtime::RuntimeToolchain;
use crate::driver::runtime::Target;
use crate::driver::toolchain::CCompiler;
use crate::driver::toolchain::Clang;
use crate::driver::toolchain::CppCompiler;
use crate::driver::toolchain::Gcc;
use crate::driver::toolchain::Intel;
use crate::driver::toolchain::Msvc;
use crate::driver::utils::format_output_file;
use crate::internal_error;
use crate::status;
use crate::success;
use crate::utils::display_command;
use crate::verbose;
use crate::with_shell;
use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
pub use runtime::RunTimeConfig;
pub use runtime::TargetKind;
pub use runtime_info::ProjectInfo;
use std::hash::{Hash, Hasher};

pub struct GlobalContext<'ctx> {
    pub cwd: PathBuf,
    pub config: RunTimeConfig<'ctx>,
    pool: Arc<ThreadPool>,
    cache: CacheFile<BuildCache>,
    all_entries: Vec<&'ctx Path>,
}

/// Global build context holding resolved configuration, toolchain state,
/// thread pool, and build cache.
///
/// This type orchestrates the full build lifecycle: compilation, linking,
/// caching, and diagnostics emission.
impl<'ctx> GlobalContext<'ctx> {
    /// Create a new `GlobalContext` from CLI arguments, manifest, and tool configuration.
    ///
    /// This resolves the runtime configuration, initializes the thread pool,
    /// loads the build cache, and precomputes executable entry points used
    /// for target filtering.
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
        let all_entries: Vec<&Path> = config
            .targets
            .iter()
            .filter_map(|target| match target.kind {
                TargetKind::Executable { entry } => Some(entry),
                _ => None,
            })
            .collect();
        let cache_path = cwd.join("build").join("build.cache");
        let cache = CacheFile::new(cache_path)?;

        Ok(Self {
            all_entries,
            cwd,
            config,
            pool,
            cache,
        })
    }

    /// Build all configured targets using the selected profile.
    ///
    /// This performs compilation and linking for each target in order,
    /// stopping early if any step fails. Successful builds populate the
    /// returned `BuildResult` with produced artifacts.
    pub fn build(&mut self) -> CoreResult<BuildResult> {
        let mut res = BuildResult {
            exe_paths: HashMap::new(),
            lib_path: None,
        };
        for target in self.config.targets.clone() {
            let CompileResult { cmds, has_cpp } = self.compile(&target)?;
            let failed = self.execute_compile_commands(&cmds)?;
            if failed {
                return Err(Box::new(CoreError::BuildFailed));
            }
            success!(&"Compiled", &format!("{}({})", &target.name, &target.kind));
            let (cmd, output) = match target.kind {
                TargetKind::Static => self.archive(&target)?,
                TargetKind::Executable { entry } => self.link(&target, has_cpp)?,
                TargetKind::Shared => self.link(&target, has_cpp)?,
            };

            let failed = self.execute_link_command(&cmd)?;
            if failed {
                return Err(Box::new(CoreError::BuildFailed));
            }
            success!(
                &"Linked".to_string(),
                &format!("{}({})", &target.name, &target.kind)
            );
            match target.kind {
                TargetKind::Executable { .. } => {
                    res.exe_paths.insert(target.name.to_string(), output);
                }
                TargetKind::Static | TargetKind::Shared => {
                    res.lib_path = Some(output);
                }
            }
        }
        success!(
            &"Finished",
            &format!("profile [{}]", &self.config.profile.name)
        );
        Ok(res)
    }

    /// Archive object files into a static library target.
    ///
    /// This constructs an archiver command based on the resolved toolchain
    /// and returns the command together with the output path.
    fn archive(&self, target: &Target) -> CoreResult<((CannonicalCommand, u64), PathBuf)> {
        let objs = self.discover_obj_files(target);
        let output = self.get_output_path(target);
        let ctx = ArchiverContext::new(
            &output,
            self.cache.seed(),
            &objs,
            self.config.toolchain.archiver,
        );
        Ok((ctx.build()?, output))
    }
    /// Link object files into an executable or shared library.
    ///
    /// The selected linker depends on the toolchain and whether any C++
    /// translation units are present.
    fn link(
        &self,
        target: &Target,
        contains_cpp: bool,
    ) -> CoreResult<((CannonicalCommand, u64), PathBuf)> {
        let objs = self.discover_obj_files(target);
        let output = self.get_output_path(target);
        let lib_dirs = vec![];
        let ctx = LinkContext::new(
            &objs,
            contains_cpp,
            &lib_dirs,
            target,
            &self.config.profile,
            self.config.toolchain.linker,
            self.cache.seed(),
        );
        Ok((ctx.build(&output)?, output))
    }

    /// Execute a single link command if it is not cached.
    ///
    /// Returns `true` if the command failed, otherwise `false`.
    /// On failure, a diagnostic is emitted to the shell.
    fn execute_link_command(&self, cmd: &(CannonicalCommand, u64)) -> CoreResult<bool> {
        let (ccmd, hash) = cmd;
        if !self.cache.contains(hash) {
            let mut cmd = std::process::Command::from(ccmd);
            verbose!("running {}", display_command(&cmd));
            let output = cmd.output()?;
            if !output.status.success() {
                let diagnostic = RustyForgeDiagnostic::BuildCommandFail {
                    cmd: display_command(&cmd),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                };
                with_shell(|sh| sh.print_miette(&diagnostic));
                return Ok(true);
            }
            self.cache.insert(hash, PathBuf::new());
        }

        Ok(false)
    }

    /// Execute all compilation commands in parallel.
    ///
    /// Commands are skipped if present in the build cache. Compilation
    /// is executed using the internal thread pool, and diagnostics are
    /// emitted immediately on failure.
    ///
    /// Returns `true` if any compilation failed.
    fn execute_compile_commands(&self, ccmds: &[(CannonicalCommand, u64)]) -> CoreResult<bool> {
        let failed = AtomicBool::new(false);

        self.pool.install(|| {
            ccmds.par_iter().for_each(|(ccmd, hash)| {
                if self.cache.contains(hash) {
                    return;
                }

                let mut cmd = std::process::Command::from(ccmd);
                verbose!("running {}", display_command(&cmd));
                let output = match cmd.output() {
                    Ok(o) => o,
                    Err(e) => {
                        failed.store(true, Ordering::SeqCst);
                        with_shell(|sh| {
                            sh.print_miette(&RustyForgeDiagnostic::BuildCommandFail {
                                cmd: display_command(&cmd),
                                stderr: format!("Failed to spawn process: {}", e),
                            });
                        });
                        return;
                    }
                };

                if !output.status.success() {
                    failed.store(true, Ordering::SeqCst);
                    let diagnostic = RustyForgeDiagnostic::BuildCommandFail {
                        cmd: display_command(&cmd),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    };
                    with_shell(|sh| {
                        sh.print_miette(&diagnostic);
                    });
                } else {
                    if !output.stderr.is_empty() {
                        with_shell(|sh| drop(sh.err().write_all(&output.stderr)));
                    }
                    self.cache.insert(hash, PathBuf::new());
                }
            });
        });

        Ok(failed.load(Ordering::SeqCst))
    }

    /// Compile all source files for a given target.
    ///
    /// This performs source discovery, applies ignore rules, resolves
    /// include directories, and generates compilation commands for both
    /// C and C++ sources.
    fn compile(&mut self, target: &Target) -> CoreResult<CompileResult> {
        let mut globs: Vec<Glob> = if let Some(patterns) = target.ignore.as_ref() {
            patterns
                .iter()
                .filter_map(|pat| Glob::new(pat).ok())
                .collect()
        } else {
            vec![]
        };

        match &target.kind {
            TargetKind::Executable { entry } => {
                let entry = Path::new(entry);

                for e in &self.all_entries {
                    let rel = e.strip_prefix(&self.cwd).unwrap_or(e);

                    if rel != entry {
                        globs.push(Glob::new(rel.to_string_lossy().as_ref())?);
                    }
                }
            }

            TargetKind::Shared | TargetKind::Static => {
                for e in &self.all_entries {
                    let rel = e.strip_prefix(&self.cwd).unwrap_or(e);
                    globs.push(Glob::new(rel.to_string_lossy().as_ref())?);
                }
            }
        }

        let ignore = Arc::new(GlobSet::new(globs)?);
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

    /// Compute the final output path for a target.
    ///
    /// The resulting path depends on the target kind (executable, static,
    /// or shared library) and the active build profile.
    fn output_path(&self, target: &Target) -> PathBuf {
        let target_dir = self.get_target_dir(target);
        let file = format_output_file(&target.kind, target.name);
        target_dir.join(file)
    }
}

pub struct BuildResult {
    pub exe_paths: HashMap<String, PathBuf>,
    pub lib_path: Option<PathBuf>,
}
